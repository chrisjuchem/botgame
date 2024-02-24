pub mod deck;
pub mod generator;
pub mod mesh;
pub mod price;
pub mod text;

use std::fmt::Debug;

use bevy::{math::UVec2, reflect::Reflect};
use bevy_mod_index::index::Index;
use serde::{Deserialize, Serialize};

use crate::{
    match_sim::{Cards, GridLocation, PlayerId},
    ui::game_scene::{GRID_H, GRID_W},
};

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Card {
    pub name: String,
    pub summon_cost: Cost,
    pub hp: u32,
    pub abilities: Vec<Ability>, // name + abilityData ??
    pub starting_energy: u32,
    pub max_energy: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(no_field_bounds)]
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct Cost {
    pub energy: u32,
}
impl Cost {
    pub const FREE: Cost = Cost { energy: 0 };
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Reflect)]
pub enum Attribute {
    Hp,
    SummonCost,
}
impl Attribute {
    pub fn get(&self, card: &Card) -> u32 {
        match self {
            Attribute::Hp => card.hp,
            Attribute::SummonCost => card.summon_cost.energy,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Reflect, Debug)]
pub enum AbilityCost {
    Static { cost: Cost },
    Derived { attribute: Attribute },
}
impl AbilityCost {
    pub fn get(&self, effect: &Effect) -> Cost {
        match self {
            AbilityCost::Static { cost } => *cost,
            AbilityCost::Derived { attribute } => {
                let Effect::SummonCard { card } = effect else {
                    panic!("DerivedCost only valid for SummonCard.")
                };
                Cost { energy: attribute.get(card) }
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
// #[reflect(where Ability: FromReflect)]
#[reflect(no_field_bounds)]
pub enum Effect {
    Attack { damage: u32, effect_type: EffectType },
    GrantAbilities { abilities: Vec<Ability> }, // Vec to avoid infinite type
    SummonCard { card: Card },
    // SharableEnergy {
    //     factor: f32,
    // },
    // Cloaking {
    //
    // },
    MultipleEffects { effects: Vec<Effect> },

    ChangeHp { amount: i32 },
    ChangeEnergy { amount: i32 },
    DestroyCard,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum PassiveEffect {
    DamageResistance { effect_type: EffectType, factor: f32 },
    WhenHit { effect: Effect, target_rules: ImplicitTargetRules },
    WhenDies { effect: Effect, target_rules: ImplicitTargetRules },
    // ModifySummonCost
    // ModifyAbilityCost ??
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Reflect)]
pub enum EffectType {
    Physical,
    Explosion,
    Fire,
    Electrical,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum ImplicitTargetRules {
    ThisUnit,
    ThatUnit,
    // AttackingUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TargetRules {
    pub amount: TargetAmount,
    pub filter: TargetFilter,
    // chosen, random, or auto
}
impl TargetRules {
    pub fn validate(
        &self,
        targets: &[GridLocation],
        loc_idx: &mut Index<GridLocation>,
        cards: &Cards,
        players: &[PlayerId],
        effect_source: &GridLocation,
    ) -> bool {
        // todo: not verify untargeted squares unless needed for `All`

        let mut total_valid = 0;
        let mut targeted_valid = 0;
        for x in 0..(GRID_H as u32 / 2) {
            for y in 0..(GRID_W as u32) {
                for p in players {
                    let loc = GridLocation { coord: UVec2::new(x, y), owner: *p };

                    let valid = self.filter.validate(&loc, loc_idx, cards, effect_source);
                    if valid {
                        total_valid += 1;
                    }

                    if targets.contains(&loc) {
                        if valid {
                            targeted_valid += 1;
                        } else {
                            return false;
                        }
                    }
                }
            }
        }

        if targeted_valid != targets.len() {
            return false;
        }

        self.amount.validate(targeted_valid, total_valid)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum TargetAmount {
    All,
    N { n: usize },
    UpToN { n: usize },
}
impl TargetAmount {
    pub fn validate(&self, targeted_valid: usize, total_valid: usize) -> bool {
        match self {
            TargetAmount::N { n } => *n == targeted_valid,
            TargetAmount::UpToN { n } => targeted_valid <= *n,
            TargetAmount::All => targeted_valid == total_valid,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(no_field_bounds)]
pub enum TargetFilter {
    Any,
    ThisUnit,
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
        target: &GridLocation,
        loc_idx: &mut Index<GridLocation>,
        cards: &Cards,
        effect_source: &GridLocation,
    ) -> bool {
        let card = loc_idx.lookup(target).next().and_then(|e| cards.get(e).ok());
        match self {
            TargetFilter::Any => true,
            TargetFilter::ThisUnit => card.is_some() && card.unwrap().grid_loc == effect_source,
            TargetFilter::Friendly => target.owner == effect_source.owner,
            TargetFilter::Enemy => target.owner != effect_source.owner,
            TargetFilter::Unoccupied => card.is_none(),
            TargetFilter::Occupied => card.is_some(),
            TargetFilter::And(conds) => {
                conds.iter().all(|c| c.validate(target, loc_idx, cards, effect_source))
            },
            TargetFilter::Or(conds) => {
                conds.iter().any(|c| c.validate(target, loc_idx, cards, effect_source))
            },
        }
    }
}
