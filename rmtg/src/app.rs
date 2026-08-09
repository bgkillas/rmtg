use crate::camera::{camera_rotation, camera_translation};
use crate::drag::drag;
use crate::events::add_events;
use crate::events::clipboard::poll_clipboards;
use crate::events::clone::update_clone;
use crate::events::hover::{update_box_select, update_hover};
use crate::events::roll::{do_roll, update_rolling};
use crate::events::scale::update_scale;
use crate::focus::{Menu, update_focus};
use crate::keybinds::KeybindsList;
use crate::mat::create_mats;
use crate::net::{Msg, Peers, net_update, receive_message};
use crate::paste::paste_card;
use crate::spatial::{Cursor, update_cursor};
use crate::startup::{spawn_objects, startup};
use crate::ui::chat::text_submission;
use crate::{APP_NAME, FONT, USER_AGENT};
use avian3d::PhysicsPlugins;
use bevy::DefaultPlugins;
use bevy::app::{
    App, AppExit, FixedUpdate, PluginGroup as _, PreUpdate, Startup, TaskPoolOptions,
    TaskPoolPlugin, TaskPoolThreadAssignmentPolicy, Update,
};
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs as _;
#[cfg(feature = "colliders")]
use bevy::gizmos::AppGizmoBuilder as _;
use bevy::image::{ImageFilterMode, ImagePlugin, ImageSamplerDescriptor};
use bevy::settings::SettingsPlugin;
use bevy::window::{Window, WindowPlugin};
use bevy_framepace::FramepacePlugin;
use bevy_p2p::plugin::P2PPlugin;
use bevy_rich_text3d::{LoadFonts, Text3dPlugin};
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
                    ..Window::default()
                }),
                ..WindowPlugin::default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..AssetPlugin::default()
            })
            .set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    mag_filter: ImageFilterMode::Linear,
                    min_filter: ImageFilterMode::Linear,
                    mipmap_filter: ImageFilterMode::Linear,
                    anisotropy_clamp: 16,
                    ..ImageSamplerDescriptor::default()
                },
            })
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
    app.add_plugins(Text3dPlugin::default());
    app.add_plugins(FramepacePlugin);
    #[cfg(feature = "fps")]
    app.add_plugins(bevy::dev_tools::fps_overlay::FpsOverlayPlugin::default());
    #[cfg(feature = "colliders")]
    app.add_plugins(avian3d::debug_render::PhysicsDebugPlugin);
    #[cfg(feature = "colliders")]
    app.insert_gizmo_config(
        avian3d::debug_render::PhysicsGizmos {
            axis_lengths: None,
            collider_color: Some(bevy::color::Color::srgba_u8(0, 0, 0, 127)),
            sleeping_color_multiplier: None,
            ..avian3d::debug_render::PhysicsGizmos::default()
        },
        bevy::gizmos::config::GizmoConfig::default(),
    );
    app.insert_resource(LoadFonts {
        font_embedded: vec![FONT],
        ..LoadFonts::default()
    });
    app.init_resource::<Menu>();
    app.init_resource::<KeybindsList>();
    app.init_resource::<Peers>();
    app.init_resource::<Client>();
    app.init_resource::<Cursor>();
    add_events(&mut app);
    app.add_systems(Startup, (startup, spawn_objects, create_mats).chain());
    app.add_systems(PreUpdate, (update_cursor, update_focus));
    app.add_systems(
        Update,
        (
            (camera_rotation, camera_translation).chain(),
            (
                (
                    (update_box_select, update_hover).chain(),
                    (
                        (do_roll, update_rolling).chain(),
                        drag,
                        update_clone,
                        update_scale,
                    ),
                )
                    .chain(),
                paste_card,
            ),
            text_submission,
        )
            .chain(),
    );
    app.add_systems(
        FixedUpdate,
        ((net_update, receive_message).chain(), poll_clipboards),
    );
    app.run()
}
#[derive(Resource)]
pub struct Client {
    pub client: importer::reqwest::Client,
}
impl Default for Client {
    fn default() -> Self {
        Self {
            client: importer::reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
        }
    }
}
