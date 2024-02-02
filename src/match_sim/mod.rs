use bevy::{ecs::query::WorldQuery, prelude::*};
use bevy_mod_index::prelude::*;
use extension_trait::extension_trait;
use serde::{Deserialize, Serialize};

use crate::{
    cards::{mesh::NeedsMesh, Ability, Card, Effect, PassiveEffect},
    utils::Uuid,
};

pub struct MatchSimPlugin;
impl Plugin for MatchSimPlugin {
    fn build(&self, app: &mut App) {
        init_events(app);
        app.add_systems(Update, (start_match, apply_deferred, effects, next_turn).chain());
        app.add_systems(Update, cleanup_match);
    }
}

// ====== Match Components ======

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct MatchId(Uuid);
impl MatchId {
    pub fn new() -> Self {
        Self(Uuid::new())
    }
}
impl IndexInfo for MatchId {
    type Components = &'static MatchId;
    type Value = MatchId;
    type Storage = HashmapStorage<Self>;
    type RefreshPolicy = SimpleRefreshPolicy;

    fn value(c: &MatchId) -> Self::Value {
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
impl IndexInfo for PlayerId {
    type Components = &'static PlayerId;
    type Value = PlayerId;
    type Storage = HashmapStorage<Self>;
    type RefreshPolicy = SimpleRefreshPolicy;

    fn value(c: &PlayerId) -> Self::Value {
        *c
    }
}

#[derive(Component)]
pub struct CurrentTurn;

#[derive(Resource)]
pub struct Us(pub PlayerId);

// ====== Card Components ======

#[derive(Component)]
pub struct BaseCard(pub Card);

#[derive(Component, Debug)]
pub struct Health(pub u32);

#[derive(Component, Debug)]
pub struct Energy {
    pub current: u32,
    pub max: u32,
}

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct GridLocation {
    pub coord: UVec2,
    pub owner: PlayerId,
}
impl IndexInfo for GridLocation {
    type Components = &'static GridLocation;
    type Value = GridLocation;
    type Storage = HashmapStorage<Self>;
    type RefreshPolicy = ConservativeRefreshPolicy;

    fn value(g: &GridLocation) -> Self::Value {
        *g
    }
}

#[derive(Component, Clone, Debug)]
pub struct Abilities(pub Vec<Ability>);

#[derive(WorldQuery, Debug)]
#[world_query(mutable, derive(Debug))]
pub struct CardQuery {
    pub entity: Entity,
    pub name: &'static Name,
    pub grid_loc: &'static mut GridLocation,
    pub abilities: &'static mut Abilities,
    pub health: &'static mut Health,
    pub energy: &'static mut Energy,
}
impl CardQuery {}

pub type Cards<'w, 's> = Query<'w, 's, CardQueryReadOnly>;
pub type CardsMut<'w, 's> = Query<'w, 's, CardQuery>;

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
    pub targets: Vec<GridLocation>,
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
            let p = commands.spawn((*match_id, *player_id, Name::new("player_id_marker"))).id();
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

    mut cards_and_loc_idx_and_match_idx: ParamSet<(CardsMut, Index<GridLocation>, Index<MatchId>)>,
    us: Option<Res<Us>>,
) {
    for EffectEvent { match_id, effect, targets } in e.read() {
        debug!("effect {effect:?} with targets {targets:?}");
        match effect {
            Effect::SummonCard { card } => {
                for t in targets {
                    commands.spawn_card(card.clone(), *match_id, *t)
                }
            },
            Effect::Attack { effect_type, damage } => {
                for t in targets {
                    let target_e = cards_and_loc_idx_and_match_idx.p1().lookup_single(t);
                    let mut final_factor = 1.;

                    let match_entities =
                        cards_and_loc_idx_and_match_idx.p2().lookup(match_id).collect::<Vec<_>>();
                    let cards = cards_and_loc_idx_and_match_idx.p0();
                    for (a, a_owner) in cards
                        .iter_many(match_entities)
                        .flat_map(|card| card.abilities.0.iter().map(|a| (a, card.grid_loc.owner)))
                    {
                        if let Ability::Passive {
                            passive_effect:
                                PassiveEffect::DamageResistance { effect_type: effect_filter, factor },
                            target_filter,
                        } = a
                        {
                            if *effect_type == *effect_filter
                                && target_filter.validate(
                                    cards.get(target_e).ok().as_ref(),
                                    t.owner,
                                    a_owner,
                                )
                            {
                                final_factor *= factor;
                            }
                        }
                    }

                    let final_dmg = *damage as f32 * final_factor;
                    cards_and_loc_idx_and_match_idx.p0().get_mut(target_e).unwrap().health.0 -=
                        final_dmg as u32;
                }
            },
            _ => unimplemented!(),
            // Effect::GrantAbility { .. } => {}
            // Effect::MultipleEffects { .. } => {}
        }
    }
}

fn cleanup_match(
    mut commands: Commands,
    mut e: EventReader<CleanupMatchEvent>,
    mut match_index: Index<MatchId>,
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
    fn spawn_card(&mut self, card: Card, mid: MatchId, loc: GridLocation) {
        let card = self.spawn((
            mid,
            Name::new(card.name.to_string()),
            Health(card.hp),
            Energy { current: 1, max: card.max_energy },
            Abilities(card.abilities.clone()),
            BaseCard(card),
            (loc, SpatialBundle::default()),
            NeedsMesh,
        ));
    }
}
