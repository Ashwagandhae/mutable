use nannou::prelude::*;

#[derive(Debug, Clone)]
pub struct Node {
    pub pos: Point2,
    pub radius: f32,
    pub vel: Vec2,
    pub accel: Vec2,
}
impl Node {
    pub fn new(pos: Point2, radius: f32) -> Node {
        Node {
            pos,
            radius,
            vel: Vec2::new(0., 0.),
            accel: Vec2::new(0., 0.),
        }
    }
    pub fn update(&mut self) {
        // verlet
        self.vel = self.vel * 0.9;
        self.vel += self.accel;
        self.pos += self.vel;
        self.accel = Vec2::new(0., 0.);
    }
}
