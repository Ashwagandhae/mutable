use super::bone::Bone;
use super::collection::GenId;
use super::math::Angle;
use super::muscle::Muscle;
use super::node::{Node, NodeKind};
use nannou::prelude::*;

mod macros;
use super::MAX_NODE_RADIUS;
use macros::{count_fields, make_gene_struct, replace_expr};

make_gene_struct!(pub BuildGene {
    node_radius: f32 = 2.0..MAX_NODE_RADIUS,
    node_energy_weight: f32 = 1.0..10.0,
    node_kind: u8 = 0..5,
    node_lifespan: u32 = 256..32_768,

    bone_length: f32 = 5.0..30.0,

    has_muscle: u8 = 0..2,
    muscle_angle: f32 = 0.0..TAU,
    muscle_strength: f32 = 0.5..2.0,
    muscle_has_movement: u8 = 0..2,
    muscle_is_sibling: u8 = 0..2,
    muscle_freq: f32 = 0.1..1.0,
    muscle_amp: f32 = 0.0..1.5,
    muscle_shift: f32 = 0.0..PI,

    starting_energy: f32 = 0.0..30.0,
});

impl BuildGene {
    pub fn build_node(
        &self,
        pos: Point2,
        gene_index: Option<usize>,
        energy: f32,
        parent_id: Option<GenId>,
    ) -> Node {
        let kind = NodeKind::from(self.node_kind);
        let energy_weight = self.node_energy_weight;
        let lifespan = self.node_lifespan;
        let radius = self.node_radius;

        Node::new(
            pos,
            radius,
            energy,
            energy_weight,
            kind,
            gene_index,
            parent_id,
            lifespan,
        )
    }
    pub fn build_bone(&self, node_1: GenId, node_2: GenId, min_length: f32) -> Bone {
        let length = self.bone_length.max(min_length);
        Bone::new(node_1, node_2, length)
    }
    pub fn build_muscle(&self, joint_id: GenId, node_1: GenId, node_2: GenId) -> Option<Muscle> {
        if self.has_muscle == 0 {
            return None;
        }
        let angle = Angle(self.muscle_angle);
        let strength = self.muscle_strength;
        let movement = if self.muscle_has_movement == 0 {
            None
        } else {
            Some((self.muscle_freq, self.muscle_amp, self.muscle_shift))
        };
        Some(Muscle::new(
            joint_id, node_1, node_2, angle, strength, movement,
        ))
    }

    pub fn energy_cost(&self) -> f32 {
        let mut cost = 0.0;
        cost += self.node_radius.powi(3) / 50.0; // up to 67.5, 7 is 6.86, 5 is 2.5, 10 is 20
        cost += self.bone_length.max(self.node_radius) / 15.0; // up to 2.0
        if self.has_muscle == 1 {
            cost += self.muscle_strength; // up to 1.0
        }
        cost += self.node_lifespan as f32 / 16_384.0; // up to 2.0
        cost += self.starting_energy;
        cost
    }
}

#[derive(Debug, Clone)]
pub enum Gene {
    Build(BuildGene),
    Repeat,
    Up,
}

impl Gene {
    pub fn random() -> Gene {
        let r = random::<f32>();
        if r < 0.5 {
            Gene::Build(BuildGene::random())
        } else if r < 0.75 {
            Gene::Repeat
        } else {
            Gene::Up
        }
    }

    pub fn mutate_one(&mut self) {
        match self {
            Gene::Build(gene) => gene.mutate_one(),
            Gene::Repeat => {}
            Gene::Up => {}
        }
    }
    pub fn mutate_one_gradual(&mut self) {
        match self {
            Gene::Build(gene) => gene.mutate_one_gradual(),
            Gene::Repeat => {}
            Gene::Up => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct Genome {
    genes: Vec<Gene>,
}

impl Genome {
    pub fn random_plant() -> Genome {
        let mut genes = Vec::new();
        let mut leaf = BuildGene::random();
        leaf.node_kind = 1;
        let mut egg = BuildGene::random();
        egg.node_kind = 0;
        genes.push(Gene::Build(leaf));
        genes.push(Gene::Build(egg));
        let mut ret = Genome { genes };
        ret.mutate();
        ret
    }
    pub fn mutate(&mut self) {
        let mutation_count = random_range(1, self.genes.len() / 5 + 2);
        for _ in 0..mutation_count {
            let r = random::<f32>();
            if r < 0.25 {
                let i = random_range(0, self.genes.len());
                self.genes[i].mutate_one();
            } else if r < 0.5 {
                let i = random_range(0, self.genes.len());
                self.genes[i].mutate_one_gradual();
            } else if r < 0.75 {
                let i = random_range(0, self.genes.len());
                self.genes.remove(i);
            } else {
                let i = random_range(0, self.genes.len());
                self.genes.insert(i, Gene::random());
            }
        }

        self.make_valid();
    }
    pub fn get(&self, index: usize) -> Option<&Gene> {
        self.genes.get(index)
    }
    pub fn get_next(&self, index: usize) -> Option<usize> {
        let mut depth = match self.genes[index] {
            Gene::Build(_) => 1,
            Gene::Repeat => 0,
            Gene::Up => panic!("Cannot get next from Up gene"),
        };
        for i in (index + 1)..self.genes.len() {
            match self.genes[i] {
                Gene::Build(_) => {
                    if depth == 0 {
                        return Some(i);
                    }
                    depth += 1;
                }
                Gene::Repeat => {
                    if depth == 0 {
                        return Some(i);
                    }
                }
                Gene::Up => {
                    depth -= 1;
                    if depth < 0 {
                        return None;
                    }
                }
            }
        }
        None
    }
    pub fn get_next_non_repeat(&self, index: usize) -> Option<usize> {
        let mut ret = self.get_next(index);
        while ret.is_some() && matches!(self.genes[ret.unwrap()], Gene::Repeat) {
            ret = self.get_next(ret.unwrap());
        }
        ret
    }
    pub fn get_next_deeper(&self, index: usize) -> Option<usize> {
        assert!(matches!(self.genes[index], Gene::Build(_)));
        self.genes
            .get(index + 1)
            .map(|gene| match gene {
                Gene::Build(_) | Gene::Repeat => Some(index + 1),
                Gene::Up => None,
            })
            .flatten()
    }
    fn make_valid(&mut self) {
        let has_build = self
            .genes
            .iter()
            .any(|gene| matches!(gene, Gene::Build { .. }));
        if !has_build {
            self.genes.insert(0, Gene::Build(BuildGene::random()));
        }
    }
    pub fn get_start_gene(&self) -> (usize, &Gene) {
        self.genes
            .iter()
            .enumerate()
            .find(|(_, gene)| matches!(gene, Gene::Build { .. }))
            .unwrap()
    }
    pub fn len(&self) -> usize {
        self.genes.len()
    }
}

use std::fmt::Display;

impl Display for Genome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut depth = 0;
        for gene in &self.genes {
            let mut tabs = depth;
            let s = match gene {
                Gene::Build(gene) => {
                    depth += 1;
                    format!("Build {} {{", gene)
                }
                Gene::Repeat => {
                    format!("Repeat")
                }
                Gene::Up => {
                    depth -= 1;
                    tabs -= 1;
                    format!("}} Up")
                }
            };

            let tabs = "\t".repeat(0.max(tabs) as usize);

            writeln!(f, "{tabs}{s}")?;
        }
        while depth > 0 {
            depth -= 1;
            let tabs = "\t".repeat(depth as usize);
            writeln!(f, "{tabs}}}")?;
        }
        Ok(())
    }
}
