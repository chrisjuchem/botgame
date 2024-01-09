use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub struct Card {
    name: &'static str,
    summon_cost: Cost,
    hp: u32,
    abilities: Vec<Ability>, // name + abilityData ??
    max_energy: u32,
    energy_regen: u32,
}

#[derive(Debug)]
pub enum Ability {
    Activated {
        effect: Effect,
        cost: AbilityCost,
        // targets
    },
    Passive {
        passive_effect: PassiveEffect,
    },
}
impl Ability {
    fn cost(&self) -> Cost {
        match self {
            Ability::Activated { effect, cost } => cost.get(effect),
            Ability::Passive { .. } => Cost::FREE,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Cost {
    energy: u32,
}
impl Cost {
    const FREE: Cost = Cost { energy: 0 };
}

trait DerivedCostFunc: Fn(&Effect) -> Cost {
    fn fn_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
impl<T> DerivedCostFunc for T where T: Fn(&Effect) -> Cost {}

pub enum AbilityCost {
    Static { cost: Cost },
    Derived { func: &'static dyn DerivedCostFunc },
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

#[derive(Debug)]
pub enum Effect {
    Attack {
        damage: u32,
        effect_type: EffectType,
    },
    GrantAbility {
        ability: Box<Ability>,
    },
    SummonCard {
        card: Card,
    },
    // SharableEnergy {
    //     factor: f32,
    // },
    // Cloaking {
    //
    // },
    MultipleEffects {
        effects: Vec<Effect>,
    },
}

#[derive(Debug)]
pub enum PassiveEffect {
    DamageResistance {
        effect_type: EffectType,
        factor: f32,
    },
    WhenHit {
        effect: Effect,
    },
    // ModifySummonCost
    // ModifyAbilityCost ??
}

#[derive(Debug)]
pub enum EffectType {
    Physical,
    Explosion,
    Fire,
    Electrical,
}

pub fn deck() -> Card {
    let deck = vec![
        Card {
            name: "bot 1",
            summon_cost: Cost { energy: 3 },
            hp: 8,
            abilities: vec![Ability::Activated {
                effect: Effect::Attack {
                    damage: 3,
                    effect_type: EffectType::Fire,
                },
                cost: AbilityCost::Static {
                    cost: Cost { energy: 2 },
                },
            }],
            max_energy: 3,
            energy_regen: 1,
        },
        Card {
            name: "bot 2",
            summon_cost: Cost { energy: 5 },
            hp: 20,
            abilities: vec![Ability::Activated {
                effect: Effect::Attack {
                    damage: 1,
                    effect_type: EffectType::Physical,
                },
                cost: AbilityCost::Static { cost: Cost::FREE },
            }],
            max_energy: 0,
            energy_regen: 0,
        },
    ];

    Card {
        name: "Command Center",
        summon_cost: Cost::FREE,
        hp: 50,
        abilities: deck
            .into_iter()
            .map(|card| Ability::Activated {
                effect: Effect::SummonCard { card },
                cost: AbilityCost::Derived { func: &summon_cost },
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
