use crate::assets::AssetManager;
use crate::events::hover::Hoverable;
use crate::events::ping_drag::{DragObject, MoveDragObject};
use crate::keybinds::Keybind;
use crate::spatial::Spatial;
use bevy::input::ButtonInput;
use bevy::prelude::Transform;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Single;
use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, Query, Res};
use bevy_query_fn_macro::query_fn;
#[derive(Component)]
pub struct SelectDrag {
    pub source: Entity,
    pub target: Entity,
}
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
    spatial: Spatial,
    transforms: Query<&Transform, With<Hoverable>>,
    assets: AssetManager,
    ping_drag: Option<Single<(Entity, &MaybeDragSource)>>,
    select_drag: Option<Single<(Entity, &SelectDrag), With<TempSelect>>>,
) {
    if keybinds.just_pressed(Keybind::SelectDrag) {
        let Some((hit, target, _)) = spatial.ray() else {
            return;
        };
        let Ok(transform) = transforms.get(hit.entity) else {
            return;
        };
        commands.spawn((
            MaybeDragSource { source: hit.entity },
            DragObject::bundle(&assets, transform.translation, target),
        ));
    } else if keybinds.just_released(Keybind::SelectDrag) {
        if let Some(ping) = ping_drag {
            commands.entity(ping.entity).despawn();
        }
        if let Some(temp) = select_drag {
            commands.entity(temp.entity).remove::<TempSelect>();
        }
    } else {
        let Some((hit, target, _)) = spatial.ray() else {
            return;
        };
        if let Some(temp) = select_drag {
            if hit.entity == temp.select_drag.target {
                return;
            }
            let Ok(from) = transforms.get(temp.select_drag.source) else {
                return;
            };
            commands.entity(temp.entity).despawn();
            if let Ok(transform) = transforms.get(hit.entity)
                && hit.entity != temp.select_drag.target
            {
                commands.spawn((
                    SelectDrag {
                        source: temp.select_drag.source,
                        target: hit.entity,
                    },
                    TempSelect,
                    DragObject::bundle(&assets, from.translation, transform.translation),
                ));
            } else {
                commands.spawn((
                    MaybeDragSource {
                        source: temp.select_drag.source,
                    },
                    DragObject::bundle(&assets, from.translation, target),
                ));
            }
        } else if let Some(drag) = ping_drag
            && let Ok(from) = transforms.get(drag.maybe_drag_source.source)
            && drag.maybe_drag_source.source != hit.entity
        {
            let Ok(transform) = transforms.get(hit.entity) else {
                return;
            };
            commands.entity(drag.entity).despawn();
            commands.spawn((
                SelectDrag {
                    source: drag.maybe_drag_source.source,
                    target: hit.entity,
                },
                TempSelect,
                DragObject::bundle(&assets, from.translation, transform.translation),
            ));
        }
    }
}
