use nannou::prelude::*;

use super::collection::GenId;

#[derive(Debug, Clone)]
pub enum NodeKind {
    Body,
    Leaf,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub pos: Point2,
    pub radius: f32,
    pub vel: Vec2,
    accel: Vec2,
    pub energy: f32,
    pub cramming: u8,
    pub age: u32,

    pub energy_weight: f32,
    pub kind: NodeKind,
    pub gene_index: usize,
    pub parent_id: Option<GenId>,

    pub dead: bool,
}
impl Node {
    pub fn new(
        pos: Point2,
        radius: f32,
        energy: f32,
        energy_weight: f32,
        kind: NodeKind,
        gene_index: usize,
        parent_id: Option<GenId>,
    ) -> Node {
        Node {
            pos,
            radius,
            vel: Vec2::new(0., 0.),
            accel: Vec2::new(0., 0.),
            cramming: 0,
            age: 0,

            energy,
            energy_weight,
            kind,
            gene_index,
            parent_id,

            dead: false,
        }
    }
    pub fn accel(&mut self, accel: Vec2) {
        if accel.is_finite() {
            self.accel += accel;
        }
    }
    pub fn update(&mut self) {
        self.vel += self.accel;
        self.vel = self.vel * 0.9;
        if !self.vel.is_finite() {
            panic!("vel not finite");
        }
        self.pos += self.vel;
        self.accel = Vec2::new(0., 0.);

        // energy
        self.energy -= 0.001;
        if let NodeKind::Leaf = self.kind {
            self.energy += 0.0001 * self.radius.powi(2);
        }
        if self.energy < 0. {
            self.dead = true;
        }
        if self.energy > 20.0 {
            self.energy = 20.0;
        }

        // age
        self.age += 1;
        if self.age > 1024 {
            self.dead = true;
        }
    }
}
