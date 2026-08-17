use crate::events::move_up::MoveUp;
use crate::pile::Pile;
use avian3d::prelude::Collider;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Children, Entity, EntityEvent, On, Query, Transform};
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
#[query_fn]
pub fn on_repaint(
    on: On<Repaint>,
    mut decks: Query<(&Pile, &Children, &mut Collider)>,
    mut transforms: Query<&mut Transform>,
    mut top: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut commands: Commands,
) {
    commands.trigger(MoveUp::new(on.entity));
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
}
