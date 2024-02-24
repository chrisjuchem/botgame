use crate::cards_v1::{Ability, Effect, PassiveEffect, TargetAmount, TargetFilter, TargetRules};

pub fn price_effect(effect: &Effect, target_rules: &TargetRules) -> f32 {
    let (mut score, positive_effect) = match effect {
        Effect::Attack { damage, .. } => ((*damage as f32).log10() * 10., false),
        _ => todo!(),
    };

    let multiplier = match target_rules.amount {
        TargetAmount::UpToN { n } => 0.7 + (n as f32 * 0.08),
        TargetAmount::N { n } => 0.5 + (n as f32 * 0.08),
        TargetAmount::All => 0.5 + (10. * 0.08),
    };
    score *= multiplier;

    score
}

pub fn price_passive_effect(passive_effect: &PassiveEffect, target_filter: &TargetFilter) -> f32 {
    let muiltiplier = match target_filter {
        TargetFilter::ThisUnit => 1.,
        _ => panic!("TODO: determine passive effect filter multipliers"),
    };

    let base_price = match passive_effect {
        PassiveEffect::DamageResistance { factor, .. } => -1. * factor.log2(),
        _ => panic!("TODO: determine passive effect prices"),
    };

    base_price * muiltiplier
}

pub fn price_card(abilities: &[Ability], hp: u32, starting_energy: u32, max_energy: u32) -> f32 {
    let mut price = 3.;

    let hp_diff = hp as f32 - 12.;
    price += hp_diff * 0.15;

    if max_energy > 0 {
        // cheaper if < 50% energey, costlier if >
        price += (2. * starting_energy as f32 / max_energy as f32) - 1.;
    }

    for ability in abilities {
        let cost_change = match ability {
            Ability::Activated { effect, cost, target_rules } => {
                let ability_price = price_effect(effect, target_rules);
                let ability_cost = cost.get(effect).energy as f32;
                ability_price - ability_cost
            },
            Ability::Passive { passive_effect, target_filter } => {
                price_passive_effect(passive_effect, target_filter)
            },
        };
        price += cost_change + 0.3 /*base ability cost*/;
    }

    price
}
