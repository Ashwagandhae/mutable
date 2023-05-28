use nannou::prelude::*;

mod bone;
mod collection;
mod collide;
mod gene;
mod init;
mod math;
mod muscle;
pub mod node;
mod organism;

use bone::Bone;
use collection::Collection;
use collide::Collider;
use init::random_organisms;
use muscle::Muscle;
use node::Node;
use organism::Organism;

pub const MAX_NODE_RADIUS: f32 = 10.0;

#[derive(Debug, Clone)]
pub struct World {
    pub nodes: Collection<Node>,
    pub bones: Collection<Bone>,
    pub muscles: Collection<Muscle>,
    pub organisms: Collection<Organism>,
    pub size: Vec2,
    pub collider: Collider,
    pub tick: u64,
}

impl World {
    pub fn new() -> World {
        let mut nodes = Collection::new();
        let mut bones = Collection::new();
        let mut muscles = Collection::new();
        let mut organisms = Collection::new();

        random_organisms(&mut nodes, &mut bones, &mut muscles, &mut organisms);

        let size = vec2(10000., 10000.);

        World {
            nodes,
            bones,
            muscles,
            organisms,
            size,
            collider: Collider::new(size),
            tick: 0,
        }
    }
    pub fn skip(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.update();
        }
    }
    pub fn update(&mut self) {
        // update nodes
        for node in self.nodes.iter_mut() {
            node.update();
        }
        for node in self.nodes.iter_mut() {
            node.update();
        }
        self.nodes.remove_when(|node| node.dead);
        // update bones
        for bone in self.bones.iter_mut() {
            bone.update(&mut self.nodes);
        }
        self.bones.remove_when(|bone| bone.dead);

        // update muscles
        for muscle in self.muscles.iter_mut() {
            muscle.update(&mut self.nodes, self.tick);
        }
        self.muscles.remove_when(|muscle| muscle.dead);

        // collide nodes with collider
        self.collider.update(&self.nodes);
        for (i, j) in self.collider.collide() {
            collide_nodes(&mut self.nodes, i, j);
        }

        // keep nodes in bounds
        for node in self.nodes.iter_mut() {
            if node.pos.x < 0.0 {
                node.pos.x = 0.0;
            }
            if node.pos.x > self.size.x {
                node.pos.x = self.size.x;
            }
            if node.pos.y < 0.0 {
                node.pos.y = 0.0;
            }
            if node.pos.y > self.size.y {
                node.pos.y = self.size.y;
            }
        }

        // grow nodes sometimes
        if self.tick % 64 == 0 {
            let mut new_organisms = Vec::new();
            for organism in self.organisms.iter_mut() {
                organism.grow(&mut self.nodes, &mut self.bones, &mut self.muscles);
                new_organisms.extend(organism.new_organisms.drain(..));
            }
            self.organisms.extend(&mut new_organisms);
        }
        self.organisms.remove_when(|organism| organism.dead);

        self.tick += 1;
    }
}

// fn collide_nodes(nodes: &mut Collection<Node>, i: usize, j: usize) {
//     let node_1 = nodes.get(i).unwrap();
//     let node_2 = nodes.get(j).unwrap();
//     let dist = node_1.pos.distance(node_2.pos);
//     let min_dist = node_1.radius + node_2.radius;
//     if dist < min_dist {
//         // move them away from each other
//         let diff = node_1.pos - node_2.pos;
//         let pos_change = diff.normalize() * (min_dist - dist) / 2.0;

//         let diff_norm = diff.normalize();
//         // get the part of vel_1 and vel_2 that is in the direction of diff_norm
//         let vel_1 = diff_norm * diff_norm.dot(node_1.vel);
//         let vel_2 = -diff_norm * -diff_norm.dot(node_2.vel);

//         nodes.get_mut(i).unwrap().pos += pos_change;
//         nodes.get_mut(j).unwrap().pos -= pos_change;
//         nodes.get_mut(i).unwrap().vel = -vel_1;
//         nodes.get_mut(j).unwrap().vel = -vel_2;
//     }
// }

// fn collide_nodes(nodes: &mut Collection<Node>, i: usize, j: usize) {
//     let node_1 = nodes.get(i).unwrap();
//     let node_2 = nodes.get(j).unwrap();
//     let dist = node_1.pos.distance(node_2.pos);
//     let min_dist = node_1.radius + node_2.radius;
//     if dist < min_dist {
//         // move them away from each other
//         let diff = node_1.pos - node_2.pos;
//         let pos_change = diff.normalize() * (min_dist - dist) / 2.0;

//         let e = 1.;

//         // Calculate the total mass
//         let total_mass = node_1.get_mass() + node_2.get_mass();

//         // Calculate the relative vel
//         let v_rel = node_2.vel - node_1.vel;

//         // Calculate the impulse
//         let impulse = (1. + e) * v_rel.dot(diff.normalize()) / total_mass;

//         // Update sphere velocities
//         let vel_1 = impulse / node_1.get_mass() * e;
//         let vel_2 = -1. * (impulse / node_2.get_mass()) * e;

//         nodes.get_mut(i).unwrap().pos += pos_change;
//         nodes.get_mut(j).unwrap().pos -= pos_change;
//         nodes.get_mut(i).unwrap().vel += vel_1;
//         nodes.get_mut(j).unwrap().vel += vel_2;
//     }
// }
fn collide_nodes(nodes: &mut Collection<Node>, i: usize, j: usize) {
    let node_1 = nodes.get(i).unwrap();
    let node_2 = nodes.get(j).unwrap();
    let dist = node_1.pos.distance(node_2.pos);
    let min_dist = node_1.radius + node_2.radius;
    if dist < min_dist {
        // move them away from each other
        let diff = node_1.pos - node_2.pos;
        let pos_change = diff.normalize() * (min_dist - dist) / 2.0;

        let node_1 = nodes.get_mut(i).unwrap();
        node_1.pos += pos_change;
        node_1.pressure += 1.;
        let node_2 = nodes.get_mut(j).unwrap();
        node_2.pos -= pos_change;
        node_2.pressure += 1.;
    }
}
