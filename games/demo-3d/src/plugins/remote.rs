use bevy::prelude::*;

pub struct RemotePluginGate;

impl Plugin for RemotePluginGate {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "remote")]
        {
            app.add_plugins(bevy_brp_extras::BrpExtrasPlugin::default());
        }
        let _ = app;
    }
}
