use std::os::linux::raw::stat;

mod deck;

#[derive(Copy, Clone)]
pub struct Cost {
    energy: u32,
    scrap: u32,
}

#[derive(Copy, Clone)]
pub enum StatKind {
    Hp,
    CombatStrength,
    MiningSpeed,
    Armor,
}

#[derive(Clone)]
pub struct CardV2 {
    name: String,
    cost: Cost,
    chassis: Chassis,
    hp: u32,
    combat_strength: Option<u32>,
    mining_speed: Option<u32>,
    support_ability: Option<SupportAbility>,
    ability: Ability,
    attribute: Attribute,
}
impl CardV2 {
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
            Some(stat_ref) => *stat_ref.saturating_add(n),
        }
    }

    pub fn lower_stat(&mut self, stat: StatKind, amount: EffectStrength) -> bool {
        let n = amount.for_target(&self);
        match self.stat_mut(stat) {
            None => false,
            Some(stat_ref) => *stat_ref.saturating_sub(n),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Chassis {
    Combat,
    Mining,
    Support,
}

#[derive(Clone)]
pub enum TriggerType {
    ManualActivation { cost: Cost },
    Etb,
    Attack,
    Mine,
    AbilityTrigger,
    AbilityActivate,
    Destroyed,
}

#[derive(Clone)]
pub struct Trigger {
    trigger_type: TriggerType,
    filter: CardFilter,
}

#[derive(Copy, Clone)]
pub struct StatRange {
    min: Option<u32>,
    max: Option<u32>,
}

#[derive(Clone)]
pub enum CardFilter {
    This,
    Friendly,
    Enemy,
    Chassis(Chassis),
    Stat { stat: StatKind, range: StatRange },
}

#[derive(Clone)]
pub struct Ability {
    trigger: Trigger,
    // targeting
    effect: Effect,
}

#[derive(Copy, Clone)]
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

pub struct Effect {
    kind: EffectKind,
    strength: EffectStrength,
}

pub enum EffectStrength {
    Constant(u32),
    SourceStat(StatKind),
    TargetStat(StatKind),
}
impl EffectStrength {
    pub fn for_target(&self, target: &CardV2) -> u32 {
        match self {
            EffectStrength::Constant(n) => *n,
            EffectStrength::SourceStat(stat) => panic!("No source available for SourceStat"),
            EffectStrength::TargetStat(stat) => target.get_stat(*stat),
        }
    }
}

pub enum EffectKind {
    DrawCards,
    RaiseStat(StatKind),
    LowerStat(StatKind),
}

// == support ==

pub struct SupportAbility;
