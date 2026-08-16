use crate::pile::Pile;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Children, Entity, EntityEvent, On, Query, Transform};
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
    decks: Query<(&Pile, &Children)>,
    mut transforms: Query<&mut Transform>,
    mut top: Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    let pile = decks.get(on.entity).unwrap();
    let mut mat = top.get_mut(pile.children[0]).unwrap();
    if let Some(new) = pile.pile.first().face().material() {
        mat.0 = new;
    }
    let [mut up, mut down, mut side] = transforms
        .get_many_mut([pile.children[0], pile.children[1], pile.children[2]])
        .unwrap();
    pile.pile.reposition_up(&mut up);
    pile.pile.reposition_down(&mut down);
    pile.pile.reposition_side(&mut side);
}
