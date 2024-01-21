use aery::prelude::*;
use bevy::prelude::*;
use bevy_mod_index::prelude::*;
use extension_trait::extension_trait;
use serde::{Deserialize, Serialize};

use crate::{
    cards::{mesh::NeedsMesh, Card, Effect, Target},
    utils::Uuid,
};

pub struct MatchSimPlugin;
impl Plugin for MatchSimPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Aery);
        init_events(app);
        app.add_systems(Update, (start_match, apply_deferred, effects, next_turn).chain());
        app.add_systems(Update, cleanup_match);
    }
}

// ====== Match Components/Relations ======

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct MatchId(Uuid);
impl MatchId {
    pub fn new() -> Self {
        Self(Uuid::new())
    }
}

struct MatchIndex;
impl IndexInfo for MatchIndex {
    type Component = MatchId;
    type Value = MatchId;
    type Storage = HashmapStorage<MatchIndex>;

    fn value(c: &Self::Component) -> Self::Value {
        *c
    }
}

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PlayerId(Uuid);
impl PlayerId {
    pub fn new() -> Self {
        Self(Uuid::new())
    }
}

struct PlayerIndex;
impl IndexInfo for PlayerIndex {
    type Component = PlayerId;
    type Value = PlayerId;
    type Storage = HashmapStorage<PlayerIndex>;

    fn value(c: &Self::Component) -> Self::Value {
        *c
    }
}

#[derive(Component)]
pub struct CurrentTurn;

#[derive(Resource)]
pub struct Us(pub PlayerId);

// ====== Card Components/Relations ======

#[derive(Relation)]
pub struct OwnedBy;

#[derive(Component)]
pub struct BaseCard(pub Card);

#[derive(Component)]
pub struct Health(pub u32);

#[derive(Component)]
pub struct Energy {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct GridLocation(pub UVec2);

// ====== Events ======

#[derive(Event, Clone)]
pub struct StartMatchEvent {
    pub match_id: MatchId,
    pub players: Vec<PlayerId>,
}

#[derive(Event, Clone)]
pub struct EffectEvent {
    pub match_id: MatchId,
    pub effect: Effect,
    pub targets: Vec<Target>,
}

#[derive(Event, Clone)]
pub struct NewTurnEvent {
    pub match_id: MatchId,
    pub next_player: PlayerId,
}

#[derive(Event, Clone)]
pub struct CleanupMatchEvent {
    match_id: MatchId,
}

fn init_events(app: &mut App) {
    app.add_event::<StartMatchEvent>();
    app.add_event::<EffectEvent>();
    app.add_event::<NewTurnEvent>();
    app.add_event::<CleanupMatchEvent>();
}

// ====== Systems ======

fn start_match(mut commands: Commands, mut e: EventReader<StartMatchEvent>) {
    for StartMatchEvent { match_id, players } in e.read() {
        info!("match {match_id:?} started");
        for player_id in players.iter() {
            let p = commands.spawn((*match_id, *player_id)).id();
        }
    }
}

fn next_turn(
    mut commands: Commands,
    mut e: EventReader<NewTurnEvent>,
    players: Query<(Entity, &MatchId, &PlayerId, Has<CurrentTurn>)>,
) {
    for NewTurnEvent { match_id, next_player } in e.read() {
        for (e, m, p, t) in players.iter() {
            if m == match_id {
                if t {
                    commands.entity(e).remove::<CurrentTurn>();
                } else if p == next_player {
                    commands.entity(e).insert(CurrentTurn);
                }
            }
        }
    }
}

fn effects(
    mut commands: Commands,
    mut e: EventReader<EffectEvent>,
    mut player_index: Index<PlayerIndex>,
) {
    for EffectEvent { match_id, effect, targets } in e.read() {
        debug!("effect {effect:?} with targets {targets:?}");
        match effect {
            Effect::SummonCard { card } => {
                for t in targets {
                    // todo: put id struct in target
                    let mut es = player_index.lookup(&t.player);
                    // todo: lookup_single
                    assert_eq!(es.len(), 1);
                    let e = es.drain().next().expect("should only be on player with id");
                    commands.spawn_card(card.clone(), *match_id, e, t.location)
                }
            },
            _ => unimplemented!(),
            // Effect::Attack { .. } => {}
            // Effect::GrantAbility { .. } => {}
            // Effect::MultipleEffects { .. } => {}
        }
    }
}

fn cleanup_match(
    mut commands: Commands,
    mut e: EventReader<CleanupMatchEvent>,
    mut match_index: Index<MatchIndex>,
) {
    for CleanupMatchEvent { match_id } in e.read() {
        for entity in match_index.lookup(match_id) {
            commands.entity(entity).despawn();
        }
    }
}

// ====== Utils ======

#[extension_trait]
impl CommandExts for Commands<'_, '_> {
    fn spawn_card(&mut self, card: Card, mid: MatchId, owner: Entity, loc: UVec2) {
        let card = self
            .spawn((
                mid,
                Health(card.hp),
                Energy { current: 1, max: card.max_energy },
                BaseCard(card),
                (GridLocation(loc), SpatialBundle::default()),
                NeedsMesh,
            ))
            .set::<OwnedBy>(owner);
    }
}
