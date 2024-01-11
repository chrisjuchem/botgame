#![feature(debug_closure_helpers)]

mod cards;
mod match_sim;

use bevy::prelude::*;
use wasm_bindgen::prelude::*;

use crate::match_sim::{
    MatchId,
    PlayerId,
    StartMatchEvent,
};

#[wasm_bindgen]
pub fn run_game() {
    println!("{:#?}", cards::deck());

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            // provide the ID selector string here
            canvas: Some("#game-canvas".into()),
            fit_canvas_to_parent: true,
            // ... any other window properties ...
            ..default()
        }),
        ..default()
    }));

    app.add_plugins(match_sim::MatchSimPlugin);

    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut e: EventWriter<StartMatchEvent>) {
    e.send(StartMatchEvent {
        match_id: MatchId::new(),
        players: vec![(PlayerId::new(), cards::deck()), (PlayerId::new(), cards::deck())],
    });
}
