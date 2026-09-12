use crate::assets::AssetManager;
use crate::events::hover::HoveredObject;
use crate::events::ping_drag::{DragObject, MoveDragObject};
use crate::keybinds::Keybind;
use crate::spatial::Spatial;
use bevy::input::ButtonInput;
use bevy::prelude::Transform;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, Query, Res};
use bevy_query_fn_macro::query_fn;
#[derive(Component)]
pub struct SelectDrag {
    pub source: Entity,
    pub target: Entity,
}
#[derive(Component)]
pub struct SelectableObject;
#[derive(Component)]
pub struct MaybeDragSource {
    pub source: Entity,
}
#[query_fn]
pub fn update_select_drags(
    drags: Query<(Entity, &SelectDrag)>,
    transforms: Query<&Transform>,
    mut commands: Commands,
) {
    for drag in drags {
        let [t1, t2] = transforms
            .get_many([drag.select_drag.source, drag.select_drag.target])
            .unwrap();
        commands.trigger(MoveDragObject::new(
            drag.entity,
            t1.translation,
            t2.translation,
        ));
    }
}
#[query_fn]
pub fn update_select_maybe_drags(
    drags: Query<(Entity, &MaybeDragSource)>,
    transforms: Query<&Transform>,
    mut commands: Commands,
    spatial: Spatial,
) {
    let Some((_, target, _)) = spatial.ray() else {
        return;
    };
    for drag in drags {
        let t1 = transforms.get(drag.maybe_drag_source.source).unwrap();
        commands.trigger(MoveDragObject::new(drag.entity, t1.translation, target));
    }
}
#[derive(Component)]
pub struct TempSelect;
#[query_fn]
pub fn add_select_drags(
    mut commands: Commands,
    keybinds: Res<ButtonInput<Keybind>>,
    hovered: Query<Entity, With<HoveredObject>>,
    can_select: Query<(), With<SelectableObject>>,
    assets: AssetManager,
    maybe_drags: Query<(Entity, &MaybeDragSource)>,
    select_drags: Query<(Entity, &SelectDrag), With<TempSelect>>,
    spatial: Spatial,
) {
    if keybinds.just_pressed(Keybind::SelectDrag) {
        for entity in hovered {
            if !can_select.contains(entity) {
                return;
            }
            commands.spawn((
                MaybeDragSource { source: entity },
                DragObject::empty(&assets),
            ));
        }
    } else if keybinds.just_released(Keybind::SelectDrag) {
        for ping in maybe_drags {
            commands.entity(ping.entity).despawn();
        }
        for temp in select_drags {
            commands.entity(temp.entity).remove::<TempSelect>();
        }
    } else if let Some((hit, _, _)) = spatial.ray() {
        for temp in select_drags {
            if hit.entity == temp.select_drag.target {
                continue;
            }
            commands.entity(temp.entity).despawn();
            if can_select.contains(hit.entity) {
                commands.spawn((
                    SelectDrag {
                        source: temp.select_drag.source,
                        target: hit.entity,
                    },
                    TempSelect,
                    DragObject::empty(&assets),
                ));
            } else {
                commands.spawn((
                    MaybeDragSource {
                        source: temp.select_drag.source,
                    },
                    DragObject::empty(&assets),
                ));
            }
        }
        if !can_select.contains(hit.entity) {
            return;
        }
        for drag in maybe_drags {
            if drag.maybe_drag_source.source == hit.entity {
                continue;
            }
            commands.entity(drag.entity).despawn();
            commands.spawn((
                SelectDrag {
                    source: drag.maybe_drag_source.source,
                    target: hit.entity,
                },
                TempSelect,
                DragObject::empty(&assets),
            ));
        }
    }
}
