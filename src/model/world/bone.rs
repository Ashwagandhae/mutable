use nannou::prelude::Vec2;

use super::{
    chunks::Chunks,
    collection::{Collection, GenId},
    math::{is_zero, Angle},
    node::{LifeState, Node},
};
#[derive(Debug, Clone)]
pub struct Bone {
    pub parent_node: GenId,
    pub child_node: GenId,
    pub len: f32,

    pub delete: bool,
}
impl Bone {
    pub fn new(parent_node: GenId, child_node: GenId, len: f32) -> Bone {
        Bone {
            parent_node,
            child_node,
            len,
            delete: false,
        }
    }
    pub fn update(&mut self, nodes: &mut Collection<Node>, chunks: &Chunks) {
        let (Some(parent_node), Some(child_node)) = (nodes.get(self.parent_node), nodes.get(self.child_node)) else {
            self.delete = true;
            return;
        };

        // move towards len
        let distance = parent_node.pos().distance(child_node.pos());
        let distance_diff = distance - self.len;
        let pos_change = (distance_diff / 2.0)
            * (child_node.pos() - parent_node.pos())
                .try_normalize()
                .unwrap_or(Vec2::new(1., 0.));

        // slow nodes who push water
        let chunk = chunks.get((parent_node.pos() + child_node.pos()) / 2.);
        // get relative vel compared to chunk tide
        let vel = (parent_node.vel + child_node.vel) / 2. - chunk.tide;
        let facing = (child_node.pos() - parent_node.pos())
            .normalize_or_zero()
            .perp();
        let stroke_amp = vel.dot(facing);
        let friction = -facing * stroke_amp * 0.8;

        let (Some(parent_node), Some(child_node)) = nodes.get_2_mut(self.parent_node, self.child_node) else {unreachable!()};
        if !is_zero(distance_diff) {
            *parent_node.pos_mut() += pos_change;
            *child_node.pos_mut() -= pos_change;
        }
        parent_node.accel(friction);
        child_node.accel(friction);

        // transfer energy
        if let (
            LifeState::Alive {
                energy_weight: weight_1,
                ..
            },
            LifeState::Alive {
                energy_weight: weight_2,
                ..
            },
        ) = (&mut parent_node.life_state, &mut child_node.life_state)
        {
            let energy_ratio = parent_node.energy / child_node.energy;
            let energy_weight_ratio = *weight_1 / *weight_2;
            let energy_change = if energy_ratio < energy_weight_ratio {
                0.1
            } else {
                -0.1
            };
            parent_node.energy += energy_change;
            child_node.energy -= energy_change;
        }

        // update angle
        let child_pos = child_node.pos();
        if let LifeState::Alive {
            parent: Some((_, ref mut angle)),
            ..
        } = child_node.life_state
        {
            *angle = Angle::from_vec2(child_pos - parent_node.pos());
        }
    }
}
