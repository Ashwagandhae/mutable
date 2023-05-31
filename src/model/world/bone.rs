use nannou::prelude::Vec2;

use super::{
    collection::{Collection, GenId},
    node::Node,
};
#[derive(Debug, Clone)]
pub struct Bone {
    pub node_1: GenId,
    pub node_2: GenId,
    pub len: f32,

    pub dead: bool,
}
impl Bone {
    pub fn new(node_1: GenId, node_2: GenId, len: f32) -> Bone {
        Bone {
            node_1,
            node_2,
            len,
            dead: false,
        }
    }
    pub fn update(&mut self, nodes: &mut Collection<Node>) {
        let (Some(node_1), Some(node_2)) = (nodes.get(self.node_1), nodes.get(self.node_2)) else {
            self.dead = true;
            return;
        };

        // move towards len
        let distance = node_1.pos.distance(node_2.pos);
        let distance_diff = distance - self.len;
        let pos_change = (distance_diff / 2.0)
            * (node_2.pos - node_1.pos)
                .try_normalize()
                .unwrap_or(Vec2::new(1., 0.));

        // push water
        let vel = (node_1.vel + node_2.vel) / 2.;
        let facing = (node_2.pos - node_1.pos).normalize_or_zero().perp();
        let stroke_amp = vel.dot(facing);
        let friction = -facing * stroke_amp * 0.5;

        // transfer energy between nodes
        let energy_ratio = node_1.energy / node_2.energy;
        let energy_weight_ratio = node_1.energy_weight / node_2.energy_weight;
        let energy_change = if energy_ratio < energy_weight_ratio {
            0.1
        } else {
            -0.1
        };

        let node_1 = nodes.get_mut(self.node_1).unwrap();
        node_1.pos += pos_change;
        node_1.accel(friction);
        node_1.energy += energy_change;
        let node_2 = nodes.get_mut(self.node_2).unwrap();
        node_2.pos -= pos_change;
        node_2.accel(friction);
        node_2.energy -= energy_change;
    }
}
