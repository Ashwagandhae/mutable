use nannou::prelude::Vec2;

use super::{
    collection::{Collection, GenId},
    math::is_zero,
    node::{LifeState, Node},
};
#[derive(Debug, Clone)]
pub struct Bone {
    pub node_1: GenId,
    pub node_2: GenId,
    pub len: f32,

    pub delete: bool,
}
impl Bone {
    pub fn new(node_1: GenId, node_2: GenId, len: f32) -> Bone {
        Bone {
            node_1,
            node_2,
            len,
            delete: false,
        }
    }
    pub fn update(&mut self, nodes: &mut Collection<Node>) {
        let (Some(node_1), Some(node_2)) = (nodes.get(self.node_1), nodes.get(self.node_2)) else {
            self.delete = true;
            return;
        };

        // move towards len
        let distance = node_1.pos().distance(node_2.pos());
        let distance_diff = distance - self.len;
        let pos_change = (distance_diff / 2.0)
            * (node_2.pos() - node_1.pos())
                .try_normalize()
                .unwrap_or(Vec2::new(1., 0.));

        // push water
        let vel = (node_1.vel + node_2.vel) / 2.;
        let facing = (node_2.pos() - node_1.pos()).normalize_or_zero().perp();
        let stroke_amp = vel.dot(facing);
        let friction = -facing * stroke_amp * 0.5;

        let (Some(node_1), Some(node_2)) = nodes.get_2_mut(self.node_1, self.node_2) else {unreachable!()};
        if !is_zero(distance_diff) {
            *node_1.pos_mut() += pos_change;
            *node_2.pos_mut() -= pos_change;
        }
        node_1.accel(friction);
        node_2.accel(friction);

        if let (
            LifeState::Alive {
                energy_weight: weight_1,
                ..
            },
            LifeState::Alive {
                energy_weight: weight_2,
                ..
            },
        ) = (&mut node_1.life_state, &mut node_2.life_state)
        {
            let energy_ratio = node_1.energy / node_2.energy;
            let energy_weight_ratio = *weight_1 / *weight_2;
            let energy_change = if energy_ratio < energy_weight_ratio {
                0.1
            } else {
                -0.1
            };
            node_1.energy += energy_change;
            node_2.energy -= energy_change;
        }
    }
}
