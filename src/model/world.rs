use nannou::prelude::*;

mod bone;
mod brain;
pub mod chunks;
pub mod collection;
mod collide;
pub mod gene;
mod init;
mod math;
mod muscle;
pub mod node;
pub mod organism;

use bone::Bone;
use chunks::Chunks;
use collection::Collection;
use collide::Collider;
use init::random_organisms;
use muscle::Muscle;
use node::Node;
use organism::Organism;

use math::Angle;
use node::{LifeState, NodeKind};
use rayon::prelude::ParallelIterator;

use self::math::is_zero_vec2;

pub const MAX_NODE_RADIUS: f32 = 15.0;

#[derive(Debug, Clone)]
pub struct World {
    pub nodes: Collection<Node>,
    pub bones: Collection<Bone>,
    pub muscles: Collection<Muscle>,
    pub organisms: Collection<Organism>,
    pub chunks: Chunks,
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
        let size = vec2(3375., 3375.);
        let chunks = Chunks::new(size, 40.0);

        random_organisms(&mut nodes, &mut bones, &mut muscles, &mut organisms, size);

        World {
            nodes,
            bones,
            muscles,
            organisms,
            size,
            chunks,
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
        self.update_bones();

        self.update_muscles();

        self.update_nodes();

        if self.tick % 64 == 0 {
            self.grow_organisms();
        }

        if self.tick % 16 == 0 {
            self.chunks.update(self.tick);
        }

        if self.nodes.len() == 0 {
            println!("All nodes dead");
            random_organisms(
                &mut self.nodes,
                &mut self.bones,
                &mut self.muscles,
                &mut self.organisms,
                self.size,
            );
        }

        self.tick += 1;
    }
    fn update_bones(&mut self) {
        for bone in self.bones.iter_mut() {
            bone.update(&mut self.nodes, &self.chunks);
        }
        self.bones.retain(|bone| !bone.delete);
    }
    fn update_muscles(&mut self) {
        for muscle in self.muscles.iter_mut() {
            muscle.update(&mut self.nodes, self.tick);
        }
        self.muscles.retain(|muscle| !muscle.delete);
    }
    fn update_nodes(&mut self) {
        self.nodes.par_iter_mut().for_each(|node| {
            node.update(self.chunks.get(node.pos()));
        });
        // kill nodes if no parent
        for i in 0..self.nodes.full_len() {
            let Some(Node {
                life_state:
                    LifeState::Alive {
                        parent_id: Some(parent_id),
                        ..
                    },
                ..
            }) = self.nodes.get_index(i) else {continue};
            let parent_dead = match self.nodes.get(*parent_id) {
                Some(node) => !node.is_alive(),
                None => true,
            };
            if parent_dead {
                self.nodes.get_index_mut(i).unwrap().die();
            }
        }

        // splat nodes if they decay
        self.nodes.par_iter_mut().for_each(|node| {
            if let LifeState::Dead { ref mut decay, .. } = node.life_state {
                // decay faster if bigger
                if *decay >= (2048.0 / node.radius * 10.0) as u32 {
                    *decay = 0;
                    node.splat = true;
                }
            }
        });
        for i in 0..self.nodes.full_len() {
            let Some(node) = self.nodes.get_index(i) else {continue};
            if node.splat {
                let new_radius = node.radius / 2.0;
                let energy = node.energy / 2.0;
                // dont splat if too small
                if new_radius > 3.0 {
                    let new_energy = energy / 2.0;
                    let splat_vec = Angle(random_range(0.0, 2.0 * PI)).to_vec2();

                    let splat_node =
                        Node::new_dead(node.pos() + splat_vec * new_radius, new_radius, new_energy);
                    self.nodes.push(splat_node);

                    let node = self.nodes.get_index_mut(i).unwrap();
                    *node.pos_mut() -= splat_vec * new_radius;
                    node.radius = new_radius;
                    node.energy = new_energy;
                    node.splat = false;
                    node.vel = vec2(0.0, 0.0);
                } else {
                    self.nodes.get_index_mut(i).unwrap().delete = true;
                }
            }
        }

        self.nodes.retain(|node| !node.delete);

        // collide nodes with collider
        self.collider.par_collide(&mut self.nodes, collide_pair);

        // keep nodes in bounds
        for node in self.nodes.iter_mut() {
            if node.pos().x < 0.0
                || node.pos().x >= self.size.x
                || node.pos().y < 0.0
                || node.pos().y >= self.size.y
            {
                *node.pos_mut() = node.pos().clamp(vec2(0.0, 0.0), self.size - vec2(0.1, 0.1));
            }
        }
    }

    fn grow_organisms(&mut self) {
        let mut new_organisms = Vec::new();
        for organism in self.organisms.iter_mut() {
            organism.grow(
                &mut self.nodes,
                &mut self.bones,
                &mut self.muscles,
                &self.collider,
            );
            new_organisms.append(&mut organism.new_organisms);
        }
        self.organisms.extend(&mut new_organisms);
        self.organisms.retain(|organism| !organism.delete);
    }
}

// fn collide_pair(nodes: &mut Collection<Node>, i: usize, j: usize) {
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

// fn collide_pair(nodes: &mut Collection<Node>, i: usize, j: usize) {
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
fn collide_pair(node_1: &mut Node, node_2: &mut Node) {
    // if let (PosChange::None, PosChange::None) = (&node_1.pos_change, &node_2.pos_change) {
    //     return;
    // }

    let dist_squared = node_1.pos().distance_squared(node_2.pos());
    let min_dist_squared = (node_1.radius + node_2.radius).powi(2);
    if dist_squared < min_dist_squared {
        // move them away from each other
        let dist = dist_squared.sqrt();
        let min_dist = node_1.radius + node_2.radius;
        let diff = node_1.pos() - node_2.pos();
        let pos_change = diff / dist * (min_dist - dist) / 2.0;
        if is_zero_vec2(pos_change) {
            return;
        }

        *node_1.pos_mut() += pos_change;
        *node_2.pos_mut() -= pos_change;

        interact_pair(node_1, node_2);
        interact_pair(node_2, node_1);
    }
}

fn interact_pair(actor: &mut Node, object: &mut Node) {
    if actor.delete || object.delete {
        return;
    }
    match &mut actor.life_state {
        LifeState::Alive { kind, .. } => match kind {
            NodeKind::Mouth => {
                if actor.radius * 0.9 < object.radius {
                    return;
                }
                actor.energy += object.energy + object.struct_energy();
                object.delete = true;
            }
            NodeKind::Spike => {
                let vel_threshold = object.radius.powi(2) / actor.radius.powi(2) * 1.0;
                // get vel towards object and compare to threshold
                let relative_vel = actor.vel - object.vel;
                let vel_towards_object =
                    relative_vel.dot((object.pos() - actor.pos()).normalize_or_zero());
                if vel_towards_object < vel_threshold {
                    return;
                }
                if object.radius < 4.0 {
                    return;
                }
                object.splat = true;
            }
            _ => {}
        },
        LifeState::Dead { .. } => (),
    }
}
