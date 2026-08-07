#![allow(clippy::shadow_reuse)]
use crate::events::hover::Hovered;
use crate::pile::Pile;
use crate::shapes::{FaceNumber, Shape};
use avian3d::spatial_query::{RayHitData, SpatialQuery, SpatialQueryFilter};
use bevy::camera::{Camera, Camera3d};
use bevy::ecs::system::SystemParam;
use bevy::math::{Ray3d, Vec3};
use bevy::prelude::{GlobalTransform, Single, Transform, With};
use bevy::window::{PrimaryWindow, Window};
use bevy_ecs::query::Without;
#[derive(SystemParam)]
pub struct Spatial<'w, 's> {
    pub spatial: SpatialQuery<'w, 's>,
    pub camera: Single<
        'w,
        's,
        (&'static Camera, &'static Transform),
        (
            With<Camera3d>,
            Without<Shape>,
            Without<Pile>,
            Without<FaceNumber>,
            Without<Hovered>,
        ),
    >,
    pub window: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
}
impl Spatial<'_, '_> {
    #[must_use]
    pub fn ray(&self) -> Option<(RayHitData, Vec3)> {
        let ray = self.cam_ray()?;
        let hit = self.spatial.cast_ray(
            ray.origin,
            ray.direction,
            f32::MAX,
            true,
            &SpatialQueryFilter::default(),
        );
        hit.map(|data| (data, ray.origin + ray.direction * data.distance))
    }
    #[must_use]
    pub fn cam_ray(&self) -> Option<Ray3d> {
        let cursor_position = self.window.cursor_position()?;
        self.camera
            .0
            .viewport_to_world(
                &GlobalTransform::from_isometry(self.camera.1.to_isometry()),
                cursor_position,
            )
            .ok()
    }
    #[must_use]
    pub fn cam_center_ray(&self) -> Option<Ray3d> {
        self.camera
            .0
            .viewport_to_world(
                &GlobalTransform::from_isometry(self.camera.1.to_isometry()),
                self.window.size() / 2.0,
            )
            .ok()
    }
}
