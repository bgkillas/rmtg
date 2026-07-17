use avian3d::parry::glamx::Vec3;
use bevy_polyline::polyline::Polyline;
pub struct CubeOutline {
    pub unit_length: f32,
}
impl CubeOutline {
    #[must_use]
    pub fn from_length(length: f32) -> Self {
        Self {
            unit_length: length / 2.0,
        }
    }
}
impl CubeOutline {
    #[must_use]
    pub fn build(&self) -> Polyline {
        let one = self.unit_length;
        let v = [
            Vec3::new(one, one, one),
            Vec3::new(-one, one, one),
            Vec3::new(one, -one, one),
            Vec3::new(one, one, -one),
            Vec3::new(-one, one, -one),
            Vec3::new(-one, -one, one),
            Vec3::new(one, -one, -one),
            Vec3::new(-one, -one, -one),
        ];
        #[rustfmt::skip]
        let vertices = vec![
            v[0], v[0], v[0],
            v[7], v[7], v[7],
            v[1], v[2], v[3],
            v[4], v[5], v[6],
            v[1], v[2], v[3],
            v[4], v[5], v[6],
            v[5], v[6], v[4],
            v[1], v[2], v[3],
        ];
        Polyline { vertices }
    }
}
