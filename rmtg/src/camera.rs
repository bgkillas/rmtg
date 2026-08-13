use crate::focus::Focus;
use crate::keybinds::Keybind;
use crate::net::{Peer, Peers};
use crate::spatial::Spatial;
use crate::{CARD_HEIGHT, CARD_THICKNESS, MAT_WIDTH, START_Y, W};
use bevy::camera::Camera3d;
use bevy::input::ButtonInput;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::math::{Dir3, EulerRot, Quat, Vec2, Vec3};
use bevy::prelude::{InfinitePlane3d, Res, Transform};
use bevy::time::Time;
use bevy_ecs::component::Component;
use bevy_ecs::query::With;
use bevy_ecs::system::{ParamSet, Single};
use bevy_query_fn_macro::query_fn;
use std::f32::consts::PI;
#[derive(Component, Default)]
pub struct CameraVelocity {
    pub vec: Vec3,
}
#[query_fn]
pub fn camera_translation(
    keybinds: Res<ButtonInput<Keybind>>,
    mouse_motion: Res<AccumulatedMouseScroll>,
    focus: Focus,
    peers: Res<Peers>,
    time: Res<Time>,
    mut spatial: ParamSet<(
        Spatial,
        Single<(&mut Transform, &mut CameraVelocity), With<Camera3d>>,
    )>,
) {
    let Some(ray) = spatial.p0().cam_center_ray() else {
        return;
    };
    let mut camera = spatial.p1();
    camera.camera_velocity.vec = Vec3::splat(0.0);
    let Some(ray_time) = ray.intersect_plane(Vec3::default(), InfinitePlane3d { normal: Dir3::Y })
    else {
        return;
    };
    let scale = MAT_WIDTH * time.delta_secs() * ray_time.max(CARD_HEIGHT) / W * 2.0;
    let fast_scale = scale * 2.0;
    let mut apply = |keybind: Keybind, fun: fn(&Transform) -> Dir3, scale: f32| {
        if keybinds.pressed(keybind) {
            let trans = fun(&camera.transform).as_vec3() * scale;
            let mut norm = trans.normalize();
            norm.y = 0.0;
            let abs = norm.length();
            if abs != 0.0 {
                let delta = norm * trans.length() / abs;
                camera.transform.translation += delta;
                camera.camera_velocity.vec += delta;
            }
        }
    };
    apply(Keybind::Up, Transform::forward, scale);
    apply(Keybind::Left, Transform::left, scale);
    apply(Keybind::Right, Transform::right, scale);
    apply(Keybind::Down, Transform::back, scale);
    apply(Keybind::UpFast, Transform::forward, fast_scale);
    apply(Keybind::LeftFast, Transform::left, fast_scale);
    apply(Keybind::RightFast, Transform::right, fast_scale);
    apply(Keybind::DownFast, Transform::back, fast_scale);
    camera.camera_velocity.vec /= time.delta_secs();
    if mouse_motion.delta.y != 0.0 && !focus.mouse_lock() {
        let mut translate = camera.transform.forward().as_vec3() * MAT_WIDTH * mouse_motion.delta.y
            / 1024.0
            * ray_time.max(CARD_HEIGHT)
            / W
            * 2.0;
        if mouse_motion.unit == MouseScrollUnit::Line {
            translate *= MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR;
        }
        if camera.transform.translation.y + translate.y <= 0.0 {
            camera.transform.translation += ray.direction * (ray_time / 2.0);
        } else {
            camera.transform.translation += translate;
        }
    }
    let epsilon = Vec3::splat(2.0 * CARD_THICKNESS);
    camera.transform.translation = camera.transform.translation.clamp(
        Vec3::new(-W, 0.0, -W) + epsilon,
        Vec3::new(W, 2.0 * W, W) - epsilon,
    );
    if keybinds.just_pressed(Keybind::Reset) {
        *camera.transform = default_cam_pos(peers.my_id.unwrap_or_default());
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
    keybinds: Res<ButtonInput<Keybind>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut spatial: ParamSet<(Spatial, Single<&mut Transform, With<Camera3d>>)>,
) {
    if keybinds.pressed(Keybind::Rotate) && mouse_motion.delta != Vec2::ZERO {
        let Some(ray) = spatial.p0().cam_center_ray() else {
            return;
        };
        let mut camera = spatial.p1();
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
