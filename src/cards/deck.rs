use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::cards::{
    generator::random_card, Ability, AbilityCost, Attribute, Card, Cost, Effect, TargetAmount,
    TargetFilter, TargetRules,
};

// #[derive(Asset, TypePath)]
#[derive(Serialize, Deserialize, Reflect)]
pub struct Deck {
    pub deck: Card,
}

// pub struct DeckLoader;
// impl AssetLoader for DeckLoader {
//     type Asset = Deck;
//     type Settings = ();
//     type Error = Box<dyn Error + Send + Sync + 'static>;
//
//     fn load<'a>(
//         &'a self,
//         reader: &'a mut Reader,
//         settings: &'a Self::Settings,
//         load_context: &'a mut LoadContext,
//     ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
//         Box::pin(async {
//             let mut bytes = Vec::new();
//             reader.read_to_end(&mut bytes).await?;
//             let deck = serde_json::from_slice(&bytes)?;
//             Ok(deck)
//         })
//     }
//
//     fn extensions(&self) -> &[&str] {
//         &["deck"]
//     }
// }

#[derive(Resource, Default)]
pub struct Decks(pub HashMap<String, Deck>);

pub fn load_decks(mut decks: ResMut<Decks>) {
    for file in std::fs::read_dir("assets/decks").unwrap() {
        let Ok(file) = file else { continue };
        let name = file.file_name().into_string().unwrap().split(".").next().unwrap().to_string();
        let Ok(fd) = std::fs::File::open(file.path()) else { continue };
        let Ok(deck) = serde_json::from_reader(fd) else { continue };
        decks.0.insert(name, deck);
    }

    decks.0.insert("Random".to_string(), random_deck());
}

pub fn random_deck() -> Deck {
    make_deck((0..5).map(|_| random_card()).collect())
}

pub fn make_deck(cards: Vec<Card>) -> Deck {
    Deck {
        deck: Card {
            name: "Command Center".to_string(),
            summon_cost: Cost::FREE,
            hp: 50,
            abilities: cards
                .into_iter()
                .map(|card| Ability::Activated {
                    effect: Effect::SummonCard { card },
                    cost: AbilityCost::Derived { attribute: Attribute::SummonCost },
                    target_rules: TargetRules {
                        amount: TargetAmount::N { n: 1 },
                        filter: TargetFilter::And(vec![
                            TargetFilter::Friendly,
                            TargetFilter::Unoccupied,
                        ]),
                    },
                })
                .chain(std::iter::once(Ability::Activated {
                    effect: Effect::MultipleEffects { effects: vec![] },
                    cost: AbilityCost::Static { cost: Cost::FREE },
                    target_rules: TargetRules {
                        amount: TargetAmount::N { n: 0 },
                        filter: TargetFilter::Any,
                    },
                }))
                .collect(),
            max_energy: 10,
            starting_energy: 3,
        },
    }
}
