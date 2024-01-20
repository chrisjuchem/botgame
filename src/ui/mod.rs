use aery::{prelude::*, tuple_traits::RelationEntries};
use bevy::{
    app::AppExit,
    prelude::{shape::Plane, *},
    render::camera::ScalingMode,
    ui::node_bundles::ButtonBundle,
};
use bevy_renet::renet::RenetClient;

use crate::{
    cards::{deck, mesh::spawn_card_mesh},
    match_sim::{Energy, GridLocation, Health, OwnedBy, PlayerId, StartMatchEvent, Us},
    network::{messages::JoinMatchmakingQueueMessage, ClientExt},
};

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
enum SceneState {
    #[default]
    MainMenu,
    Match,
}

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<SceneState>();

        app.add_systems(OnEnter(SceneState::MainMenu), spawn_main_menu);
        app.add_systems(OnExit(SceneState::MainMenu), despawn_all_with_marker::<MainMenu>);
        app.add_systems(OnEnter(SceneState::Match), spawn_match);
        app.add_systems(OnExit(SceneState::Match), despawn_all_with_marker::<MatchScenery>);
        app.add_systems(Update, handle_button.run_if(in_state(SceneState::MainMenu)));
        app.add_systems(Update, transition_to_match.run_if(on_event::<StartMatchEvent>()));
        app.add_systems(
            Update,
            (spawn_card_mesh, apply_deferred, update_card_transforms, update_stat_overlays)
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

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
enum MenuButton {
    FindMatch,
    Quit,
}

fn handle_button(
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

fn spawn_main_menu(mut commands: Commands) {
    debug!("spawn_main_menu");
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

fn transition_to_match(e: EventReader<StartMatchEvent>, mut s: ResMut<NextState<SceneState>>) {
    if !e.is_empty() {
        s.0 = Some(SceneState::Match)
    }
}

#[derive(Component)]
struct MatchScenery;

const BATTLEFIELD_H: f32 = 20.;
const BATTLEFIELD_W: f32 = 30.;

fn spawn_match(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera>>,
) {
    // table
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(BATTLEFIELD_H))),
            material: materials.add(StandardMaterial {
                perceptual_roughness: 0.9,
                ..Color::rgb(0.3, 0.5, 0.3).into()
            }),
            transform: Transform::from_xyz(0., 0., 0.)
                .looking_to(Vec3::Y, Vec3::Z)
                .with_scale(Vec3::new(BATTLEFIELD_W / BATTLEFIELD_H, 1., 1.)),
            ..default()
        },
        MatchScenery,
    ));

    // light
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 2500.0,
                // color: Default::default(),
                shadows_enabled: true,
                // shadow_projection: Default::default(),
                // shadow_depth_bias: 0.0,
                // shadow_normal_bias: 0.0,
                ..default()
            },
            transform: Transform::from_xyz(-0.8, 1., 5.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MatchScenery,
    ));

    let (mut t, mut p) = camera.single_mut();
    *t = Transform::from_xyz(0., -60., 60.).looking_at(Vec3::new(0., -1., 0.), Vec3::Y);
    *p = Projection::Perspective(PerspectiveProjection { fov: 0.2, ..default() });
    // *p = Projection::Orthographic(OrthographicProjection {
    //     area: Rect { min: Vec2 { x: -160. / 9., y: -10.0 }, max: Vec2 { x: 160. / 9., y: 10.0 } },
    //     scaling_mode: ScalingMode::FixedVertical(20.),
    //     near: 0.,
    //     far: 20.1,
    //     ..default()
    // });
}

const GRID_H: f32 = 4.;
const GRID_W: f32 = 5.;

fn update_card_transforms(
    mut cards: Query<((&GridLocation, &mut Transform), Relations<OwnedBy>), Changed<GridLocation>>,
    players: Query<&PlayerId>,
    us: Res<Us>,
) {
    for ((loc, mut t), ownership) in &mut cards {
        ownership.join::<Up<OwnedBy>>(&players).for_each(|pid| {
            // 5x4 -> 20x12   4x/3x
            let row = if *pid == us.0 { loc.0.x as f32 } else { GRID_H - loc.0.x as f32 - 1. };
            let col = loc.0.y as f32;

            let scale_w = BATTLEFIELD_W / GRID_W as f32;
            let scale_h = BATTLEFIELD_H / GRID_H as f32;

            t.translation.x = ((col + 0.5) * scale_w) - (BATTLEFIELD_W / 2.);
            t.translation.y = ((row + 0.5) * scale_h) - (BATTLEFIELD_H / 2.);
        })
    }
}

#[derive(Component)]
pub struct StatsPanel(pub Entity);

fn update_stat_overlays(
    cards: Query<(&Energy, &Health, &Transform)>,
    mut stats: Query<(&mut Text, &mut Style, &Node, &StatsPanel)>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (cam, cam_pos) = camera.single();

    for (mut txt, mut style, node, source) in &mut stats {
        let (src_e, src_h, src_t) = cards.get(source.0).unwrap();

        let Some(coord) = cam.world_to_viewport(cam_pos, src_t.translation) else { continue };
        style.position_type = PositionType::Absolute;
        style.top = Val::Px(coord.y + 15.);
        style.left = Val::Px(coord.x - (node.size().x / 2.));

        *txt = Text {
            sections: vec![TextSection::new(
                format!("energy: {}/{}\n {} hp", src_e.current, src_e.max, src_h.0),
                TextStyle { font_size: 15.0, color: Color::WHITE, ..default() },
            )],
            alignment: TextAlignment::Center,
            ..default()
        };
    }
}
