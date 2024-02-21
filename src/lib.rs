#![feature(debug_closure_helpers)] // Debug impls
#![feature(exclusive_range_pattern)]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod cards;
pub mod macros;
pub mod match_sim;
pub mod network;
pub mod ui;
pub mod utils;

use bevy::{input::common_conditions::input_toggle_active, log::LogPlugin, prelude::*};
use wasm_bindgen::prelude::*;

use crate::{
    match_sim::MatchSimPlugin,
    network::{ClientPlugin, NwDebugPlugin, ServerPlugin},
    ui::ScenePlugin,
};

fn log_plugin() -> LogPlugin {
    let mut log_config = LogPlugin::default();
    log_config.filter.push_str(",botgame=debug");
    log_config
}

pub fn run_server() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, log_plugin()));
    app.add_plugins((ServerPlugin, MatchSimPlugin { server: true }, NwDebugPlugin));
    app.run();
}

#[wasm_bindgen]
pub fn run_client() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(log_plugin()));
    app.add_plugins((ClientPlugin, MatchSimPlugin { server: false }, NwDebugPlugin, ScenePlugin));

    app.add_plugins(
        bevy_inspector_egui::quick::WorldInspectorPlugin::new()
            .run_if(input_toggle_active(false, KeyCode::KeyI)),
    );
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera3dBundle::default());
    });
    app.run();
}
