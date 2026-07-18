use crate::APP_NAME;
use crate::camera::{camera_rotation, camera_translation};
use crate::focus::Menu;
use crate::keybinds::KeybindsList;
use crate::net::{Msg, Peers, connect_failed, on_connect, on_disconnect, receive_message};
use crate::startup::startup;
use avian3d::PhysicsPlugins;
use bevy::DefaultPlugins;
use bevy::app::{
    App, AppExit, FixedPostUpdate, FixedUpdate, PluginGroup as _, Startup, TaskPoolOptions,
    TaskPoolPlugin, TaskPoolThreadAssignmentPolicy, Update,
};
use bevy::asset::{AssetMetaCheck, AssetPlugin};
#[cfg(feature = "colliders")]
use bevy::gizmos::AppGizmoBuilder as _;
use bevy::image::ImagePlugin;
use bevy::prelude::MeshPickingPlugin;
use bevy::settings::SettingsPlugin;
use bevy::window::{PresentMode, Window, WindowPlugin};
use bevy_p2p::plugin::P2PPlugin;
use bevy_polyline::PolylinePlugin;
use bevy_rich_text3d::Text3dPlugin;
#[must_use]
pub fn app_run() -> AppExit {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "rmtg".to_owned(),
                    resizable: true,
                    fit_canvas_to_parent: true,
                    present_mode: PresentMode::Immediate,
                    ..Window::default()
                }),
                ..WindowPlugin::default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..AssetPlugin::default()
            })
            .set(ImagePlugin::default_nearest())
            .set(TaskPoolPlugin {
                task_pool_options: TaskPoolOptions {
                    min_total_threads: 1,
                    max_total_threads: usize::MAX,
                    io: TaskPoolThreadAssignmentPolicy {
                        min_threads: 1,
                        max_threads: 1,
                        percent: 0.25,
                        on_thread_spawn: None,
                        on_thread_destroy: None,
                    },
                    async_compute: TaskPoolThreadAssignmentPolicy {
                        min_threads: 1,
                        max_threads: 1,
                        percent: 0.25,
                        on_thread_spawn: None,
                        on_thread_destroy: None,
                    },
                    compute: TaskPoolThreadAssignmentPolicy {
                        min_threads: 1,
                        max_threads: usize::MAX,
                        percent: 1.0,
                        on_thread_spawn: None,
                        on_thread_destroy: None,
                    },
                },
            }),
    );
    app.add_plugins(PhysicsPlugins::default());
    app.add_plugins(SettingsPlugin::new(APP_NAME));
    app.add_plugins(P2PPlugin::<Msg>::new());
    app.add_plugins(MeshPickingPlugin);
    app.add_plugins(PolylinePlugin);
    app.add_plugins(Text3dPlugin::default());
    #[cfg(feature = "colliders")]
    app.add_plugins(avian2d::debug_render::PhysicsDebugPlugin);
    #[cfg(feature = "fps")]
    app.add_plugins(bevy::dev_tools::fps_overlay::FpsOverlayPlugin::default());
    #[cfg(feature = "colliders")]
    app.insert_gizmo_config(
        avian2d::debug_render::PhysicsGizmos {
            axis_lengths: None,
            collider_color: Some(bevy::color::Color::srgba_u8(0, 0, 0, 127)),
            sleeping_color_multiplier: None,
            ..avian2d::debug_render::PhysicsGizmos::default()
        },
        bevy::gizmos::config::GizmoConfig::default(),
    );
    app.insert_resource(Menu::default());
    app.insert_resource(KeybindsList::default());
    app.insert_resource(Peers::default());
    app.add_systems(Startup, startup);
    app.add_systems(Update, (camera_translation, camera_rotation));
    app.add_systems(FixedUpdate, (connect_failed, on_connect, receive_message));
    app.add_systems(FixedPostUpdate, on_disconnect);
    app.run()
}
