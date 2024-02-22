use std::vec;

use botgame::cards::{
    deck::{make_deck, random_deck, Deck},
    generator::random_card,
    summon_cost, Ability, AbilityCost, Card, Cost, Effect,
    Effect::MultipleEffects,
    EffectType, ImplicitTargetRules, PassiveEffect, TargetAmount, TargetFilter, TargetRules,
};

fn aoe_deck() -> Deck {
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
                    effect: Effect::ChangeEnergy { amount: 2 },
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
                        amount: TargetAmount::UpToN { n: 3 },
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
                        amount: TargetAmount::UpToN { n: 3 },
                        filter: TargetFilter::Occupied,
                    },
                },
            ],
            starting_energy: 2,
            max_energy: 5,
        },
    ];

    make_deck(deck)
}

pub fn gigablaster_deck() -> Deck {
    let deck = vec![
        Card {
            name: "GIGABLASTER".to_string(),
            summon_cost: Cost { energy: 0 },
            hp: 15,
            abilities: vec![
                Ability::Activated {
                    effect: Effect::Attack { damage: 50, effect_type: EffectType::Explosion },
                    cost: AbilityCost::Static { cost: Cost { energy: 50 } },
                    target_rules: TargetRules {
                        amount: TargetAmount::N { n: 1 },
                        filter: TargetFilter::Occupied,
                    },
                },
                Ability::Passive {
                    passive_effect: PassiveEffect::WhenDies {
                        effect: Effect::ChangeEnergy { amount: 5 },
                        target_rules: ImplicitTargetRules::ThisUnit,
                    },
                    target_filter: TargetFilter::And(vec![
                        TargetFilter::Friendly,
                        TargetFilter::Occupied,
                    ]),
                },
            ],
            starting_energy: 3,
            max_energy: 50,
        },
        Card {
            name: "Self destruct bot".to_string(),
            summon_cost: Cost { energy: 2 },
            hp: 3,
            abilities: vec![{
                Ability::Activated {
                    effect: Effect::Attack { damage: 3, effect_type: EffectType::Explosion },
                    cost: AbilityCost::Static { cost: Cost { energy: 1 } },
                    //todo two phase targeting
                    target_rules: TargetRules {
                        amount: TargetAmount::N { n: 2 },
                        filter: TargetFilter::Occupied,
                    },
                }
            }],
            starting_energy: 0,
            max_energy: 1,
        },
        Card {
            name: "Protection Bot".to_string(),
            summon_cost: Cost { energy: 2 },
            hp: 4,
            abilities: vec![
                Ability::Activated {
                    effect: Effect::ChangeHp { amount: 5 },
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
                Ability::Activated {
                    effect: Effect::GrantAbility {
                        ability: Box::new(Ability::Passive {
                            passive_effect: PassiveEffect::DamageResistance {
                                effect_type: EffectType::Explosion,
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
                Ability::Activated {
                    effect: Effect::GrantAbility {
                        ability: Box::new(Ability::Passive {
                            passive_effect: PassiveEffect::DamageResistance {
                                effect_type: EffectType::Electrical,
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
                Ability::Activated {
                    effect: Effect::GrantAbility {
                        ability: Box::new(Ability::Passive {
                            passive_effect: PassiveEffect::DamageResistance {
                                effect_type: EffectType::Fire,
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

    make_deck(deck)
}

fn main() {
    println!("{}", serde_json::to_string_pretty(&random_deck()).unwrap())
}
