use crate::{
    cards_v1::generator::random_name,
    cards_v2::{
        Ability, Attribute, CardFilter, CardV2, Chassis, Cost, Effect, EffectKind, EffectStrength,
        Trigger, TriggerType,
    },
};

pub fn demo_deck() -> Vec<CardV2> {
    vec![CardV2 {
        name: random_name(),
        cost: Cost { energy: 1, scrap: 0 },
        chassis: Chassis::Combat,
        hp: 2,
        combat_strength: Some(1),
        mining_speed: None,
        support_ability: None,
        ability: Ability {
            trigger: Trigger { trigger_type: TriggerType::Attack, filter: CardFilter::This },
            effect: Effect { kind: EffectKind::GainHp, strength: EffectStrength::Constant(2) },
        },
        attribute: Attribute::None,
    }]
}
