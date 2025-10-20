use crate::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAnchor, TextAtlas};
use std::f32::consts::FRAC_PI_2;
#[derive(Encode, Decode, Component, Copy, Clone, Debug)]
pub enum Shape {
    Cube,
    Icosahedron,
    Dodecahedron,
    Octohedron,
    Tetrahedron,
    Disc,
}
impl Shape {
    pub fn create<'a>(
        &self,
        transform: Transform,
        commands: &'a mut Commands,
        materials: &mut Assets<StandardMaterial>,
        meshes: &mut Assets<Mesh>,
    ) -> EntityCommands<'a> {
        match self {
            Shape::Cube => spawn_cube(256.0, transform, commands, meshes, materials),
            Shape::Icosahedron => spawn_ico(96.0, transform, commands, meshes, materials),
            Shape::Dodecahedron => spawn_dodec(96.0, transform, commands, meshes, materials),
            Shape::Octohedron => spawn_oct(192.0, transform, commands, meshes, materials),
            Shape::Tetrahedron => spawn_tetra(128.0, transform, commands, meshes, materials),
            Shape::Disc => spawn_coin(96.0, transform, commands, meshes, materials),
        }
    }
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
        CollisionLayers::new(0b11, LayerMask::ALL),
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
pub fn spawn_oct<'a>(
    m: f32,
    transform: Transform,
    commands: &'a mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> EntityCommands<'a> {
    let mut verticies: Vec<[f32; 3]> = Vec::with_capacity(6);
    for x in [-m, m] {
        verticies.push([x, 0.0, 0.0]);
        verticies.push([0.0, x, 0.0]);
        verticies.push([0.0, 0.0, x])
    }
    let mut f = Vec::with_capacity(24);
    for (i, a) in verticies.iter().enumerate() {
        let [ax, ay, az] = a;
        let an = ax * ax + ay * ay + az * az;
        for (j, b) in verticies.iter().enumerate() {
            let [bx, by, bz] = b;
            let bn = bx * bx + by * by + bz * bz;
            let t = (ax * bx + ay * by + az * bz) / (an * bn).sqrt();
            let t = t.acos();
            if (t - FRAC_PI_2).abs() < 0.125 {
                f.push([i as u16, j as u16]);
            }
        }
    }
    let mut indecies = Vec::with_capacity(24);
    let mut faces = Vec::with_capacity(8);
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
        CollisionLayers::new(0b11, LayerMask::ALL),
        Collider::convex_hull_from_mesh(&mesh).unwrap(),
        transform,
        Shape::Octohedron,
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
pub fn spawn_tetra<'a>(
    m: f32,
    transform: Transform,
    commands: &'a mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> EntityCommands<'a> {
    let verticies: Vec<[f32; 3]> = vec![[m, m, m], [m, -m, -m], [-m, m, -m], [-m, -m, m]];
    let mut f = Vec::with_capacity(12);
    for (i, a) in verticies.iter().enumerate() {
        let [ax, ay, az] = a;
        let an = ax * ax + ay * ay + az * az;
        for (j, b) in verticies.iter().enumerate() {
            let [bx, by, bz] = b;
            let bn = bx * bx + by * by + bz * bz;
            let t = (ax * bx + ay * by + az * bz) / (an * bn).sqrt();
            let t = t.acos();
            if (t - 1.9106332).abs() < 0.125 {
                f.push([i as u16, j as u16]);
            }
        }
    }
    let mut indecies = Vec::with_capacity(12);
    let mut faces = Vec::with_capacity(4);
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
        CollisionLayers::new(0b11, LayerMask::ALL),
        Collider::convex_hull_from_mesh(&mesh).unwrap(),
        transform,
        Shape::Tetrahedron,
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
        CollisionLayers::new(0b11, LayerMask::ALL),
        Collider::compound(vec![
            (
                Position::default(),
                Rotation::default(),
                Collider::cylinder(m, m / ratio),
            ),
            (
                Position::default(),
                Rotation::default(),
                Collider::cylinder(m + 1.0, m / (ratio * 16.0)),
            ),
        ]),
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
    let phi = 0.5 + 5.0f64.sqrt() / 2.0;
    let phir = (phi.recip() * m as f64) as f32;
    let phi = (phi * m as f64) as f32;
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
            if (t - 0.72972).abs() < 0.125 {
                f.push([i as u16, j as u16]);
            }
        }
    }
    let mut indecies = Vec::with_capacity(180);
    let mut faces = Vec::with_capacity(12);
    for a in &f {
        for b in &f {
            if a[1] != b[0] || a[0] > b[0] {
                continue;
            }
            for c in &f {
                if b[1] != c[0] || a[0] > c[0] {
                    continue;
                }
                for d in &f {
                    if c[1] != d[0] || a[0] > d[0] {
                        continue;
                    }
                    for e in &f {
                        if d[1] != e[0]
                            || e[1] != a[0]
                            || a[0] > e[0]
                            || b[0] > e[0]
                            || [a, b, c, d, e].iter().enumerate().any(|(i, x)| {
                                [a, b, c, d, e].iter().enumerate().any(|(j, y)| {
                                    (x == y && i != j) || (x[0] == y[1] && x[1] == y[0])
                                })
                            })
                        {
                            continue;
                        }
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
                            indecies.push(e[0]);
                            indecies.push(a[0]);
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
                            indecies.push(b[0]);
                            indecies.push(a[0]);
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
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verticies)
    .with_inserted_indices(Indices::U16(indecies));
    let mut ent = commands.spawn((
        CollisionLayers::new(0b11, LayerMask::ALL),
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

pub fn spawn_cube<'a>(
    m: f32,
    transform: Transform,
    commands: &'a mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> EntityCommands<'a> {
    let d = m / 2.0 + 1.0;
    let mut cube = commands.spawn((
        CollisionLayers::new(0b11, LayerMask::ALL),
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
