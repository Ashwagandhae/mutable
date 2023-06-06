use nannou::prelude::*;

use super::bone::Bone;
use super::collection::Collection;
use super::collection::GenId;
use super::gene::Gene;
use super::gene::Genome;
use super::gene::SkipGene;
use super::math::Angle;
use super::muscle::Muscle;
use super::node::Node;

#[derive(Debug, Clone)]
pub struct Organism {
    genome: Genome,
    node_ids: Vec<GenId>,
    bone_ids: Vec<GenId>,
    muscle_ids: Vec<GenId>,
    pub new_organisms: Vec<Organism>,
    pub delete: bool,
    next_child_genome: Option<Genome>,
}

impl Organism {
    pub fn build(pos: Vec2, genome: Genome, energy: f32, nodes: &mut Collection<Node>) -> Organism {
        let (index, gene) = genome.get_start_gene();
        let Gene::Build(gene) = gene else { unreachable!() };
        let node_id = nodes.push(gene.build_node(pos, index + 1, energy, None));
        Organism {
            genome,
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
        self.bone_ids.retain(|id| bones.get(*id).is_some());
        self.muscle_ids.retain(|id| muscles.get(*id).is_some());
        if self.node_ids.len() == 0 {
            self.delete = true;
        }
    }
    #[inline(never)]
    pub fn grow(
        &mut self,
        nodes: &mut Collection<Node>,
        bones: &mut Collection<Bone>,
        muscles: &mut Collection<Muscle>,
    ) {
        self.clear_dead(nodes, bones, muscles);
        // grow nodes
        for i in (0..self.node_ids.len()).rev() {
            let node_id = self.node_ids[i];
            let gene_index = nodes[node_id].unwrap_gene_index_mut();
            let gene = self.genome.get(*gene_index);
            // deal with skip
            match gene {
                Gene::Skip(SkipGene { skip }) => {
                    *gene_index += skip;
                    continue;
                }
                Gene::Junk => {
                    *gene_index += 1;
                    continue;
                }
                Gene::Stop => {
                    continue;
                }
                _ => (),
            }

            let spawn_direction = {
                let children = &self
                    .node_ids
                    .iter()
                    .map(|id| &nodes[*id])
                    .filter(|node| *node.unwrap_parent_id() == Some(node_id))
                    .collect::<Vec<_>>();
                let mut pos = Point2::new(0., 0.);
                for child in children {
                    pos += child.pos;
                }
                let mut average_pos = pos / children.len() as f32;
                if average_pos == Point2::new(0., 0.) || !average_pos.is_finite() {
                    average_pos = Point2::new(1., 0.);
                }

                // get angle from parent to average pos and reverse
                let parent = nodes.get(node_id).unwrap();
                let angle =
                    Angle((average_pos.y - parent.pos.y).atan2(average_pos.x - parent.pos.x))
                        + Angle(PI);

                // create vec of right length
                angle.to_vec2()
            };

            if let Gene::Egg(gene) = gene {
                // save genome for child, so that high energy cost genes aren't deleted
                self.next_child_genome.get_or_insert_with(|| {
                    let mut new_genome = self.genome.clone();
                    if random::<f32>() < 0.2 {
                        for _ in 0..random_range(1, 3) {
                            new_genome.mutate();
                        }
                    }
                    new_genome
                });
                let new_genome = self.next_child_genome.take().unwrap();

                let Gene::Build(build_gene) = new_genome.get_start_gene().1 else {unreachable!()};
                let energy_cost = build_gene.energy_cost() + gene.starting_energy;

                if nodes[node_id].energy < energy_cost {
                    continue;
                }
                nodes[node_id].energy -= energy_cost;
                let gene_index = nodes[node_id].unwrap_gene_index_mut();
                *gene_index += 1;

                let child_start_pos = nodes[node_id].pos + spawn_direction * gene.child_distance;

                let organism =
                    Organism::build(child_start_pos, new_genome, gene.starting_energy, nodes);
                self.new_organisms.push(organism);
            } else if let Gene::Build(gene) = gene {
                let energy_cost = gene.energy_cost();

                if nodes[node_id].energy < energy_cost {
                    continue;
                }
                nodes[node_id].energy -= energy_cost;
                let gene_index = nodes[node_id].unwrap_gene_index_mut();
                *gene_index += 1;

                let gene_index = *gene_index;

                let child_start_pos = nodes[node_id].pos + spawn_direction * gene.bone_length;
                // build node
                let child_id = nodes.push(gene.build_node(
                    child_start_pos,
                    gene_index + gene.child_skip,
                    gene.starting_energy,
                    Some(node_id),
                ));
                self.node_ids.push(child_id);

                // build bone
                let min_length = nodes[node_id].radius + nodes[child_id].radius;
                let bone_id = bones.push(gene.build_bone(node_id, child_id, min_length));
                self.bone_ids.push(bone_id);

                // build muscle
                let parent_id = nodes[node_id].unwrap_parent_id();
                if gene.has_muscle == 0 || parent_id.is_none() {
                    continue;
                }
                // unwrap is safe because we know it has a muscle and a parent
                let parent_id = parent_id.unwrap();
                let muscle_id =
                    muscles.push(gene.build_muscle(node_id, parent_id, child_id).unwrap());
                self.muscle_ids.push(muscle_id);
            } else {
                unreachable!()
            };
        }
    }
}
