use std::fmt::{format, Display, Formatter};

use crate::cards::{Ability, AbilityCost, Card, Effect, EffectType, PassiveEffect};

impl Card {
    pub fn full_text(&self) -> String {
        let Card { name, summon_cost, hp, abilities, max_energy, energy_regen } = self;
        let fmtd_abilities =
            abilities.iter().map(Ability::full_text).collect::<Vec<_>>().join("\n");

        format!(
            "\
{name}
{hp} max HP
{energy_regen} energy/turn up to {max_energy}
{fmtd_abilities}"
        )
    }
}

impl Ability {
    pub fn full_text(&self) -> String {
        match self {
            Ability::Activated { effect, cost } => {
                let energy_cost = self.cost().energy;
                let effect_str = effect.full_text();

                format!("{{{energy_cost}}}: {effect_str}",)
            },
            Ability::Passive { passive_effect } => passive_effect.full_text(),
        }
    }
}

impl Effect {
    pub fn full_text(&self) -> String {
        match self {
            Effect::Attack { damage, effect_type } => {
                format!("Deal {damage} {effect_type} damage.")
            },
            Effect::GrantAbility { ability } => {
                format!("Give a card \"{}\"", ability.full_text())
            },
            Effect::SummonCard { card } => {
                format!("Summon a card called:\n\n{}\n", card.full_text())
            },
            Effect::MultipleEffects { effects } => {
                effects.iter().map(Effect::full_text).collect::<Vec<_>>().join(" ")
            },
        }
    }
}

impl PassiveEffect {
    pub fn full_text(&self) -> String {
        match self {
            PassiveEffect::DamageResistance { effect_type, factor } => {
                format!("This card takes {factor}x damage from {effect_type} attacks.")
            },
            PassiveEffect::WhenHit { effect } => {
                format!("Whenever this card is hit, {}", effect.full_text())
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
