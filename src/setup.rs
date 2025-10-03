use crate::download::get_from_img;
#[cfg(feature = "steam")]
use crate::sync::Packet;
#[cfg(feature = "steam")]
use crate::sync::SendSleeping;
use crate::sync::{Shape, SyncObjectMe, spawn_hand};
use crate::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_rand::global::GlobalRng;
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAnchor, TextAtlas};
use bytes::Bytes;
#[cfg(feature = "steam")]
use net::{Client, ClientTrait, Reliability};
#[cfg(feature = "steam")]
use std::collections::HashMap;
#[cfg(feature = "steam")]
use std::collections::hash_map::Entry::Vacant;
#[cfg(feature = "steam")]
use std::env::args;
use std::f32::consts::PI;
use std::fs;
const MAT_SCALE: f32 = 10.0;
pub const MAT_WIDTH: f32 = 872.0 * MAT_SCALE;
pub const MAT_HEIGHT: f32 = 525.0 * MAT_SCALE;
pub const T: f32 = 256.0;
pub const W: f32 = 16384.0;
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut framepace: ResMut<FramepaceSettings>,
    #[cfg(feature = "steam")] mut client: ResMut<Client>,
    mut rand: Single<&mut WyRand, With<GlobalRng>>,
    mut count: ResMut<SyncCount>,
    #[cfg(feature = "steam")] send_sleep: Res<SendSleeping>,
) {
    #[cfg(feature = "steam")]
    {
        let who = Arc::new(Mutex::new(HashMap::new()));
        let who2 = who.clone();
        let send = send_sleep.0.clone();
        client
            .init_steam(
                Some(Box::new(move |client, peer| {
                    if client.is_host() {
                        let mut k = 1;
                        {
                            let mut who = who.lock().unwrap();
                            loop {
                                if let Vacant(e) = who.entry(k) {
                                    e.insert(peer);
                                    break;
                                }
                                k += 1;
                            }
                        }
                        client
                            .send_message(peer, &Packet::SetUser(k), Reliability::Reliable)
                            .unwrap();
                    }
                    send.store(true, std::sync::atomic::Ordering::Relaxed);
                })),
                Some(Box::new(move |client, peer| {
                    if client.is_host() {
                        let mut who = who2.lock().unwrap();
                        who.retain(|_, p| *p != peer)
                    }
                })),
            )
            .unwrap();
        let mut next = false;
        let mut lobby = None;
        let mut f = |arg: &str| {
            if arg == "+connect_lobby" {
                next = true;
            } else if next {
                lobby = Some(arg.parse::<u64>().unwrap());
            }
        };
        for arg in args().skip(1) {
            f(&arg)
        }
        for arg in client.args().split(' ') {
            f(arg)
        }
        if let Some(lobby) = lobby {
            client.join_steam(lobby).unwrap();
        }
    }
    let font = include_bytes!("../assets/noto.ttf");
    let font = asset_server.add(Font::try_from_bytes(font.to_vec()).unwrap());
    commands.insert_resource(FontRes(font.clone()));
    let _ = fs::create_dir("./cache");
    framepace.limiter = Limiter::from_framerate(60.0);
    let card_stock = meshes.add(Rectangle::new(CARD_WIDTH, CARD_HEIGHT));
    let bytes = include_bytes!("../assets/back.jpg");
    let card_back = get_from_img(Bytes::from(bytes.as_slice()), &asset_server).unwrap();
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(card_back),
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        ..default()
    });
    let card_side = materials.add(StandardMaterial {
        base_color: bevy::prelude::Color::srgb_u8(0x11, 0x0F, 0x02),
        unlit: true,
        ..default()
    });
    commands.insert_resource(CardBase {
        stock: card_stock,
        back: material_handle,
        side: card_side,
    });
    let bytes = include_bytes!("../assets/mat.png");
    let mat = get_from_img(Bytes::from(bytes.as_slice()), &asset_server).unwrap();
    let playmat = materials.add(StandardMaterial {
        base_color_texture: Some(mat),
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        ..default()
    });
    let mat_mesh = meshes.add(Rectangle::new(MAT_WIDTH, MAT_HEIGHT));
    let mut transform = Transform::from_xyz(MAT_WIDTH / 2.0, 0.0, MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    commands.spawn((
        Mesh3d(mat_mesh.clone()),
        MeshMaterial3d(playmat.clone()),
        transform,
    ));
    let mut transform = Transform::from_xyz(-MAT_WIDTH / 2.0, 0.0, MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    commands.spawn((
        Mesh3d(mat_mesh.clone()),
        MeshMaterial3d(playmat.clone()),
        transform,
    ));
    let mut transform = Transform::from_xyz(MAT_WIDTH / 2.0, 0.0, -MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    transform.rotate_y(PI);
    commands.spawn((
        Mesh3d(mat_mesh.clone()),
        MeshMaterial3d(playmat.clone()),
        transform,
    ));
    let mut transform = Transform::from_xyz(-MAT_WIDTH / 2.0, 0.0, -MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    transform.rotate_y(PI);
    commands.spawn((Mesh3d(mat_mesh), MeshMaterial3d(playmat), transform));
    spawn_hand(0, &mut commands);
    commands.spawn((
        Transform::from_xyz(0.0, -T, 0.0),
        Collider::cuboid(2.0 * W, 2.0 * T, 2.0 * W),
        RigidBody::Static,
        Floor,
        Mesh3d(meshes.add(Cuboid::new(2.0 * W, 2.0 * T - 2.0, 2.0 * W))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::BLACK,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(0.0, 2.0 * (W - T), 0.0),
        Collider::cuboid(2.0 * W, 2.0 * T, 2.0 * W),
        RigidBody::Static,
        Ceiling,
        Mesh3d(meshes.add(Cuboid::new(2.0 * W, 2.0 * T, 2.0 * W))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::BLACK,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(W + T / 2.0, W - T, 0.0),
        Collider::cuboid(2.0 * T, 2.0 * W, 2.0 * W),
        RigidBody::Static,
        Wall,
        Mesh3d(meshes.add(Cuboid::new(2.0 * T, 2.0 * W, 2.0 * W))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::BLACK,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(-(W + T / 2.0), W - T, 0.0),
        Collider::cuboid(2.0 * T, 2.0 * W, 2.0 * W),
        RigidBody::Static,
        Wall,
        Mesh3d(meshes.add(Cuboid::new(2.0 * T, 2.0 * W, 2.0 * W))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::BLACK,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(0.0, W - T, W + T / 2.0),
        Collider::cuboid(2.0 * W, 2.0 * W, 2.0 * T),
        RigidBody::Static,
        Wall,
        Mesh3d(meshes.add(Cuboid::new(2.0 * W, 2.0 * W, 2.0 * T))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::BLACK,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(0.0, W - T, -(W + T / 2.0)),
        Collider::cuboid(2.0 * W, 2.0 * W, 2.0 * T),
        RigidBody::Static,
        Wall,
        Mesh3d(meshes.add(Cuboid::new(2.0 * W, 2.0 * W, 2.0 * T))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::BLACK,
            ..default()
        })),
    ));
    commands.spawn((
        Camera3d::default(),
        Msaa::Sample8,
        Transform::from_xyz(0.0, START_Y, START_Z).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let mut cube = spawn_cube(
        256.0,
        Transform::from_xyz(0.0, 128.0, 0.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    cube.insert(SyncObjectMe::new(&mut rand, &mut count));
    let mut ico = spawn_ico(
        96.0,
        Transform::from_xyz(320.0, 128.0, 0.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    ico.insert(SyncObjectMe::new(&mut rand, &mut count));
    let mut ico = spawn_ico(
        96.0,
        Transform::from_xyz(640.0, 128.0, 0.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    ico.insert(SyncObjectMe::new(&mut rand, &mut count));
    let mut dodec = spawn_dodec(
        96.0,
        Transform::from_xyz(896.0, 128.0, 0.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    dodec.insert(SyncObjectMe::new(&mut rand, &mut count));
    let mut coin = spawn_coin(
        96.0,
        Transform::from_xyz(0.0, 128.0, 256.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    coin.insert(SyncObjectMe::new(&mut rand, &mut count));
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        EscMenu,
        Visibility::Hidden,
        BackgroundColor(bevy::color::Color::srgba_u8(0, 0, 0, 127)),
    ));
    #[cfg(feature = "steam")]
    commands.spawn((
        Node {
            width: Val::Px(0.0),
            height: Val::Px(0.0),
            ..default()
        },
        Text(String::new()),
        SteamInfo,
        EscMenu,
        Visibility::Hidden,
        TextFont {
            font,
            font_size: FONT_SIZE,
            ..default()
        },
    ));
}
#[derive(Component)]
pub struct EscMenu;
#[cfg(feature = "steam")]
#[derive(Component)]
pub struct SteamInfo;
#[derive(Resource)]
#[allow(dead_code)]
pub struct FontRes(Handle<Font>);
pub fn spawn_cube<'a>(
    m: f32,
    transform: Transform,
    commands: &'a mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> EntityCommands<'a> {
    let d = m / 2.0 + 1.0;
    let mut cube = commands.spawn((
        Collider::cuboid(m, m, m),
        transform,
        RigidBody::Dynamic,
        LinearDamping(LIN_DAMPING),
        AngularDamping(ANG_DAMPING),
        SLEEP,
        GravityScale(GRAVITY),
        Shape::Cube,
        Mesh3d(meshes.add(Cuboid::from_length(m))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::WHITE,
            unlit: true,
            ..default()
        })),
    ));
    cube.with_children(|parent| {
        for i in 1..=6 {
            let (x, y, z) = match i {
                1 => (0.0, d, 0.0),
                2 => (d, 0.0, 0.0),
                3 => (0.0, 0.0, d),
                4 => (0.0, 0.0, -d),
                5 => (-d, 0.0, 0.0),
                6 => (0.0, -d, 0.0),
                _ => unreachable!(),
            };
            parent.spawn((
                Transform::from_xyz(x, y, z).looking_at(Vec3::default(), Dir3::Z),
                Text3d::new(i.to_string()),
                Side(i),
                Mesh3d(meshes.add(Rectangle::new(m / 2.0, m / 2.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
                    alpha_mode: AlphaMode::Premultiplied,
                    base_color: bevy::prelude::Color::BLACK,
                    unlit: true,
                    ..default()
                })),
                Text3dStyling {
                    size: m / 2.0,
                    anchor: TextAnchor::CENTER,
                    ..default()
                },
            ));
        }
    });
    cube
}
pub fn spawn_ico<'a>(
    m: f32,
    transform: Transform,
    commands: &'a mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> EntityCommands<'a> {
    let phi = ((0.5 + 5.0f64.sqrt() / 2.0) * m as f64) as f32;
    let mut verticies: Vec<[f32; 3]> = Vec::with_capacity(12);
    for y in [-m, m] {
        for z in [-phi, phi] {
            verticies.push([0.0, y, z])
        }
    }
    for x in [-m, m] {
        for y in [-phi, phi] {
            verticies.push([x, y, 0.0])
        }
    }
    for x in [-phi, phi] {
        for z in [-m, m] {
            verticies.push([x, 0.0, z])
        }
    }
    let mut f = Vec::with_capacity(60);
    for (i, a) in verticies.iter().enumerate() {
        let [ax, ay, az] = a;
        let an = ax * ax + ay * ay + az * az;
        for (j, b) in verticies.iter().enumerate() {
            let [bx, by, bz] = b;
            let bn = bx * bx + by * by + bz * bz;
            let t = (ax * bx + ay * by + az * bz) / (an * bn).sqrt();
            let t = t.acos();
            if (t - 1.1071488).abs() < 0.125 {
                f.push([i as u16, j as u16]);
            }
        }
    }
    let mut indecies = Vec::with_capacity(60);
    let mut faces = Vec::with_capacity(20);
    for a in &f {
        for b in &f {
            for c in &f {
                if a[1] == b[0] && b[1] == c[0] && c[1] == a[0] && a[0] < b[0] && b[0] < c[0] {
                    let [ox, oy, oz] = verticies[a[0] as usize];
                    let u = verticies[b[0] as usize];
                    let v = verticies[c[0] as usize];
                    let n = [
                        u[1] * v[2] - u[2] * v[1],
                        u[2] * v[0] - u[0] * v[2],
                        u[0] * v[1] - u[1] * v[0],
                    ];
                    let dot = n[0] * ox + n[1] * oy + n[2] * oz;
                    indecies.push(a[0]);
                    if dot > 0.0 {
                        indecies.push(b[0]);
                        indecies.push(c[0]);
                    } else {
                        indecies.push(c[0]);
                        indecies.push(b[0]);
                    }
                    faces.push([
                        (ox + u[0] + v[0]) / 3.0,
                        (oy + u[1] + v[1]) / 3.0,
                        (oz + u[2] + v[2]) / 3.0,
                    ])
                }
            }
        }
    }
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verticies)
    .with_inserted_indices(Indices::U16(indecies));
    let mut ent = commands.spawn((
        Collider::convex_hull_from_mesh(&mesh).unwrap(),
        transform,
        Shape::Icosahedron,
        RigidBody::Dynamic,
        LinearDamping(LIN_DAMPING),
        AngularDamping(ANG_DAMPING),
        SLEEP,
        GravityScale(GRAVITY),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::WHITE,
            unlit: true,
            ..default()
        })),
    ));
    ent.with_children(|parent| {
        for (i, [mut x, mut y, mut z]) in faces.into_iter().enumerate() {
            if x < 0.0 {
                x -= 1.0;
            } else {
                x += 1.0;
            }
            if y < 0.0 {
                y -= 1.0;
            } else {
                y += 1.0;
            }
            if z < 0.0 {
                z -= 1.0;
            } else {
                z += 1.0;
            }
            parent.spawn((
                Transform::from_xyz(x, y, z).looking_at(Vec3::default(), Dir3::Z),
                Text3d::new((i + 1).to_string()),
                Side(i + 1),
                Mesh3d(meshes.add(Rectangle::new(m / 2.0, m / 2.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
                    alpha_mode: AlphaMode::Premultiplied,
                    base_color: bevy::prelude::Color::BLACK,
                    unlit: true,
                    ..default()
                })),
                Text3dStyling {
                    size: m / 2.0,
                    anchor: TextAnchor::CENTER,
                    ..default()
                },
            ));
        }
    });
    ent
}
pub fn spawn_coin<'a>(
    m: f32,
    transform: Transform,
    commands: &'a mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> EntityCommands<'a> {
    let ratio = 8.0;
    let mut ent = commands.spawn((
        Collider::cylinder(m, m / ratio),
        transform,
        Shape::Disc,
        ColliderDensity(1.0 / 32.0),
        RigidBody::Dynamic,
        LinearDamping(LIN_DAMPING),
        AngularDamping(ANG_DAMPING),
        SLEEP,
        GravityScale(GRAVITY),
        Mesh3d(meshes.add(Cylinder::new(m, m / ratio))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::WHITE,
            unlit: true,
            ..default()
        })),
    ));
    ent.with_children(|parent| {
        for (i, [mut y]) in [[m / ratio], [-m / ratio]].into_iter().enumerate() {
            if y < 0.0 {
                y -= 1.0;
            } else {
                y += 1.0;
            }
            parent.spawn((
                Transform::from_xyz(0.0, y, 0.0).looking_at(Vec3::default(), Dir3::Z),
                Text3d::new(i.to_string()),
                Side(i),
                Mesh3d(meshes.add(Rectangle::new(m / 2.0, m / 2.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
                    alpha_mode: AlphaMode::Premultiplied,
                    base_color: bevy::prelude::Color::BLACK,
                    unlit: true,
                    ..default()
                })),
                Text3dStyling {
                    size: m / 2.0,
                    anchor: TextAnchor::CENTER,
                    ..default()
                },
            ));
        }
    });
    ent
}
#[allow(dead_code)]
#[derive(Component)]
pub struct Side(pub usize);
pub fn spawn_dodec<'a>(
    m: f32,
    transform: Transform,
    commands: &'a mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> EntityCommands<'a> {
    let phi = (0.5 + 5.0f64.sqrt() / 2.0) * m as f64;
    let phir = phi.recip() as f32;
    let phi = phi as f32;
    let mut verticies: Vec<[f32; 3]> = Vec::with_capacity(20);
    for x in [-m, m] {
        for y in [-m, m] {
            for z in [-m, m] {
                verticies.push([x, y, z])
            }
        }
    }
    for y in [-phir, phir] {
        for z in [-phi, phi] {
            verticies.push([0.0, y, z])
        }
    }
    for x in [-phir, phir] {
        for y in [-phi, phi] {
            verticies.push([x, y, 0.0])
        }
    }
    for x in [-phi, phi] {
        for z in [-phir, phir] {
            verticies.push([x, 0.0, z])
        }
    }
    let mut f = Vec::with_capacity(60);
    for (i, a) in verticies.iter().enumerate() {
        let [ax, ay, az] = a;
        let an = ax * ax + ay * ay + az * az;
        for (j, b) in verticies.iter().enumerate() {
            let [bx, by, bz] = b;
            let bn = bx * bx + by * by + bz * bz;
            let t = (ax * bx + ay * by + az * bz) / (an * bn).sqrt();
            let t = t.acos();
            if (t - 0.9552873).abs() < 0.125 {
                f.push([i as u16, j as u16]);
            }
        }
    }
    let mut indecies = Vec::with_capacity(60);
    let mut faces = Vec::with_capacity(12);
    for a in &f {
        for b in &f {
            for c in &f {
                for d in &f {
                    for e in &f {
                        if a[1] == b[0]
                            && b[1] == c[0]
                            && c[1] == d[0]
                            && d[1] == e[0]
                            && e[1] == a[0]
                            //&& a[0] < b[0]
                            //&& b[0] < c[0]
                            //&& c[0] < d[0]
                            //&& d[0] < e[0]
                        {
                            let [ox, oy, oz] = verticies[a[0] as usize];
                            let u = verticies[b[0] as usize];
                            let v = verticies[c[0] as usize];
                            let x = verticies[d[0] as usize];
                            let y = verticies[e[0] as usize];
                            let n = [
                                u[1] * v[2] - u[2] * v[1],
                                u[2] * v[0] - u[0] * v[2],
                                u[0] * v[1] - u[1] * v[0],
                            ];
                            let dot = n[0] * ox + n[1] * oy + n[2] * oz;
                            indecies.push(a[0]);
                            if dot > 0.0 {
                                indecies.push(b[0]);
                                indecies.push(verticies.len() as u16);
                                indecies.push(b[0]);
                                indecies.push(c[0]);
                                indecies.push(verticies.len() as u16);
                                indecies.push(c[0]);
                                indecies.push(d[0]);
                                indecies.push(verticies.len() as u16);
                                indecies.push(d[0]);
                                indecies.push(e[0]);
                                indecies.push(verticies.len() as u16);
                            } else {
                                indecies.push(e[0]);
                                indecies.push(verticies.len() as u16);
                                indecies.push(e[0]);
                                indecies.push(d[0]);
                                indecies.push(verticies.len() as u16);
                                indecies.push(d[0]);
                                indecies.push(c[0]);
                                indecies.push(verticies.len() as u16);
                                indecies.push(c[0]);
                                indecies.push(b[0]);
                                indecies.push(verticies.len() as u16);
                            }
                            let a = [
                                (ox + u[0] + v[0] + x[0] + y[0]) / 5.0,
                                (oy + u[1] + v[1] + x[1] + y[1]) / 5.0,
                                (oz + u[2] + v[2] + x[2] + y[2]) / 5.0,
                            ];
                            verticies.push(a);
                            faces.push(a)
                        }
                    }
                }
            }
        }
    }
    println!("{} {}", faces.len(), f.len());
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verticies)
    .with_inserted_indices(Indices::U16(indecies));
    let mut ent = commands.spawn((
        Collider::convex_hull_from_mesh(&mesh).unwrap(),
        transform,
        Shape::Dodecahedron,
        RigidBody::Dynamic,
        LinearDamping(LIN_DAMPING),
        AngularDamping(ANG_DAMPING),
        SLEEP,
        GravityScale(GRAVITY),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::WHITE,
            unlit: true,
            ..default()
        })),
    ));
    ent.with_children(|parent| {
        for (i, [mut x, mut y, mut z]) in faces.into_iter().enumerate() {
            if x < 0.0 {
                x -= 1.0;
            } else {
                x += 1.0;
            }
            if y < 0.0 {
                y -= 1.0;
            } else {
                y += 1.0;
            }
            if z < 0.0 {
                z -= 1.0;
            } else {
                z += 1.0;
            }
            parent.spawn((
                Transform::from_xyz(x, y, z).looking_at(Vec3::default(), Dir3::Z),
                Text3d::new((i + 1).to_string()),
                Side(i + 1),
                Mesh3d(meshes.add(Rectangle::new(m / 2.0, m / 2.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
                    alpha_mode: AlphaMode::Premultiplied,
                    base_color: bevy::prelude::Color::BLACK,
                    unlit: true,
                    ..default()
                })),
                Text3dStyling {
                    size: m / 2.0,
                    anchor: TextAnchor::CENTER,
                    ..default()
                },
            ));
        }
    });
    ent
}
#[derive(Component)]
pub struct Wall;
#[derive(Component)]
pub struct Floor;
#[derive(Component)]
pub struct Ceiling;
