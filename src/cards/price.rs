use crate::cards::{AbilityCost, Cost, Effect, TargetAmount, TargetRules};

pub fn price_effect(effect: &Effect, target_rules: &TargetRules) -> AbilityCost {
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

    AbilityCost::Static { cost: Cost { energy: score as u32 } }
}
