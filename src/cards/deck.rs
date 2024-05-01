use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::cards::{
    generator::random_card, Ability, AbilityCost, Attribute, Card, Cost, Effect, TargetAmount,
    TargetFilter, TargetRules,
};

// #[derive(Asset, TypePath)]
#[derive(Serialize, Deserialize, Reflect, Debug, Clone, Component)]
pub struct Decklist {
    pub cards: Vec<Card>,
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
pub struct Decks(pub HashMap<String, Decklist>);

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

pub fn random_deck() -> Decklist {
    make_deck((0..5).map(|_| random_card()).collect())
}

pub fn make_deck(cards: Vec<Card>) -> Decklist {
    Decklist { cards: cards }
}
