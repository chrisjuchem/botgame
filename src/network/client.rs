use std::{net::UdpSocket, time::SystemTime};

use bevy::{log, prelude::*};
use bevy_renet::{
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport},
        ConnectionConfig, DefaultChannel, RenetClient,
    },
    transport::NetcodeClientPlugin,
    RenetClientPlugin,
};
use extension_trait::extension_trait;

use crate::{
    match_sim::StartMatchEvent,
    network::messages::{NetworkMessage, ProtocolError},
};

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RenetClientPlugin, NetcodeClientPlugin));
        app.insert_resource(RenetClient::new(ConnectionConfig::default()));

        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let authentication = ClientAuthentication::Unsecure {
            server_addr: "127.0.0.1:5000".parse().unwrap(),
            client_id: socket.local_addr().unwrap().port() as u64,
            user_data: None,
            protocol_id: 0,
        };
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

        app.insert_resource(transport);

        app.add_systems(Update, updates);
    }
}

#[extension_trait]
pub impl ClientExt for RenetClient {
    fn next_msg(&mut self) -> Option<NetworkMessage> {
        let msg = self.receive_message(DefaultChannel::ReliableOrdered)?;

        bincode::deserialize::<NetworkMessage>(&*msg)
            .map_err(|e| {
                self.send_error(format!("Failed to parse message: {e}"));
            })
            .ok()
            .or_else(|| self.next_msg())
    }
    fn send(&mut self, msg: impl Into<NetworkMessage>) {
        let nwm = msg.into();
        let Ok(msg_bytes) = bincode::serialize(&nwm) else {
            log::error!("Serializing NetworkMessage failed: {nwm:?}");
            panic!("Serializing NetworkMessage failed");
        };
        self.send_message(DefaultChannel::ReliableOrdered, msg_bytes);
    }
    fn send_error(&mut self, msg: impl std::fmt::Display) {
        log::error!("ProtocolError sent to server: {msg}");
        self.send(ProtocolError { msg: msg.to_string() })
    }
}

fn updates(mut client: ResMut<RenetClient>, mut start_match: EventWriter<StartMatchEvent>) {
    while let Some(msg) = client.next_msg() {
        match msg {
            NetworkMessage::MatchStarted(data) => {
                // insert resource => data.you
                start_match.send(StartMatchEvent { match_id: data.match_id, players: data.players })
            },

            NetworkMessage::ProtocolError(ProtocolError { msg }) => {
                log::error!("ProtocolError from server: {msg}")
            },
            other => {
                client.send_error(format!("Unhandleable NetworkMessage: {other:?}"));
            },
        }
    }
}
