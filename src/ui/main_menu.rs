use bevy::{app::AppExit, prelude::*};
use bevy_mod_picking::prelude::*;
use bevy_renet::renet::RenetClient;

use crate::{
    network::{messages::JoinMatchmakingQueueMessage, ClientConfig, ClientExt},
    ui::button::{ClickHandler, GameButton},
};

#[derive(Component)]
pub struct MainMenu;

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

            base.spawn((base_button.clone(), GameButton {
                bg_color: Color::WHITE,
                hover_color: Color::GREEN,
                disabled_color: Color::GRAY,
                click_handler: ClickHandler::new(
                    |listener: Listener<Pointer<Click>>,
                     mut client: ResMut<RenetClient>,
                     config: Res<ClientConfig>,
                     mut btns: Query<&mut GameButton>| {
                        client.send(JoinMatchmakingQueueMessage {
                            player_name: "player".to_string(),
                            deck: config.deck.clone(),
                        });
                        btns.get_mut(listener.listener()).unwrap().active = false;
                    },
                ),
                active: true,
            }))
            .with_children(|btn| {
                btn.spawn(button_text("Find Match"));
            });
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
