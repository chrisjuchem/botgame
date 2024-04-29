pub mod events;
pub mod server;

use bevy::{ecs::query::QueryData, prelude::*};
use bevy_mod_index::prelude::*;
use extension_trait::extension_trait;
use serde::{Deserialize, Serialize};

use crate::{
    cards_v2::{deck::Decklist, mesh::NeedsMesh, Card},
    match_sim::{
        events::{
            add_cards_to_deck, cleanup_match, draw_cards, init_events, next_turn, start_match,
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

        app.register_type::<Hand>();
        app.register_type::<Deck>();

        let mut sim_systems = OrderedSystemList::new();
        sim_systems.push(start_match);
        if self.server {
            sim_systems.push(fill_decks_and_hands)
        }
        sim_systems.push(add_cards_to_deck);
        sim_systems.push(draw_cards);

        sim_systems.push(next_turn);
        sim_systems.push(cleanup_match);

        app.add_systems(Update, sim_systems);
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

#[derive(Component)]
pub struct Scrap(u32);

#[derive(Component)]
pub struct Minerals(u32);

#[derive(Component)]
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

// ====== Card Components ======

#[derive(Component)]
pub struct BaseCard(pub Card);

#[derive(Component, Debug)]
pub struct Health(pub i32);

#[derive(Component, Debug)]
pub struct CombatStrength(pub u32);

#[derive(Component, Debug)]
pub struct MiningSpeed(pub u32);

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
pub struct Abilities(pub Vec<crate::cards_v2::Ability>);

#[derive(QueryData, Debug)]
#[query_data(mutable, derive(Debug))]
pub struct CardQuery {
    pub entity: Entity,
    pub match_id: &'static MatchId,
    pub name: &'static Name,
    pub grid_loc: &'static GridLocation,
    pub abilities: &'static mut Abilities,
    pub health: &'static mut Health,
    pub attack_strength: Option<&'static mut CombatStrength>,
    pub mining_speed: Option<&'static mut CombatStrength>,
}
impl CardQuery {}

pub type Cards<'w, 's> = Query<'w, 's, CardQueryReadOnly>;
pub type CardsMut<'w, 's> = Query<'w, 's, CardQuery>;

// ====== Utils ======

#[extension_trait]
impl CommandExts for Commands<'_, '_> {
    fn spawn_card(&mut self, card: Card, mid: MatchId, loc: GridLocation) {
        let mut cmds = self.spawn((
            mid,
            Name::new(card.name.clone()),
            Health(card.hp as i32),
            Abilities(vec![card.ability.clone()]),
            (loc, SpatialBundle::default()),
            NeedsMesh,
        ));

        if let Some(str) = card.combat_strength {
            cmds.insert(CombatStrength(str));
        }

        if let Some(spd) = card.mining_speed {
            cmds.insert(MiningSpeed(spd));
        }

        cmds.insert(BaseCard(card));

        // pub struct Card {
        //     cost: Cost,
        //     chassis: Chassis,
        //     support_ability: Option<SupportAbility>,
        //     attribute: Attribute,
        // }
    }
}
