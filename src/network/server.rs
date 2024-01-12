use std::{net::UdpSocket, time::SystemTime};

use bevy::{
    log,
    prelude::*,
    utils::{hashbrown::hash_map::Entry, HashMap},
};
use bevy_renet::{
    renet::{
        transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
        ClientId, ConnectionConfig, DefaultChannel, RenetServer, ServerEvent,
    },
    transport::NetcodeServerPlugin,
    RenetServerPlugin,
};
use extension_trait::extension_trait;

use crate::{
    cards::Card,
    match_sim::{MatchId, PlayerId, StartMatchEvent},
    network::messages::{JoinMatchmakingQueue, MatchStarted, NetworkMessage, ProtocolError},
};

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RenetServerPlugin, NetcodeServerPlugin));
        app.insert_resource(RenetServer::new(ConnectionConfig::default()));

        let server_addr = "127.0.0.1:5000".parse().unwrap();
        let socket = UdpSocket::bind(server_addr).unwrap();
        let server_config = ServerConfig {
            current_time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap(),
            max_clients: 64,
            protocol_id: 0,
            public_addresses: vec![server_addr],
            authentication: ServerAuthentication::Unsecure,
        };
        let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
        app.insert_resource(transport);

        app.insert_resource(ConnectedClients::default());
        app.insert_resource(MMQueue::default());
        app.insert_resource(PlayerMap::default());
        app.add_systems(Update, update);
        app.add_systems(Update, matchmaking.run_if(resource_changed::<MMQueue>()));
        app.add_systems(Update, send_match_start);
    }
}

#[derive(Resource, Default)]
struct ConnectedClients(HashMap<ClientId, Option<PlayerId>>);
#[derive(Resource, Default)]
struct PlayerMap(HashMap<PlayerId, ClientId>);

#[derive(Resource, Default)]
struct MMQueue(HashMap<ClientId, QueueInfo>);
struct QueueInfo {
    deck: Card,
    player_name: String,
}

#[extension_trait]
pub impl ServerExt for RenetServer {
    fn next(&mut self, client_id: &ClientId) -> Option<NetworkMessage> {
        let msg = self.receive_message(*client_id, DefaultChannel::ReliableOrdered)?;

        bincode::deserialize::<NetworkMessage>(&*msg)
            .map_err(|e| {
                self.send_error(client_id, format!("Failed to parse message: {e}\n{msg:?}"));
            })
            .ok()
            .or_else(|| self.next(client_id))
    }
    fn send(&mut self, client_id: &ClientId, msg: impl Into<NetworkMessage>) {
        let nwm = msg.into();
        let Ok(msg_bytes) = bincode::serialize(&nwm) else {
            log::error!("Serializing NetworkMessage failed: {nwm:?}");
            panic!("Serializing NetworkMessage failed");
        };
        self.send_message(*client_id, DefaultChannel::ReliableOrdered, msg_bytes);
    }
    fn send_error(&mut self, client_id: &ClientId, msg: impl std::fmt::Display) {
        log::error!("ProtocolError sent to client {client_id}: {msg}");
        self.send(client_id, ProtocolError { msg: msg.to_string() })
    }
}

fn update(
    mut server_events: EventReader<ServerEvent>,
    mut server: ResMut<RenetServer>,
    mut clients: ResMut<ConnectedClients>,
    mut mm_queue: ResMut<MMQueue>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Client {client_id} connected");
                clients.0.insert(*client_id, None);
            },
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Client {client_id} disconnected: {reason}");
                clients.0.remove(client_id);
            },
        }
    }

    for (client_id, match_id) in clients.0.iter() {
        while let Some(msg) = server.next(client_id) {
            match msg {
                NetworkMessage::JoinMatchmakingQueue(JoinMatchmakingQueue {
                    deck,
                    player_name,
                }) => match mm_queue.0.entry(*client_id) {
                    Entry::Vacant(e) => {
                        e.insert(QueueInfo { player_name, deck: deck.clone() });
                    },
                    Entry::Occupied(_) => {
                        server.send_error(client_id, "already in queue".to_string());
                    },
                },
                NetworkMessage::ProtocolError(ProtocolError { msg }) => {
                    log::error!("ProtocolError from client {client_id}: {msg}")
                },
                other => {
                    server.send_error(client_id, format!("Unhandleable NetworkMessage: {other:?}"));
                },
            }
        }
    }
}

fn matchmaking(
    mut mm_queue: ResMut<MMQueue>,
    mut start_match: EventWriter<StartMatchEvent>,
    mut player_map: ResMut<PlayerMap>,
    mut clients: ResMut<ConnectedClients>,
) {
    debug!("{} players in queue", mm_queue.0.len());

    if mm_queue.0.len() < 2 {
        return;
    }

    let mut i = mm_queue.0.drain();
    let players = i
        .by_ref()
        .take(2)
        .map(|(client_id, info)| {
            let pid = PlayerId::new();
            clients.0.insert(client_id, Some(pid));
            player_map.0.insert(pid, client_id);
            (pid, info.deck)
        })
        .collect();
    *mm_queue = MMQueue(i.collect());

    start_match.send(StartMatchEvent { match_id: MatchId::new(), players });
}

fn send_match_start(
    mut start_match: EventReader<StartMatchEvent>,
    mut server: ResMut<RenetServer>,
    player_map: Res<PlayerMap>,
) {
    for StartMatchEvent { match_id, players } in start_match.read() {
        for (pid, _) in players {
            let client_id = player_map.0.get(pid).unwrap();
            server.send(client_id, MatchStarted {
                match_id: *match_id,
                players: players.clone(),
                you: *pid,
            })
        }
    }
}
