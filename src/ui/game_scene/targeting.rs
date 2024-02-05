use bevy::{prelude::*, utils::HashMap};
use bevy_mod_index::prelude::Index;
use bevy_mod_picking::prelude::*;
use bevy_renet::renet::RenetClient;

use crate::{
    cards::Ability,
    match_sim::{BaseCard, Cards, GridLocation, MatchId, PlayerId, Us},
    network::{messages::ActivateAbilityMessage, ClientExt},
    ui::{
        button::{ClickHandler, GameButton},
        game_scene::{create_ability_overlay, BATTLEFIELD_H, BATTLEFIELD_W, GRID_H, GRID_W},
    },
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
    pub chosen: Vec<GridLocation>,
}

pub fn start_targeting(
    targeting: Res<Targeting>,
    cards: Query<(Entity, &GridLocation), With<BaseCard>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<&PlayerId>,
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
                GameButton {
                    bg_color: Color::GRAY,
                    hover_color: Color::hex("#5aad65").unwrap(),
                    disabled_color: Color::RED,
                    click_handler: ClickHandler::new(submit_targets),
                    active: true,
                },
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
                GameButton {
                    bg_color: Color::GRAY,
                    hover_color: Color::hex("#5aad65").unwrap(),
                    disabled_color: Color::RED,
                    click_handler: ClickHandler::new(
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
                    active: true,
                },
            ));
        });

    let mut indicators = HashMap::new();
    for p in players.iter() {
        for x in 0..(GRID_H as u32 / 2) {
            for y in 0..(GRID_W as u32) {
                let loc = GridLocation { coord: UVec2 { x, y }, owner: *p };
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
                indicators.insert(loc, e);
            }
        }
    }

    for (e, loc) in &cards {
        let mut cmds = commands.entity(e);
        cmds.add_child(*indicators.get(loc).unwrap());
        indicators.insert(*loc, e);
        // remove overlay click handler
        cmds.remove::<On<Pointer<Click>>>();
    }

    for (loc, e) in indicators.into_iter() {
        commands.entity(e).insert(loc).insert(On::<Pointer<Click>>::run(
            |listener: Listener<Pointer<Click>>,
             cards: Query<(Entity, &GridLocation, Option<&Children>)>,
             mut targeting: ResMut<Targeting>,
             mut indicators: Query<&mut Handle<StandardMaterial>, With<TargetingIndicator>>,
             mut materials: ResMut<Assets<StandardMaterial>>| {
                let (card_e, loc, children) = cards.get(listener.listener()).unwrap();

                // find indicator between when either
                //   - this listener is the indicator
                //   - this card's indicator is its child
                let mut indicator_iter = indicators.iter_many_mut(
                    children.iter().flat_map(|c| c.iter()).chain(std::iter::once(&card_e)),
                );
                if targeting.chosen.contains(loc) {
                    targeting.chosen.retain(|x| x != loc);
                    while let Some(mut i) = indicator_iter.fetch_next() {
                        *i = materials.add(StandardMaterial {
                            perceptual_roughness: 0.9,
                            ..Color::rgba(0.8, 0., 0., 0.1).into()
                        });
                    }
                } else {
                    targeting.chosen.push(*loc);
                    while let Some(mut i) = indicator_iter.fetch_next() {
                        *i = materials.add(StandardMaterial {
                            perceptual_roughness: 0.9,
                            ..Color::rgba(0.8, 0., 0., 0.6).into()
                        });
                    }
                }
            },
        ));
    }
}

pub fn check_targets(
    cards: Cards,
    targeting: Res<Targeting>,
    mut btn: Query<&mut GameButton, With<TargetingSubmit>>,
    mut grid_idx: Index<GridLocation>,
    us: Res<Us>,
) {
    let ability =
        cards.get(targeting.source).unwrap().abilities.0.get(targeting.ability_idx).unwrap();
    let Ability::Activated { target_rules, .. } = ability else {
        panic!("Activated passive abillity!");
    };

    let targets_valid = target_rules.validate(&targeting.chosen, &mut grid_idx, &cards, us.0);

    let mut btn = btn.single_mut();
    if targets_valid && !btn.active {
        btn.active = true;
    } else if !targets_valid && btn.active {
        btn.active = false;
    }
}

fn submit_targets(
    ui: Query<Entity, With<TargetingUI>>,
    indicators: Query<Entity, With<TargetingIndicator>>,
    cards: Query<(Entity, &MatchId, &GridLocation), With<BaseCard>>,
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    targeting: Res<Targeting>,
) {
    commands.entity(ui.single()).despawn_recursive();
    for e in &indicators {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<Targeting>();

    for (card_e, _, _) in &cards {
        // restore overlay click handler
        commands.entity(card_e).insert(On::<Pointer<Click>>::run(create_ability_overlay));
    }

    let (_, mid, loc) = cards.get(targeting.source).unwrap();
    client.send(ActivateAbilityMessage {
        match_id: *mid,
        unit_location: loc.coord,
        ability_idx: targeting.ability_idx,
        targets: targeting.chosen.clone(), // todo: mem swap
    })
}
