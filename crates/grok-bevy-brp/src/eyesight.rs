//! Agent eyesight: packets, crops, motion strips, black-frame checks, packs.
//!
//! Sensory helpers for agents (not a human editor). Schema: `grok-bevy.eyesight/v1`.

use crate::{capture_viewport_image, BrpClient, CapturedImage};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

/// Eyesight packet schema version string.
pub const EYESIGHT_SCHEMA: &str = "grok-bevy.eyesight/v1";

/// Default max frames in a motion strip (V6 hardening: short strips).
pub const DEFAULT_MOTION_FRAMES: u32 = 6;

/// Default delay between motion frames.
pub const DEFAULT_MOTION_INTERVAL_MS: u64 = 80;

/// Default fovea half-size in pixels when cropping around a point.
pub const DEFAULT_CROP_HALF: u32 = 96;

/// Role of a capture within an eyesight packet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaptureRole {
    Full,
    Crop,
    Frame,
    Baseline,
    After,
    Unlit,
    Top,
    Side,
    Strip,
    Diff,
}

impl CaptureRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Crop => "crop",
            Self::Frame => "frame",
            Self::Baseline => "baseline",
            Self::After => "after",
            Self::Unlit => "unlit",
            Self::Top => "top",
            Self::Side => "side",
            Self::Strip => "strip",
            Self::Diff => "diff",
        }
    }
}

/// One capture entry in an eyesight packet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaptureEntry {
    pub role: String,
    pub abs_path: String,
    pub bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub black_frame_warning: Option<bool>,
}

impl CaptureEntry {
    pub fn from_path(role: CaptureRole, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = fs::read(path).with_context(|| format!("read capture {}", path.display()))?;
        crate::validate_png_header(&bytes)?;
        let (w, h) = crate::read_png_ihdr_size_pub(&bytes)?;
        let meta = fs::metadata(path)?;
        let abs = abs_path_string(path);
        let black = is_mostly_black_png(&bytes, 0.04)?;
        Ok(Self {
            role: role.as_str().to_string(),
            abs_path: abs,
            bytes: meta.len(),
            width: w,
            height: h,
            note: None,
            black_frame_warning: if black { Some(true) } else { None },
        })
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

/// Named subject grounding pixels to ECS.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EyesightSubject {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_screen_estimate: Option<bool>,
}

/// Stimulus applied before temporal capture.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StimulusInfo {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Value>,
}

/// Full agent eyesight packet (`grok-bevy.eyesight/v1`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EyesightPacket {
    pub schema: String,
    pub subject_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_state: Option<String>,
    pub intent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style_intent: Option<String>,
    pub captures: Vec<CaptureEntry>,
    #[serde(default)]
    pub subjects: Vec<EyesightSubject>,
    #[serde(default)]
    pub stimulus: StimulusInfo,
    pub agent_must: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

impl EyesightPacket {
    pub fn new(subject_class: impl Into<String>, intent: impl Into<String>) -> Self {
        Self {
            schema: EYESIGHT_SCHEMA.into(),
            subject_class: subject_class.into(),
            app_state: None,
            intent: intent.into(),
            style_intent: None,
            captures: Vec::new(),
            subjects: Vec::new(),
            stimulus: StimulusInfo {
                kind: "none".into(),
                detail: None,
            },
            agent_must: vec!["open_and_read_each_capture_image".into()],
            target: None,
            port: None,
            pack: None,
            warnings: None,
        }
    }

    pub fn to_json_value(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }

    pub fn to_pretty_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn write_json(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.to_pretty_json()?)?;
        Ok(path.to_path_buf())
    }

    pub fn push_warning(&mut self, w: impl Into<String>) {
        self.warnings.get_or_insert_with(Vec::new).push(w.into());
    }

    /// Validate schema + non-empty captures with positive byte counts.
    pub fn validate(&self) -> Result<()> {
        if self.schema != EYESIGHT_SCHEMA {
            return Err(anyhow!(
                "unexpected schema '{}' (want {})",
                self.schema,
                EYESIGHT_SCHEMA
            ));
        }
        if self.captures.is_empty() {
            return Err(anyhow!("eyesight packet has no captures"));
        }
        for c in &self.captures {
            if c.bytes == 0 {
                return Err(anyhow!("capture {} has zero bytes", c.abs_path));
            }
            if c.abs_path.is_empty() {
                return Err(anyhow!("capture missing abs_path"));
            }
        }
        Ok(())
    }
}

/// Absolute path display helper.
pub fn abs_path_string(path: &Path) -> String {
    match path.canonicalize() {
        Ok(p) => p.display().to_string(),
        Err(_) if path.is_absolute() => path.display().to_string(),
        Err(_) => std::env::current_dir()
            .map(|cwd| cwd.join(path).display().to_string())
            .unwrap_or_else(|_| path.display().to_string()),
    }
}

/// Ensure captures/eyesight (or custom) directory exists; return joined path.
pub fn eyesight_path(base: impl AsRef<Path>, file: &str) -> PathBuf {
    let dir = base.as_ref().join("captures").join("eyesight");
    let _ = fs::create_dir_all(&dir);
    dir.join(file)
}

// ── PNG decode / crop / black-frame / strip ──────────────────────────────────

#[derive(Debug, Clone)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA8
}

impl RgbaImage {
    pub fn decode_png(bytes: &[u8]) -> Result<Self> {
        crate::validate_png_header(bytes)?;
        let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
        let mut reader = decoder.read_info().context("png read_info")?;
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).context("png next_frame")?;
        let width = info.width;
        let height = info.height;
        let rgba = match info.color_type {
            png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
            png::ColorType::Rgb => {
                let rgb = &buf[..info.buffer_size()];
                let mut out = Vec::with_capacity((width * height * 4) as usize);
                for chunk in rgb.chunks(3) {
                    out.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
                }
                out
            }
            png::ColorType::Grayscale => {
                let g = &buf[..info.buffer_size()];
                let mut out = Vec::with_capacity((width * height * 4) as usize);
                for &v in g {
                    out.extend_from_slice(&[v, v, v, 255]);
                }
                out
            }
            png::ColorType::GrayscaleAlpha => {
                let ga = &buf[..info.buffer_size()];
                let mut out = Vec::with_capacity((width * height * 4) as usize);
                for chunk in ga.chunks(2) {
                    out.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
                }
                out
            }
            other => return Err(anyhow!("unsupported PNG color type: {other:?}")),
        };
        Ok(Self {
            width,
            height,
            pixels: rgba,
        })
    }

    pub fn encode_png(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut out, self.width, self.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().context("png write_header")?;
            writer
                .write_image_data(&self.pixels)
                .context("png write_image_data")?;
        }
        Ok(out)
    }

    pub fn save_png(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.encode_png()?)?;
        Ok(path.to_path_buf())
    }
}

/// Axis-aligned crop in pixel space (inclusive min, exclusive max clamped).
pub fn crop_rgba(img: &RgbaImage, x: u32, y: u32, w: u32, h: u32) -> Result<RgbaImage> {
    if w == 0 || h == 0 {
        return Err(anyhow!("crop size must be non-zero"));
    }
    let x1 = x.min(img.width.saturating_sub(1));
    let y1 = y.min(img.height.saturating_sub(1));
    let x2 = (x1 + w).min(img.width);
    let y2 = (y1 + h).min(img.height);
    let cw = x2 - x1;
    let ch = y2 - y1;
    if cw == 0 || ch == 0 {
        return Err(anyhow!("crop rectangle empty after clamp"));
    }
    let mut pixels = Vec::with_capacity((cw * ch * 4) as usize);
    for row in y1..y2 {
        let start = ((row * img.width + x1) * 4) as usize;
        let end = start + (cw as usize) * 4;
        pixels.extend_from_slice(&img.pixels[start..end]);
    }
    Ok(RgbaImage {
        width: cw,
        height: ch,
        pixels,
    })
}

/// Crop a full-frame PNG bytes to a rect; write result.
pub fn crop_png_file(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Result<PathBuf> {
    let bytes = fs::read(src.as_ref())?;
    let img = RgbaImage::decode_png(&bytes)?;
    let cropped = crop_rgba(&img, x, y, w, h)?;
    cropped.save_png(dest)
}

/// Crop centered on (cx, cy) with half extents.
pub fn crop_png_around(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    cx: u32,
    cy: u32,
    half_w: u32,
    half_h: u32,
) -> Result<PathBuf> {
    let x = cx.saturating_sub(half_w);
    let y = cy.saturating_sub(half_h);
    crop_png_file(src, dest, x, y, half_w.saturating_mul(2), half_h.saturating_mul(2))
}

/// True if the frame is *empty* black (minimized / no draw), not merely a dark scene.
///
/// Space games have very low **mean** luminance (black backdrop). We only flag empty
/// when there is also no significant bright content: low max luminance **and** almost
/// no pixels above a mid-bright bar (HUD, sprites, stars).
pub fn is_mostly_black_png(bytes: &[u8], threshold: f32) -> Result<bool> {
    let img = RgbaImage::decode_png(bytes)?;
    if img.pixels.is_empty() {
        return Ok(true);
    }
    let mut sum: f64 = 0.0;
    let mut max_l: f64 = 0.0;
    let mut n: f64 = 0.0;
    let mut bright: f64 = 0.0;
    // Dense-enough sample: every 4th pixel (RGBA stride * 4).
    for chunk in img.pixels.chunks(4).step_by(4) {
        if chunk.len() < 3 {
            continue;
        }
        let r = chunk[0] as f64;
        let g = chunk[1] as f64;
        let b = chunk[2] as f64;
        let l = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255.0;
        sum += l;
        if l > max_l {
            max_l = l;
        }
        // Any mid-bright content (sprites, HUD, stars, UI) counts as "not empty".
        if l > 0.18 {
            bright += 1.0;
        }
        n += 1.0;
    }
    if n == 0.0 {
        return Ok(true);
    }
    let mean = (sum / n) as f32;
    let bright_frac = (bright / n) as f32;
    // Empty black only when mean is dark, peak is dim, AND almost no bright pixels.
    // Dark-but-alive scenes (space + ship) have bright_frac > ~0.001 or max_l high.
    Ok(mean < threshold && (max_l as f32) < 0.35 && bright_frac < 0.0005)
}

/// Horizontal montage of equal-height frames (scaled to min height).
pub fn montage_horizontal(frames: &[RgbaImage], gap: u32) -> Result<RgbaImage> {
    if frames.is_empty() {
        return Err(anyhow!("montage requires at least one frame"));
    }
    let h = frames.iter().map(|f| f.height).min().unwrap();
    let mut scaled: Vec<RgbaImage> = Vec::with_capacity(frames.len());
    for f in frames {
        if f.height == h {
            scaled.push(f.clone());
        } else {
            // Nearest-neighbor scale to height h.
            let scale = h as f32 / f.height as f32;
            let nw = ((f.width as f32) * scale).max(1.0) as u32;
            scaled.push(scale_nearest(f, nw, h)?);
        }
    }
    let total_w: u32 = scaled.iter().map(|f| f.width).sum::<u32>()
        + gap * (scaled.len().saturating_sub(1) as u32);
    let mut pixels = vec![0u8; (total_w * h * 4) as usize];
    let mut x_off = 0u32;
    for f in &scaled {
        for row in 0..h {
            let src_start = (row * f.width * 4) as usize;
            let src_end = src_start + (f.width * 4) as usize;
            let dst_start = ((row * total_w + x_off) * 4) as usize;
            let dst_end = dst_start + (f.width * 4) as usize;
            pixels[dst_start..dst_end].copy_from_slice(&f.pixels[src_start..src_end]);
        }
        x_off += f.width + gap;
    }
    Ok(RgbaImage {
        width: total_w,
        height: h,
        pixels,
    })
}

fn scale_nearest(img: &RgbaImage, new_w: u32, new_h: u32) -> Result<RgbaImage> {
    if new_w == 0 || new_h == 0 {
        return Err(anyhow!("scale target must be non-zero"));
    }
    let mut pixels = vec![0u8; (new_w * new_h * 4) as usize];
    for y in 0..new_h {
        let sy = (y as u64 * img.height as u64 / new_h as u64) as u32;
        for x in 0..new_w {
            let sx = (x as u64 * img.width as u64 / new_w as u64) as u32;
            let si = ((sy * img.width + sx) * 4) as usize;
            let di = ((y * new_w + x) * 4) as usize;
            pixels[di..di + 4].copy_from_slice(&img.pixels[si..si + 4]);
        }
    }
    Ok(RgbaImage {
        width: new_w,
        height: new_h,
        pixels,
    })
}

/// Simple abs-diff montage hint: average absolute channel delta as f32 0..1.
pub fn mean_abs_diff(a: &RgbaImage, b: &RgbaImage) -> Result<f32> {
    if a.width != b.width || a.height != b.height {
        return Err(anyhow!("diff requires equal dimensions"));
    }
    let n = a.pixels.len();
    if n == 0 {
        return Ok(0.0);
    }
    let mut sum: f64 = 0.0;
    for i in 0..n {
        sum += (a.pixels[i] as i16 - b.pixels[i] as i16).unsigned_abs() as f64;
    }
    Ok((sum / (n as f64) / 255.0) as f32)
}

/// Write a crude RGB abs-diff PNG (same size).
pub fn write_diff_png(a_path: &Path, b_path: &Path, dest: &Path) -> Result<(PathBuf, f32)> {
    let a = RgbaImage::decode_png(&fs::read(a_path)?)?;
    let b = RgbaImage::decode_png(&fs::read(b_path)?)?;
    // If sizes differ, scale b to a.
    let b = if b.width != a.width || b.height != a.height {
        scale_nearest(&b, a.width, a.height)?
    } else {
        b
    };
    let mut pixels = vec![0u8; a.pixels.len()];
    for i in (0..a.pixels.len()).step_by(4) {
        for c in 0..3 {
            let d = (a.pixels[i + c] as i16 - b.pixels[i + c] as i16).unsigned_abs() as u8;
            pixels[i + c] = d.saturating_mul(3).min(255); // boost visibility
        }
        pixels[i + 3] = 255;
    }
    let diff = RgbaImage {
        width: a.width,
        height: a.height,
        pixels,
    };
    let score = mean_abs_diff(&a, &b)?;
    let path = diff.save_png(dest)?;
    Ok((path, score))
}

// ── BRP subject parsing ──────────────────────────────────────────────────────

/// Parse subjects from a BRP `world.query` result (Name + Transform).
pub fn subjects_from_query(query: &Value) -> Vec<EyesightSubject> {
    let mut out = Vec::new();
    let rows = match query {
        Value::Array(a) => a.clone(),
        Value::Object(o) => o
            .get("result")
            .and_then(|r| r.as_array())
            .cloned()
            .or_else(|| o.get("entities").and_then(|e| e.as_array()).cloned())
            .unwrap_or_default(),
        _ => Vec::new(),
    };

    for row in rows {
        let entity = row
            .get("entity")
            .and_then(|e| e.as_u64())
            .or_else(|| row.get("id").and_then(|e| e.as_u64()));
        let comps = row
            .get("components")
            .cloned()
            .unwrap_or_else(|| row.clone());
        let name = extract_name(&comps).unwrap_or_else(|| "unnamed".into());
        let translation = extract_translation(&comps);
        out.push(EyesightSubject {
            name,
            entity,
            translation,
            on_screen_estimate: None,
        });
    }
    out
}

fn extract_name(comps: &Value) -> Option<String> {
    if let Some(obj) = comps.as_object() {
        for (k, v) in obj {
            if k.ends_with("Name") || k == "Name" || k.contains("name::Name") {
                if let Some(s) = v.as_str() {
                    return Some(s.to_string());
                }
                if let Some(s) = v.get("name").and_then(|n| n.as_str()) {
                    return Some(s.to_string());
                }
                if let Some(s) = v.get("0").and_then(|n| n.as_str()) {
                    return Some(s.to_string());
                }
            }
        }
    }
    None
}

fn extract_translation(comps: &Value) -> Option<[f64; 3]> {
    let obj = comps.as_object()?;
    for (k, v) in obj {
        if k.ends_with("Transform") || k == "Transform" || k.contains("transform::Transform") {
            let t = v.get("translation")?;
            if let Some(arr) = t.as_array() {
                if arr.len() >= 3 {
                    return Some([
                        arr[0].as_f64().unwrap_or(0.0),
                        arr[1].as_f64().unwrap_or(0.0),
                        arr[2].as_f64().unwrap_or(0.0),
                    ]);
                }
            }
            // {x,y,z}
            let x = t.get("x").and_then(|v| v.as_f64())?;
            let y = t.get("y").and_then(|v| v.as_f64())?;
            let z = t.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0);
            return Some([x, y, z]);
        }
    }
    None
}

/// Map world XY to screen for a simple orthographic camera centered at (cam_x, cam_y).
pub fn world_to_screen_ortho(
    world_x: f64,
    world_y: f64,
    cam_x: f64,
    cam_y: f64,
    visible_half_w: f64,
    visible_half_h: f64,
    screen_w: u32,
    screen_h: u32,
) -> (u32, u32) {
    let nx = ((world_x - cam_x) / (2.0 * visible_half_w) + 0.5).clamp(0.0, 1.0);
    let ny = (0.5 - (world_y - cam_y) / (2.0 * visible_half_h)).clamp(0.0, 1.0);
    let sx = (nx * screen_w as f64) as u32;
    let sy = (ny * screen_h as f64) as u32;
    (sx.min(screen_w.saturating_sub(1)), sy.min(screen_h.saturating_sub(1)))
}

// ── High-level see_* operations ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SeeOptions {
    pub out_dir: PathBuf,
    pub subject_class: String,
    pub intent: String,
    pub style_intent: Option<String>,
    pub app_state: Option<String>,
    pub target_name: Option<String>,
    pub port: Option<u16>,
}

impl Default for SeeOptions {
    fn default() -> Self {
        Self {
            out_dir: PathBuf::from("."),
            subject_class: "scene".into(),
            intent: "verify scene appearance".into(),
            style_intent: None,
            app_state: None,
            target_name: None,
            port: None,
        }
    }
}

/// E0: full-frame capture + BRP Name/Transform subjects → packet.
pub fn see_scene(client: &BrpClient, opts: &SeeOptions) -> Result<EyesightPacket> {
    let full_path = eyesight_path(&opts.out_dir, "scene_full.png");
    let img = capture_viewport_image(client, &full_path)?;
    let entry = CaptureEntry::from_path(CaptureRole::Full, &img.path)?
        .with_note("E0 full frame");

    let mut packet = EyesightPacket::new(&opts.subject_class, &opts.intent);
    packet.app_state = opts.app_state.clone();
    packet.style_intent = opts.style_intent.clone();
    packet.target = opts.target_name.clone();
    packet.port = Some(client.target.port);
    packet.captures.push(entry);

    // Query subjects (best-effort).
    let comps = [
        "bevy_ecs::name::Name",
        "bevy_transform::components::transform::Transform",
    ];
    if let Ok(q) = client.query(&comps) {
        packet.subjects = subjects_from_query(&q);
    } else if let Ok(q) = client.query(&["Name", "Transform"]) {
        packet.subjects = subjects_from_query(&q);
    }

    if packet
        .captures
        .iter()
        .any(|c| c.black_frame_warning == Some(true))
    {
        packet.push_warning(
            "black_frame: window may be minimized, camera wrong, no lights, or still Loading",
        );
    }

    let json_path = eyesight_path(&opts.out_dir, "scene_packet.json");
    packet.write_json(json_path)?;
    packet.validate()?;
    Ok(packet)
}

/// E1: full capture + crop around screen point (or center).
pub fn see_entity(
    client: &BrpClient,
    opts: &SeeOptions,
    entity_name: &str,
    screen_x: Option<u32>,
    screen_y: Option<u32>,
    half: u32,
) -> Result<EyesightPacket> {
    let full_path = eyesight_path(&opts.out_dir, "entity_full.png");
    let img = capture_viewport_image(client, &full_path)?;
    let full = CaptureEntry::from_path(CaptureRole::Full, &img.path)?;

    let (w, h) = (
        full.width.unwrap_or(1280),
        full.height.unwrap_or(720),
    );
    let cx = screen_x.unwrap_or(w / 2);
    let cy = screen_y.unwrap_or(h / 2);
    let crop_path = eyesight_path(
        &opts.out_dir,
        &format!("entity_{}_crop.png", sanitize_name(entity_name)),
    );
    crop_png_around(&img.path, &crop_path, cx, cy, half, half)?;
    let crop = CaptureEntry::from_path(CaptureRole::Crop, &crop_path)?
        .with_note(format!("E1 fovea on '{entity_name}' @ ({cx},{cy}) half={half}"));

    let mut packet = EyesightPacket::new("entity", &opts.intent);
    packet.style_intent = opts.style_intent.clone();
    packet.target = opts.target_name.clone();
    packet.port = Some(client.target.port);
    packet.captures.push(full);
    packet.captures.push(crop);

    if let Ok(q) = client.query(&[
        "bevy_ecs::name::Name",
        "bevy_transform::components::transform::Transform",
    ]) {
        let all = subjects_from_query(&q);
        packet.subjects = all
            .into_iter()
            .filter(|s| s.name == entity_name || entity_name == "*")
            .collect();
        if packet.subjects.is_empty() {
            packet.subjects.push(EyesightSubject {
                name: entity_name.into(),
                entity: None,
                translation: None,
                on_screen_estimate: Some(true),
            });
        }
    } else {
        packet.subjects.push(EyesightSubject {
            name: entity_name.into(),
            entity: None,
            translation: None,
            on_screen_estimate: Some(true),
        });
    }

    let json_path = eyesight_path(
        &opts.out_dir,
        &format!("entity_{}_packet.json", sanitize_name(entity_name)),
    );
    packet.write_json(json_path)?;
    packet.validate()?;
    Ok(packet)
}

/// E1 region crop by explicit pixel rect.
pub fn see_region(
    client: &BrpClient,
    opts: &SeeOptions,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    label: &str,
) -> Result<EyesightPacket> {
    let full_path = eyesight_path(&opts.out_dir, "region_full.png");
    let img = capture_viewport_image(client, &full_path)?;
    let full = CaptureEntry::from_path(CaptureRole::Full, &img.path)?;
    let crop_path = eyesight_path(
        &opts.out_dir,
        &format!("region_{}_crop.png", sanitize_name(label)),
    );
    crop_png_file(&img.path, &crop_path, x, y, w, h)?;
    let crop = CaptureEntry::from_path(CaptureRole::Crop, &crop_path)?
        .with_note(format!("E1 region '{label}' rect=({x},{y},{w},{h})"));

    let mut packet = EyesightPacket::new(opts.subject_class.clone(), &opts.intent);
    packet.style_intent = opts.style_intent.clone();
    packet.target = opts.target_name.clone();
    packet.port = Some(client.target.port);
    packet.captures.push(full);
    packet.captures.push(crop);
    packet.subjects.push(EyesightSubject {
        name: label.into(),
        entity: None,
        translation: None,
        on_screen_estimate: Some(true),
    });

    let json_path = eyesight_path(
        &opts.out_dir,
        &format!("region_{}_packet.json", sanitize_name(label)),
    );
    packet.write_json(json_path)?;
    packet.validate()?;
    Ok(packet)
}

/// E2 temporal: N frames with optional key stimulus; montage strip.
pub fn see_motion(
    client: &BrpClient,
    opts: &SeeOptions,
    frames: u32,
    interval_ms: u64,
    keys: Option<Vec<String>>,
) -> Result<EyesightPacket> {
    let frames = frames.clamp(2, 12);
    let mut packet = EyesightPacket::new(
        if opts.subject_class == "scene" {
            "physics_motion".into()
        } else {
            opts.subject_class.clone()
        },
        &opts.intent,
    );
    packet.style_intent = opts.style_intent.clone();
    packet.target = opts.target_name.clone();
    packet.port = Some(client.target.port);

    if let Some(ref k) = keys {
        // Best-effort: brp_extras/send_keys shapes vary; try common params.
        let params = json!({ "keys": k, "duration": 0.05 });
        let _ = client.call("brp_extras/send_keys", Some(params));
        packet.stimulus = StimulusInfo {
            kind: "keys".into(),
            detail: Some(json!({ "keys": k })),
        };
    }

    let mut decoded = Vec::new();
    for i in 0..frames {
        let path = eyesight_path(&opts.out_dir, &format!("motion_frame_{i:02}.png"));
        let img = capture_viewport_image(client, &path)?;
        let entry = CaptureEntry::from_path(CaptureRole::Frame, &img.path)?
            .with_note(format!("E2 frame {i}/{frames}"));
        if entry.black_frame_warning == Some(true) {
            packet.push_warning(format!("black_frame on motion frame {i}"));
        }
        packet.captures.push(entry);
        if let Ok(bytes) = fs::read(&img.path) {
            if let Ok(rgba) = RgbaImage::decode_png(&bytes) {
                decoded.push(rgba);
            }
        }
        if i + 1 < frames {
            thread::sleep(Duration::from_millis(interval_ms.max(16)));
        }
    }

    if decoded.len() >= 2 {
        if let Ok(strip) = montage_horizontal(&decoded, 2) {
            let strip_path = eyesight_path(&opts.out_dir, "motion_strip.png");
            strip.save_png(&strip_path)?;
            packet.captures.push(
                CaptureEntry::from_path(CaptureRole::Strip, &strip_path)?
                    .with_note("E2 horizontal montage"),
            );
        }
    }

    let json_path = eyesight_path(&opts.out_dir, "motion_packet.json");
    packet.write_json(json_path)?;
    packet.validate()?;
    Ok(packet)
}

/// E3: capture after using an existing baseline path; optional diff image.
pub fn see_diff(
    client: &BrpClient,
    opts: &SeeOptions,
    baseline_path: impl AsRef<Path>,
) -> Result<EyesightPacket> {
    let baseline = baseline_path.as_ref();
    if !baseline.is_file() {
        return Err(anyhow!("baseline not found: {}", baseline.display()));
    }
    let after_path = eyesight_path(&opts.out_dir, "diff_after.png");
    let img = capture_viewport_image(client, &after_path)?;

    let mut packet = EyesightPacket::new(&opts.subject_class, &opts.intent);
    packet.style_intent = opts.style_intent.clone();
    packet.target = opts.target_name.clone();
    packet.port = Some(client.target.port);
    packet.captures.push(
        CaptureEntry::from_path(CaptureRole::Baseline, baseline)?
            .with_note("E3 baseline"),
    );
    packet.captures.push(
        CaptureEntry::from_path(CaptureRole::After, &img.path)?.with_note("E3 after"),
    );

    let diff_path = eyesight_path(&opts.out_dir, "diff_abs.png");
    if let Ok((p, score)) = write_diff_png(baseline, &img.path, &diff_path) {
        packet.captures.push(
            CaptureEntry::from_path(CaptureRole::Diff, &p)?
                .with_note(format!("E3 abs-diff mean={score:.4}")),
        );
    }

    let json_path = eyesight_path(&opts.out_dir, "diff_packet.json");
    packet.write_json(json_path)?;
    packet.validate()?;
    Ok(packet)
}

/// E4/E5 pack presets: entity_craft | landscape | water | physics_jump | lighting
pub fn see_pack(
    client: &BrpClient,
    opts: &SeeOptions,
    pack: &str,
) -> Result<EyesightPacket> {
    let pack = pack.to_ascii_lowercase();
    let mut packet = EyesightPacket::new(
        match pack.as_str() {
            "water" => "water",
            "landscape" => "landscape",
            "physics_jump" | "physics" => "physics_motion",
            "lighting" => "lighting",
            _ => "entity",
        },
        format!("pack:{pack} — {}", opts.intent),
    );
    packet.pack = Some(pack.clone());
    packet.style_intent = opts.style_intent.clone();
    packet.target = opts.target_name.clone();
    packet.port = Some(client.target.port);

    match pack.as_str() {
        "entity_craft" | "entity" => {
            let scene = see_scene(client, opts)?;
            packet.subjects = scene.subjects;
            packet.captures.extend(scene.captures);
            // Center crop craft view
            if let Some(full) = packet.captures.iter().find(|c| c.role == "full") {
                let crop_path = eyesight_path(&opts.out_dir, "pack_entity_crop.png");
                let w = full.width.unwrap_or(1280);
                let h = full.height.unwrap_or(720);
                if crop_png_around(&full.abs_path, &crop_path, w / 2, h / 2, 128, 128).is_ok() {
                    packet.captures.push(
                        CaptureEntry::from_path(CaptureRole::Crop, &crop_path)?
                            .with_note("pack entity_craft center crop"),
                    );
                }
            }
        }
        "landscape" => {
            let full_path = eyesight_path(&opts.out_dir, "pack_landscape.png");
            let img = capture_viewport_image(client, &full_path)?;
            packet.captures.push(
                CaptureEntry::from_path(CaptureRole::Full, &img.path)?
                    .with_note("landscape game cam"),
            );
            let w = packet.captures[0].width.unwrap_or(1280);
            let h = packet.captures[0].height.unwrap_or(720);
            // Horizon band crop (upper-mid strip)
            let crop_path = eyesight_path(&opts.out_dir, "pack_landscape_horizon.png");
            crop_png_file(&img.path, &crop_path, 0, h / 6, w, h / 3)?;
            packet.captures.push(
                CaptureEntry::from_path(CaptureRole::Crop, &crop_path)?
                    .with_note("landscape horizon band"),
            );
            // Lower third terrain
            let terrain_path = eyesight_path(&opts.out_dir, "pack_landscape_terrain.png");
            crop_png_file(&img.path, &terrain_path, 0, h / 2, w, h / 2)?;
            packet.captures.push(
                CaptureEntry::from_path(CaptureRole::Crop, &terrain_path)?
                    .with_note("landscape terrain lower half"),
            );
        }
        "water" => {
            let full_path = eyesight_path(&opts.out_dir, "pack_water.png");
            let img = capture_viewport_image(client, &full_path)?;
            packet.captures.push(
                CaptureEntry::from_path(CaptureRole::Full, &img.path)?
                    .with_note("water establishing"),
            );
            let w = packet.captures[0].width.unwrap_or(1280);
            let h = packet.captures[0].height.unwrap_or(720);
            let crop_path = eyesight_path(&opts.out_dir, "pack_water_surface.png");
            crop_png_file(&img.path, &crop_path, w / 4, h / 4, w / 2, h / 2)?;
            packet.captures.push(
                CaptureEntry::from_path(CaptureRole::Crop, &crop_path)?
                    .with_note("water surface crop"),
            );
            // Short motion
            let mut motion_opts = opts.clone();
            motion_opts.subject_class = "water".into();
            motion_opts.intent = "water motion strip".into();
            if let Ok(m) = see_motion(client, &motion_opts, 4, 100, None) {
                for c in m.captures {
                    if c.role == "strip" || c.role == "frame" {
                        packet.captures.push(c);
                    }
                }
            }
        }
        "physics_jump" | "physics" => {
            let mut motion_opts = opts.clone();
            motion_opts.subject_class = "physics_motion".into();
            let m = see_motion(client, &motion_opts, 6, 70, None)?;
            packet.captures = m.captures;
            packet.stimulus = m.stimulus;
            packet.subjects = m.subjects;
        }
        "lighting" => {
            let full_path = eyesight_path(&opts.out_dir, "pack_lighting.png");
            let img = capture_viewport_image(client, &full_path)?;
            packet.captures.push(
                CaptureEntry::from_path(CaptureRole::Full, &img.path)?
                    .with_note("lighting lit capture (unlit requires game debug mode)"),
            );
            packet.push_warning(
                "unlit diagnostic not automatic; spawn agent debug camera/material if needed",
            );
        }
        other => {
            return Err(anyhow!(
                "unknown pack '{other}' (entity_craft|landscape|water|physics_jump|lighting)"
            ));
        }
    }

    if packet
        .captures
        .iter()
        .any(|c| c.black_frame_warning == Some(true))
    {
        packet.push_warning("black_frame detected in pack captures");
    }

    let json_path = eyesight_path(&opts.out_dir, &format!("pack_{}_packet.json", sanitize_name(&pack)));
    packet.write_json(json_path)?;
    packet.validate()?;
    Ok(packet)
}

fn sanitize_name(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Build MCP text+image content blocks from a packet (first capture image + full JSON).
pub fn packet_to_mcp_content(packet: &EyesightPacket) -> Result<Value> {
    let mut content = Vec::new();
    // Prefer strip or crop for fovea, else first capture.
    let prefer = packet
        .captures
        .iter()
        .find(|c| c.role == "strip" || c.role == "crop" || c.role == "diff")
        .or_else(|| packet.captures.first());
    if let Some(c) = prefer {
        if let Ok(img) = CapturedImage::from_png_path(&c.abs_path) {
            if let Some(arr) = img.to_mcp_content_blocks().as_array() {
                content.extend(arr.iter().cloned());
            }
        }
    }
    content.push(json!({
        "type": "text",
        "text": packet.to_pretty_json()?
    }));
    Ok(json!({ "content": content, "isError": false }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD as B64, Engine};

    fn fixture_png_bytes() -> Vec<u8> {
        let b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
        B64.decode(b64).expect("fixture b64")
    }

    fn solid_png(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
        let img = RgbaImage {
            width: w,
            height: h,
            pixels: rgba.repeat((w * h) as usize),
        };
        img.encode_png().unwrap()
    }

    #[test]
    fn packet_schema_validate_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let png = dir.path().join("a.png");
        fs::write(&png, solid_png(8, 8, [10, 20, 30, 255])).unwrap();
        let mut p = EyesightPacket::new("scene", "test");
        p.captures
            .push(CaptureEntry::from_path(CaptureRole::Full, &png).unwrap());
        p.validate().unwrap();
        let v = p.to_json_value().unwrap();
        assert_eq!(v["schema"], EYESIGHT_SCHEMA);
        assert!(v["captures"][0]["bytes"].as_u64().unwrap() > 0);
        let back: EyesightPacket = serde_json::from_value(v).unwrap();
        assert_eq!(back.schema, EYESIGHT_SCHEMA);
    }

    #[test]
    fn crop_rgba_center_region() {
        let img = RgbaImage {
            width: 10,
            height: 10,
            pixels: {
                let mut p = vec![0u8; 10 * 10 * 4];
                // mark (5,5) red
                let i = (5 * 10 + 5) * 4;
                p[i] = 255;
                p[i + 3] = 255;
                p
            },
        };
        let c = crop_rgba(&img, 4, 4, 3, 3).unwrap();
        assert_eq!(c.width, 3);
        assert_eq!(c.height, 3);
        // center of crop is original (5,5)
        let i = (1 * 3 + 1) * 4;
        assert_eq!(c.pixels[i], 255);
    }

    #[test]
    fn crop_png_file_writes_smaller() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src.png");
        fs::write(&src, solid_png(32, 32, [40, 80, 120, 255])).unwrap();
        let dest = dir.path().join("crop.png");
        crop_png_file(&src, &dest, 4, 4, 8, 8).unwrap();
        let out = RgbaImage::decode_png(&fs::read(&dest).unwrap()).unwrap();
        assert_eq!(out.width, 8);
        assert_eq!(out.height, 8);
    }

    #[test]
    fn black_frame_detects_near_black() {
        let black = solid_png(16, 16, [0, 0, 0, 255]);
        assert!(is_mostly_black_png(&black, 0.04).unwrap());
        let bright = solid_png(16, 16, [200, 200, 200, 255]);
        assert!(!is_mostly_black_png(&bright, 0.04).unwrap());
    }

    #[test]
    fn black_frame_allows_dark_scene_with_bright_sprite() {
        // Mostly black "space" with a bright 4x4 ship in the center — not empty.
        let mut img = RgbaImage {
            width: 64,
            height: 64,
            pixels: [0u8, 0, 0, 255].repeat(64 * 64),
        };
        for y in 30..34 {
            for x in 30..34 {
                let i = ((y * 64 + x) * 4) as usize;
                img.pixels[i] = 240;
                img.pixels[i + 1] = 240;
                img.pixels[i + 2] = 255;
                img.pixels[i + 3] = 255;
            }
        }
        let bytes = img.encode_png().unwrap();
        assert!(
            !is_mostly_black_png(&bytes, 0.04).unwrap(),
            "dark space with bright sprite must not be flagged empty"
        );
    }

    #[test]
    fn montage_horizontal_width_sums() {
        let a = RgbaImage {
            width: 4,
            height: 4,
            pixels: [255, 0, 0, 255].repeat(16),
        };
        let b = RgbaImage {
            width: 4,
            height: 4,
            pixels: [0, 255, 0, 255].repeat(16),
        };
        let m = montage_horizontal(&[a, b], 2).unwrap();
        assert_eq!(m.width, 4 + 2 + 4);
        assert_eq!(m.height, 4);
    }

    #[test]
    fn subjects_from_query_parses_name_transform() {
        let q = json!([
            {
                "entity": 7,
                "components": {
                    "bevy_ecs::name::Name": "Player",
                    "bevy_transform::components::transform::Transform": {
                        "translation": { "x": 1.0, "y": 2.0, "z": 3.0 }
                    }
                }
            }
        ]);
        let s = subjects_from_query(&q);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].name, "Player");
        assert_eq!(s[0].entity, Some(7));
        assert_eq!(s[0].translation, Some([1.0, 2.0, 3.0]));
    }

    #[test]
    fn world_to_screen_ortho_center() {
        let (sx, sy) = world_to_screen_ortho(0.0, 0.0, 0.0, 0.0, 10.0, 10.0, 100, 100);
        assert!((sx as i32 - 50).abs() <= 1);
        assert!((sy as i32 - 50).abs() <= 1);
    }

    #[test]
    fn write_diff_png_score_positive_when_different() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.png");
        let b = dir.path().join("b.png");
        let d = dir.path().join("d.png");
        fs::write(&a, solid_png(8, 8, [0, 0, 0, 255])).unwrap();
        fs::write(&b, solid_png(8, 8, [255, 255, 255, 255])).unwrap();
        let (_p, score) = write_diff_png(&a, &b, &d).unwrap();
        assert!(score > 0.5);
        assert!(d.is_file());
    }

    #[test]
    fn sanitize_and_eyesight_path() {
        let dir = tempfile::tempdir().unwrap();
        let p = eyesight_path(dir.path(), "x.png");
        assert!(p.ends_with("captures/eyesight/x.png") || p.to_string_lossy().contains("eyesight"));
        assert_eq!(sanitize_name("Player Ship!"), "Player_Ship_");
    }

    #[test]
    fn fixture_png_still_valid() {
        validate_roundtrip_fixture();
    }

    fn validate_roundtrip_fixture() {
        let bytes = fixture_png_bytes();
        let img = RgbaImage::decode_png(&bytes).unwrap();
        assert_eq!(img.width, 1);
        assert_eq!(img.height, 1);
    }
}
