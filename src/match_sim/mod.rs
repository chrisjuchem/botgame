pub mod events;
pub mod server;

use bevy::{
    ecs::{event::ManualEventReader, query::QueryData, system::BoxedSystem},
    prelude::*,
};
use bevy_mod_index::prelude::*;
use bevy_mod_picking::pointer::PressDirection::Up;
use extension_trait::extension_trait;
use serde::{Deserialize, Serialize};

use crate::{
    cards::{
        deck::Decklist, mesh::NeedsMesh, Ability, Card, Effect, ImplicitTargetRules,
        PassiveAbility, PassiveEffect, Robot,
    },
    match_sim::{
        events::{
            add_cards_to_deck, cleanup_match, client_effects, common_effects, draw_cards,
            init_events, next_turn, server_effects, server_state_based, start_match,
        },
        server::fill_decks_and_hands,
    },
    utils::{OrderedSystemList, Uuid},
};

pub struct MatchSimPlugin {
    pub(crate) server: bool,
}
impl Plugin for MatchSimPlugin {
    fn build(&self, app: &mut App) {
        init_events(app);

        app.register_type::<MatchId>()
            .register_type::<PlayerId>()
            .register_type::<Uuid>()
            .register_type::<Attack>()
            .register_type::<Health>()
            .register_type::<GridLocation>()
            .register_type::<Abilities>()
            .register_type::<Hand>()
            .register_type::<Deck>()
            .register_type::<Decklist>()
            .register_type::<Energy>();

        let mut sim_systems = OrderedSystemList::new();
        sim_systems.push(start_match);
        if self.server {
            sim_systems.push(server_effects);
        } else {
            sim_systems.push(client_effects);
        }
        sim_systems.push(common_effects);
        if self.server {
            sim_systems.push(server_state_based);
            sim_systems.push(fill_decks_and_hands);
        }
        sim_systems.push(add_cards_to_deck);
        sim_systems.push(draw_cards);
        sim_systems.push(next_turn);
        sim_systems.push(cleanup_match);

        app.add_systems(Update, sim_systems);
    }
}

// ====== Match Components ======

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Reflect)]
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

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Reflect)]
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

#[derive(Component, Reflect)]
pub struct CurrentTurn;

#[derive(Component, Reflect)]
pub struct Energy {
    max: u32,
    available: u32,
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize, Reflect)]
pub enum UnplayedCard {
    Known(Card),
    Unknown,
}

#[derive(Component, Reflect)]
pub struct Hand(Vec<UnplayedCard>);

#[derive(Component, Reflect)]
pub struct Deck(Vec<UnplayedCard>);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Us(pub PlayerId);
impl Default for Us {
    fn default() -> Self {
        Us(PlayerId(Uuid::new_blank()))
    }
}

// ====== Card Components ======

#[derive(Component)]
pub struct BaseCard(pub Robot);

#[derive(Component, Debug, Reflect)]
pub struct Health(pub i32);

#[derive(Component, Debug, Reflect)]
pub struct Attack(pub i32);

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Reflect)]
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

#[derive(Component, Clone, Debug, Reflect)]
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
    pub attack: &'static mut Attack,
}
impl CardQuery {}

pub type Cards<'w, 's> = Query<'w, 's, CardQueryReadOnly>;
pub type CardsMut<'w, 's> = Query<'w, 's, CardQuery>;

// ====== Utils ======

#[extension_trait]
impl CommandExts for Commands<'_, '_> {
    fn spawn_robot(&mut self, robot: Robot, mid: MatchId, loc: GridLocation) {
        let card = self.spawn((
            mid,
            Name::new("Robot"),
            Health(robot.hp as i32),
            Attack(robot.attack as i32),
            Abilities(robot.abilities.clone()),
            BaseCard(robot),
            (loc, SpatialBundle::default()),
            NeedsMesh,
        ));
    }
}
