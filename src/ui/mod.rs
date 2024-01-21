pub mod game_scene;
pub mod main_menu;

use bevy::prelude::*;
use bevy_framepace::FramepacePlugin;
use bevy_mod_picking::DefaultPickingPlugins;

use crate::{
    cards::mesh::spawn_card_mesh,
    match_sim::StartMatchEvent,
    ui::{
        game_scene::{
            add_overlays_for_new_cards, follow_mouse, spawn_match, transition_to_match,
            update_card_transforms, update_stat_overlays, MatchScenery,
        },
        main_menu::{handle_button, spawn_main_menu, MainMenu},
    },
};

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum SceneState {
    #[default]
    MainMenu,
    Match,
}

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPickingPlugins);
        app.add_plugins(FramepacePlugin);

        app.add_state::<SceneState>();

        app.add_systems(OnEnter(SceneState::MainMenu), spawn_main_menu);
        app.add_systems(OnExit(SceneState::MainMenu), despawn_all_with_marker::<MainMenu>);
        app.add_systems(OnEnter(SceneState::Match), spawn_match);
        app.add_systems(OnExit(SceneState::Match), despawn_all_with_marker::<MatchScenery>);
        app.add_systems(Update, handle_button.run_if(in_state(SceneState::MainMenu)));
        app.add_systems(Update, transition_to_match.run_if(on_event::<StartMatchEvent>()));
        app.add_systems(
            Update,
            (
                spawn_card_mesh,
                add_overlays_for_new_cards,
                apply_deferred,
                update_card_transforms,
                update_stat_overlays,
                follow_mouse,
            )
                .chain()
                .run_if(in_state(SceneState::Match)),
        );
    }
}

fn despawn_all_with_marker<M: Component>(mut commands: Commands, all: Query<Entity, With<M>>) {
    for e in &all {
        commands.entity(e).despawn_recursive();
    }
}
