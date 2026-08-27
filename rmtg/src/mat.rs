use crate::card_spot::{CardSpot, SpotType};
use crate::net::Peer;
use crate::{CARD_HEIGHT, CARD_THICKNESS, CARD_WIDTH, MAT_BAR, MAT_HEIGHT, MAT_WIDTH, PLAYER};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Commands, Component, InheritedVisibility, Rectangle, Transform};
use bevy_ecs::system::ResMut;
use std::f32::consts::PI;
#[derive(Component)]
pub struct PlayMat {
    pub player: Peer,
}
pub fn create_mats(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let player0 = Transform::from_xyz(MAT_WIDTH / 2.0, -CARD_THICKNESS, MAT_HEIGHT / 2.0);
    let player1 = Transform::from_xyz(-MAT_WIDTH / 2.0, -CARD_THICKNESS, MAT_HEIGHT / 2.0);
    let mut player2 = Transform::from_xyz(-MAT_WIDTH / 2.0, -CARD_THICKNESS, -MAT_HEIGHT / 2.0);
    player2.rotate_y(PI);
    let mut player3 = Transform::from_xyz(MAT_WIDTH / 2.0, -CARD_THICKNESS, -MAT_HEIGHT / 2.0);
    player3.rotate_y(PI);
    for (i, (transform, right)) in [
        (player0, true),
        (player1, false),
        (player2, true),
        (player3, false),
    ]
    .into_iter()
    .enumerate()
    {
        make_mat(
            &mut materials,
            &mut meshes,
            &mut commands,
            transform,
            right,
            PLAYER[i],
            Peer::new(i as u64),
        );
    }
}
fn make_mat(
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    commands: &mut Commands,
    transform: Transform,
    right: bool,
    color: Color,
    player: Peer,
) {
    let mat = materials.add(StandardMaterial {
        unlit: true,
        base_color: color,
        ..StandardMaterial::default()
    });
    let trans = |x: f32, y: f32, z: f32| -> Transform {
        Transform::from_xyz(if right { x } else { -x }, y, z)
    };
    commands
        .spawn((transform, PlayMat { player }, InheritedVisibility::VISIBLE))
        .with_children(|p| {
            p.spawn((
                Mesh3d(meshes.add(Rectangle::new(MAT_WIDTH, MAT_BAR))),
                MeshMaterial3d(mat.clone()),
                trans(0.0, 0.0, MAT_HEIGHT / 2.0 - MAT_BAR / 2.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            p.spawn((
                Mesh3d(meshes.add(Rectangle::new(MAT_WIDTH, MAT_BAR))),
                MeshMaterial3d(mat.clone()),
                trans(0.0, 0.0, MAT_BAR / 2.0 - MAT_HEIGHT / 2.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            p.spawn((
                Mesh3d(meshes.add(Rectangle::new(MAT_BAR, MAT_HEIGHT))),
                MeshMaterial3d(mat.clone()),
                trans(MAT_WIDTH / 2.0 - MAT_BAR / 2.0, 0.0, 0.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            p.spawn((
                Mesh3d(meshes.add(Rectangle::new(MAT_BAR, MAT_HEIGHT))),
                MeshMaterial3d(mat.clone()),
                trans(MAT_BAR / 2.0 - MAT_WIDTH / 2.0, 0.0, 0.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            for i in 1..5 {
                p.spawn((
                    Mesh3d(meshes.add(Rectangle::new(CARD_WIDTH, MAT_BAR))),
                    MeshMaterial3d(mat.clone()),
                    trans(
                        MAT_WIDTH / 2.0 - CARD_WIDTH / 2.0 - MAT_BAR,
                        0.0,
                        i as f32 * (CARD_HEIGHT + MAT_BAR) - MAT_HEIGHT / 2.0 + MAT_BAR / 2.0,
                    )
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
                ));
            }
            for i in 0..5 {
                p.spawn((
                    trans(
                        MAT_WIDTH / 2.0 - MAT_BAR - CARD_WIDTH / 2.0,
                        CARD_THICKNESS / 2.0,
                        MAT_HEIGHT / 2.0
                            - MAT_BAR
                            - CARD_HEIGHT / 2.0
                            - i as f32 * (CARD_HEIGHT + MAT_BAR),
                    ),
                    match i {
                        4 => CardSpot::new(SpotType::CommanderMain),
                        3 => CardSpot::new(SpotType::CommanderAlt),
                        2 => CardSpot::new(SpotType::Exile),
                        1 => CardSpot::new(SpotType::Main),
                        0 => CardSpot::new(SpotType::Graveyard),
                        _ => unreachable!(),
                    },
                    player,
                ));
            }
            p.spawn((
                Mesh3d(meshes.add(Rectangle::new(MAT_BAR, MAT_HEIGHT))),
                MeshMaterial3d(mat.clone()),
                trans(MAT_WIDTH / 2.0 - MAT_BAR * 1.5 - CARD_WIDTH, 0.0, 0.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            p.spawn((
                Mesh3d(meshes.add(Rectangle::new(
                    MAT_WIDTH - CARD_WIDTH - 2.0 * MAT_BAR,
                    MAT_BAR,
                ))),
                MeshMaterial3d(mat.clone()),
                trans(
                    -CARD_WIDTH / 2.0 - MAT_BAR,
                    0.0,
                    MAT_HEIGHT / 2.0 - MAT_BAR * 1.5 - CARD_HEIGHT * 1.5,
                )
                .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
        });
}
