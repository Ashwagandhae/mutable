use nannou::prelude::*;

use super::{chunks::Chunk, collection::GenId, math::is_zero_vec2};

#[derive(Debug, Clone)]
pub enum NodeKind {
    Storage,
    Leaf,
    Mouth,
    Spike,
}
impl From<u8> for NodeKind {
    fn from(n: u8) -> NodeKind {
        match n {
            0 => NodeKind::Storage,
            1 => NodeKind::Leaf,
            2 => NodeKind::Mouth,
            3 => NodeKind::Spike,
            _ => panic!("invalid node kind"),
        }
    }
}
#[derive(Debug, Clone)]
pub enum LifeState {
    Alive {
        kind: NodeKind,
        parent_id: Option<GenId>,
        age: u32,
        gene_index: usize,
        energy_weight: f32,
        lifespan: u32,
    },
    Dead {
        decay: u32,
    },
}

#[derive(Debug, Clone)]
pub struct Node {
    pos: Point2,
    pub radius: f32,
    pub vel: Vec2,
    accel: Vec2,

    pub life_state: LifeState,
    pub energy: f32,

    pub delete: bool,
    pub splat: bool,
}
impl Node {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pos: Point2,
        radius: f32,
        energy: f32,
        energy_weight: f32,
        kind: NodeKind,
        gene_index: usize,
        parent_id: Option<GenId>,
        lifespan: u32,
    ) -> Node {
        Node {
            pos,
            radius,
            vel: Vec2::new(0., 0.),
            accel: Vec2::new(0., 0.),

            life_state: LifeState::Alive {
                kind,
                parent_id,
                age: 0,
                gene_index,
                energy_weight,
                lifespan,
            },

            energy,

            delete: false,
            splat: false,
        }
    }
    pub fn new_dead(pos: Point2, radius: f32, energy: f32) -> Node {
        Node {
            pos,
            radius,
            vel: Vec2::new(0., 0.),
            accel: Vec2::new(0., 0.),

            life_state: LifeState::Dead { decay: 0 },

            energy,

            delete: false,
            splat: false,
        }
    }
    pub fn accel(&mut self, accel: Vec2) {
        if accel.is_finite() {
            self.accel += accel;
        }
    }
    pub fn pos(&self) -> Point2 {
        self.pos
    }
    pub fn pos_mut(&mut self) -> &mut Point2 {
        &mut self.pos
    }
    pub fn max_energy(&self) -> f32 {
        self.radius.powi(3) / 50.0 * 8.
    }

    pub fn update(&mut self, chunk: &Chunk) {
        self.vel += self.accel;
        self.vel += chunk.tide;

        let friction = 1. - (self.radius / 15.0 * 0.4);
        self.vel *= friction;
        if !self.vel.is_finite() {
            println!("vel not finite, {:?}", self.vel);
            self.vel = Vec2::new(0., 0.);
        }
        if !is_zero_vec2(self.vel) {
            self.pos += self.vel;
        }
        self.accel = Vec2::new(0., 0.);

        let max_energy = self.max_energy();

        match &mut self.life_state {
            LifeState::Alive {
                ref mut age,
                lifespan,
                kind,
                ..
            } => {
                const LEAF_ENERGY_RATE: f32 = 0.000_04;
                const ENERGY_LOSS_RATE: f32 = 0.000_002;
                if let NodeKind::Leaf = kind {
                    self.energy += LEAF_ENERGY_RATE * self.radius.powi(2) * chunk.sun;
                }
                match kind {
                    NodeKind::Storage => {
                        self.energy -= ENERGY_LOSS_RATE * self.radius.powi(3) * 0.25;
                        if self.energy > max_energy * 4. {
                            self.energy = max_energy * 4.;
                        }
                    }
                    _ => {
                        self.energy -= ENERGY_LOSS_RATE * self.radius.powi(3);
                        if self.energy > max_energy {
                            self.energy = max_energy;
                        }
                    }
                }

                *age += 1;
                if self.energy < 0. || *age > *lifespan {
                    self.die();
                }
            }
            LifeState::Dead { ref mut decay, .. } => {
                *decay += 1;
            }
        }
        // age
    }
    pub fn die(&mut self) {
        if let LifeState::Alive { .. } = self.life_state {
            self.life_state = LifeState::Dead { decay: 0 };
        }
    }
    pub fn is_alive(&self) -> bool {
        matches!(self.life_state, LifeState::Alive { .. })
    }

    pub fn unwrap_parent_id(&self) -> &Option<GenId> {
        match self.life_state {
            LifeState::Alive { ref parent_id, .. } => parent_id,
            LifeState::Dead { .. } => panic!("dead node has no parent id"),
        }
    }

    pub fn unwrap_gene_index_mut(&mut self) -> &mut usize {
        match self.life_state {
            LifeState::Alive {
                ref mut gene_index, ..
            } => gene_index,
            LifeState::Dead { .. } => panic!("dead node has no gene index"),
        }
    }
    pub fn struct_energy(&self) -> f32 {
        self.radius.powi(3) / 50.0
    }
}
