use crate::CARD_THICKNESS;
use crate::shapes::{NewShape, Shape, ShapeMesh, ShapeOutline};
use avian3d::prelude::Collider;
use bevy::mesh::{Mesh, MeshBuilder};
#[derive(Clone, Copy)]
pub struct Drag {
    pub x: f32,
    pub y: f32,
}
impl ShapeMesh for Drag {
    type Outline = DragOutline;
    type const VERTICES: usize = 4;
    type const FACES: usize = 2;
    type const FACE_VERTICES: usize = 4;
    type const TRIANGLES: usize = 2;
    const SHAPE: Shape = Shape::Cube;
    fn collider(height: f32, _: &Mesh) -> Collider {
        Collider::cuboid(height, height, height)
    }
    fn text_size(_: f32) -> f32 {
        unreachable!()
    }
    fn convert_height(_: f32) -> f32 {
        unreachable!()
    }
    fn face_indices() -> [[u16; Self::FACE_VERTICES]; Self::FACES] {
        unreachable!()
    }
    fn vertices(self) -> [[f32; 3]; Self::VERTICES] {
        [
            [self.x, 0.0, self.y],
            [-self.x, 0.0, self.y],
            [self.x, 0.0, -self.y],
            [-self.x, 0.0, -self.y],
        ]
    }
    fn convert_to_triangles(_: [u16; Self::FACE_VERTICES]) -> [[u16; 3]; Self::TRIANGLES] {
        unreachable!()
    }
    fn oriented_vertices(self) -> [[f32; 3]; Self::VERTICES] {
        self.vertices()
    }
    fn unit_length(self) -> f32 {
        unreachable!()
    }
}
impl ShapeOutline for DragOutline {
    type Mesh = Drag;
    type const EDGES: usize = 4;
    const THICKNESS: f32 = 4.0 * CARD_THICKNESS;
    fn edge_indices() -> [[usize; 2]; Self::EDGES] {
        [[0, 1], [0, 2], [3, 2], [3, 1]]
    }
    fn unit_length(self) -> f32 {
        unreachable!()
    }
}
#[derive(Clone, Copy)]
pub struct DragOutline {
    pub x: f32,
    pub y: f32,
}
impl MeshBuilder for Drag {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
impl NewShape for Drag {
    fn from_height(_: f32) -> Self {
        unreachable!()
    }
}
impl NewShape for DragOutline {
    fn from_height(_: f32) -> Self {
        unreachable!()
    }
}
impl MeshBuilder for DragOutline {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
impl From<DragOutline> for Drag {
    fn from(value: DragOutline) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
impl From<Drag> for DragOutline {
    fn from(value: Drag) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
