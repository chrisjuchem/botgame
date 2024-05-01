use std::fmt::{Display, Formatter};

use crate::{
    cards::{
        Ability, ActivatedAbility, Card, Effect, EffectType, ImplicitTargetRules, PassiveAbility,
        PassiveEffect, Robot, TargetAmount, TargetFilter, TargetRules,
    },
    utils::StrJoin,
};

impl Card {
    pub fn full_text(&self) -> String {
        let Card { name, ability } = self;
        let fmtd = ability.full_text();
        format!("{name}\n{fmtd}")
    }
}

impl Robot {
    pub fn full_text(&self) -> String {
        let Robot { hp, attack, size, abilities } = self;
        let fmtd_abilities = abilities.iter().map(Ability::full_text).join("\n");

        format!(
            "\
{attack} ATK, {hp} HP
{fmtd_abilities}"
        )
    }
}

impl Ability {
    pub fn full_text(&self) -> String {
        match self {
            Ability::Activated(ability) => ability.full_text(),
            Ability::Passive(ability) => ability.full_text(),
        }
    }
}

impl ActivatedAbility {
    pub fn full_text(&self) -> String {
        let ActivatedAbility { effect, cost, target_rules } = self;
        {
            let energy_cost = self.cost().energy;
            let effect_str = effect.full_text(target_rules.text());

            format!("{{{energy_cost}}}: {effect_str}",)
        }
    }
}

impl PassiveAbility {
    pub fn full_text(&self) -> String {
        let PassiveAbility { passive_effect, target_filter } = self;
        passive_effect.full_text(target_filter.text())
    }
}

impl Effect {
    pub fn full_text(&self, target_str: String) -> String {
        match self {
            Effect::Attack {} => {
                format!("Fight {target_str}.")
            },
            Effect::GrantAbilities { abilities } => std::iter::once(format!("Give {target_str}:"))
                .chain(abilities.iter().map(Ability::full_text))
                .join("\n"),
            Effect::SummonRobot { robot } => {
                format!("Summon the following unit to {target_str}:\n\n{}\n", robot.full_text())
            },
            Effect::MultipleEffects { effects } => {
                effects.iter().map(|e| e.full_text(target_str.clone())).join(" ")
            },
            Effect::ChangeHp { amount } => {
                let (change, n) = if *amount > 0 { ("gains", *amount) } else { ("loses", -amount) };
                format!("{target_str} {change} {n} health.",)
            },
            Effect::ChangeEnergy { amount } => {
                let (change, n) = if *amount > 0 { ("gains", *amount) } else { ("loses", -amount) };
                format!("{target_str} {change} {n} energy.",)
            },
            Effect::DestroyCard => {
                format!("Destroy {target_str}")
            },
        }
    }
}

impl PassiveEffect {
    pub fn full_text(&self, target_str: String) -> String {
        match self {
            PassiveEffect::DamageResistance { effect_type, factor } => {
                format!("{target_str} takes {factor}x damage from {effect_type} attacks.")
            },
            PassiveEffect::WhenHit { effect, target_rules } => {
                format!("Whenever a {target_str} is hit, {}", effect.full_text(target_rules.text()))
            },
            PassiveEffect::WhenDies { effect, target_rules } => {
                format!(
                    "Whenever a {target_str} is destroyed, {}",
                    effect.full_text(target_rules.text())
                )
            },
        }
    }
}

impl Display for EffectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            EffectType::Physical => "physical",
            EffectType::Explosion => "explosion",
            EffectType::Fire => "fire",
            EffectType::Electrical => "electrical",
        })
    }
}

impl ImplicitTargetRules {
    pub fn text(&self) -> String {
        match self {
            ImplicitTargetRules::ThisUnit => "this unit",
            ImplicitTargetRules::ThatUnit => "that unit",
            // ImplicitTargetRules::AttackingUnit => "the attacking unit",
        }
        .to_string()
    }
}

impl TargetRules {
    pub fn text(&self) -> String {
        match self.amount {
            TargetAmount::All => format!("all {}", self.filter.text()),
            TargetAmount::N { n } => format!("{n} {}", self.filter.text()),
            TargetAmount::UpToN { n } => format!("up to {n} {}", self.filter.text()),
        }
    }
}

impl TargetFilter {
    pub fn text(&self) -> String {
        match self {
            TargetFilter::Any => "location(s)".to_string(),
            TargetFilter::ThisUnit => "this unit".to_string(),
            TargetFilter::Friendly => "friendly".to_string(),
            TargetFilter::Enemy => "enemy".to_string(),
            TargetFilter::Unoccupied => "open location(s)".to_string(),
            TargetFilter::Occupied => "unit(s)".to_string(),
            TargetFilter::And(conds) => conds.iter().map(|f| f.text()).join(" "),
            TargetFilter::Or(conds) => conds.iter().map(|f| f.text()).join(" or "),
        }
    }
}
