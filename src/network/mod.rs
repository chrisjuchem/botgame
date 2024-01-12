mod client;
pub mod messages;
mod server;

pub use client::{ClientExt, ClientPlugin};
pub use server::{ServerExt, ServerPlugin};

pub struct NwDebugPlugin;
impl bevy::app::Plugin for NwDebugPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        // use bevy::prelude::*;
        //
        // use crate::{
        //     cards::*,
        //     network::messages::{JoinMatchmakingQueue, NetworkMessage},
        // };

        // app.add_systems(Startup, || {
        //     let msg = JoinMatchmakingQueue {
        //         player_name: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        //         deck: Card {
        //             name: "BBBBBBBBBBBBBBBBBB".to_string(),
        //             summon_cost: Cost { energy: 0xEEEEEEEE },
        //             hp: 0xDDDDDDDD,
        //             abilities: vec![],
        //             max_energy: 0xCCCCCCCC,
        //             energy_regen: 0xBBBBBBBB,
        //         },
        //     };
        //
        //     info!("{:?}", bincode::serialize(&NetworkMessage::JoinMatchmakingQueue(msg)).unwrap())
        // });
    }
}
