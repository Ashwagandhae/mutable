use std::collections::HashMap;

use nannou::prelude::*;

use super::bone::Bone;
use super::brain::Brain;
use super::collection::GenId;
use super::collection::{Collection, CollectionView};
use super::collide::Collider;
use super::gene::BuildId;
use super::gene::Gene;
use super::genome::Genome;
use super::math::Angle;
use super::muscle::Muscle;
use super::node::NodeKind;
use super::node::{LifeState, Node};

const MAX_NODE_CHILDREN: usize = 4;
fn get_node_children(
    nodes: &Collection<Node>,
    parent_id: GenId,
    possible_child_ids: &[GenId],
) -> Vec<GenId> {
    possible_child_ids
        .iter()
        .filter(|id| {
            nodes.get(**id).is_some_and(|node| match node.life_state {
                LifeState::Alive {
                    parent: Some((id, _)),
                    ..
                } if id == parent_id => true,
                _ => false,
            })
        })
        .map(|id| *id)
        .collect::<Vec<_>>()
}
fn get_spawn_direction(nodes: &Collection<Node>, spawn_pos: Point2, children: &[GenId]) -> Vec2 {
    let mut pos = Point2::new(0., 0.);
    for child_id in children {
        pos += nodes[*child_id].pos();
    }
    let mut average_pos = pos / children.len() as f32;
    if average_pos == Point2::new(0., 0.) || !average_pos.is_finite() {
        average_pos = Angle(random_range(0., TAU)).to_vec2() + spawn_pos;
    }

    // get angle from parent to average pos and reverse

    let angle = Angle((average_pos.y - spawn_pos.y).atan2(average_pos.x - spawn_pos.x)) + Angle(PI);

    // create unit vec
    angle.to_vec2()
}

#[derive(Debug, Clone)]
pub struct Organism {
    pub genome: Genome,
    pub brain: Brain,
    build_id_map: HashMap<BuildId, GenId>,
    node_ids: Vec<GenId>,
    pub new_organisms: Vec<Organism>,
    pub delete: bool,
    next_child_genome: Option<Genome>,
}

impl Organism {
    pub fn new(pos: Vec2, genome: Genome, energy: f32, nodes: &mut Collection<Node>) -> Organism {
        let (index, gene) = genome.body.get_start_gene();
        let Gene::Build((gene, _)) = gene else { unreachable!() };
        let index = genome.body.get_next_deeper(index);
        let brain = Brain::from_plan(&genome.brain);
        let node_id =
            nodes.push(gene.build_node(pos, index, energy, None, brain.does_calculate_neurons()));
        Organism {
            genome,
            brain,
            build_id_map: HashMap::new(),
            node_ids: vec![node_id],

            new_organisms: Vec::new(),
            delete: false,
            next_child_genome: None,
        }
    }

    pub fn clear_dead(&mut self, nodes: &Collection<Node>) {
        self.node_ids.retain(|id| match nodes.get(*id) {
            Some(node) => node.is_alive(),
            None => false,
        });
        self.build_id_map.retain(|_, id| nodes.get(*id).is_some());
        if self.node_ids.is_empty() {
            self.delete = true;
        }
    }
    pub fn think(&mut self, nodes: &mut CollectionView<Node>, tick: u64) {
        let think_energy = self
            .brain
            .step(&self.genome.brain, &self.build_id_map, nodes, tick);
        for node_id in &self.node_ids {
            // safe because no 2 organisms share nodes
            nodes.get_mut(*node_id).map(|node| {
                node.energy -= think_energy / self.node_ids.len() as f32;
            });
        }
    }
    pub fn grow(
        &mut self,
        nodes: &mut Collection<Node>,
        bones: &mut Collection<Bone>,
        muscles: &mut Collection<Muscle>,
    ) {
        for i in (0..self.node_ids.len()).rev() {
            self.grow_node(self.node_ids[i], nodes, bones, muscles);
        }
    }
    pub fn reproduce(&mut self, nodes: &mut Collection<Node>, collider: &Collider) {
        for i in (0..self.node_ids.len()).rev() {
            self.reproduce_node(self.node_ids[i], nodes, collider);
        }
    }
    fn grow_node(
        &mut self,
        node_id: GenId,
        nodes: &mut Collection<Node>,
        bones: &mut Collection<Bone>,
        muscles: &mut Collection<Muscle>,
    ) {
        // make sure node is alive
        let Some(Node { life_state: LifeState::Alive { .. }, .. }) = nodes.get(node_id) else { return };
        // get next build gene if there is one
        let Some(real_gene_index) = nodes[node_id].unwrap_gene_index() else {return};
        let Some((build_gene_index, (gene, build_id))) = self.genome.body.get_build(*real_gene_index) else {
                *nodes[node_id].unwrap_gene_index_mut() = None;
                return;
            };

        let energy_cost = gene.energy_cost();
        if nodes[node_id].energy < energy_cost {
            return;
        }
        let children = get_node_children(nodes, node_id, &self.node_ids);
        if children.len() >= MAX_NODE_CHILDREN {
            return;
        }
        *nodes[node_id].unwrap_gene_index_mut() = self.genome.body.get_next(*real_gene_index);
        let child_gene_index = self.genome.body.get_next_deeper(build_gene_index);
        nodes[node_id].energy -= energy_cost;

        // build node
        let spawn_direction = get_spawn_direction(nodes, nodes[node_id].pos(), &children);
        let child_start_pos = nodes[node_id].pos() + spawn_direction * gene.bone_length;
        let child_id = nodes.push(gene.build_node(
            child_start_pos,
            child_gene_index,
            gene.starting_energy,
            Some((node_id, nodes[node_id].pos())),
            self.brain.does_calculate_neurons(),
        ));
        self.node_ids.push(child_id);
        self.build_id_map.insert(build_id.clone(), child_id);

        // build bone
        let min_length = nodes[node_id].radius + nodes[child_id].radius;
        bones.push(gene.build_bone(node_id, child_id, min_length));

        // build muscle
        if gene.has_muscle == 0 {
            return;
        }
        let node_1 = if gene.muscle_is_sibling == 1 {
            // attach muscle to last_child--(node)--child
            children.last().map(|id| *id)
        } else {
            // attach muscle to parent--(node)--child
            nodes[node_id].unwrap_parent_id()
        };
        let Some(node_1) = node_1 else {return};
        muscles.push(gene.build_muscle(node_id, node_1, child_id).unwrap());
    }
    pub fn reproduce_node(
        &mut self,
        node_id: GenId,
        nodes: &mut Collection<Node>,
        collider: &Collider,
    ) {
        // make sure node is alive
        let Some(Node { life_state: LifeState::Alive { .. }, .. }) = nodes.get(node_id) else { return };
        // make sure node is an egg
        let NodeKind::Egg = nodes[node_id].unwrap_kind() else { return };

        let children = get_node_children(nodes, node_id, &self.node_ids);
        let spawn_direction = get_spawn_direction(nodes, nodes[node_id].pos(), &children);

        // save genome for child, so that high energy cost genes aren't deleted
        self.next_child_genome.get_or_insert_with(|| {
            let mut new_genome = self.genome.clone();
            if random::<f32>() < 0.5 {
                new_genome.mutate();
            }
            new_genome
        });
        let new_genome = self.next_child_genome.take().unwrap();

        let Gene::Build((build_gene, _)) = new_genome.body.get_start_gene().1 else {unreachable!()};
        let energy_cost = build_gene.energy_cost() + new_genome.body.len() as f32 / 10.;

        let min_child_distance = nodes[node_id].radius + build_gene.node_radius;
        let child_start_pos =
            nodes[node_id].pos() + spawn_direction * min_child_distance.max(build_gene.bone_length);

        // check if child will collide with anything, if so, don't spawn
        let child_would_collide = collider
            .pos_collides_iter(&nodes.view(), child_start_pos)
            .any(|node| {
                let dist_squared = node.pos().distance_squared(child_start_pos);
                let min_dist_squared = (node.radius + build_gene.node_radius).powi(2);
                dist_squared < min_dist_squared
            });
        if child_would_collide {
            return;
        }

        if nodes[node_id].energy < energy_cost {
            return;
        }
        nodes[node_id].energy -= energy_cost;

        let starting_energy = build_gene.starting_energy;
        let organism = Organism::new(child_start_pos, new_genome, starting_energy, nodes);
        self.new_organisms.push(organism);
    }
    pub fn node_ids(&self) -> &[GenId] {
        &self.node_ids
    }
}
