use crate::pile::Pile;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Children, Entity, EntityEvent, On, Query};
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
pub fn on_repaint(
    on: On<Repaint>,
    decks: Query<(&Pile, &Children)>,
    mut top: Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    let (deck, children) = decks.get(on.entity).unwrap();
    let mut mat = top.get_mut(children[1]).unwrap();
    mat.0 = deck.first().face().material();
}
