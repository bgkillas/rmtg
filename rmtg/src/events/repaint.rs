use crate::CARD_THICKNESS;
use crate::assets::AssetManager;
use crate::pile::Pile;
use avian3d::prelude::Collider;
use bevy::math::Vec3;
use bevy::mesh::Mesh3d;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Children, Entity, EntityEvent, On, Query, Transform};
use bevy_ecs::lifecycle::Add;
use bevy_ecs::system::Commands;
use bevy_query_fn_macro::query_fn;
#[derive(EntityEvent)]
pub struct Repaint {
    pub entity: Entity,
}
impl Repaint {
    #[must_use]
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
pub fn on_pile_added(
    on: On<Add, Pile>,
    piles: Query<&Pile>,
    mut commands: Commands,
    asset: AssetManager,
) {
    let pile = piles.get(on.entity).unwrap();
    commands.entity(on.entity).with_children(|parent| {
        parent.spawn(pile.up(&asset));
        parent.spawn(pile.down(&asset));
        parent.spawn(pile.sides(&asset));
        parent.spawn(pile.outline(
            &asset,
            Transform::from_xyz(0.0, pile.thickness() / 2.0, 0.0),
        ));
        parent.spawn(pile.outline(
            &asset,
            Transform::from_xyz(0.0, -pile.thickness() / 2.0, 0.0),
        ));
    });
    commands.trigger(Repaint::new(on.entity));
}
#[query_fn]
pub fn on_repaint(
    on: On<Repaint>,
    mut decks: Query<(&Pile, &Children, &mut Collider)>,
    mut transforms: Query<&mut Transform>,
    mut top: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut commands: Commands,
    assets: AssetManager,
) {
    let mut pile = decks.get_mut(on.entity).unwrap();
    *pile.collider = pile.pile.collider();
    let mut mat = top.get_mut(pile.children[0]).unwrap();
    if let Some(new) = pile.pile.first().face().material() {
        mat.0 = new;
    }
    let [mut up, mut down, mut side, mut outline_up, mut outline_down] = transforms
        .get_many_mut([
            pile.children[0],
            pile.children[1],
            pile.children[2],
            pile.children[3],
            pile.children[4],
        ])
        .unwrap();
    pile.pile.reposition_up(&mut up);
    pile.pile.reposition_down(&mut down);
    pile.pile.reposition_side(&mut side);
    pile.pile.reposition_up(&mut outline_up);
    pile.pile.reposition_down(&mut outline_down);
    if pile.children.len() - 5 != pile.pile.len() - 1 {
        for &ent in &pile.children[5..] {
            commands.entity(ent).despawn();
        }
        commands.entity(on.entity).with_children(|parent| {
            for i in 1..pile.pile.len() {
                parent.spawn((
                    Transform::from_xyz(
                        0.0,
                        (i as f32 - pile.pile.len() as f32 / 2.0) * CARD_THICKNESS,
                        0.0,
                    )
                    .with_scale(Vec3::new(
                        1.0 + 1.0 / 4096.0,
                        1.0 / 4.0,
                        1.0 + 1.0 / 4096.0,
                    )),
                    Mesh3d(assets.card.side.clone()),
                    MeshMaterial3d(assets.card.inbetween_color.clone()),
                ));
            }
        });
    }
}
