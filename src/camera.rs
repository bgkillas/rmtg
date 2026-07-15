use crate::focus::Focus;
use crate::keybinds::{Keybind, Keybinds};
use crate::net::Peers;
use crate::{CARD_THICKNESS, MAT_WIDTH, START_Y, W};
use bevy::camera::{Camera, Camera3d};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::math::{Dir3, EulerRot, Quat, Vec2, Vec3};
use bevy::prelude::{GlobalTransform, InfinitePlane3d, KeyCode, Res, Single, Transform, With};
use bevy::window::{PrimaryWindow, Window};
use std::f32::consts::PI;
pub fn camera_translation(
    keybinds: Keybinds,
    mouse_motion: Res<AccumulatedMouseScroll>,
    camera: Single<(&mut Transform, &Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    focus: Focus,
    peers: Res<Peers>,
) {
    if focus.key_lock() {
        return;
    }
    let (mut cam_transform, cam, cam_global) = camera.into_inner();
    let scale = CARD_THICKNESS * 4.0;
    let apply = |translate: Vec3, cam: &mut Transform| {
        let mut norm = translate.normalize();
        norm.y = 0.0;
        let abs = norm.length();
        if abs != 0.0 {
            let translate = norm * translate.length() / abs;
            cam.translation += translate;
        }
    };
    if !keybinds
        .keyboard
        .any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
    {
        if keybinds.pressed(Keybind::Up) {
            let translate = cam_transform.forward().as_vec3() * scale;
            apply(translate, &mut cam_transform)
        }
        if keybinds.pressed(Keybind::Left) {
            let translate = cam_transform.left().as_vec3() * scale;
            apply(translate, &mut cam_transform)
        }
        if keybinds.pressed(Keybind::Right) {
            let translate = cam_transform.right().as_vec3() * scale;
            apply(translate, &mut cam_transform)
        }
        if keybinds.pressed(Keybind::Down) {
            let translate = cam_transform.back().as_vec3() * scale;
            apply(translate, &mut cam_transform)
        }
        let scale = scale * 4.0;
        if keybinds.pressed(Keybind::UpFast) {
            let translate = cam_transform.forward().as_vec3() * scale;
            apply(translate, &mut cam_transform)
        }
        if keybinds.pressed(Keybind::LeftFast) {
            let translate = cam_transform.left().as_vec3() * scale;
            apply(translate, &mut cam_transform)
        }
        if keybinds.pressed(Keybind::RightFast) {
            let translate = cam_transform.right().as_vec3() * scale;
            apply(translate, &mut cam_transform)
        }
        if keybinds.pressed(Keybind::DownFast) {
            let translate = cam_transform.back().as_vec3() * scale;
            apply(translate, &mut cam_transform)
        }
    }
    if mouse_motion.delta.y != 0.0 && !focus.mouse_lock() {
        let mut translate = cam_transform.forward().as_vec3() * scale * mouse_motion.delta.y * 16.0;
        if mouse_motion.unit != MouseScrollUnit::Line {
            translate /= 4.0;
        }
        if cam_transform.translation.y + translate.y <= 0.0 {
            let Ok(ray) = cam.viewport_to_world(cam_global, window.size() / 2.0) else {
                return;
            };
            if let Some(time) =
                ray.intersect_plane(Vec3::default(), InfinitePlane3d { normal: Dir3::Y })
            {
                cam_transform.translation += ray.direction * (time / 2.0);
            }
        } else {
            cam_transform.translation += translate;
        }
    }
    let epsilon = Vec3::splat(CARD_THICKNESS);
    cam_transform.translation = cam_transform.translation.clamp(
        Vec3::new(-W, 0.0, -W) + epsilon,
        Vec3::new(W, 2.0 * W, W) - epsilon,
    );
    if keybinds.just_pressed(Keybind::Reset) {
        *cam_transform = default_cam_pos(peers.my_id.unwrap_or_default());
    }
}
pub fn default_cam_pos(n: usize) -> Transform {
    let x = if n / 2 == 0 {
        MAT_WIDTH / 2.0
    } else {
        -MAT_WIDTH / 2.0
    };
    let z = if n.is_multiple_of(2) {
        MAT_WIDTH
    } else {
        -MAT_WIDTH
    };
    Transform::from_xyz(x, START_Y, z).looking_at(Vec3::new(x, 0.0, 0.0), Vec3::Y)
}
pub fn camera_rotation(
    keybinds: Keybinds,
    mouse_motion: Res<AccumulatedMouseMotion>,
    cam: Single<(&mut Transform, &Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    focus: Focus,
) {
    if focus.mouse_lock() {
        return;
    }
    let (mut cam_transform, cam, cam_global) = cam.into_inner();
    if keybinds.pressed(Keybind::Rotate) && mouse_motion.delta != Vec2::ZERO {
        let Ok(ray) = cam.viewport_to_world(cam_global, window.size() / 2.0) else {
            return;
        };
        let delta_yaw = -mouse_motion.delta.x * 0.001;
        let delta_pitch = -mouse_motion.delta.y * 0.001;
        let (yaw, pitch, roll) = cam_transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;
        let pitch = (pitch + delta_pitch)
            .max((-PI / 2.0).next_up())
            .min(-PI / 12.0);
        cam_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        let Some(time) = ray.intersect_plane(Vec3::default(), InfinitePlane3d { normal: Dir3::Y })
        else {
            return;
        };
        let orig = cam_transform.translation + ray.direction * time;
        cam_transform.translation = orig - cam_transform.rotation * Dir3::NEG_Z * time;
    }
}
