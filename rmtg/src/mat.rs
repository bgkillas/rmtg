use crate::assets::Asset;
use crate::card_spot::{CardSpot, SpotType};
use crate::net::Peer;
use crate::{CARD_HEIGHT, CARD_THICKNESS, CARD_WIDTH, MAT_BAR, MAT_HEIGHT, MAT_WIDTH, PLAYER};
use bevy::color::Color;
use bevy::material::AlphaMode;
use bevy::math::Vec3;
use bevy::mesh::Mesh3d;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Commands, InheritedVisibility, Rectangle, Transform};
use std::f32::consts::PI;
pub fn create_mats(mut assets: Asset, mut commands: Commands) {
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
            &mut assets,
            &mut commands,
            transform,
            right,
            PLAYER[i],
            Peer::new(u32::try_from(i).unwrap()),
        );
    }
}
fn make_mat(
    assets: &mut Asset,
    commands: &mut Commands,
    transform: Transform,
    right: bool,
    color: Color,
    player: Peer,
) {
    let mat = assets.materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        base_color: color,
        ..StandardMaterial::default()
    });
    let trans = |x: f32, y: f32, z: f32| -> Transform {
        Transform::from_xyz(if right { x } else { -x }, y, z)
    };
    commands
        .spawn((transform, InheritedVisibility::VISIBLE))
        .with_children(|p| {
            p.spawn((
                Mesh3d(assets.meshes.add(Rectangle::new(MAT_WIDTH, MAT_BAR))),
                MeshMaterial3d(mat.clone()),
                trans(0.0, 0.0, MAT_HEIGHT / 2.0 - MAT_BAR / 2.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            p.spawn((
                Mesh3d(assets.meshes.add(Rectangle::new(MAT_WIDTH, MAT_BAR))),
                MeshMaterial3d(mat.clone()),
                trans(0.0, 0.0, MAT_BAR / 2.0 - MAT_HEIGHT / 2.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            p.spawn((
                Mesh3d(assets.meshes.add(Rectangle::new(MAT_BAR, MAT_HEIGHT))),
                MeshMaterial3d(mat.clone()),
                trans(MAT_WIDTH / 2.0 - MAT_BAR / 2.0, 0.0, 0.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            p.spawn((
                Mesh3d(assets.meshes.add(Rectangle::new(MAT_BAR, MAT_HEIGHT))),
                MeshMaterial3d(mat.clone()),
                trans(MAT_BAR / 2.0 - MAT_WIDTH / 2.0, 0.0, 0.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            for i in 1..5 {
                p.spawn((
                    Mesh3d(assets.meshes.add(Rectangle::new(CARD_WIDTH, MAT_BAR))),
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
                Mesh3d(assets.meshes.add(Rectangle::new(MAT_BAR, MAT_HEIGHT))),
                MeshMaterial3d(mat.clone()),
                trans(MAT_WIDTH / 2.0 - MAT_BAR * 1.5 - CARD_WIDTH, 0.0, 0.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            p.spawn((
                Mesh3d(assets.meshes.add(Rectangle::new(
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
