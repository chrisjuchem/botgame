use std::vec;

use botgame::cards::{
    summon_cost, Ability, AbilityCost, Card, Cost, Effect, Effect::ChangeEnergy, EffectType,
    ImplicitTargetRules, PassiveEffect, TargetAmount, TargetFilter, TargetRules,
};
use serde::Serialize;

#[derive(Serialize)]
struct Deck {
    deck: Card,
}

pub fn deck1() -> Card {
    let deck = vec![
        Card {
            name: "Shrapnel Bot".to_string(),
            summon_cost: Cost { energy: 3 },
            hp: 6,
            abilities: vec![Ability::Activated {
                effect: Effect::Attack { damage: 1, effect_type: EffectType::Physical },
                cost: AbilityCost::Static { cost: Cost { energy: 2 } },
                target_rules: TargetRules {
                    amount: TargetAmount::All,
                    filter: TargetFilter::Occupied,
                },
            }],
            starting_energy: 1,
            max_energy: 3,
        },
        Card {
            name: "Charge Bot".to_string(),
            summon_cost: Cost { energy: 4 },
            hp: 10,
            abilities: vec![Ability::Passive {
                passive_effect: PassiveEffect::WhenHit {
                    effect: ChangeEnergy { amount: 2 },
                    target_rules: ImplicitTargetRules::ThatUnit,
                },
                target_filter: TargetFilter::And(vec![
                    TargetFilter::Friendly,
                    TargetFilter::Occupied,
                ]),
            }],
            starting_energy: 0,
            max_energy: 0,
        },
        Card {
            name: "Support Bot".to_string(),
            summon_cost: Cost { energy: 1 },
            hp: 10,
            abilities: vec![
                Ability::Activated {
                    effect: Effect::GrantAbility {
                        ability: Box::new(Ability::Passive {
                            passive_effect: PassiveEffect::DamageResistance {
                                effect_type: EffectType::Physical,
                                factor: 2.0,
                            },
                            //fixme
                            target_filter: TargetFilter::ThisUnit,
                        }),
                    },
                    cost: AbilityCost::Static { cost: Cost { energy: 3 } },
                    target_rules: TargetRules {
                        amount: TargetAmount::N { n: 1 },
                        filter: TargetFilter::Occupied,
                    },
                },
                Ability::Activated {
                    effect: Effect::GrantAbility {
                        ability: Box::new(Ability::Passive {
                            passive_effect: PassiveEffect::DamageResistance {
                                effect_type: EffectType::Physical,
                                factor: 0.5,
                            },
                            target_filter: TargetFilter::ThisUnit,
                        }),
                    },
                    cost: AbilityCost::Static { cost: Cost { energy: 3 } },
                    target_rules: TargetRules {
                        amount: TargetAmount::N { n: 1 },
                        filter: TargetFilter::Occupied,
                    },
                },
            ],
            starting_energy: 2,
            max_energy: 5,
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
                        TargetFilter::Friendly,
                        TargetFilter::Unoccupied,
                    ]),
                },
            })
            .collect(),
        max_energy: 10,
        starting_energy: 3,
    }
}

fn main() {
    println!("{}", serde_json::to_string_pretty(&Deck { deck: deck1() }).unwrap())
}
