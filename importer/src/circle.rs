pub struct Circumference {
    x: u32,
    y: u32,
    error: u32,
}
impl Iterator for Circumference {
    type Item = (u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        if self.x.cast_signed() < self.y.cast_signed() {
            None
        } else {
            let (x, y) = (self.x, self.y);
            self.y += 1;
            self.error += self.y;
            if let Some(e) = self.error.checked_sub(self.x) {
                self.error = e;
                self.x = self.x.overflowing_sub(1).0;
            }
            Some((x, y))
        }
    }
}
impl Circumference {
    #[must_use]
    pub fn new(r: u32) -> Self {
        Self {
            x: r,
            y: 0,
            error: r / 16,
        }
    }
}
#[derive(Clone, Copy)]
pub enum Octant {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}
impl Octant {
    #[must_use]
    pub fn octant(self, (x0, y0): (u32, u32), (dx, dy): (u32, u32)) -> (u32, u32) {
        match self {
            Octant::Zero => (x0 + dx, y0 + dy),
            Octant::One => (x0 + dy, y0 + dx),
            Octant::Two => (x0 - dy, y0 + dx),
            Octant::Three => (x0 - dx, y0 + dy),
            Octant::Four => (x0 - dx, y0 - dy),
            Octant::Five => (x0 - dy, y0 - dx),
            Octant::Six => (x0 + dy, y0 - dx),
            Octant::Seven => (x0 + dx, y0 - dy),
        }
    }
}
