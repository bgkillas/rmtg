use crate::assets::AssetManager;
use crate::events::hover::Hoverable;
use crate::events::ping_drag::{DragObject, MoveDragObject, PingDrag};
use crate::keybinds::Keybind;
use crate::spatial::Spatial;
use bevy::input::ButtonInput;
use bevy::prelude::Transform;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::lifecycle::Remove;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::Single;
use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, Query, Res};
use bevy_query_fn_macro::query_fn;
#[derive(Event)]
pub struct NewSelectDrag {
    pub source: Entity,
    pub target: Entity,
}
#[derive(Component)]
pub struct SelectDrag {
    pub source: Entity,
    pub target: Entity,
}
#[derive(Component)]
pub struct SelectDragSource {
    pub target: Entity,
    pub entity: Entity,
}
#[derive(Component)]
pub struct SelectDragTarget {
    pub source: Entity,
    pub entity: Entity,
}
#[derive(Component)]
pub struct MaybeDragSource {
    pub source: Entity,
}
#[query_fn]
pub fn on_select_drag_removed(
    event: On<Remove, SelectDrag>,
    sources: Query<&SelectDrag>,
    mut commands: Commands,
) {
    let Ok(source) = sources.get(event.entity) else {
        return;
    };
    commands.entity(source.source).try_despawn();
    commands.entity(source.target).try_despawn();
}
#[query_fn]
pub fn on_select_drag_source_removed(
    event: On<Remove, SelectDragSource>,
    sources: Query<&SelectDragSource>,
    mut commands: Commands,
) {
    let Ok(source) = sources.get(event.entity) else {
        return;
    };
    commands.entity(source.entity).try_despawn();
    commands.entity(source.target).try_despawn();
}
#[query_fn]
pub fn on_select_drag_target_removed(
    event: On<Remove, SelectDragTarget>,
    targets: Query<&SelectDragTarget>,
    mut commands: Commands,
) {
    let Ok(target) = targets.get(event.entity) else {
        return;
    };
    commands.entity(target.entity).try_despawn();
    commands.entity(target.source).try_despawn();
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
pub fn add_select_drags(
    mut commands: Commands,
    keybinds: Res<ButtonInput<Keybind>>,
    spatial: Spatial,
    transforms: Query<&Transform, With<Hoverable>>,
    assets: AssetManager,
    ping_drag: Option<Single<(Entity, &MaybeDragSource), With<PingDrag>>>,
) {
    let Some((hit, target, _)) = spatial.ray() else {
        return;
    };
    if keybinds.just_pressed(Keybind::SelectDrag) {
        let Ok(transform) = transforms.get(hit.entity) else {
            return;
        };
        commands.spawn((
            PingDrag {
                from: transform.translation,
            },
            MaybeDragSource { source: hit.entity },
            DragObject::bundle(&assets, transform.translation, target),
        ));
    } else if keybinds.just_released(Keybind::SelectDrag)
        && let Some(drag) = ping_drag
    {
        commands.entity(drag.entity).despawn();
        let Ok(transform) = transforms.get(hit.entity) else {
            return;
        };
        if drag.maybe_drag_source.source != hit.entity {
            commands.spawn((
                SelectDrag {
                    source: drag.maybe_drag_source.source,
                    target: hit.entity,
                },
                DragObject::bundle(&assets, transform.translation, target),
            ));
        }
    }
}
