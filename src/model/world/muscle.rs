use super::collection::Collection;
use super::math::Angle;
use super::node::Node;

#[derive(Debug, Clone)]
pub struct Muscle {
    pub joint_node: usize,
    pub node_1: usize,
    pub node_2: usize,

    pub angle: Angle,
    pub strength: f32,
    pub dead: bool,
}
impl Muscle {
    pub fn new(
        joint_node: usize,
        node_1: usize,
        node_2: usize,
        angle: Angle,
        strength: f32,
    ) -> Muscle {
        Muscle {
            joint_node,
            node_1,
            node_2,
            angle,
            strength,
            dead: false,
        }
    }
    pub fn update(&mut self, nodes: &mut Collection<Node>, _tick: u64) {
        // let joint_node = nodes
        //     .get(self.joint_node)
        // let node_1 = nodes.get(self.node_1);
        // let node_2 = nodes.get(self.node_2);
        let (Some(joint_node), Some(node_1), Some(node_2)) =
            (nodes.get(self.joint_node), nodes.get(self.node_1), nodes.get(self.node_2)) else {
                self.dead = true;
                return;
            };

        // update angle by slow sin wave

        let angle_diff = Angle::from_pi_pi_range(
            (node_1.pos - joint_node.pos).angle_between(node_2.pos - joint_node.pos),
        )
        .0 - self.angle.0;

        // move towards each other
        let accel_change_1 =
            (node_1.pos - joint_node.pos).perp().normalize() * angle_diff * self.strength;
        let accel_change_2 =
            -(node_2.pos - joint_node.pos).perp().normalize() * angle_diff * self.strength;

        // apply
        let node_1 = nodes.get_mut(self.node_1).unwrap();
        node_1.accel += accel_change_1;
        let node_2 = nodes.get_mut(self.node_2).unwrap();
        node_2.accel += accel_change_2;
    }
}
