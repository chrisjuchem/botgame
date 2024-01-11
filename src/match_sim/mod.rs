use aery::prelude::*;
use bevy::{
    prelude::*,
    utils::Uuid,
};
use bevy_mod_index::prelude::*;
use extension_trait::extension_trait;

use crate::cards::{
    Card,
    Effect,
    Target,
};

pub struct MatchSimPlugin;
impl Plugin for MatchSimPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, start_match);
        app.add_systems(Update, effects);
        app.add_systems(Update, next_turn);
        app.add_systems(Update, cleanup_match);
    }
}

// ====== Match Components/Relations ======

#[derive(Component, Copy, Clone, Eq, PartialEq, Hash)]
pub struct MatchId(Uuid);
impl MatchId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
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

#[derive(Component, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PlayerId(Uuid);
impl PlayerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
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
struct CurrentTurn;

// ====== Card Components/Relations ======

#[derive(Relation)]
struct OwnedBy;

#[derive(Component)]
struct BaseCard(Card);

#[derive(Component)]
struct Health(u32);

#[derive(Component)]
struct Energy {
    current: u32,
    max: u32,
}

// ====== Events ======

#[derive(Event)]
pub struct StartMatchEvent {
    pub match_id: MatchId,
    pub players: Vec<(PlayerId, Card)>,
}

#[derive(Event)]
pub struct EffectEvent {
    pub match_id: MatchId,
    pub effect: Effect,
    pub target: Target,
}

#[derive(Event)]
pub struct NewTurnEvent {
    pub match_id: MatchId,
    pub next_player: PlayerId,
}

#[derive(Event)]
pub struct CleanupMatchEvent {
    match_id: MatchId,
}

// ====== Systems ======

fn start_match(
    mut commands: Commands,
    mut e: EventReader<StartMatchEvent>,
    mut effects: EventWriter<EffectEvent>,
    mut turns: EventWriter<NewTurnEvent>,
) {
    for StartMatchEvent { match_id, players } in e.read() {
        for (player_id, card) in players.iter() {
            let p = commands.spawn((*match_id, *player_id)).id();
            commands.spawn_card(card.clone(), *match_id, p);
            effects.
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
    for EffectEvent { match_id, effect, target } in e.read() {
        match effect {
            Effect::SummonCard { card } => {
                let Target::Players(players) = target else {
                    panic!("only players can spawn cards")
                };

                for p in players {
                    // todo: put id struct in target
                    let mut es = player_index.lookup(&PlayerId(*p));
                    // todo: lookup_single
                    assert_eq!(es.len(), 1);
                    let e = es.drain().next().expect("should only be on player with id");
                    commands.spawn_card(card.clone(), *match_id, e)
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
    fn spawn_card(&mut self, card: Card, mid: MatchId, owner: Entity) {
        self.spawn((
            mid,
            Health(card.hp),
            Energy { current: 1, max: card.max_energy },
            BaseCard(card),
        ))
        .set::<OwnedBy>(owner);
    }
}
