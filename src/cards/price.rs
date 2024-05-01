use crate::cards::{
    Ability, ActivatedAbility, Effect, PassiveAbility, PassiveEffect, TargetAmount, TargetFilter,
    TargetRules,
};

pub fn price_effect(effect: &Effect, target_rules: &TargetRules) -> f32 {
    let (mut score, detrimental) = match effect {
        Effect::Attack {} => (0., false),
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

pub fn price_card(abilities: &[Ability], hp: u32, attack: u32) -> f32 {
    let mut price = 3.;

    let hp_diff = hp as f32 - 4.;
    price += hp_diff * 0.15;
    let atk_diff = attack as f32 - 4.;
    price += atk_diff * 0.15;

    for ability in abilities {
        let cost_change = match ability {
            Ability::Activated(ActivatedAbility { effect, cost, target_rules }) => {
                let ability_price = price_effect(effect, target_rules);
                let ability_cost = cost.get(effect).energy as f32;
                ability_price - ability_cost
            },
            Ability::Passive(PassiveAbility { passive_effect, target_filter }) => {
                price_passive_effect(passive_effect, target_filter)
            },
        };
        price += cost_change + 0.3 /*base ability cost*/;
    }

    price
}
