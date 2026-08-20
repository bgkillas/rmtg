#![expect(clippy::shadow_reuse)]
use crate::events::hover::HoveredObject;
use crate::pile::Pile;
use crate::shapes::{FaceNumber, Shape};
use avian3d::spatial_query::{RayHitData, SpatialQuery, SpatialQueryFilter};
use bevy::camera::{Camera, Camera3d};
use bevy::ecs::system::SystemParam;
use bevy::math::{Dir3, Ray3d, Vec2, Vec3};
use bevy::prelude::{GlobalTransform, InfinitePlane3d, Single, Transform, With};
use bevy::window::{PrimaryWindow, Window};
use bevy_ecs::query::{QueryData, Without};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Res, ResMut};
#[derive(QueryData)]
pub struct CameraQuery {
    pub camera: &'static Camera,
    pub transform: &'static Transform,
}
#[derive(SystemParam)]
pub struct Spatial<'w, 's> {
    pub spatial: SpatialQuery<'w, 's>,
    pub camera: Single<
        'w,
        's,
        CameraQuery,
        (
            With<Camera3d>,
            Without<Shape>,
            Without<Pile>,
            Without<FaceNumber>,
            Without<HoveredObject>,
        ),
    >,
    pub window: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
    pub cursor: Res<'w, Cursor>,
}
#[derive(Resource, Default)]
pub struct Cursor {
    pub pos: Vec2,
}
pub fn update_cursor(window: Single<&Window, With<PrimaryWindow>>, mut cursor: ResMut<Cursor>) {
    if let Some(cur) = window.cursor_position() {
        cursor.pos = cur;
    }
}
impl Spatial<'_, '_> {
    #[must_use]
    pub fn ray(&self) -> Option<(RayHitData, Vec3, Vec3)> {
        let ray = self.cam_ray()?;
        let hit = self.spatial.cast_ray(
            ray.origin,
            ray.direction,
            f32::MAX,
            true,
            &SpatialQueryFilter::default(),
        );
        let dist = ray.intersect_plane(Vec3::splat(0.0), InfinitePlane3d::new(Dir3::Y))?;
        hit.map(|data| {
            (
                data,
                ray.origin + ray.direction * data.distance,
                ray.origin + ray.direction * dist,
            )
        })
    }
    #[must_use]
    pub fn cam_ray(&self) -> Option<Ray3d> {
        self.camera
            .camera
            .viewport_to_world(
                &GlobalTransform::from_isometry(self.camera.transform.to_isometry()),
                self.cursor.pos,
            )
            .ok()
    }
    #[must_use]
    pub fn cam_center_ray(&self) -> Option<Ray3d> {
        self.camera
            .camera
            .viewport_to_world(
                &GlobalTransform::from_isometry(self.camera.transform.to_isometry()),
                self.window.size() / 2.0,
            )
            .ok()
    }
}
