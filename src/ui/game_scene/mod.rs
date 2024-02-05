pub mod targeting;

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};
use bevy_mod_index::prelude::Index;
use bevy_mod_picking::prelude::*;

use crate::{
    cards::Ability,
    match_sim::{
        BaseCard, Cards, CurrentTurn, Energy, GridLocation, Health, PlayerId, StartMatchEvent, Us,
    },
    ui::{
        button::{ClickHandler, GameButton},
        game_scene::targeting::Targeting,
        SceneState,
    },
};

pub fn transition_to_match(e: EventReader<StartMatchEvent>, mut s: ResMut<NextState<SceneState>>) {
    if !e.is_empty() {
        s.0 = Some(SceneState::Match)
    }
}

#[derive(Component)]
pub struct MatchScenery;

const BATTLEFIELD_H: f32 = 20.;
const BATTLEFIELD_W: f32 = 30.;

pub fn spawn_match(
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

pub const GRID_H: f32 = 4.;
pub const GRID_W: f32 = 5.;

pub fn update_card_transforms(
    mut cards: Query<(&GridLocation, &mut Transform), Changed<GridLocation>>,
    players: Query<&PlayerId>,
    us: Res<Us>,
) {
    for (GridLocation { owner, coord }, mut t) in &mut cards {
        // 5x4 -> 20x12   4x/3x
        let row = if *owner == us.0 { coord.x as f32 } else { GRID_H - coord.x as f32 - 1. };
        let col = coord.y as f32;

        let scale_w = BATTLEFIELD_W / GRID_W as f32;
        let scale_h = BATTLEFIELD_H / GRID_H as f32;

        t.translation.x = ((col + 0.5) * scale_w) - (BATTLEFIELD_W / 2.);
        t.translation.y = ((row + 0.5) * scale_h) - (BATTLEFIELD_H / 2.);
    }
}

#[derive(Component)]
pub struct StatsPanel(pub Entity);
#[derive(Component)]
pub struct HoverPanel(pub Entity);

pub fn update_stat_overlays(
    cards: Query<(&Name, &Energy, &Health, &Transform)>,
    mut stats: Query<(&mut Text, &mut Style, &Node, &StatsPanel)>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (cam, cam_pos) = camera.single();

    for (mut txt, mut style, node, source) in &mut stats {
        let (name, energy, health, transform) = cards.get(source.0).unwrap();

        let Some(coord) = cam.world_to_viewport(cam_pos, transform.translation) else { continue };
        style.position_type = PositionType::Absolute;
        style.top = Val::Px(coord.y + 15.);
        style.left = Val::Px(coord.x - (node.size().x / 2.));

        *txt = Text {
            sections: vec![TextSection::new(
                format!("{}\nenergy: {}/{}\n {} hp", name, energy.current, energy.max, health.0),
                TextStyle { font_size: 15.0, color: Color::WHITE, ..default() },
            )],
            alignment: TextAlignment::Center,
            ..default()
        };
    }
}

pub fn setup_new_cards(cards: Query<Entity, Added<BaseCard>>, mut commands: Commands) {
    for e in &cards {
        commands.spawn((
            TextBundle::default(),
            StatsPanel(e),
            MatchScenery,
            Name::new("stats_panel"),
        ));
        // commands.spawn((TextBundle::default(), HoverPanel(e), MatchScenery));
        commands.entity(e).insert((
            MatchScenery,
            // On::<Pointer<Over>>::run(show_hover_overlay),
            // On::<Pointer<Out>>::run(hide_hover_overlay),
            On::<Pointer<Click>>::run(create_ability_overlay),
        ));
    }
}

// fn show_hover_overlay(
//     event: Listener<Pointer<Over>>,
//     mut panel: Query<(Entity, &mut Text, &mut Style, &Node, &HoverPanel)>,
//     cards: Query<&BaseCard>,
//     mut commands: Commands,
// ) {
//     let target = event.listener();
//     let card = cards.get(target).unwrap();
//     let (e, mut txt, mut style, node, _) =
//         panel.iter_mut().filter(|(_, _, _, _, panel)| panel.0 == target).next().unwrap();
//
//     style.display = Display::Flex;
//     style.position_type = PositionType::Absolute;
//     style.max_width = Val::Percent(20.);
//
//     *txt = Text {
//         sections: vec![TextSection::new(&card.0.full_text(), TextStyle {
//             font_size: 15.0,
//             color: Color::WHITE,
//             ..default()
//         })],
//         alignment: TextAlignment::Center,
//         ..default()
//     };
//
//     commands.entity(e).insert(FollowMouse { offset: Vec2::new(15., node.size().y / 2.) });
// }
//
// #[derive(Component)]
// pub struct FollowMouse {
//     offset: Vec2,
// }
//
// fn hide_hover_overlay(event: Listener<Pointer<Out>>, mut panel: Query<(&mut Style, &HoverPanel)>) {
//     let card = event.listener();
//     let mut style = panel
//         .iter_mut()
//         .filter_map(|(style, panel)| (panel.0 == card).then_some(style))
//         .next()
//         .unwrap();
//
//     style.display = Display::None;
// }
//
// pub fn follow_mouse(mut nodes: Query<(&mut Style, &Node, &FollowMouse)>, window: Query<&Window>) {
//     let Ok(window) = window.get_single() else { return };
//     let Some(mouse_pos) = window.cursor_position() else { return };
//     for (mut style, node, follow) in &mut nodes {
//         let width = 15. + node.size().x;
//
//         style.top = Val::Px(f32::max(0., mouse_pos.y - node.size().y / 2.));
//         if mouse_pos.x + width < window.width() {
//             style.left = Val::Px(mouse_pos.x + 15.);
//         } else {
//             style.left = Val::Px(mouse_pos.x - node.size().x - 5.);
//         }
//     }
// }

#[derive(Component, Default)]
pub struct Scroll {
    current: f32,
    step: f32,
    target: f32,
}

pub fn create_ability_overlay(
    event: Listener<Pointer<Click>>,
    mut commands: Commands,
    cards: Cards,
    current_turns: Query<Has<CurrentTurn>>,
    mut player_idx: Index<PlayerId>,
    us: Res<Us>,
    window: Query<&Window>,
) {
    let card_entity = event.listener();
    let card = cards.get(card_entity).unwrap();
    let window = window.single();

    let size = Vec2::new(0.2 * window.width(), 0.4 * window.height());
    let side_offest = 20.;
    let margin = UiRect::all(Val::Px(3.));
    let mouse_pos = window.cursor_position().unwrap();

    let top = (mouse_pos.y - (size.y / 2.)).clamp(0., window.height() - size.y);
    let left = if mouse_pos.x + size.x + side_offest < window.width() {
        mouse_pos.x + side_offest
    } else {
        mouse_pos.x - size.x - side_offest
    };

    let owners_turn = current_turns.get(player_idx.lookup_single(&card.grid_loc.owner)).unwrap();
    let buttons_active = owners_turn && card.grid_loc.owner == us.0;

    let scrollbar = commands
        .spawn((Name::new("scrollbar"), NodeBundle {
            style: Style {
                width: Val::Px(4.),
                height: Val::Percent(50.),
                position_type: PositionType::Relative,
                top: Val::Px(0.), //moved by scroll system
                margin,
                ..default()
            },
            background_color: BackgroundColor(Color::GRAY),
            ..default()
        }))
        .id();

    let content = commands
        .spawn((Name::new("abilities_list_window"), NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip_y(),
                flex_grow: 1.,
                ..default()
            },
            ..default()
        }))
        .with_children(|base| {
            base.spawn((Name::new("abilities_list"), NodeBundle {
                style: Style {
                    position_type: PositionType::Relative,
                    top: Val::Px(0.), //moved by scroll system
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            }))
            .with_children(|base| {
                for (i, ability) in card.abilities.0.iter().enumerate() {
                    let active = buttons_active
                        && match ability {
                            Ability::Activated { effect, cost, .. } => {
                                let energy_cost = cost.get(effect).energy;
                                card.energy.current >= energy_cost
                            },
                            Ability::Passive { .. } => false,
                        };

                    base.spawn((
                        TextBundle {
                            style: Style { margin, ..default() },
                            text: Text::from_section(ability.full_text(), TextStyle {
                                font_size: 15.0,
                                color: Color::WHITE,
                                ..default()
                            }),
                            ..default()
                        },
                        GameButton {
                            bg_color: Color::GRAY,
                            hover_color: Color::hex("#5aad65").unwrap(),
                            disabled_color: if buttons_active { Color::RED } else { Color::GRAY },
                            click_handler: ClickHandler::new(move |mut commands: Commands| {
                                commands.insert_resource(Targeting {
                                    source: card_entity,
                                    ability_idx: i,
                                    chosen: vec![],
                                })
                            }),
                            active,
                        },
                    ));
                }
            });
        })
        .id();

    let scene_shield = commands
        .spawn((
            MatchScenery,
            Name::new("scene_guard"),
            NodeBundle {
                style: Style { width: Val::Vw(100.), height: Val::Vh(100.), ..default() },
                ..default()
            },
            On::<Pointer<Click>>::run(
                |listener: Listener<Pointer<Click>>, mut commands: Commands| {
                    commands.entity(listener.listener()).despawn_recursive();
                },
            ),
        ))
        .id();

    let panel = commands
        .spawn((
            MatchScenery,
            Name::new("ability_panel"),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Px(size.x),
                    height: Val::Px(size.y),
                    top: Val::Px(top),
                    left: Val::Px(left),
                    ..default()
                },
                background_color: Color::WHITE.into(),
                ..default()
            },
            Interaction::default(),
            Scroll::default(),
            On::<Pointer<Click>>::run(
                move |mut listener: ListenerMut<Pointer<Click>>, mut commands: Commands| {
                    listener.stop_propagation();
                    if buttons_active {
                        commands.entity(scene_shield).despawn_recursive();
                    }
                },
            ),
        ))
        .add_child(content)
        .add_child(scrollbar)
        .id();

    commands.entity(scene_shield).add_child(panel);
}

pub fn scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut container: Query<(&mut Scroll, &Children, &Interaction, &Node)>,
    mut inners: Query<(&mut Style, &Node, Option<&Children>)>,
) {
    let events = mouse_wheel_events.read().collect::<Vec<_>>();

    for (mut scroll, children, interaction, parent_node) in &mut container {
        if parent_node.size().y == 0. {
            // initial frame
            continue;
        }

        let (_, content_window_node, content_window_kids) =
            inners.get_mut(*children.get(0).unwrap()).unwrap();
        let content_window_h = content_window_node.size().y;
        let content: Entity = *content_window_kids.unwrap().get(0).unwrap();
        let (mut content_style, content_node, _) = inners.get_mut(content).unwrap();

        let max_scroll = (content_node.size().y - parent_node.size().y).max(0.);

        for e in events.iter() {
            if matches!(interaction, Interaction::Hovered) {
                scroll.target -= match e.unit {
                    MouseScrollUnit::Line => e.y * 20.,
                    MouseScrollUnit::Pixel => e.y,
                };
            }
            scroll.target = scroll.target.clamp(0., max_scroll);
        }

        let dist = (scroll.target - scroll.current).abs();
        if dist > scroll.step * 4. {
            scroll.step = dist / 4.;
        } else if scroll.target == scroll.current {
            scroll.step = 0.;
        }

        if dist < scroll.step {
            scroll.current = scroll.target;
        } else if scroll.current < scroll.target {
            scroll.current += scroll.step;
        } else {
            scroll.current -= scroll.step;
        }
        content_style.top = Val::Px(-scroll.current);
        let content_h = content_node.size().y;

        let (mut bar_style, bar_node, _) = inners.get_mut(*children.get(1).unwrap()).unwrap();
        let margin = match (bar_style.margin.top, bar_style.margin.bottom) {
            (Val::Px(top), Val::Px(bottom)) => top + bottom,
            _ => panic!("scrollbar margin must be in px"),
        };
        let h = parent_node.size().y - margin;
        let bar_length = h * content_window_h / content_h;
        let bar_top =
            if max_scroll == 0. { 0. } else { (h - bar_length) * scroll.current / max_scroll };

        if bar_length >= h {
            bar_style.display = Display::None;
        } else {
            bar_style.display = Display::Flex;
            bar_style.height = Val::Px(bar_length);
            bar_style.top = Val::Px(bar_top);
        }
        // println!(
        //     "bar_length:{bar_length} bar_top:{bar_top} h:{h}, content_h:{content_h}, \
        //      max_scroll:{max_scroll} scroll.current:{}, scroll.target:{}",
        //     scroll.current, scroll.target
        // );
    }
}
