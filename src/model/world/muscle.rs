use super::collection::{CollectionView, GenId};
use super::node::{Node, MUSCLE_ENERGY_RATE};

#[derive(Debug, Clone)]
pub struct Muscle {
    pub joint_node: GenId,
    pub node_1: GenId,
    pub node_2: GenId,

    pub len: f32,
    pub strength: f32,
    pub delete: bool,
    // pub movement: Option<(f32, f32, f32)>,
}
impl Muscle {
    pub fn new(joint_node: GenId, node_1: GenId, node_2: GenId, len: f32, strength: f32) -> Muscle {
        Muscle {
            joint_node,
            node_1,
            node_2,
            len,
            strength,
            delete: false,
        }
    }
    pub fn update(&mut self, nodes: &mut CollectionView<Node>) {
        let (Some(joint_node), Some(node_1), Some(node_2)) =
            (nodes.get(self.joint_node), nodes.get(self.node_1), nodes.get(self.node_2)) else {
                self.delete = true;
                return;
            };
        // dont move if joint node is dead
        if !joint_node.is_alive() {
            return;
        }

        // let real_angle = match self.movement {
        //     Some((freq, amp, shift)) => {
        //         let angle = amp * (freq * tick as f32 + shift).sin();
        //         self.angle + Angle(angle)
        //     }
        //     None => self.angle,
        // };
        let min_len = node_1.radius + node_2.radius;
        let real_len = (self.len * joint_node.unwrap_activate().clamp(0.1, 2.0)).max(min_len);
        let dist_diff = node_1.pos().distance(node_2.pos()) - real_len;
        let accel_mag = dist_diff * 0.0625 * self.strength;

        // move towards each other
        let accel_change_1 = (node_2.pos() - node_1.pos()).normalize_or_zero() * accel_mag;
        let accel_change_2 = -accel_change_1;

        // apply
        let node_1 = nodes.get_mut(self.node_1).unwrap();
        node_1.accel(accel_change_1);
        let node_2 = nodes.get_mut(self.node_2).unwrap();
        node_2.accel(accel_change_2);
        let joint_node = nodes.get_mut(self.joint_node).unwrap();
        joint_node.energy -= accel_change_1.length() * MUSCLE_ENERGY_RATE;
    }
    // pub fn update(&mut self, nodes: &mut CollectionView<Node>) {
    //     let (Some(joint_node), Some(node_1), Some(node_2)) =
    //         (nodes.get(self.joint_node), nodes.get(self.node_1), nodes.get(self.node_2)) else {
    //             self.delete = true;
    //             return;
    //         };
    //     // dont move if joint node is dead
    //     if !joint_node.is_alive() {
    //         return;
    //     }

    //     // let real_angle = match self.movement {
    //     //     Some((freq, amp, shift)) => {
    //     //         let angle = amp * (freq * tick as f32 + shift).sin();
    //     //         self.angle + Angle(angle)
    //     //     }
    //     //     None => self.angle,
    //     // };
    //     // get activation of joint node and add to angle
    //     let real_angle = self.angle + Angle(joint_node.unwrap_activate().clamp(-PI, PI));

    //     let angle_diff = Angle::from_pi_pi_range(
    //         (node_1.pos() - joint_node.pos()).angle_between(node_2.pos() - joint_node.pos()),
    //     )
    //     .0 - real_angle.0;

    //     // move towards each other
    //     let accel_change_1 =
    //         (node_1.pos() - joint_node.pos()).perp().normalize() * angle_diff * self.strength;
    //     let accel_change_2 =
    //         -(node_2.pos() - joint_node.pos()).perp().normalize() * angle_diff * self.strength;

    //     // apply
    //     let node_1 = nodes.get_mut(self.node_1).unwrap();
    //     node_1.accel(accel_change_1);
    //     let node_2 = nodes.get_mut(self.node_2).unwrap();
    //     node_2.accel(accel_change_2);
    //     let joint_node = nodes.get_mut(self.joint_node).unwrap();
    //     joint_node.energy -= accel_change_1.length() * ENERGY_LOSS_RATE * 0.25;
    // }
}
