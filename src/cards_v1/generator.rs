use rand::{prelude::SliceRandom, thread_rng, Rng};

use crate::cards_v1::{
    price::{price_card, price_effect},
    Ability, AbilityCost, Card, Cost, Effect, EffectType, PassiveEffect, TargetAmount,
    TargetFilter, TargetRules,
};

/// Returns a random number in `[1, limit)`
fn rnd(limit: usize) -> usize {
    thread_rng().gen_range(0..limit)
}

/// Returns a random number in `[1, max]` where each number is twice as likely
/// to appear as the next highest number.
fn rnd_log(max: u32) -> u32 {
    let r = thread_rng().gen_range(1..(2usize.pow(max)));
    max - r.ilog2()
}

fn rnd_log_n(max: u32, n: u32) -> u32 {
    let mut sum = 0;
    for i in 0..n {
        sum += rnd_log(max) - 1;
    }
    sum
}

fn random_effect_type() -> EffectType {
    match rnd(4) {
        0 => EffectType::Fire,
        1 => EffectType::Electrical,
        2 => EffectType::Explosion,
        _ => EffectType::Physical,
    }
}

pub fn random_card() -> Card {
    let mut abilities = vec![random_active_ability()];
    if rnd(4) < 2 {
        abilities.push(random_ability());
    }
    abilities.push(random_passive_ability());
    if rnd(3) < 2 {
        abilities.push(random_passive_ability());
    }

    let max_energy = abilities
        .iter()
        .filter_map(|a| match a {
            Ability::Activated { cost, effect, .. } => Some(cost.get(effect).energy),
            Ability::Passive { .. } => None,
        })
        .max()
        .unwrap();
    let starting_energy = max_energy.min(rnd_log_n(3, 3));
    let hp = rnd_log_n(8, 12);

    Card {
        name: random_name(),
        summon_cost: Cost {
            energy: price_card(&abilities, hp, starting_energy, max_energy) as u32,
        },
        hp,
        abilities,
        starting_energy,
        max_energy,
    }
}

pub fn random_ability() -> Ability {
    match rnd(100) {
        0..80 => random_active_ability(),
        _ => random_passive_ability(),
    }
}

fn random_active_ability() -> Ability {
    let (effect, target_rules) = match rnd(100) {
        _ => {
            let n = rnd_log(10) as usize;
            let amount = if rnd(5) < 2 { TargetAmount::UpToN { n } } else { TargetAmount::N { n } };
            let target_rules = TargetRules {
                amount,
                filter: TargetFilter::And(vec![TargetFilter::Enemy, TargetFilter::Occupied]),
            };

            let damage = rnd_log_n(10, 3) + 1;
            let effect = Effect::Attack { damage, effect_type: random_effect_type() };

            (effect, target_rules)
        },
    };

    let score = price_effect(&effect, &target_rules);
    let jitter = rnd(3) as f32 - 1.;
    let cost = AbilityCost::Static { cost: Cost { energy: (score + jitter) as u32 } };
    Ability::Activated { effect, cost, target_rules }
}

fn random_passive_ability() -> Ability {
    let (passive_effect, target_filter) = match rnd(100) {
        _ => {
            let effect = PassiveEffect::DamageResistance {
                effect_type: random_effect_type(),
                factor: if rnd(2) == 1 { 0.5 } else { 2.0 },
            };
            let target_filter = TargetFilter::ThisUnit;
            (effect, target_filter)
        },
    };

    Ability::Passive { passive_effect, target_filter }
}

fn random_name() -> String {
    let mut rng = thread_rng();

    let adj = ADJECTIVES.choose(&mut rng).unwrap();
    let noun = NOUNS.choose(&mut rng).unwrap();

    if rng.gen_ratio(1, 20) {
        let n = rnd_log(10) * 1000;
        format!("{adj} {noun} {n}")
    } else {
        format!("{adj} {noun}")
    }
}

const ADJECTIVES: [&'static str; 184] = [
    "Adept",
    "Aggressive",
    "Agreeable",
    "Ambitious",
    "Brave",
    "Calm",
    "Delightful",
    "Eager",
    "Extreme",
    "Faithful",
    "Gentle",
    "Happy",
    "Jolly",
    "Kind",
    "Lively",
    "Nice",
    "Obedient",
    "Patient",
    "Polite",
    "Proud",
    "Silly",
    "Thankful",
    "Victorious",
    "Witty",
    "Wonderful",
    "Zealous",
    "Angry",
    "Bewildered",
    "Clumsy",
    "Defeated",
    "Embarrassed",
    "Fierce",
    "Grumpy",
    "Helpless",
    "Itchy",
    "Jealous",
    "Lazy",
    "Mysterious",
    "Nervous",
    "Obnoxious",
    "Panicky",
    "Pitiful",
    "Repulsive",
    "Scary",
    "Thoughtless",
    "Uptight",
    "Worried",
    "Better",
    "Careful",
    "Clever",
    "Dead",
    "Easy",
    "Famous",
    "Gifted",
    "Hallowed",
    "Helpful",
    "Important",
    "Inexpensive",
    "Mealy",
    "Mushy",
    "Odd",
    "Poor",
    "Powerful",
    "Powerful",
    "Rich",
    "Shy",
    "Tender",
    "Unimportant",
    "Uninterested",
    "Attractive",
    "Bald",
    "Beautiful",
    "Blinding",
    "Chubby",
    "Clean",
    "Dazzling",
    "Drab",
    "Elegant",
    "Fancy",
    "Fit",
    "Flabby",
    "Glamorous",
    "Gorgeous",
    "Handsome",
    "Long",
    "Magnificent",
    "Muscular",
    "Plain ol'",
    "Plump",
    "Quaint",
    "Scruffy",
    "Shapely",
    "Short",
    "Skinny",
    "Stocky",
    "Ugly",
    "Unkempt",
    "Unsightly",
    "Big",
    "Colossal",
    "Fat",
    "Gigantic",
    "Great",
    "Huge",
    "Immense",
    "Large",
    "Little",
    "Mammoth",
    "Massive",
    "Microscopic",
    "Miniature",
    "Petite",
    "Puny",
    "Scrawny",
    "Short",
    "Small",
    "Tall",
    "Teeny",
    "Tiny",
    "Crashing",
    "Deafening",
    "Echoing",
    "Faint",
    "Harsh",
    "Hissing",
    "Howling",
    "Loud",
    "Melodic",
    "Noisy",
    "Purring",
    "Quiet",
    "Rapping",
    "Raspy",
    "Rhythmic",
    "Screeching",
    "Shrilling",
    "Squeaking",
    "Thundering",
    "Twinkling",
    "Wailing",
    "Whining",
    "Whispering",
    "Bumpy",
    "Chilly",
    "Cold",
    "Cool",
    "Cuddly",
    "Damaged",
    "Damp",
    "Dirty",
    "Dry",
    "Flaky",
    "Fluffy",
    "Freezing",
    "Greasy",
    "Hot",
    "Icy",
    "Loose",
    "Melted",
    "Prickly",
    "Rough",
    "Shaggy",
    "Sharp",
    "Slimy",
    "Sticky",
    "Strong",
    "Uneven",
    "Warm",
    "Weak",
    "Wet",
    "Carbon-Fiber",
    "Steel",
    "Copper",
    "Bronze",
    "Iron",
    "Gold",
    "Silicon",
    "Quartz",
    "Alluminum",
    "Elder",
    "Party",
    "Jovial",
    "Invisible",
    "Secret",
];
const NOUNS: [&'static str; 83] = [
    "Cow",
    "Bunny",
    "Wolf",
    "Duck",
    "Moose",
    "Pig",
    "Lion",
    "Eagle",
    "Hawk",
    "Dragon",
    "Raven",
    "Viper",
    "Worm",
    "Slug",
    "Snail",
    "Shark",
    "Tiger",
    "Eel",
    "Crab",
    "Stingray",
    "Wasp",
    "Cockroach",
    "Beetle",
    "Beverage",
    "Weapon",
    "Army",
    "Algorithm",
    "Calculator",
    "Concept",
    "Intuition",
    "Energy",
    "Warmth",
    "Trust",
    "Wrath",
    "Wisdom",
    "Knowledge",
    "Violence",
    "Secret",
    "Planet",
    "Quasar",
    "Galaxy",
    "Supernova",
    "Comet",
    "Rain",
    "Storm",
    "Wind",
    "Tornado",
    "Tsunami",
    "Sand",
    "Grass",
    "Dwarf",
    "Ranger",
    "Paladin",
    "Priest",
    "Wizard",
    "Warrior",
    "Soldier",
    "Farmer",
    "Weaponsmith",
    "Tinkerer",
    "Inventor",
    "Creator",
    "Minion",
    "Agent",
    "Maid",
    "Mage",
    "Politician",
    "Judge",
    "Destroyer",
    "Rogue",
    "Sentinel",
    "Observer",
    "Colossus",
    "Wisp",
    "Master",
    "Mastermind",
    "Elder",
    "Ghost",
    "Goblin",
    "Darkness",
    "Drone",
    "Bot",
    "Robot",
];
