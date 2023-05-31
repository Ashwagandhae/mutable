use nannou::prelude::*;

use super::bone::Bone;
use super::collection::Collection;
use super::collection::GenId;
use super::gene::Gene;
use super::gene::Genome;
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
    pub dead: bool,
}

impl Organism {
    pub fn build(pos: Vec2, genome: Genome, energy: f32, nodes: &mut Collection<Node>) -> Organism {
        // find first build in genome
        let (index, gene) = genome
            .genes
            .iter()
            .enumerate()
            .find(|(_, gene)| matches!(gene, Gene::Build { .. }))
            .unwrap();

        let Gene::Build {node: node_build, ..} = gene else { unreachable!() };

        let node_id = nodes.push(node_build.build(pos, index + 1, energy, None));
        Organism {
            genome,
            node_ids: vec![node_id],
            bone_ids: Vec::new(),
            muscle_ids: Vec::new(),
            new_organisms: Vec::new(),
            dead: false,
        }
    }
    fn clear_dead(
        &mut self,
        nodes: &Collection<Node>,
        bones: &Collection<Bone>,
        muscles: &Collection<Muscle>,
    ) {
        // clear dead nodes
        self.node_ids.retain(|id| nodes.get(*id).is_some());
        // clear dead bones
        self.bone_ids.retain(|id| bones.get(*id).is_some());
        // clear dead muscles
        self.muscle_ids.retain(|id| muscles.get(*id).is_some());
        // if all empty, die
        if self.node_ids.len() == 0 {
            self.dead = true;
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
            let gene = self.genome.get(nodes.get(node_id).unwrap().gene_index);
            // deal with skip

            match gene {
                Gene::Skip(index) => nodes.get_mut(node_id).unwrap().gene_index += index,
                _ => nodes.get_mut(node_id).unwrap().gene_index += 1,
            }

            // build gene
            let (Gene::Build { starting_energy, .. } | Gene::Egg{ starting_energy }) = gene else { continue };
            let build_energy = gene.build_energy_cost();
            let energy = *starting_energy + build_energy;
            if nodes.get(node_id).unwrap().energy < energy {
                continue;
            }
            nodes.get_mut(node_id).unwrap().energy -= energy;

            let spawn_direction = {
                let children = &self
                    .node_ids
                    .iter()
                    .map(|id| nodes.get(*id).unwrap())
                    .filter(|node| node.parent_id == Some(node_id))
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
                angle.as_vec2()
            };

            if let Gene::Egg { .. } = gene {
                // if egg, spawn organism
                let mut new_genome = self.genome.clone();
                if random::<f32>() < 0.2 {
                    new_genome.mutate();
                }
                let organism = Organism::build(
                    nodes.get(node_id).unwrap().pos + spawn_direction * 20.,
                    new_genome,
                    *starting_energy,
                    nodes,
                );
                self.new_organisms.push(organism);
                continue;
            }
            let Gene::Build { node: node_build, bone: bone_build, muscle: muscle_build, child_skip, .. } = gene else { continue };

            let child_node_id = nodes.push(node_build.build(
                nodes.get(node_id).unwrap().pos + spawn_direction * bone_build.length,
                nodes.get(node_id).unwrap().gene_index + child_skip,
                *starting_energy,
                Some(node_id),
            ));
            self.node_ids.push(child_node_id);
            let bone_id = bones.push(bone_build.build(node_id, child_node_id));
            self.bone_ids.push(bone_id);

            let parent_id = nodes.get(node_id).unwrap().parent_id;
            if muscle_build.is_none() || parent_id.is_none() {
                continue;
            }
            let muscle_build = muscle_build.as_ref().unwrap();
            let muscle_id =
                muscles.push(muscle_build.build(parent_id.unwrap(), child_node_id, node_id));
            self.muscle_ids.push(muscle_id);
        }
    }
}
