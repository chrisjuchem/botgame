use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

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
use serde::Deserialize;

use crate::{
    cards::Card,
    match_sim::{EffectEvent, NewTurnEvent, StartMatchEvent, Us},
    network::{
        messages::{EffectMessage, NetworkMessage, NewTurnMessage, ProtocolErrorMessage},
        PORT,
    },
};

#[derive(Resource, Deserialize)]
pub struct ClientConfig {
    pub server_ip: IpAddr,
    pub client_id: u64,
    pub deck: Card,
}

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RenetClientPlugin, NetcodeClientPlugin));
        app.insert_resource(RenetClient::new(ConnectionConfig::default()));

        let config_path = std::env::args().nth(1).expect("Must provide config path!");
        let config: ClientConfig = serde_json::from_reader(
            std::fs::File::open(config_path).expect("could not open config file"),
        )
        .expect("invalid config");

        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        let authentication = ClientAuthentication::Unsecure {
            server_addr: SocketAddr::new(config.server_ip, PORT),
            client_id: config.client_id,
            user_data: None,
            protocol_id: 0,
        };
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

        app.insert_resource(config);
        app.insert_resource(transport);

        app.add_systems(First, read_messages);
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
        self.send(ProtocolErrorMessage { msg: msg.to_string() })
    }
}

fn read_messages(
    mut client: ResMut<RenetClient>,
    mut start_match: EventWriter<StartMatchEvent>,
    mut effects: EventWriter<EffectEvent>,
    mut turns: EventWriter<NewTurnEvent>,
    mut commands: Commands,
) {
    while let Some(msg) = client.next_msg() {
        match msg {
            NetworkMessage::MatchStartedMessage(data) => {
                commands.insert_resource(Us(data.you));
                start_match.send(StartMatchEvent { match_id: data.match_id, players: data.players })
            },
            NetworkMessage::EffectMessage(EffectMessage { match_id, effect, targets }) => {
                effects.send(EffectEvent { match_id, effect, targets });
            },
            NetworkMessage::NewTurnMessage(NewTurnMessage { match_id, next_player }) => {
                turns.send(NewTurnEvent { match_id, next_player })
            },
            NetworkMessage::ProtocolErrorMessage(ProtocolErrorMessage { msg }) => {
                log::error!("ProtocolError from server: {msg}")
            },
            other => {
                client.send_error(format!("Unhandleable NetworkMessage: {other:?}"));
            },
        }
    }
}
