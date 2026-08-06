use crate::focus::Focus;
use crate::keybinds::{Keybind, Keybinds};
use crate::net::{Peer, Peers};
use crate::spatial::Spatial;
use crate::{CARD_HEIGHT, CARD_THICKNESS, MAT_WIDTH, START_Y, W};
use bevy::camera::Camera3d;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::math::{Dir3, EulerRot, Quat, Vec2, Vec3};
use bevy::prelude::{InfinitePlane3d, Res, Single, Transform, With};
use bevy::time::Time;
use std::f32::consts::PI;
pub fn camera_translation(
    keybinds: Keybinds,
    mouse_motion: Res<AccumulatedMouseScroll>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
    focus: Focus,
    peers: Res<Peers>,
    time: Res<Time>,
    spatial: Spatial,
) {
    let Some(ray) = spatial.cam_center_ray() else {
        return;
    };
    let Some(ray_time) = ray.intersect_plane(Vec3::default(), InfinitePlane3d { normal: Dir3::Y })
    else {
        return;
    };
    let scale = MAT_WIDTH * time.delta_secs() * ray_time.max(CARD_HEIGHT) / W * 2.0;
    let fast_scale = scale * 2.0;
    let apply = |keybind: Keybind, fun: fn(&Transform) -> Dir3, scale: f32, cam: &mut Transform| {
        if keybinds.pressed(keybind) {
            let trans = fun(cam).as_vec3() * scale;
            let mut norm = trans.normalize();
            norm.y = 0.0;
            let abs = norm.length();
            if abs != 0.0 {
                cam.translation += norm * trans.length() / abs;
            }
        }
    };
    apply(Keybind::Up, Transform::forward, scale, &mut camera);
    apply(Keybind::Left, Transform::left, scale, &mut camera);
    apply(Keybind::Right, Transform::right, scale, &mut camera);
    apply(Keybind::Down, Transform::back, scale, &mut camera);
    apply(Keybind::UpFast, Transform::forward, fast_scale, &mut camera);
    apply(Keybind::LeftFast, Transform::left, fast_scale, &mut camera);
    apply(
        Keybind::RightFast,
        Transform::right,
        fast_scale,
        &mut camera,
    );
    apply(Keybind::DownFast, Transform::back, fast_scale, &mut camera);
    if mouse_motion.delta.y != 0.0 && !focus.mouse_lock() {
        let mut translate = camera.forward().as_vec3() * MAT_WIDTH * mouse_motion.delta.y / 1024.0;
        if mouse_motion.unit == MouseScrollUnit::Line {
            translate *= MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR;
        }
        if camera.translation.y + translate.y <= 0.0 {
            camera.translation += ray.direction * (ray_time / 2.0);
        } else {
            camera.translation += translate;
        }
    }
    let epsilon = Vec3::splat(2.0 * CARD_THICKNESS);
    camera.translation = camera.translation.clamp(
        Vec3::new(-W, 0.0, -W) + epsilon,
        Vec3::new(W, 2.0 * W, W) - epsilon,
    );
    if keybinds.just_pressed(Keybind::Reset) {
        **camera = default_cam_pos(peers.my_id.unwrap_or_default());
    }
}
#[must_use]
pub fn default_cam_pos(n: Peer) -> Transform {
    let (rev_x, rev_z) = match n.id {
        0 => (false, false),
        1 => (true, false),
        2 => (true, true),
        _ => (false, true),
    };
    let x = if rev_x {
        -MAT_WIDTH / 2.0
    } else {
        MAT_WIDTH / 2.0
    };
    let z = if rev_z { -MAT_WIDTH } else { MAT_WIDTH };
    Transform::from_xyz(x, START_Y, z).looking_at(Vec3::new(x, 0.0, 0.0), Vec3::Y)
}
pub fn camera_rotation(
    keybinds: Keybinds,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
    spatial: Spatial,
) {
    if keybinds.pressed(Keybind::Rotate) && mouse_motion.delta != Vec2::ZERO {
        let Some(ray) = spatial.cam_center_ray() else {
            return;
        };
        let delta_yaw = -mouse_motion.delta.x * 0.001;
        let delta_pitch = -mouse_motion.delta.y * 0.001;
        let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);
        let new_yaw = yaw + delta_yaw;
        let new_pitch = (pitch + delta_pitch)
            .max((-PI / 2.0).next_up())
            .min(-PI / 12.0);
        camera.rotation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, roll);
        let Some(time) = ray.intersect_plane(Vec3::default(), InfinitePlane3d { normal: Dir3::Y })
        else {
            return;
        };
        let orig = camera.translation + ray.direction * time;
        camera.translation = orig - camera.rotation * Dir3::NEG_Z * time;
    }
}
