use bevy::prelude::{shape::Cube, *};

use crate::match_sim::BaseCard;

#[derive(Component)]
pub struct NeedsMesh;

pub fn spawn_card_mesh(
    cards: Query<(Entity, &BaseCard), With<NeedsMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for (e, c) in &cards {
        commands.entity(e).remove::<NeedsMesh>().with_children(|root| {
            root.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(Cube::new(1.5))),
                material: materials.add(StandardMaterial {
                    perceptual_roughness: 0.9,
                    ..Color::rgb(0.6, 0.5, 0.3).into()
                }),
                transform: Transform::from_xyz(0., 0., 1.)
                    .with_rotation(Quat::from_axis_angle(Vec3::Z, std::f32::consts::FRAC_PI_4)),
                ..default()
            });
        });
    }
}
