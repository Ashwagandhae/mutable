use nannou::prelude::*;

use super::{chunks::Chunk, collection::GenId};

#[derive(Debug, Clone)]
pub enum NodeKind {
    Storage,
    Leaf,
    Mouth,
}
impl From<u8> for NodeKind {
    fn from(n: u8) -> NodeKind {
        match n {
            0 => NodeKind::Storage,
            1 => NodeKind::Leaf,
            2 => NodeKind::Mouth,
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
        energy: f32,
        lifespan: u32,
    },
    Dead {
        decay: u32,
        energy: f32,
    },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub pos: Point2,
    pub radius: f32,
    pub vel: Vec2,
    accel: Vec2,
    pub cramming: u8,

    pub life_state: LifeState,

    pub delete: bool,
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
        lifespan: u32,
    ) -> Node {
        Node {
            pos,
            radius,
            vel: Vec2::new(0., 0.),
            accel: Vec2::new(0., 0.),
            cramming: 0,

            life_state: LifeState::Alive {
                kind,
                parent_id,
                age: 0,
                gene_index,
                energy_weight,
                energy,
                lifespan,
            },

            delete: false,
        }
    }
    pub fn new_dead(pos: Point2, radius: f32, energy: f32) -> Node {
        Node {
            pos,
            radius,
            vel: Vec2::new(0., 0.),
            accel: Vec2::new(0., 0.),
            cramming: 0,

            life_state: LifeState::Dead { decay: 0, energy },

            delete: false,
        }
    }
    pub fn accel(&mut self, accel: Vec2) {
        if accel.is_finite() {
            self.accel += accel;
        }
    }
    pub fn update(&mut self, chunk: &Chunk) {
        self.vel += self.accel;
        self.vel = self.vel * 0.9;
        if !self.vel.is_finite() {
            panic!("vel not finite");
        }
        self.pos += self.vel;
        self.accel = Vec2::new(0., 0.);

        match &mut self.life_state {
            LifeState::Alive {
                ref mut energy,
                ref mut age,
                lifespan,
                kind,
                ..
            } => {
                if let NodeKind::Leaf = kind {
                    *energy += 0.0001 * self.radius.powi(2) * chunk.sun;
                }

                match kind {
                    NodeKind::Storage => {
                        *energy -= 0.00025;
                        if *energy > 60.0 {
                            *energy = 60.0;
                        }
                    }
                    _ => {
                        *energy -= 0.001;
                        if *energy > 30.0 {
                            *energy = 30.0;
                        }
                    }
                }

                *age += 1;
                if *energy < 0. || *age > *lifespan {
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
        if let LifeState::Alive { ref energy, .. } = self.life_state {
            self.life_state = LifeState::Dead {
                decay: 0,
                energy: *energy,
            };
        }
    }
    pub fn is_alive(&self) -> bool {
        matches!(self.life_state, LifeState::Alive { .. })
    }
    pub fn get_energy(&self) -> f32 {
        match self.life_state {
            LifeState::Alive { ref energy, .. } => *energy,
            LifeState::Dead { ref energy, .. } => *energy,
        }
    }

    pub fn unwrap_parent_id(&self) -> &Option<GenId> {
        match self.life_state {
            LifeState::Alive { ref parent_id, .. } => parent_id,
            LifeState::Dead { .. } => panic!("dead node has no parent id"),
        }
    }

    pub fn unwrap_parent_id_mut(&mut self) -> &mut Option<GenId> {
        match self.life_state {
            LifeState::Alive {
                ref mut parent_id, ..
            } => parent_id,
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
    pub fn unwrap_energy_mut(&mut self) -> &mut f32 {
        match self.life_state {
            LifeState::Alive { ref mut energy, .. } => energy,
            LifeState::Dead { .. } => panic!("dead node has no mutable energy"),
        }
    }
}
