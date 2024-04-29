pub mod deck;
pub mod mesh;

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub struct Cost {
    pub energy: u32,
    pub scrap: u32,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub enum StatKind {
    Hp,
    CombatStrength,
    MiningSpeed,
    Armor,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub struct Card {
    pub name: String,
    pub cost: Cost,
    pub chassis: Chassis,
    pub hp: u32,
    pub combat_strength: Option<u32>,
    pub mining_speed: Option<u32>,
    pub support_ability: Option<SupportAbility>,
    pub ability: Ability,
    pub attribute: Attribute,
}
impl Card {
    pub fn get_stat(&self, stat: StatKind) -> u32 {
        match stat {
            StatKind::Hp => self.hp,
            StatKind::CombatStrength => self.combat_strength.unwrap_or(0),
            StatKind::MiningSpeed => self.mining_speed.unwrap_or(0),
            StatKind::Armor => {
                if let Attribute::Armored(armor) = self.attribute {
                    armor
                } else {
                    0
                }
            },
        }
    }

    fn stat_mut(&mut self, stat: StatKind) -> Option<&mut u32> {
        match stat {
            StatKind::Hp => Some(&mut self.hp),
            StatKind::CombatStrength => self.combat_strength.as_mut(),
            StatKind::MiningSpeed => self.mining_speed.as_mut(),
            StatKind::Armor => {
                if let Attribute::Armored(ref mut armor) = self.attribute {
                    Some(armor)
                } else {
                    None
                }
            },
        }
    }

    pub fn raise_stat(&mut self, stat: StatKind, amount: EffectStrength) -> bool {
        let n = amount.for_target(&self);
        match self.stat_mut(stat) {
            None => false,
            Some(stat_ref) => {
                *stat_ref = stat_ref.saturating_add(n);
                true
            },
        }
    }

    pub fn lower_stat(&mut self, stat: StatKind, amount: EffectStrength) -> bool {
        let n = amount.for_target(&self);
        match self.stat_mut(stat) {
            None => false,
            Some(stat_ref) => {
                *stat_ref = stat_ref.saturating_sub(n);
                true
            },
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub enum Chassis {
    Combat,
    Mining,
    Support,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub enum TriggerType {
    ManualActivation { cost: Cost },
    Etb,
    Attack,
    Mine,
    AbilityTrigger,
    AbilityActivate,
    Destroyed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub struct Trigger {
    pub trigger_type: TriggerType,
    pub filter: CardFilter,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub struct StatRange {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub enum CardFilter {
    This,
    Friendly,
    Enemy,
    Chassis(Chassis),
    Stat { stat: StatKind, range: StatRange },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub struct Ability {
    pub trigger: Trigger,
    // targeting
    pub effect: Effect,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub enum Attribute {
    None,
    Ranged,
    Armored(u32),
    Cloaked,
    Reconfigurable,
    Mobile,
    StartingHand,
}

// == effects ==

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub struct Effect {
    pub kind: EffectKind,
    pub strength: EffectStrength,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub enum EffectStrength {
    Constant(u32),
    SourceStat(StatKind),
    TargetStat(StatKind),
}
impl EffectStrength {
    pub fn for_target(&self, target: &Card) -> u32 {
        match self {
            EffectStrength::Constant(n) => *n,
            EffectStrength::SourceStat(stat) => panic!("No source available for SourceStat"),
            EffectStrength::TargetStat(stat) => target.get_stat(*stat),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub enum EffectKind {
    DrawCards,
    RaiseStat(StatKind),
    LowerStat(StatKind),
}

// == support ==

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub struct SupportAbility;
