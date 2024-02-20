#![feature(debug_closure_helpers)]
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

// pub fn run_game() {
//     println!("{:#?}", cards::deck());
//
//     let mut app = App::new();
//     app.add_plugins(DefaultPlugins.set(WindowPlugin {
//         primary_window: Some(Window {
//             // provide the ID selector string here
//             canvas: Some("#game-canvas".into()),
//             fit_canvas_to_parent: true,
//             // ... any other window properties ...
//             ..default()
//         }),
//         ..default()
//     }));
//
//     app.add_plugins(match_sim::MatchSimPlugin);
//
//     app.add_systems(Startup, setup);
//     app.run();
// }

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
    // app.add_systems(Startup, |mut c: ResMut<RenetClient>| {
    //     c.send(JoinMatchmakingQueueMessage { player_name: "p1".to_string(), deck: cards::deck() })
    // });
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera3dBundle::default());
    });
    app.run();
}
