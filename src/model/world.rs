use nannou::prelude::*;

mod bone;
mod collection;
mod collide;
mod init;
mod math;
mod muscle;
mod node;
mod organism;

use bone::Bone;
use collection::Collection;
use collide::Collider;
use muscle::Muscle;
use node::Node;
// use organism::Organism;

use init::random_trees;

use self::init::fish;

pub const MAX_NODE_RADIUS: f32 = 10.0;

#[derive(Debug, Clone)]
pub struct World {
    pub nodes: Collection<Node>,
    pub bones: Collection<Bone>,
    pub muscles: Collection<Muscle>,
    pub size: Vec2,
    pub collider: Collider,
    pub tick: u64,
}

impl World {
    pub fn new() -> World {
        let mut nodes = Collection::new();
        let mut bones = Collection::new();
        let mut muscles = Collection::new();

        fish(&mut nodes, &mut bones, &mut muscles);
        random_trees(&mut nodes, &mut bones, &mut muscles);
        let size = vec2(10000., 10000.);

        World {
            nodes,
            bones,
            muscles,
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
        // update bones
        for bone in self.bones.iter_mut() {
            bone.update(&mut self.nodes);
        }
        // update muscles
        for muscle in self.muscles.iter_mut() {
            muscle.update(&mut self.nodes, self.tick);
        }
        // collide nodes with collider
        if self.tick % 1 == 0 {
            self.collider.update(&self.nodes);
            for (i, j) in self.collider.collide() {
                let node_1 = self.nodes.get(i).unwrap();
                let node_2 = self.nodes.get(j).unwrap();
                let dist = node_1.pos.distance(node_2.pos);
                let min_dist = node_1.radius + node_2.radius;
                if dist < min_dist {
                    // move them away from each other
                    let diff = node_1.pos - node_2.pos;
                    let diff = diff.normalize() * (min_dist - dist) / 2.0;
                    self.nodes.get_mut(i).unwrap().pos += diff;
                    self.nodes.get_mut(j).unwrap().pos -= diff;
                }
            }
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
        self.tick += 1;
    }
}
