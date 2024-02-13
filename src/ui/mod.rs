pub mod button;
pub mod game_scene;
pub mod main_menu;

use bevy::prelude::*;
use bevy_eventlistener::event_dispatcher::EventDispatcher;
use bevy_framepace::FramepacePlugin;
use bevy_mod_picking::prelude::{Pointer, *};

use crate::{
    cards::{
        deck::{load_decks, Decks},
        mesh::spawn_card_mesh,
    },
    match_sim::StartMatchEvent,
    ui::{
        button::update_buttons,
        game_scene::{
            scroll, setup_new_cards, spawn_match,
            targeting::{check_targets, start_targeting, Targeting},
            transition_to_match, update_card_transforms, update_stat_overlays, MatchScenery,
        },
        main_menu::{spawn_main_menu, MainMenu},
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

        app.init_resource::<Decks>();
        app.add_systems(Startup, load_decks);

        app.add_systems(OnEnter(SceneState::MainMenu), spawn_main_menu);
        app.add_systems(OnExit(SceneState::MainMenu), despawn_all_with_marker::<MainMenu>);
        app.add_systems(OnEnter(SceneState::Match), spawn_match);
        app.add_systems(OnExit(SceneState::Match), despawn_all_with_marker::<MatchScenery>);
        // Hack to ensure `Out` handlers run before `Over`
        app.add_systems(
            PreUpdate,
            (|| {})
                .after(EventDispatcher::<Pointer<Out>>::bubble_events)
                .before(EventDispatcher::<Pointer<Over>>::bubble_events),
        );

        app.add_systems(Update, update_buttons);
        app.add_systems(Update, transition_to_match.run_if(on_event::<StartMatchEvent>()));
        app.add_systems(
            Update,
            (
                spawn_card_mesh,
                setup_new_cards,
                apply_deferred,
                update_card_transforms,
                update_stat_overlays,
                scroll,
            )
                .chain()
                .run_if(in_state(SceneState::Match)),
        );
        app.add_systems(
            Update,
            (start_targeting, apply_deferred, check_targets)
                .chain()
                .run_if(resource_exists_and_changed::<Targeting>()),
        );
    }
}

fn despawn_all_with_marker<M: Component>(mut commands: Commands, all: Query<Entity, With<M>>) {
    for e in &all {
        commands.entity(e).despawn_recursive();
    }
}
