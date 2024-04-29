use bevy::{app::AppExit, prelude::*};
use bevy_mod_picking::prelude::*;
use bevy_renet::renet::RenetClient;

use crate::{
    cards_v2::deck::Decks,
    network::{messages::JoinMatchmakingQueueMessage, ClientConfig, ClientExt},
    ui::{
        button::{ClickHandler, GameButton},
        font::{CustomText, DefaultFont},
        UiManager,
    },
};

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct QueueButton;

pub fn spawn_main_menu(
    mut commands: Commands,
    decks: Res<Decks>,
    font: Res<DefaultFont>,
    mut ui: UiManager,
) {
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
                    width: Val::Vh(45.),
                    margin: UiRect::all(Val::Vh(0.4)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },

                ..default()
            };

            let text = CustomText::default().size(30.).color(Color::NAVY).centered();
            let mut decks = deck_names.iter().cloned().collect::<Vec<_>>();
            decks.sort();
            for name in decks {
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
                                deck: decks.0.get(&name).unwrap().clone(),
                            });
                            for mut btn in &mut btns {
                                btn.active = false;
                            }
                        },
                    ),
                    active: true,
                }))
                .add_child(
                    ui.spawn_text(text.clone().text(format!("Find Match ({})", name_cloned))).id(),
                );
            }

            base.spawn((base_button, GameButton {
                bg_color: Color::WHITE,
                hover_color: Color::GREEN,
                disabled_color: Color::GRAY,
                click_handler: ClickHandler::new(|mut quit: EventWriter<AppExit>| {
                    quit.send(AppExit);
                }),
                active: true,
            }))
            .add_child(ui.spawn_text(text.text("Quit")).id());
        });
}
