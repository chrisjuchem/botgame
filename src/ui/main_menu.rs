use bevy::{app::AppExit, prelude::*, ui::node_bundles::ButtonBundle};
use bevy_renet::renet::RenetClient;

use crate::{
    cards::deck,
    network::{messages::JoinMatchmakingQueueMessage, ClientExt},
};

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub enum MenuButton {
    FindMatch,
    Quit,
}

pub fn handle_button(
    interactions: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut quit: EventWriter<AppExit>,
    mut client: ResMut<RenetClient>,
) {
    for (i, b) in &interactions {
        if *i == Interaction::Pressed {
            match b {
                MenuButton::FindMatch => client.send(JoinMatchmakingQueueMessage {
                    player_name: "player".to_string(),
                    deck: deck(),
                }),
                MenuButton::Quit => quit.send(AppExit),
            }
        }
    }
}

pub fn spawn_main_menu(mut commands: Commands) {
    commands
        .spawn((MainMenu, NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                justify_items: JustifyItems::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }))
        .with_children(|base| {
            let base_button = ButtonBundle {
                style: Style {
                    width: Val::Vw(30.),
                    margin: UiRect::all(Val::Px(3.)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },

                ..default()
            };
            fn button_text(text: impl Into<String>) -> TextBundle {
                TextBundle {
                    text: Text {
                        sections: vec![TextSection::new(text, TextStyle {
                            font_size: 60.0,
                            color: Color::NAVY,
                            ..default()
                        })],
                        alignment: TextAlignment::Center,
                        ..default()
                    },
                    style: Style { border: UiRect::all(Val::Px(2.)), ..default() },

                    ..default()
                }
            }

            base.spawn((base_button.clone(), MenuButton::FindMatch)).with_children(|btn| {
                btn.spawn(button_text("Find Match"));
            });
            base.spawn((base_button, MenuButton::Quit)).with_children(|btn| {
                btn.spawn(button_text("Quit"));
            });
        });
}
