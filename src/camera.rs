use crate::focus::Focus;
use crate::keybinds::{Keybind, Keybinds};
use bevy::camera::{Camera, Camera3d};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::math::{Dir3, EulerRot, Quat, Vec2, Vec3};
use bevy::prelude::{GlobalTransform, InfinitePlane3d, Res, Single, Transform, With};
use bevy::window::{PrimaryWindow, Window};
use std::f32::consts::PI;
/*pub fn cam_translation(
    keybinds: Keybinds,
    mouse_motion: Res<AccumulatedMouseScroll>,
    mut cam: Single<&mut Transform, With<Camera3d>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    focus: Focus,
) {
    if focus.key_lock() {
        return;
    }
    let scale = CARD_THICKNESS * 16.0;
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
            let translate = cam.forward().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if keybinds.pressed(Keybind::Left) {
            let translate = cam.left().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if keybinds.pressed(Keybind::Right) {
            let translate = cam.right().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if keybinds.pressed(Keybind::Down) {
            let translate = cam.back().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        let scale = scale * 4.0;
        if keybinds.pressed(Keybind::UpFast) {
            let translate = cam.forward().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if keybinds.pressed(Keybind::LeftFast) {
            let translate = cam.left().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if keybinds.pressed(Keybind::RightFast) {
            let translate = cam.right().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if keybinds.pressed(Keybind::DownFast) {
            let translate = cam.back().as_vec3() * scale;
            apply(translate, &mut cam)
        }
    }
    if mouse_motion.delta.y != 0.0 && !focus.mouse_lock() {
        let mut translate = cam.forward().as_vec3() * scale * mouse_motion.delta.y * 16.0;
        if mouse_motion.unit != MouseScrollUnit::Line {
            translate /= 4.0;
        }
        if cam.translation.y + translate.y <= 0.0 {
            let (camera, camera_transform) = camera.into_inner();
            let Ok(ray) = camera.viewport_to_world(camera_transform, window.size() / 2.0) else {
                return;
            };
            if let Some(time) =
                ray.intersect_plane(Vec3::default(), InfinitePlane3d { normal: Dir3::Y })
            {
                cam.translation += ray.direction * (time / 2.0);
            }
        } else {
            cam.translation += translate;
        }
    }
    let epsilon = Vec3::splat(CARD_THICKNESS);
    cam.translation = cam.translation.clamp(
        Vec3::new(-W, 0.0, -W) + epsilon,
        Vec3::new(W, 2.0 * W, W) - epsilon,
    );
    if keybinds.just_pressed(Keybind::Reset) {
        *cam.into_inner() = default_cam_pos(peers.me.unwrap_or_default());
    }
}*/
pub fn camera_rotation(
    keybinds: Keybinds,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut cam: Single<(&mut Transform, &Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    focus: Focus,
) {
    println!("a");
    if focus.mouse_lock() {
        return;
    }
    println!("b");
    if keybinds.pressed(Keybind::Rotate) && mouse_motion.delta != Vec2::ZERO {
        println!("c");
        let Ok(ray) = cam.1.viewport_to_world(cam.2, window.size() / 2.0) else {
            return;
        };
        println!("d");
        let delta_yaw = -mouse_motion.delta.x * 0.001;
        let delta_pitch = -mouse_motion.delta.y * 0.001;
        let (yaw, pitch, roll) = cam.0.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;
        let pitch = (pitch + delta_pitch)
            .max((-PI / 2.0).next_up())
            .min(-PI / 12.0);
        cam.0.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        let Some(time) = ray.intersect_plane(Vec3::default(), InfinitePlane3d { normal: Dir3::Y })
        else {
            return;
        };
        let orig = cam.0.translation + ray.direction * time;
        cam.0.translation = orig - cam.0.rotation * Dir3::NEG_Z * time;
    }
}
