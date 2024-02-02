pub mod mesh;
pub mod text;

use std::fmt::{Debug, Formatter};

use bevy_mod_index::index::Index;
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::match_sim::{CardQueryReadOnlyItem, Cards, GridLocation, PlayerId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    pub summon_cost: Cost,
    pub hp: u32,
    pub abilities: Vec<Ability>, // name + abilityData ??
    pub max_energy: u32,
    pub energy_regen: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Ability {
    Activated { effect: Effect, cost: AbilityCost, target_rules: TargetRules },
    Passive { passive_effect: PassiveEffect, target_filter: TargetFilter },
}
impl Ability {
    fn cost(&self) -> Option<Cost> {
        match self {
            Ability::Activated { effect, cost, .. } => Some(cost.get(effect)),
            Ability::Passive { .. } => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Cost {
    pub(crate) energy: u32,
}
impl Cost {
    const FREE: Cost = Cost { energy: 0 };
}

pub trait DerivedCostFunc: Sync + Fn(&Effect) -> Cost {
    fn fn_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
impl dyn DerivedCostFunc {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.fn_name())
    }
    fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<&'static Self, D::Error> {
        struct DerivedCostFuncVisitor;
        impl Visitor<'_> for DerivedCostFuncVisitor {
            type Value = &'static dyn DerivedCostFunc;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("the name of a function to use to derive costs")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(&summon_cost) // TODO
            }
        }

        de.deserialize_str(DerivedCostFuncVisitor)
    }
}
impl<T> DerivedCostFunc for T where T: Sync + Fn(&Effect) -> Cost {}

#[derive(Clone, Serialize, Deserialize)]
pub enum AbilityCost {
    Static {
        cost: Cost,
    },
    Derived {
        #[serde(with = "DerivedCostFunc")]
        func: &'static dyn DerivedCostFunc,
    },
}
impl AbilityCost {
    fn get(&self, effect: &Effect) -> Cost {
        match self {
            AbilityCost::Static { cost } => *cost,
            AbilityCost::Derived { func } => func(effect),
        }
    }
}
impl Debug for AbilityCost {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AbilityCost::Static { cost } => f.debug_struct("Static").field("cost", cost).finish(),
            AbilityCost::Derived { func } => f
                .debug_struct("Derived")
                .field_with("func", |f| f.write_str((*func).fn_name()))
                .finish(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    Attack { damage: u32, effect_type: EffectType },
    GrantAbility { ability: Box<Ability> },
    SummonCard { card: Card },
    // SharableEnergy {
    //     factor: f32,
    // },
    // Cloaking {
    //
    // },
    MultipleEffects { effects: Vec<Effect> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PassiveEffect {
    DamageResistance { effect_type: EffectType, factor: f32 },
    WhenHit { effect: Effect, target_rules: TargetRules },
    // ModifySummonCost
    // ModifyAbilityCost ??
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum EffectType {
    Physical,
    Explosion,
    Fire,
    Electrical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetRules {
    amount: TargetAmount,
    filter: TargetFilter,
    // chosen, random, or auto
}
impl TargetRules {
    pub(crate) fn validate(
        &self,
        targets: &[GridLocation],
        loc_idx: &mut Index<GridLocation>,
        cards: &Cards,
        effect_owner: PlayerId,
    ) -> bool {
        if !self.amount.validate(targets.len()) {
            return false;
        }

        targets.iter().all(|loc| {
            let card = loc_idx.lookup(loc).next().and_then(|e| cards.get(e).ok());
            self.filter.validate(card.as_ref(), loc.owner, effect_owner)
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetAmount {
    All,
    N { n: usize },
    UpToN { n: usize },
}
impl TargetAmount {
    pub fn validate(&self, x: usize) -> bool {
        match self {
            TargetAmount::N { n } => *n == x,
            TargetAmount::UpToN { n } => x <= *n,
            TargetAmount::All => panic!("TODO: implement 'all' targeting"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetFilter {
    Any,
    Friendly,
    Enemy,
    Unoccupied,
    Occupied,
    And(Vec<TargetFilter>),
    Or(Vec<TargetFilter>),
}
impl TargetFilter {
    pub fn validate(
        &self,
        card: Option<&CardQueryReadOnlyItem>,
        target_owner: PlayerId,
        effect_owner: PlayerId,
    ) -> bool {
        match self {
            TargetFilter::Any => true,
            TargetFilter::Friendly => target_owner == effect_owner,
            TargetFilter::Enemy => target_owner != effect_owner,
            TargetFilter::Unoccupied => card.is_none(),
            TargetFilter::Occupied => card.is_some(),
            TargetFilter::And(conds) => {
                conds.iter().all(|c| c.validate(card, target_owner, effect_owner))
            },
            TargetFilter::Or(conds) => {
                conds.iter().any(|c| c.validate(card, target_owner, effect_owner))
            },
        }
    }
}

pub fn deck() -> Card {
    let deck = vec![
        Card {
            name: "bot 1".to_string(),
            summon_cost: Cost { energy: 3 },
            hp: 8,
            abilities: vec![Ability::Activated {
                effect: Effect::Attack { damage: 3, effect_type: EffectType::Fire },
                cost: AbilityCost::Static { cost: Cost { energy: 2 } },
                target_rules: TargetRules {
                    amount: TargetAmount::N { n: 1 },
                    filter: TargetFilter::And(vec![TargetFilter::Enemy, TargetFilter::Occupied]),
                },
            }],
            max_energy: 3,
            energy_regen: 1,
        },
        Card {
            name: "bot 2".to_string(),
            summon_cost: Cost { energy: 5 },
            hp: 20,
            abilities: vec![Ability::Activated {
                effect: Effect::Attack { damage: 1, effect_type: EffectType::Physical },
                cost: AbilityCost::Static { cost: Cost::FREE },
                target_rules: TargetRules {
                    amount: TargetAmount::N { n: 1 },
                    filter: TargetFilter::And(vec![TargetFilter::Enemy, TargetFilter::Occupied]),
                },
            }],
            max_energy: 0,
            energy_regen: 0,
        },
        Card {
            name: "bot 3".to_string(),
            summon_cost: Cost { energy: 3 },
            hp: 8,
            abilities: vec![Ability::Activated {
                effect: Effect::Attack { damage: 3, effect_type: EffectType::Fire },
                cost: AbilityCost::Static { cost: Cost { energy: 2 } },
                target_rules: TargetRules {
                    amount: TargetAmount::N { n: 1 },
                    filter: TargetFilter::And(vec![TargetFilter::Enemy, TargetFilter::Occupied]),
                },
            }],
            max_energy: 3,
            energy_regen: 1,
        },
        Card {
            name: "bot 4".to_string(),
            summon_cost: Cost { energy: 5 },
            hp: 20,
            abilities: vec![Ability::Activated {
                effect: Effect::Attack { damage: 1, effect_type: EffectType::Physical },
                cost: AbilityCost::Static { cost: Cost::FREE },
                target_rules: TargetRules {
                    amount: TargetAmount::N { n: 1 },
                    filter: TargetFilter::And(vec![TargetFilter::Enemy, TargetFilter::Occupied]),
                },
            }],
            max_energy: 0,
            energy_regen: 0,
        },
        Card {
            name: "bot 5".to_string(),
            summon_cost: Cost { energy: 5 },
            hp: 20,
            abilities: vec![Ability::Activated {
                effect: Effect::Attack { damage: 1, effect_type: EffectType::Physical },
                cost: AbilityCost::Static { cost: Cost::FREE },
                target_rules: TargetRules {
                    amount: TargetAmount::N { n: 1 },
                    filter: TargetFilter::And(vec![TargetFilter::Enemy, TargetFilter::Occupied]),
                },
            }],
            max_energy: 0,
            energy_regen: 0,
        },
    ];

    Card {
        name: "Command Center".to_string(),
        summon_cost: Cost::FREE,
        hp: 50,
        abilities: deck
            .into_iter()
            .map(|card| Ability::Activated {
                effect: Effect::SummonCard { card },
                cost: AbilityCost::Derived { func: &summon_cost },
                target_rules: TargetRules {
                    amount: TargetAmount::N { n: 1 },
                    filter: TargetFilter::And(vec![
                        TargetFilter::Unoccupied,
                        TargetFilter::Friendly,
                    ]),
                },
            })
            .collect(),
        max_energy: 10,
        energy_regen: 1,
    }
}

fn summon_cost(effect: &Effect) -> Cost {
    let Effect::SummonCard { card } = effect else {
        panic!("Can't get summon cost of {effect:?}");
    };
    card.summon_cost
}
