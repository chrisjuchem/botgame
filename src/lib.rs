#![feature(debug_closure_helpers)]
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate core;

mod cards;
mod macros;
mod match_sim;
mod network;
mod ui;
mod utils;

use bevy::{log::LogPlugin, prelude::*};
use bevy_renet::renet::RenetClient;
use wasm_bindgen::prelude::*;

use crate::{
    match_sim::MatchSimPlugin,
    network::{
        messages::JoinMatchmakingQueueMessage, ClientExt, ClientPlugin, NwDebugPlugin, ServerPlugin,
    },
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
//
// fn setup(mut e: EventWriter<StartMatchEvent>) {
//     e.send(StartMatchEvent {
//         match_id: MatchId::new(),
//         players: vec![(PlayerId::new(), cards::deck()), (PlayerId::new(), cards::deck())],
//     });
// }

fn log_plugin() -> LogPlugin {
    let mut log_config = LogPlugin::default();
    log_config.filter.push_str(",botgame=debug");
    log_config
}

pub fn run_server() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, log_plugin()));
    app.add_plugins((ServerPlugin, MatchSimPlugin, NwDebugPlugin));
    app.run();
}

#[wasm_bindgen]
pub fn run_client() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(log_plugin()));
    app.add_plugins((ClientPlugin, MatchSimPlugin, NwDebugPlugin, ScenePlugin));

    app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
    // app.add_systems(Startup, |mut c: ResMut<RenetClient>| {
    //     c.send(JoinMatchmakingQueueMessage { player_name: "p1".to_string(), deck: cards::deck() })
    // });
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera3dBundle::default());
    });
    app.run();
}
