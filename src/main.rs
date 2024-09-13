use bevy_replicon::RepliconPlugins;
use bevy_replicon_renet::RepliconRenetPlugins;

use bevy::{
    prelude::*,
    winit::{UpdateMode::Continuous, WinitSettings},
};

mod mmo_client;

use mmo_client::SimpleBoxPlugin;

fn main() {
    App::new()
        // Makes the server/client update continuously even while unfocused.
        .insert_resource(WinitSettings {
            focused_mode: Continuous,
            unfocused_mode: Continuous,
        })
        .add_plugins((
            DefaultPlugins,
            RepliconPlugins,
            RepliconRenetPlugins,
            SimpleBoxPlugin,
        ))
        .run();
}
