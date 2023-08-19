use int_enum::IntEnum;
use nannou::prelude::*;
use strum_macros::{EnumCount, EnumIter};

use crate::model::world::math::sense_angle_diff;

use super::{
    chunks::Chunk,
    collection::GenId,
    math::{is_zero_vec2, Angle},
};

// pub const LEAF_ENERGY_RATE: f32 = 0.000_08;
pub const LEAF_ENERGY_RATE: f32 = 0.000_08;
pub const ENERGY_LOSS_RATE: f32 = 0.000_002;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, IntEnum, EnumIter, EnumCount)]
pub enum NodeKind {
    Egg = 0,
    Leaf = 1,
    Mouth = 2,
    Spike = 3,
    Storage = 4,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, IntEnum, EnumIter, EnumCount)]
pub enum SenseKind {
    Sun = 0,
    Energy = 1,
    Age = 2,

    TideAngle = 3,
    TideSpeed = 4,

    CollideAngle = 5,
    CollideKind = 6,
    CollideRadius = 7,
    CollideSpeed = 8,
}

#[derive(Debug, Clone)]
pub enum LifeState {
    Alive {
        kind: NodeKind,
        parent: Option<(GenId, Angle)>,
        age: u32,
        gene_index: Option<usize>,
        energy_weight: f32,
        lifespan: u32,

        sense: Option<(SenseKind, f32)>,
        activate: f32,
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
        gene_index: Option<usize>,
        parent: Option<(GenId, Angle)>,
        lifespan: u32,
        sense_kind: Option<SenseKind>,
    ) -> Node {
        Node {
            pos,
            radius,
            vel: Vec2::new(0., 0.),
            accel: Vec2::new(0., 0.),

            life_state: LifeState::Alive {
                kind,
                parent,
                age: 0,
                gene_index,
                energy_weight,
                lifespan,

                sense: sense_kind.map(|sense| (sense, 0.)),
                activate: 0.,
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
        self.radius.powi(3) / 50.0 * 16.
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
                ref mut sense,
                parent,
                ..
            } => {
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
                // sense sunlight
                if let Some((kind, ref mut sense)) = sense {
                    use SenseKind::*;
                    *sense = match kind {
                        Sun => chunk.sun,
                        Energy => self.energy,
                        Age => *age as f32,
                        TideSpeed => chunk.tide.length(),
                        TideAngle => parent
                            .map(|(_, a)| sense_angle_diff(a, Angle::from_vec2(chunk.tide)))
                            .unwrap_or(0.),
                        // handled by collider
                        CollideAngle | CollideKind | CollideRadius | CollideSpeed => 0.,
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

    pub fn unwrap_parent_id(&self) -> Option<GenId> {
        match self.life_state {
            LifeState::Alive { ref parent, .. } => parent.map(|(parent_id, _)| parent_id),
            LifeState::Dead { .. } => panic!("dead node has no parent id"),
        }
    }

    pub fn unwrap_gene_index_mut(&mut self) -> &mut Option<usize> {
        match self.life_state {
            LifeState::Alive {
                ref mut gene_index, ..
            } => gene_index,
            LifeState::Dead { .. } => panic!("dead node has no gene index"),
        }
    }
    pub fn unwrap_gene_index(&self) -> &Option<usize> {
        match self.life_state {
            LifeState::Alive { ref gene_index, .. } => gene_index,
            LifeState::Dead { .. } => panic!("dead node has no gene index"),
        }
    }
    pub fn unwrap_kind(&self) -> &NodeKind {
        match self.life_state {
            LifeState::Alive { ref kind, .. } => kind,
            LifeState::Dead { .. } => panic!("dead node has no kind"),
        }
    }
    pub fn struct_energy(&self) -> f32 {
        self.radius.powi(3) / 50.0
    }

    pub fn unwrap_activate(&self) -> &f32 {
        match self.life_state {
            LifeState::Alive { ref activate, .. } => activate,
            LifeState::Dead { .. } => panic!("dead node has no activate"),
        }
    }
    pub fn sense(&self) -> Option<f32> {
        match self.life_state {
            LifeState::Alive { ref sense, .. } => sense.map(|(_, sense)| sense),
            LifeState::Dead { .. } => None,
        }
    }
    pub fn activate(&mut self, new_activate: f32) {
        match self.life_state {
            LifeState::Alive {
                ref mut activate, ..
            } => *activate = new_activate,
            LifeState::Dead { .. } => {}
        }
    }
}
