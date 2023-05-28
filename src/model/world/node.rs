use nannou::prelude::*;

#[derive(Debug, Clone)]
pub enum NodeKind {
    Body,
    Leaf,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub pos: Point2,
    pub last_pos: Point2,
    pub radius: f32,
    pub accel: Vec2,
    pub energy: f32,
    pub pressure: f32,

    pub energy_weight: f32,
    pub kind: NodeKind,
    pub gene_index: usize,
    pub parent_id: Option<usize>,

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
        parent_id: Option<usize>,
    ) -> Node {
        Node {
            pos,
            last_pos: pos,
            radius,
            accel: Vec2::new(0., 0.),
            pressure: 0.,

            energy,
            energy_weight,
            kind,
            gene_index,
            parent_id,

            dead: false,
        }
    }
    pub fn update(&mut self) {
        // verlet
        let vel = self.get_vel() * 0.9;
        self.last_pos = self.pos;
        self.pos += vel + self.accel;
        self.accel = Vec2::new(0., 0.);

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

        if self.pressure > 12. {
            self.dead = true;
        }
        self.pressure *= 0.1;
    }
    pub fn get_mass(&self) -> f32 {
        self.radius.powi(2) * PI * 0.1
    }
    pub fn get_vel(&self) -> Vec2 {
        self.pos - self.last_pos
    }
}
