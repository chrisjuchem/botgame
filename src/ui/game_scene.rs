use aery::prelude::{Up as Reverse, *};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::{
    match_sim::{BaseCard, Energy, GridLocation, Health, OwnedBy, PlayerId, StartMatchEvent, Us},
    ui::SceneState,
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

const GRID_H: f32 = 4.;
const GRID_W: f32 = 5.;

pub fn update_card_transforms(
    mut cards: Query<((&GridLocation, &mut Transform), Relations<OwnedBy>), Changed<GridLocation>>,
    players: Query<&PlayerId>,
    us: Res<Us>,
) {
    for ((loc, mut t), ownership) in &mut cards {
        ownership.join::<Reverse<OwnedBy>>(&players).for_each(|pid| {
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
#[derive(Component)]
pub struct HoverPanel(pub Entity);

pub fn update_stat_overlays(
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

pub fn add_overlays_for_new_cards(cards: Query<Entity, Added<BaseCard>>, mut commands: Commands) {
    for e in &cards {
        commands.spawn((TextBundle::default(), StatsPanel(e), MatchScenery));
        commands.spawn((TextBundle::default(), HoverPanel(e), MatchScenery));
        commands.entity(e).insert((
            MatchScenery,
            On::<Pointer<Over>>::run(show_hover_overlay),
            On::<Pointer<Out>>::run(hide_hover_overlay),
        ));
    }
}

fn show_hover_overlay(
    event: Listener<Pointer<Over>>,
    mut panel: Query<(Entity, &mut Text, &mut Style, &Node, &HoverPanel)>,
    cards: Query<&BaseCard>,
    mut commands: Commands,
) {
    let target = event.listener();
    let card = cards.get(target).unwrap();
    let (e, mut txt, mut style, node, _) =
        panel.iter_mut().filter(|(_, _, _, _, panel)| panel.0 == target).next().unwrap();

    style.display = Display::Flex;
    style.position_type = PositionType::Absolute;
    style.max_width = Val::Percent(20.);

    *txt = Text {
        sections: vec![TextSection::new(&card.0.full_text(), TextStyle {
            font_size: 15.0,
            color: Color::WHITE,
            ..default()
        })],
        alignment: TextAlignment::Center,
        ..default()
    };

    commands.entity(e).insert(FollowMouse { offset: Vec2::new(15., node.size().y / 2.) });
}

#[derive(Component)]
pub struct FollowMouse {
    offset: Vec2,
}

fn hide_hover_overlay(event: Listener<Pointer<Out>>, mut panel: Query<(&mut Style, &HoverPanel)>) {
    let target = event.listener();
    let mut style = panel
        .iter_mut()
        .filter_map(|(style, panel)| (panel.0 == target).then_some(style))
        .next()
        .unwrap();

    style.display = Display::None;
}

pub fn follow_mouse(mut nodes: Query<(&mut Style, &Node, &FollowMouse)>, window: Query<&Window>) {
    let Ok(window) = window.get_single() else { return };
    let Some(mouse_pos) = window.cursor_position() else { return };
    for (mut style, node, follow) in &mut nodes {
        let width = 15. + node.size().x;

        style.top = Val::Px(f32::max(0., mouse_pos.y - node.size().y / 2.));
        if mouse_pos.x + width < window.width() {
            style.left = Val::Px(mouse_pos.x + 15.);
        } else {
            style.left = Val::Px(mouse_pos.x - node.size().x - 5.);
        }
    }
}
