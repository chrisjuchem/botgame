use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::cards_v2::{
    Ability, Attribute, Card, CardFilter, Chassis, Cost, Effect, EffectKind, EffectStrength,
    StatKind, Trigger, TriggerType,
};

#[derive(Resource, Default)]
pub struct Decks(pub HashMap<String, Decklist>);

#[derive(Clone, Debug, Serialize, Deserialize, Component)]
pub struct Decklist {
    pub cards: Vec<Card>,
}

pub fn demo_deck() -> Decklist {
    Decklist {
        cards: vec![Card {
            name: "One Drop".to_string(),
            cost: Cost { energy: 1, scrap: 0 },
            chassis: Chassis::Combat,
            hp: 2,
            combat_strength: Some(1),
            mining_speed: None,
            support_ability: None,
            ability: Ability {
                trigger: Trigger { trigger_type: TriggerType::Attack, filter: CardFilter::This },
                effect: Effect {
                    kind: EffectKind::RaiseStat(StatKind::Hp),
                    strength: EffectStrength::Constant(2),
                },
            },
            attribute: Attribute::None,
        }],
    }
}
