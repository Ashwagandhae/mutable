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
    pub fn to_vec2(self) -> nannou::prelude::Vec2 {
        Vec2::new(self.0.cos(), self.0.sin())
    }
    pub fn from_vec2(v: Vec2) -> Angle {
        Angle::from_pi_pi_range(v.y.atan2(v.x))
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

pub fn is_zero(f: f32) -> bool {
    f.abs() < 0.0001
}

pub fn is_zero_vec2(v: Vec2) -> bool {
    is_zero(v.x) && is_zero(v.y)
}

/// returns the difference between the two angles, normalized for sense
/// returns 0 if the angles are the same
/// returns 1/-1 if the angles are opposite
/// returns 0.5/-0.5 if the angles are 90 degrees apart
pub fn sense_angle_diff(node_angle: Angle, angle: Angle) -> f32 {
    let diff = (node_angle - angle).0;
    if diff < PI {
        diff / PI
    } else {
        (diff - 2. * PI) / PI
    }
}

/// returns the velocity of the object towards the other object
pub fn vel_towards(pos_1: Vec2, vel_2: Vec2, pos_2: Vec2, vel_1: Vec2) -> f32 {
    let relative_vel = vel_1 - vel_2;
    relative_vel.dot((pos_2 - pos_1).normalize_or_zero())
}
