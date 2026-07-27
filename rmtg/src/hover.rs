use crate::PLAYER;
use crate::assets::Asset;
use crate::shapes::{OUTLINE_COLOR, OUTLINE_DEPTH_BIAS};
use crate::spatial::Spatial;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Children, Commands, Component, Entity, Query, Single, With};
#[derive(Component)]
pub struct Hovered;
pub fn update_hover(
    old: Option<Single<(Entity, &Children), With<Hovered>>>,
    children: Query<&Children>,
    mut query: Query<&mut MeshMaterial3d<StandardMaterial>>,
    spatial: Spatial,
    mut commands: Commands,
    mut asset: Asset,
) {
    if let Some((hit, _)) = spatial.ray()
        && old.as_deref().is_none_or(|(e, _)| *e != hit.entity)
    {
        if let Some((ent, children)) = old.as_deref() {
            commands.entity(*ent).remove::<Hovered>();
            let mut mat = query.get_mut(children[0]).unwrap();
            mat.0 = asset.materials.add(StandardMaterial {
                base_color: OUTLINE_COLOR,
                unlit: true,
                depth_bias: OUTLINE_DEPTH_BIAS,
                ..StandardMaterial::default()
            });
        }
        let Ok(childs) = children.get(hit.entity) else {
            return;
        };
        let mut mat = query.get_mut(childs[0]).unwrap();
        mat.0 = asset.materials.add(StandardMaterial {
            base_color: PLAYER[0],
            unlit: true,
            depth_bias: OUTLINE_DEPTH_BIAS,
            ..StandardMaterial::default()
        });
        commands.entity(hit.entity).insert(Hovered);
    }
}
