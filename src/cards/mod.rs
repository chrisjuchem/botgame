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
    pub ability: ActivatedAbility,
}
impl Card {
    pub fn as_robot(&self) -> Option<&Robot> {
        match &self.ability {
            ActivatedAbility { effect: Effect::SummonRobot { robot }, .. } => Some(robot),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Robot {
    pub size: u32,
    pub attack: u32,
    pub hp: u32,
    pub abilities: Vec<Ability>, // name + abilityData ??
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(no_field_bounds)]
pub enum Ability {
    Activated(ActivatedAbility),
    Passive(PassiveAbility),
}
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(no_field_bounds)]
pub struct ActivatedAbility {
    pub effect: Effect,
    pub cost: AbilityCost,
    pub target_rules: TargetRules,
}
impl ActivatedAbility {
    pub fn cost(&self) -> Cost {
        self.cost.get(&self.effect)
    }

    pub fn basic_summon(robot: Robot) -> Self {
        Self {
            effect: Effect::SummonRobot { robot },
            cost: AbilityCost::SUMMON_COST,
            target_rules: TargetRules {
                amount: TargetAmount::N { n: 1 },
                filter: TargetFilter::And(vec![TargetFilter::Friendly, TargetFilter::Unoccupied]),
            },
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(no_field_bounds)]
pub struct PassiveAbility {
    pub passive_effect: PassiveEffect,
    pub target_filter: TargetFilter,
}

impl Ability {
    pub const BASIC_ATTACK: Ability = Ability::Activated(ActivatedAbility {
        cost: AbilityCost::Static { cost: Cost::FREE },
        effect: Effect::Attack,
        target_rules: TargetRules { amount: TargetAmount::N { n: 1 }, filter: TargetFilter::Enemy },
    });

    pub fn cost(&self) -> Option<Cost> {
        match self {
            Ability::Activated(ability) => Some(ability.cost()),
            Ability::Passive(_) => None,
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
    Size,
    Attack,
    Hp,
    EnergyCost,
}
impl Attribute {
    pub fn get(&self, card: &Card) -> Option<u32> {
        match self {
            Attribute::Hp | Attribute::Attack | Attribute::Size => {
                card.as_robot().map(|bot| self.get_robot(bot).unwrap())
            },
            Attribute::EnergyCost => Some(card.ability.cost().energy),
        }
    }

    pub fn get_robot(&self, robot: &Robot) -> Option<u32> {
        match self {
            Attribute::Hp => Some(robot.hp),
            Attribute::Attack => Some(robot.attack),
            Attribute::Size => Some(robot.size),
            Attribute::EnergyCost => None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Reflect, Debug)]
pub enum AbilityCost {
    Static { cost: Cost },
    Derived { attribute: Attribute },
}
impl AbilityCost {
    const SUMMON_COST: AbilityCost = AbilityCost::Derived { attribute: Attribute::Size };

    pub fn get(&self, effect: &Effect) -> Cost {
        match self {
            AbilityCost::Static { cost } => *cost,
            AbilityCost::Derived { attribute } => {
                let Effect::SummonRobot { robot } = effect else {
                    panic!("DerivedCost only valid for SummonRobot.")
                };
                Cost { energy: attribute.get_robot(robot).unwrap_or(0) }
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(no_field_bounds)]
pub enum Effect {
    Attack,
    GrantAbilities { abilities: Vec<Ability> }, // Vec to avoid infinite type
    SummonRobot { robot: Robot },
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
