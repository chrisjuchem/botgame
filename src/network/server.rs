use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::{
    log,
    prelude::*,
    utils::{hashbrown::hash_map::Entry, HashMap},
};
use bevy_mod_index::prelude::Index;
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
    cards::{Ability, Card, Effect},
    match_sim::{
        Cards, CurrentTurn, EffectEvent, GridLocation, MatchId, NewTurnEvent, OwnerIndex, PlayerId,
        StartMatchEvent,
    },
    network::{
        messages::{
            ActivateAbilityMessage, EffectMessage, JoinMatchmakingQueueMessage,
            MatchStartedMessage, NetworkMessage, NewTurnMessage, ProtocolErrorMessage,
        },
        PORT,
    },
};

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RenetServerPlugin, NetcodeServerPlugin));
        app.insert_resource(RenetServer::new(ConnectionConfig::default()));

        let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), PORT);
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
        app.insert_resource(MatchClientMap::default());
        app.insert_resource(AbilityQueue::default());
        app.add_systems(First, read_messages);
        app.add_systems(PreUpdate, matchmaking.run_if(resource_changed::<MMQueue>()));
        app.add_systems(
            Update,
            (process_abilities, send_match_start, send_effects, send_turn_change).chain(),
        );
    }
}

#[derive(Resource, Default)]
struct ConnectedClients(HashMap<ClientId, Option<PlayerId>>);
#[derive(Resource, Default)]
struct MatchClientMap(HashMap<MatchId, Vec<ClientId>>);

#[derive(Resource, Default)]
struct MMQueue(HashMap<ClientId, QueueInfo>);
struct QueueInfo {
    deck: Card,
    player_name: String,
}

#[derive(Resource, Default)]
struct AbilityQueue(Vec<(ClientId, ActivateAbilityMessage)>);

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
        self.send(client_id, ProtocolErrorMessage { msg: msg.to_string() })
    }
}

fn read_messages(
    mut server_events: EventReader<ServerEvent>,
    mut server: ResMut<RenetServer>,
    mut clients: ResMut<ConnectedClients>,
    mut mm_queue: ResMut<MMQueue>,
    mut ability_queue: ResMut<AbilityQueue>,
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
                NetworkMessage::JoinMatchmakingQueueMessage(JoinMatchmakingQueueMessage {
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
                NetworkMessage::ActivateAbilityMessage(msg) => {
                    ability_queue.0.push((*client_id, msg))
                },
                NetworkMessage::ProtocolErrorMessage(ProtocolErrorMessage { msg }) => {
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
    mut effects: EventWriter<EffectEvent>,
    mut start_turn: EventWriter<NewTurnEvent>,
    mut match_map: ResMut<MatchClientMap>,
    mut clients: ResMut<ConnectedClients>,
) {
    debug!("{} players in queue", mm_queue.0.len());

    if mm_queue.0.len() < 2 {
        return;
    }

    let match_id = MatchId::new();

    let mut i = mm_queue.0.drain();
    let mut players = i
        .by_ref()
        .take(2)
        .map(|(client_id, info)| {
            let pid = PlayerId::new();
            clients.0.insert(client_id, Some(pid));
            match_map.0.entry(match_id).or_insert(vec![]).push(client_id);
            (pid, Some(info.deck))
        })
        .collect::<Vec<_>>();
    *mm_queue = MMQueue(i.collect());

    for (pid, card) in &mut players {
        effects.send(EffectEvent {
            match_id,
            effect: Effect::SummonCard { card: card.take().unwrap() },
            targets: vec![GridLocation { owner: *pid, coord: UVec2::new(0, 2) }],
        });
    }

    let players = players.into_iter().map(|(pid, _)| pid).collect::<Vec<_>>();
    let p1 = players[0]; //todo random
    start_match.send(StartMatchEvent { match_id, players });
    start_turn.send(NewTurnEvent { match_id, next_player: p1 });
}

fn send_match_start(
    mut start_match: EventReader<StartMatchEvent>,
    mut server: ResMut<RenetServer>,
    client_map: Res<MatchClientMap>,
    player_map: Res<ConnectedClients>,
) {
    for StartMatchEvent { match_id, players } in start_match.read() {
        for client_id in client_map.0.get(match_id).unwrap() {
            server.send(client_id, MatchStartedMessage {
                match_id: *match_id,
                players: players.clone(),
                you: player_map.0.get(client_id).unwrap().unwrap(),
            })
        }
    }
}

fn send_effects(
    mut effects: EventReader<EffectEvent>,
    mut server: ResMut<RenetServer>,
    client_map: Res<MatchClientMap>,
) {
    for EffectEvent { match_id, effect, targets } in effects.read() {
        for client_id in client_map.0.get(match_id).unwrap() {
            server.send(client_id, EffectMessage {
                match_id: *match_id,
                effect: effect.clone(),
                targets: targets.clone(),
            });
        }
    }
}

fn send_turn_change(
    mut turns: EventReader<NewTurnEvent>,
    mut server: ResMut<RenetServer>,
    client_map: Res<MatchClientMap>,
) {
    for NewTurnEvent { match_id, next_player } in turns.read() {
        for client_id in client_map.0.get(match_id).unwrap() {
            server
                .send(client_id, NewTurnMessage { match_id: *match_id, next_player: *next_player });
        }
    }
}

fn process_abilities(
    mut ability_queue: ResMut<AbilityQueue>,
    mut effects: EventWriter<EffectEvent>,
    mut turns: EventWriter<NewTurnEvent>,
    cards: Cards,
    cur_turns: Query<Has<CurrentTurn>>,
    mut player_idx: Index<PlayerId>,
    mut owner_idx: Index<OwnerIndex>,
    mut loc_idx: Index<GridLocation>,
    clients: Res<ConnectedClients>,
    client_map: Res<MatchClientMap>,
    mut server: ResMut<RenetServer>,
) {
    for (client_id, activation) in ability_queue.0.drain(..) {
        let Some(Some(pid)) = clients.0.get(&client_id) else {
            server.send_error(&client_id, "Not in a match.");
            continue;
        };

        if !cur_turns.get(player_idx.lookup_single(&pid)).unwrap() {
            server.send_error(&client_id, "Not your turn.");
            continue;
        }

        let Ok(card) = cards.get(
            loc_idx.lookup_single(&GridLocation { owner: *pid, coord: activation.unit_location }),
        ) else {
            server.send_error(&client_id, "No unit there.");
            continue;
        };

        let Some(ability) = card.abilities.0.get(activation.ability_idx) else {
            server.send_error(&client_id, "No such ability.");
            continue;
        };

        let Ability::Activated { effect, cost, target_rules } = ability else {
            server.send_error(&client_id, "Ability is passive.");
            continue;
        };

        let energy_cost = cost.get(&effect).energy;
        if energy_cost > card.energy.current {
            server.send_error(&client_id, "Not enough energy.")
        }

        // todo validate targets

        effects.send(EffectEvent {
            match_id: activation.match_id,
            effect: Effect::ChangeEnergy { amount: -(energy_cost as i32) },
            targets: vec![*card.grid_loc],
        });
        effects.send(EffectEvent {
            match_id: activation.match_id,
            effect: effect.clone(),
            targets: activation.targets,
        });

        let next_player = clients
            .0
            .get(
                client_map
                    .0
                    .get(&activation.match_id)
                    .unwrap()
                    .iter()
                    .filter(|c| **c != client_id)
                    .next()
                    .unwrap(),
            )
            .unwrap()
            .unwrap();
        turns.send(NewTurnEvent { match_id: activation.match_id, next_player });
        effects.send(EffectEvent {
            match_id: activation.match_id,
            effect: Effect::ChangeEnergy { amount: 1 },
            targets: cards
                .iter_many(owner_idx.lookup(&next_player))
                .map(|card| *card.grid_loc)
                .collect(),
        });
    }
}
