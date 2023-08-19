use std::collections::HashMap;

use nannou::prelude::*;

use super::bone::Bone;
use super::brain::Brain;
use super::collection::Collection;
use super::collection::GenId;
use super::collide::Collider;
use super::gene::BuildId;
use super::gene::Gene;
use super::genome::Genome;
use super::math::Angle;
use super::muscle::Muscle;
use super::node::Node;
use super::node::NodeKind;

const MAX_NODE_CHILDREN: usize = 4;
#[derive(Debug, Clone)]
pub struct Organism {
    pub genome: Genome,
    pub brain: Brain,
    build_id_map: HashMap<BuildId, GenId>,
    node_ids: Vec<GenId>,
    bone_ids: Vec<GenId>,
    muscle_ids: Vec<GenId>,
    pub new_organisms: Vec<Organism>,
    pub delete: bool,
    next_child_genome: Option<Genome>,
}

impl Organism {
    pub fn new(pos: Vec2, genome: Genome, energy: f32, nodes: &mut Collection<Node>) -> Organism {
        let (index, gene) = genome.body.get_start_gene();
        let Gene::Build((gene, _)) = gene else { unreachable!() };
        let index = genome.body.get_next_deeper(index);
        let node_id = nodes.push(gene.build_node(pos, index, energy, None));
        let brain = Brain::from_plan(&genome.brain);
        Organism {
            genome,
            brain,
            build_id_map: HashMap::new(),
            node_ids: vec![node_id],
            bone_ids: Vec::new(),
            muscle_ids: Vec::new(),
            new_organisms: Vec::new(),
            delete: false,
            next_child_genome: None,
        }
    }
    fn clear_dead(
        &mut self,
        nodes: &Collection<Node>,
        bones: &Collection<Bone>,
        muscles: &Collection<Muscle>,
    ) {
        self.node_ids.retain(|id| match nodes.get(*id) {
            Some(node) => node.is_alive(),
            None => false,
        });
        self.build_id_map.retain(|_, id| nodes.get(*id).is_some());
        self.bone_ids.retain(|id| bones.get(*id).is_some());
        self.muscle_ids.retain(|id| muscles.get(*id).is_some());
        if self.node_ids.is_empty() {
            self.delete = true;
        }
    }
    pub fn think(&mut self, nodes: &mut Collection<Node>, tick: u64) {
        let think_energy = self
            .brain
            .step(&self.genome.brain, &self.build_id_map, nodes, tick);
        for node_id in &self.node_ids {
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
        collider: &Collider,
    ) {
        self.clear_dead(nodes, bones, muscles);
        // grow nodes
        for i in (0..self.node_ids.len()).rev() {
            let node_id = self.node_ids[i];

            let Some(real_gene_index) = nodes[node_id].unwrap_gene_index() else {continue};
            let (gene_index, gene) = {
                let Some(real_gene) = self.genome.body.get(*real_gene_index) else { unreachable!() };
                match real_gene {
                    // if its repeat, try to find first non repeat gene
                    Gene::Repeat => {
                        let Some((index, gene)) = self.genome.body.get_next_non_repeat(*real_gene_index).map(|index|{
                        self.genome.body.get(index).map(|gene| (index, gene))
                    }).flatten() else {
                        // if no non repeat gene is found, we are at the end, so stop gene_index
                        *nodes[node_id].unwrap_gene_index_mut() = None;
                        continue;
                    };
                        (index, gene)
                    }
                    Gene::Build(_) => (*real_gene_index, real_gene),
                    _ => {
                        unreachable!()
                    }
                }
            };

            let children = &self
                .node_ids
                .iter()
                .map(|id| (*id, &nodes[*id]))
                .filter(|(_, node)| node.unwrap_parent_id() == Some(node_id))
                .map(|(id, _)| id)
                .collect::<Vec<_>>();
            let spawn_direction = {
                let mut pos = Point2::new(0., 0.);
                for child_id in children {
                    pos += nodes[*child_id].pos();
                }
                let mut average_pos = pos / children.len() as f32;
                if average_pos == Point2::new(0., 0.) || !average_pos.is_finite() {
                    average_pos = Angle(random_range(0., TAU)).to_vec2() + nodes[node_id].pos();
                }

                // get angle from parent to average pos and reverse

                let angle = Angle(
                    (average_pos.y - nodes[node_id].pos().y)
                        .atan2(average_pos.x - nodes[node_id].pos().x),
                ) + Angle(PI);

                // create vec of right length
                angle.to_vec2()
            };

            //TODO make this separate from build function
            if let NodeKind::Egg = nodes[node_id].unwrap_kind() {
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
                let child_start_pos = nodes[node_id].pos()
                    + spawn_direction * min_child_distance.max(build_gene.bone_length);

                // check if child will collide with anything, if so, don't spawn
                let child_would_collide =
                    collider
                        .pos_collides_iter(nodes, child_start_pos)
                        .any(|node| {
                            let dist_squared = node.pos().distance_squared(child_start_pos);
                            let min_dist_squared = (node.radius + build_gene.node_radius).powi(2);
                            dist_squared < min_dist_squared
                        });
                if child_would_collide {
                    continue;
                }

                if nodes[node_id].energy < energy_cost {
                    continue;
                }
                nodes[node_id].energy -= energy_cost;
                // dont advance gene index, so that the egg is built again forever
                // *nodes[node_id].unwrap_gene_index_mut() = new_genome.get_next(*gene_index);

                let starting_energy = build_gene.starting_energy;
                let organism = Organism::new(child_start_pos, new_genome, starting_energy, nodes);
                self.new_organisms.push(organism);
            } else if let Gene::Build((gene, build_id)) = gene {
                let energy_cost = gene.energy_cost();

                if nodes[node_id].energy < energy_cost {
                    continue;
                }
                if children.len() >= MAX_NODE_CHILDREN {
                    continue;
                }
                let child_index = self.genome.body.get_next_deeper(gene_index);
                *nodes[node_id].unwrap_gene_index_mut() =
                    self.genome.body.get_next(*real_gene_index);
                nodes[node_id].energy -= energy_cost;

                let child_start_pos = nodes[node_id].pos() + spawn_direction * gene.bone_length;
                // build node
                let child_id = nodes.push(gene.build_node(
                    child_start_pos,
                    child_index,
                    gene.starting_energy,
                    Some((node_id, nodes[node_id].pos())),
                ));
                self.node_ids.push(child_id);
                self.build_id_map.insert(build_id.clone(), child_id);

                // build bone
                let min_length = nodes[node_id].radius + nodes[child_id].radius;
                let bone_id = bones.push(gene.build_bone(node_id, child_id, min_length));
                self.bone_ids.push(bone_id);

                // build muscle
                if gene.has_muscle == 0 {
                    continue;
                }
                let node_1 = if gene.muscle_is_sibling == 1 {
                    // attach muscle to last_child--(node)--child
                    children.last().map(|id| *id)
                } else {
                    // attach muscle to parent--(node)--child
                    nodes[node_id].unwrap_parent_id()
                };
                let Some(node_1) = node_1 else {continue};
                let muscle_id = muscles.push(gene.build_muscle(node_id, node_1, child_id).unwrap());
                self.muscle_ids.push(muscle_id);
            } else {
                unreachable!()
            };
        }
    }
    pub fn node_ids(&self) -> &[GenId] {
        &self.node_ids
    }
}
