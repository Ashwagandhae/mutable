use nannou::prelude::*;

mod bone;
pub mod chunks;
pub mod collection;
mod collide;
mod gene;
mod init;
mod math;
mod muscle;
pub mod node;
mod organism;

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

pub const MAX_NODE_RADIUS: f32 = 20.0;

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
        let size = vec2(2000., 2000.);
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

        self.tick += 1;
    }
    fn update_bones(&mut self) {
        for bone in self.bones.iter_mut() {
            bone.update(&mut self.nodes);
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
        for node in self.nodes.iter_mut() {
            node.update(self.chunks.get(node.pos));
        }
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
        for node in self.nodes.iter_mut() {
            match node.life_state {
                LifeState::Dead { ref mut decay, .. } => {
                    // decay faster if bigger
                    if *decay >= (2048.0 / node.radius * 10.0) as u32 {
                        *decay = 0;
                        node.splat = true;
                    }
                }
                _ => {}
            }
        }
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
                        Node::new_dead(node.pos + splat_vec * new_radius, new_radius, new_energy);
                    self.nodes.push(splat_node);

                    let node = self.nodes.get_index_mut(i).unwrap();
                    node.pos = node.pos - splat_vec * new_radius;
                    node.radius = new_radius;
                    node.energy = new_energy;
                    node.splat = false;
                } else {
                    self.nodes.get_index_mut(i).unwrap().delete = true;
                }
            }
        }

        self.nodes.retain(|node| !node.delete);

        // collide nodes with collider
        self.collider.par_collide(&mut self.nodes, collide_pair);
        // self.nodes.retain(|node| node.cramming < 6); // TODO: revert this
        for node in self.nodes.iter_mut() {
            if node.cramming > 8 {
                node.die();
            }
            node.cramming = 0;
        }

        // keep nodes in bounds
        for node in self.nodes.iter_mut() {
            node.pos.x = node.pos.x.clamp(0.0, self.size.x);
            node.pos.y = node.pos.y.clamp(0.0, self.size.y);
        }
    }

    fn grow_organisms(&mut self) {
        let mut new_organisms = Vec::new();
        for organism in self.organisms.iter_mut() {
            organism.grow(&mut self.nodes, &mut self.bones, &mut self.muscles);
            new_organisms.extend(organism.new_organisms.drain(..));
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
    let dist = node_1.pos.distance(node_2.pos);
    let min_dist = node_1.radius + node_2.radius;
    if dist < min_dist {
        // move them away from each other
        let diff = node_1.pos - node_2.pos;
        let pos_change = diff.normalize_or_zero() * (min_dist - dist) / 2.0;

        node_1.pos += pos_change;
        node_1.cramming = node_1.cramming.saturating_add(1);
        node_2.pos -= pos_change;
        node_2.cramming = node_2.cramming.saturating_add(1);

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
                actor.energy += object.energy;
                object.energy = 0.;
                object.delete = true;
            }
            NodeKind::Spike => {
                let vel_threshold = object.radius.powi(2) / actor.radius.powi(2) * 0.5;
                // get vel towards object and compare to threshold
                let vel_towards_object =
                    actor.vel.dot((object.pos - actor.pos).normalize_or_zero());
                if vel_towards_object < vel_threshold {
                    return;
                }
                if object.radius < 3.0 {
                    return;
                }
                object.splat = true;
            }
            _ => {}
        },
        LifeState::Dead { .. } => (),
    }
}
