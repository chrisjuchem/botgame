use bevy::{prelude::*, utils::HashMap};
use bevy_mod_index::prelude::Index;
use bevy_mod_picking::prelude::*;
use bevy_renet::renet::RenetClient;

use crate::{
    cards::Ability,
    match_sim::{BaseCard, Cards, GridLocation, MatchId, PlayerId},
    network::{messages::ActivateAbilityMessage, ClientExt},
    ui::{
        button::{ClickHandler, GameButton},
        font::CustomText,
        game_scene::{create_ability_overlay, BATTLEFIELD_H, BATTLEFIELD_W, GRID_H, GRID_W},
        UiManager,
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
    cards: Cards,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut loc_idx: Index<GridLocation>,
    players: Query<&PlayerId>,
    mut ui: UiManager,
) {
    if !targeting.is_added() {
        return;
    }
    commands
        .spawn((Name::new("target_ui"), TargetingUI, NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                bottom: Val::Vh(0.),
                right: Val::Vh(0.),
                ..default()
            },
            background_color: Color::BLUE.into(),
            ..default()
        }))
        .with_children(|base| {
            let margin = UiRect::all(Val::Vh(1.));
            base.spawn((
                Name::new("targeting_submit"),
                TargetingSubmit,
                NodeBundle {
                    style: Style { margin, padding: margin, ..default() },
                    background_color: BackgroundColor(Color::GRAY), // replaced by targeting validation
                    ..default()
                },
                GameButton {
                    bg_color: Color::GRAY,
                    hover_color: Color::hex("#5aad65").unwrap(),
                    disabled_color: Color::RED,
                    click_handler: ClickHandler::new(submit_targets),
                    active: true, // replaced by targeting validation
                },
            ))
            .add_child(ui.spawn_text(CustomText::new("Submit").color(Color::WHITE).size(15.)).id());
            base.spawn((
                Name::new("targeting_cancel"),
                NodeBundle {
                    style: Style { margin, padding: margin, ..default() },
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
            ))
            .add_child(ui.spawn_text(CustomText::new("Cancel").color(Color::WHITE).size(15.)).id());
        });

    let source_card = cards.get(targeting.source).unwrap();
    let Ability::Activated { target_rules, .. } =
        source_card.abilities.0.get(targeting.ability_idx).unwrap()
    else {
        panic!("Activated passive abillity!");
    };
    let mut indicators = HashMap::new();
    for p in players.iter() {
        for x in 0..(GRID_H as u32 / 2) {
            for y in 0..(GRID_W as u32) {
                let loc = GridLocation { coord: UVec2 { x, y }, owner: *p };

                if !target_rules.filter.validate(&loc, &mut loc_idx, &cards, source_card.grid_loc) {
                    continue;
                }

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

    for card in &cards {
        let mut cmds = commands.entity(card.entity);
        if let Some(indicator_e) = indicators.get(card.grid_loc) {
            cmds.add_child(*indicator_e);
            indicators.insert(*card.grid_loc, card.entity);
            // remove overlay click handler
            cmds.remove::<On<Pointer<Click>>>();
        }
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
    players: Query<&PlayerId>,
) {
    let card = cards.get(targeting.source).unwrap();
    let ability = card.abilities.0.get(targeting.ability_idx).unwrap();
    let Ability::Activated { target_rules, .. } = ability else {
        panic!("Activated passive abillity!");
    };

    let players = players.iter().copied().collect::<Vec<PlayerId>>();
    let targets_valid =
        target_rules.validate(&targeting.chosen, &mut grid_idx, &cards, &players, card.grid_loc);

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
