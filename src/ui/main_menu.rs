use bevy::{app::AppExit, prelude::*};
use bevy_mod_picking::prelude::*;
use bevy_renet::renet::RenetClient;

use crate::{
    cards::deck::Decks,
    network::{messages::JoinMatchmakingQueueMessage, ClientConfig, ClientExt},
    ui::button::{ClickHandler, GameButton},
};

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct QueueButton;

pub fn spawn_main_menu(mut commands: Commands, decks: Res<Decks>) {
    let deck_names = decks.0.keys().cloned().collect::<Vec<_>>();

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
            let base_button = NodeBundle {
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
                            font_size: 30.0,
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

            for name in deck_names.iter().cloned() {
                let name_cloned = name.clone();
                base.spawn((base_button.clone(), QueueButton, GameButton {
                    bg_color: Color::WHITE,
                    hover_color: Color::GREEN,
                    disabled_color: Color::GRAY,
                    click_handler: ClickHandler::new(
                        move |listener: Listener<Pointer<Click>>,
                              mut client: ResMut<RenetClient>,
                              config: Res<ClientConfig>,
                              mut btns: Query<&mut GameButton, With<QueueButton>>,
                              decks: Res<Decks>| {
                            client.send(JoinMatchmakingQueueMessage {
                                player_name: "player".to_string(),
                                deck: decks.0.get(&name).unwrap().deck.clone(),
                            });
                            for mut btn in &mut btns {
                                btn.active = false;
                            }
                        },
                    ),
                    active: true,
                }))
                .with_children(|btn| {
                    btn.spawn(button_text(format!("Find Match ({})", name_cloned)));
                });
            }

            base.spawn((base_button, GameButton {
                bg_color: Color::WHITE,
                hover_color: Color::GREEN,
                disabled_color: Color::GRAY,
                click_handler: ClickHandler::new(|mut quit: EventWriter<AppExit>| {
                    quit.send(AppExit)
                }),
                active: true,
            }))
            .with_children(|btn| {
                btn.spawn(button_text("Quit"));
            });
        });
}
