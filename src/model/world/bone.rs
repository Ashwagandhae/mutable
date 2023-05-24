use super::{collection::Collection, node::Node};
use nannou::prelude::*;
#[derive(Debug, Clone)]
pub struct Bone {
    pub node_1: usize,
    pub node_2: usize,
    pub dist: f32,
}
impl Bone {
    pub fn new(node_1: usize, node_2: usize, dist: f32) -> Bone {
        Bone {
            node_1,
            node_2,
            dist,
        }
    }
    pub fn update(&mut self, nodes: &mut Collection<Node>) {
        let node_1 = nodes.get(self.node_1).expect("node 1 not found for bone");
        let node_2 = nodes.get(self.node_2).expect("node 2 not found for bone");
        // move towards dist
        let dist = node_1.pos.distance(node_2.pos);
        let dist_diff = dist - self.dist;
        let pos_change = (dist_diff / 2.0) * (node_2.pos - node_1.pos).normalize();

        let vel = (node_1.vel + node_2.vel) / 2.;
        let facing = (node_2.pos - node_1.pos).normalize().perp();
        let stroke_amp = vel.dot(facing);
        let mut friction = -facing * stroke_amp * 0.5;

        // apply
        if !friction.is_finite() {
            friction = Vec2::new(0., 0.);
        }
        let node_1 = nodes.get_mut(self.node_1).unwrap();
        node_1.pos += pos_change;
        node_1.accel += friction;
        let node_2 = nodes.get_mut(self.node_2).unwrap();
        node_2.pos -= pos_change;
        node_2.accel += friction;
    }
}
