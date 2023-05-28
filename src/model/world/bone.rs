use super::{collection::Collection, node::Node};
#[derive(Debug, Clone)]
pub struct Bone {
    pub node_1: usize,
    pub node_2: usize,
    pub len: f32,

    pub dead: bool,
}
impl Bone {
    pub fn new(node_1: usize, node_2: usize, len: f32) -> Bone {
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
        let pos_change = (distance_diff / 2.0) * (node_2.pos - node_1.pos).normalize();

        // push water
        let vel = (node_1.get_vel() + node_2.get_vel()) / 2.;
        let facing = (node_2.pos - node_1.pos).normalize().perp();
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
        node_1.accel += friction;
        node_1.energy += energy_change;
        let node_2 = nodes.get_mut(self.node_2).unwrap();
        node_2.pos -= pos_change;
        node_2.accel += friction;
        node_2.energy -= energy_change;
    }
}
