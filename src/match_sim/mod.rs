use bevy::{
    ecs::{event::ManualEventReader, query::QueryData, system::BoxedSystem},
    prelude::*,
};
use bevy_mod_index::prelude::*;
use extension_trait::extension_trait;
use serde::{Deserialize, Serialize};

use crate::{
    cards_v1::{mesh::NeedsMesh, Ability, Card, Effect, ImplicitTargetRules, PassiveEffect},
    utils::Uuid,
};

pub struct MatchSimPlugin {
    pub(crate) server: bool,
}
impl Plugin for MatchSimPlugin {
    fn build(&self, app: &mut App) {
        init_events(app);
        let (specialized_effects, state_based_effects): (BoxedSystem, BoxedSystem) = if self.server
        {
            (
                Box::new(IntoSystem::into_system(server_effects)),
                Box::new(IntoSystem::into_system(server_state_based)),
            )
        } else {
            (
                Box::new(IntoSystem::into_system(client_effects)),
                Box::new(IntoSystem::into_system(|| {})),
            )
        };

        // TODO: use manual stack for effects instead of reading event queue

        app.add_systems(
            Update,
            (
                start_match,
                // apply_deferred,
                specialized_effects,
                common_effects,
                // apply_deferred,
                state_based_effects,
                next_turn,
            )
                .chain(),
        );
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
    type Component = MatchId;
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
    type Component = PlayerId;
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
pub struct Health(pub i32);

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
    type Component = GridLocation;
    type Value = GridLocation;
    type Storage = HashmapStorage<Self>;
    type RefreshPolicy = ConservativeRefreshPolicy;

    fn value(g: &GridLocation) -> Self::Value {
        *g
    }
}
pub struct OwnerIndex;
impl IndexInfo for OwnerIndex {
    type Component = GridLocation;
    type Value = PlayerId;
    type Storage = HashmapStorage<Self>;
    type RefreshPolicy = ConservativeRefreshPolicy;

    fn value(g: &GridLocation) -> Self::Value {
        g.owner
    }
}

#[derive(Component, Clone, Debug)]
pub struct Abilities(pub Vec<Ability>);

#[derive(QueryData, Debug)]
#[query_data(mutable, derive(Debug))]
pub struct CardQuery {
    pub entity: Entity,
    pub match_id: &'static MatchId,
    pub name: &'static Name,
    pub grid_loc: &'static GridLocation,
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

fn client_effects(mut e: EventReader<EffectEvent>) {
    for EffectEvent { match_id, effect, targets } in e.read() {}
}

fn common_effects(
    mut commands: Commands,
    mut e: EventReader<EffectEvent>,
    mut cards: CardsMut,
    mut loc_idx: Index<GridLocation>,
) {
    for EffectEvent { match_id, effect, targets } in e.read() {
        debug!("effect {effect:?} with targets {targets:?}");
        match effect {
            Effect::SummonCard { card } => {
                for t in targets {
                    commands.spawn_card(card.clone(), *match_id, *t)
                }
            },
            Effect::GrantAbilities { abilities } => {
                for t in targets {
                    cards
                        .get_mut(loc_idx.single(t))
                        .unwrap()
                        .abilities
                        .0
                        .extend_from_slice(abilities);
                }
            },
            Effect::ChangeHp { amount } => {
                for t in targets {
                    cards.get_mut(loc_idx.single(t)).unwrap().health.0 += amount;
                }
            },
            Effect::ChangeEnergy { amount } => {
                for t in targets {
                    let mut e = cards.get_mut(loc_idx.single(t)).unwrap().energy;
                    e.current = (e.current as i32 + amount).clamp(0, e.max as i32) as u32;
                }
            },
            Effect::DestroyCard => {
                // triggering abilities handled by server effects
                for t in targets {
                    println!("despawn {t:?}");
                    commands.entity(loc_idx.single(t)).despawn_recursive();
                }
            },
            Effect::Attack { .. } | Effect::MultipleEffects { .. } => {
                // Handled by server_effects
            },
        }
    }
}

// todo read from a stack instead of using event queue
fn server_effects(
    mut e: ResMut<Events<EffectEvent>>,
    mut e_reader: Local<ManualEventReader<EffectEvent>>,
    cards: Cards,
    mut loc_idx: Index<GridLocation>,
    mut match_idx: Index<MatchId>,
) {
    let mut new_events = vec![];
    for EffectEvent { match_id, effect, targets } in e_reader.read(&*e) {
        match effect {
            Effect::Attack { effect_type, damage } => {
                for t in targets {
                    let mut final_factor = 1.;
                    for (ability, ability_source_loc) in cards
                        .iter_many(match_idx.lookup(match_id))
                        .flat_map(|card| card.abilities.0.iter().map(|a| (a, *card.grid_loc)))
                    {
                        if let Ability::Passive { passive_effect, target_filter } = ability {
                            if target_filter.validate(t, &mut loc_idx, &cards, &ability_source_loc)
                            {
                                match passive_effect {
                                    PassiveEffect::DamageResistance {
                                        effect_type: effect_filter,
                                        factor,
                                    } => {
                                        if *effect_type == *effect_filter {
                                            final_factor *= factor;
                                        }
                                    },
                                    PassiveEffect::WhenHit { effect, target_rules } => {
                                        let target = match target_rules {
                                            ImplicitTargetRules::ThisUnit => ability_source_loc,
                                            ImplicitTargetRules::ThatUnit => *t,
                                        };

                                        new_events.push(EffectEvent {
                                            match_id: *match_id,
                                            effect: effect.clone(),
                                            targets: vec![target],
                                        })
                                    },
                                    PassiveEffect::WhenDies { .. } => {},
                                }
                            }
                        }
                    }

                    let final_dmg = *damage as f32 * final_factor;

                    new_events.push(EffectEvent {
                        match_id: *match_id,
                        effect: Effect::ChangeHp { amount: -(final_dmg as i32) },
                        targets: vec![*t],
                    });
                }
            },
            Effect::MultipleEffects { effects } => {
                for e in effects {
                    new_events.push(EffectEvent {
                        match_id: *match_id,
                        effect: e.clone(),
                        targets: targets.clone(),
                    })
                }
            },
            Effect::DestroyCard => {
                // despawning handled by common effects
                for t in targets {
                    for (ability, ability_source_loc) in cards
                        .iter_many(match_idx.lookup(match_id))
                        .flat_map(|card| card.abilities.0.iter().map(|a| (a, *card.grid_loc)))
                    {
                        if let Ability::Passive { passive_effect, target_filter } = ability {
                            if target_filter.validate(t, &mut loc_idx, &cards, &ability_source_loc)
                            {
                                if let PassiveEffect::WhenDies { effect, target_rules } =
                                    passive_effect
                                {
                                    let target = match target_rules {
                                        ImplicitTargetRules::ThisUnit => ability_source_loc,
                                        ImplicitTargetRules::ThatUnit => *t,
                                    };

                                    new_events.push(EffectEvent {
                                        match_id: *match_id,
                                        effect: effect.clone(),
                                        targets: vec![target],
                                    })
                                }
                            }
                        }
                    }
                }
            },
            Effect::SummonCard { .. }
            | Effect::GrantAbilities { .. }
            | Effect::ChangeHp { .. }
            | Effect::ChangeEnergy { .. } => {
                // Handled by common_effects
            },
        }
    }
    for ev in new_events.into_iter() {
        e.send(ev);
    }
}

fn server_state_based(mut e: EventWriter<EffectEvent>, cards: Cards) {
    for card in &cards {
        if card.health.0 <= 0 {
            e.send(EffectEvent {
                match_id: *card.match_id,
                effect: Effect::DestroyCard,
                targets: vec![*card.grid_loc],
            });
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
            Health(card.hp as i32),
            Energy { current: card.starting_energy, max: card.max_energy },
            Abilities(card.abilities.clone()),
            BaseCard(card),
            (loc, SpatialBundle::default()),
            NeedsMesh,
        ));
    }
}
