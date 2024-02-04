mod client;
pub mod messages;
mod server;

pub use client::{ClientExt, ClientPlugin};
pub use server::{ServerExt, ServerPlugin};

pub const PORT: u16 = 17922;

pub struct NwDebugPlugin;
impl bevy::app::Plugin for NwDebugPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        use bevy::prelude::*;

        use crate::match_sim::MatchId;
        // use crate::{
        //     cards::*,
        //     network::messages::{JoinMatchmakingQueue, NetworkMessage},
        // };

        app.add_systems(
            Update,
            |mut last_count: Local<usize>, all: Query<Entity, With<MatchId>>| {
                let count = all.iter().count();
                if *last_count != count {
                    *last_count = count;
                    debug!("{} match entities", count);
                }
            },
        );
    }
}
