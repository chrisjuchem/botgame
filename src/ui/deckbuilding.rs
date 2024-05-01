use bevy::prelude::*;

use crate::cards::deck::{random_deck, Decklist};

#[derive(Resource, Reflect)]
pub struct CustomDeck(pub Decklist);

pub struct DeckbuildingPlugin;
impl Plugin for DeckbuildingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CustomDeck>();
        app.insert_resource(CustomDeck(random_deck()));
        app.add_plugins(bevy_inspector_egui::quick::ResourceInspectorPlugin::<CustomDeck>::new());
    }
}
