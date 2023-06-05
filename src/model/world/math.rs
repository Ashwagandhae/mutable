use nannou::prelude::{Vec2, PI};

#[derive(Debug, Clone, Copy)]
pub struct Angle(pub f32);
/// angle is between 0 and 2PI
impl Angle {
    // converts an angle from -PI to PI to 0 to 2PI
    pub fn from_pi_pi_range(angle: f32) -> Angle {
        if angle < 0. {
            Angle(angle + 2. * PI)
        } else {
            Angle(angle)
        }
    }
    pub fn to_vec2(&self) -> nannou::prelude::Vec2 {
        Vec2::new(self.0.cos(), self.0.sin())
    }
}

impl std::ops::AddAssign for Angle {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = (self.0 + rhs.0) % (2. * PI);
    }
}

impl std::ops::SubAssign for Angle {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = (self.0 - rhs.0) % (2. * PI);
    }
}

impl std::ops::Add for Angle {
    type Output = Angle;
    fn add(self, rhs: Self) -> Self::Output {
        Angle((self.0 + rhs.0) % (2. * PI))
    }
}

impl std::ops::Sub for Angle {
    type Output = Angle;
    fn sub(self, rhs: Self) -> Self::Output {
        Angle((self.0 - rhs.0) % (2. * PI))
    }
}
