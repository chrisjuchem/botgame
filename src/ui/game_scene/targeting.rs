use aery::prelude::{Up, *};
use bevy::{
    ecs::{query::WorldQuery, system::SystemParam},
    prelude::*,
    utils::HashMap,
};
use bevy_mod_index::prelude::Index;
use bevy_mod_picking::prelude::*;

use crate::{
    cards::Ability,
    match_sim::{
        Abilities, BaseCard, Energy, GridLocation, Health, OwnedBy, PlayerId, PlayerIndex, Us,
    },
    ui::game_scene::{create_ability_overlay, BATTLEFIELD_H, BATTLEFIELD_W},
};

#[derive(Component)]
pub struct TargetingUI;
#[derive(Component)]
pub struct TargetingSubmit;
#[derive(Component)]
pub struct TargetingIndicator;

#[derive(Resource)]
pub struct Targeting {
    pub source: Entity,
    pub ability_idx: usize,
    pub chosen: Vec<(UVec2, PlayerId)>,
}

pub fn start_targeting(
    targeting: Res<Targeting>,
    cards: Query<(Entity, &GridLocation, Relations<OwnedBy>), With<BaseCard>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<&PlayerId>,
    mut player_idx: Index<PlayerIndex>,
) {
    if !targeting.is_added() {
        return;
    }
    commands
        .spawn((Name::new("target_ui"), TargetingUI, NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                bottom: Val::Px(0.),
                right: Val::Px(0.),
                ..default()
            },
            background_color: Color::BLUE.into(),
            ..default()
        }))
        .with_children(|base| {
            let margin = UiRect::all(Val::Px(10.));
            base.spawn((
                Name::new("targeting_submit"),
                TargetingSubmit,
                TextBundle {
                    style: Style { margin, padding: margin, ..default() },
                    text: Text::from_section("Submit", TextStyle {
                        font_size: 15.0,
                        color: Color::WHITE,
                        ..default()
                    }),
                    background_color: BackgroundColor(Color::RED),
                    ..default()
                },
                On::<Pointer<Over>>::listener_component_mut::<BackgroundColor>(|e, color| {
                    color.0 = Color::hex("#5aad65").unwrap();
                }),
                On::<Pointer<Out>>::listener_component_mut::<BackgroundColor>(|e, color| {
                    color.0 = Color::GRAY;
                }),
            ));
            base.spawn((
                Name::new("targeting_cancel"),
                TextBundle {
                    style: Style { margin, padding: margin, ..default() },
                    text: Text::from_section("Cancel", TextStyle {
                        font_size: 15.0,
                        color: Color::WHITE,
                        ..default()
                    }),
                    background_color: BackgroundColor(Color::GRAY),
                    ..default()
                },
                On::<Pointer<Over>>::listener_component_mut::<BackgroundColor>(|e, color| {
                    color.0 = Color::hex("#5aad65").unwrap();
                }),
                On::<Pointer<Out>>::listener_component_mut::<BackgroundColor>(|e, color| {
                    color.0 = Color::GRAY;
                }),
                On::<Pointer<Click>>::run(
                    |indicators: Query<Entity, With<TargetingIndicator>>,
                     cards: Query<Entity, With<BaseCard>>,
                     ui: Query<Entity, With<TargetingUI>>,
                     mut commands: Commands| {
                        commands.entity(ui.single()).despawn_recursive();
                        for e in &indicators {
                            commands.entity(e).despawn();
                        }
                        commands.remove_resource::<Targeting>();
                        for card in &cards {
                            // restore overlay click handler
                            commands
                                .entity(card)
                                .insert(On::<Pointer<Click>>::run(create_ability_overlay));
                        }
                    },
                ),
            ));
        });

    let mut indicators = HashMap::new();
    for p in players.iter() {
        for x in 0..2 {
            for y in 0..5 {
                let target = (UVec2 { x, y }, *p);
                let e = commands
                    .spawn((Name::new("floor_targeting_helper"), TargetingIndicator, PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Plane::from_size(BATTLEFIELD_H / 4.5))),
                        material: materials.add(StandardMaterial {
                            perceptual_roughness: 0.9,
                            ..Color::rgba(0.8, 0., 0., 0.1).into()
                        }),
                        transform: Transform::from_xyz(0., 0., 0.01)
                            .looking_to(Vec3::Y, Vec3::Z)
                            .with_scale(Vec3::new(BATTLEFIELD_W / 5. / BATTLEFIELD_H * 4., 1., 1.)),
                        ..default()
                    }))
                    .id();
                indicators.insert(target, e);
            }
        }
    }

    for (e, loc, ownership) in &cards {
        let mut cmds = commands.entity(e);
        let mut pid = None;
        ownership.join::<Up<OwnedBy>>(&players).for_each(|p| pid = Some(*p));
        let target = (loc.0, pid.unwrap());

        cmds.add_child(*indicators.get(&target).unwrap());
        indicators.insert(target, e);
        // remove overlay click handler
        cmds.remove::<On<Pointer<Click>>>();
    }

    for ((l, p), e) in indicators.into_iter() {
        let p_e = player_idx.lookup_single(&p);

        commands.entity(e).insert(GridLocation(l)).set::<OwnedBy>(p_e).insert(
            On::<Pointer<Click>>::run(
                |listener: Listener<Pointer<Click>>,
                 cards: Query<(Entity, &GridLocation, Relations<OwnedBy>, Option<&Children>)>,
                 players: Query<&PlayerId>,
                 mut targeting: ResMut<Targeting>,
                 mut indicators: Query<&mut Handle<StandardMaterial>, With<TargetingIndicator>>,
                 mut materials: ResMut<Assets<StandardMaterial>>| {
                    let (card_e, grid, ownership, children) =
                        cards.get(listener.listener()).unwrap();
                    let mut pid = None;
                    ownership.join::<Up<OwnedBy>>(&players).for_each(|p| pid = Some(*p));
                    let target = (grid.0, pid.unwrap());

                    // find indicator between when either
                    //   - this listener is the indicator
                    //   - this card's indicator is its child
                    let mut indicator_iter = indicators.iter_many_mut(
                        children.iter().flat_map(|c| c.iter()).chain(std::iter::once(&card_e)),
                    );
                    if targeting.chosen.contains(&target) {
                        targeting.chosen.retain(|x| *x != target);
                        while let Some(mut i) = indicator_iter.fetch_next() {
                            *i = materials.add(StandardMaterial {
                                perceptual_roughness: 0.9,
                                ..Color::rgba(0.8, 0., 0., 0.1).into()
                            });
                        }
                    } else {
                        targeting.chosen.push(target);
                        while let Some(mut i) = indicator_iter.fetch_next() {
                            *i = materials.add(StandardMaterial {
                                perceptual_roughness: 0.9,
                                ..Color::rgba(0.8, 0., 0., 0.6).into()
                            });
                        }
                    }
                },
            ),
        );
    }
}

pub fn check_targets(
    cards: Cards,
    targeting: Res<Targeting>,
    mut btn: Query<(Entity, &mut BackgroundColor, Has<On<Pointer<Click>>>), With<TargetingSubmit>>,
    mut grid_idx: Index<GridLocation>,
    mut commands: Commands,
    us: Res<Us>,
) {
    let (btn, mut btn_bg, btn_active) = btn.single_mut();

    let ability =
        cards.cards.get(targeting.source).unwrap().abilities.0.get(targeting.ability_idx).unwrap();
    let Ability::Activated { target_rules, .. } = ability else {
        panic!("Activated passive abillity!");
    };

    let targets_valid = target_rules.validate(&targeting.chosen, &mut grid_idx, &cards, us.0);

    if targets_valid && !btn_active {
        commands.entity(btn).insert(On::<Pointer<Click>>::run(submit_targets));
        *btn_bg = Color::GRAY.into();
    } else if !targets_valid && btn_active {
        commands.entity(btn).remove::<On<Pointer<Click>>>();
        *btn_bg = Color::RED.into();
    }
}

fn submit_targets(
    ui: Query<Entity, With<TargetingUI>>,
    indicators: Query<Entity, With<TargetingIndicator>>,
    cards: Query<Entity, With<BaseCard>>,
    mut commands: Commands,
) {
    commands.entity(ui.single()).despawn_recursive();
    for e in &indicators {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<Targeting>();

    for card in &cards {
        // restore overlay click handler
        commands.entity(card).insert(On::<Pointer<Click>>::run(create_ability_overlay));
    }

    // todo: send activated msg
}

#[derive(WorldQuery)]
pub struct CardQuery {
    pub entity: Entity,
    pub grid_loc: &'static GridLocation,
    pub abilities: &'static Abilities,
    pub health: &'static Health,
    pub energy: &'static Energy,
    // pub ownership: Relations<OwnedBy>,
}
impl CardQuery {
    // pub fn owner(&mut self, players: Query<&PlayerId>) -> PlayerId {
    //     let mut ret = None;
    //     self.ownership.join::<Up<OwnedBy>>().for_each(|pid| ret = Some(pid));
    //     ret.unwrap()
    // }
}

#[derive(SystemParam)]
pub struct Cards<'w, 's> {
    pub cards: Query<'w, 's, CardQuery>,
    // pub players: Query<'w, 's, &'static PlayerId>,
}
