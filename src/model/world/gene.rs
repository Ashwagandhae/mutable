use super::bone::Bone;
use super::brain::BrainPlan;
use super::collection::GenId;
use super::math::Angle;
use super::muscle::Muscle;
use super::node::{Node, NodeKind, SenseKind};
use int_enum::IntEnum;

use nannou::prelude::*;
use strum::EnumCount;

mod macros;
use super::MAX_NODE_RADIUS;
use macros::{count_fields, make_gene_struct, replace_expr};

make_gene_struct!(pub BuildGene {
    node_radius: f32 = 2.0..MAX_NODE_RADIUS,
    node_energy_weight: f32 = 1.0..10.0,
    node_kind: u8 = 0..(NodeKind::COUNT),
    node_lifespan: u32 = 256..32_768,

    has_sense: u8 = 0..2,
    sense_kind: u8 = 0..(SenseKind::COUNT),

    bone_length: f32 = 5.0..30.0,

    has_muscle: u8 = 0..2,
    muscle_length: f32 = 5.0..30.0,
    muscle_strength: f32 = 0.5..2.0,
    muscle_has_movement: u8 = 0..2,
    muscle_is_sibling: u8 = 0..2,

    starting_energy: f32 = 0.0..30.0,
});

impl BuildGene {
    pub fn build_node(
        &self,
        pos: Point2,
        gene_index: Option<usize>,
        energy: f32,
        parent: Option<(GenId, Point2)>,
    ) -> Node {
        let kind = NodeKind::from_int(self.node_kind).unwrap();
        let energy_weight = self.node_energy_weight;
        let lifespan = self.node_lifespan;
        let radius = self.node_radius;
        let sense_kind = if self.has_sense == 0 {
            None
        } else {
            Some(SenseKind::from_int(self.sense_kind).unwrap())
        };

        let parent = parent.map(|(id, parent_pos)| {
            let angle = Angle::from_vec2(pos - parent_pos);
            (id, angle)
        });

        Node::new(
            pos,
            radius,
            energy,
            energy_weight,
            kind,
            gene_index,
            parent,
            lifespan,
            sense_kind,
        )
    }
    pub fn build_bone(&self, parent_node: GenId, child_node: GenId, min_length: f32) -> Bone {
        let length = self.bone_length.max(min_length);
        Bone::new(parent_node, child_node, length)
    }
    pub fn build_muscle(&self, joint_id: GenId, node_1: GenId, node_2: GenId) -> Option<Muscle> {
        if self.has_muscle == 0 {
            return None;
        }
        let length = self.muscle_length;
        let strength = self.muscle_strength;
        Some(Muscle::new(joint_id, node_1, node_2, length, strength))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BuildId(usize);

impl BuildId {
    fn new() -> BuildId {
        BuildId(random())
    }
}

#[derive(Debug, Clone)]
pub enum Gene {
    Build((BuildGene, BuildId)),
    Repeat,
    Up,
}

impl Gene {
    pub fn random() -> Gene {
        let r = random::<f32>();
        if r < 0.5 {
            Gene::Build((BuildGene::random(), BuildId::new()))
        } else if r < 0.75 {
            Gene::Repeat
        } else {
            Gene::Up
        }
    }

    pub fn mutate_one(&mut self) {
        match self {
            Gene::Build((gene, _)) => gene.mutate_one(),
            Gene::Repeat => {}
            Gene::Up => {}
        }
    }
    pub fn mutate_one_gradual(&mut self) {
        match self {
            Gene::Build((gene, _)) => gene.mutate_one_gradual(),
            Gene::Repeat => {}
            Gene::Up => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct BodyPlan {
    genes: Vec<Gene>,
}

pub enum Mutation {
    Add,
    Delete,
    Edit,
    EditGradual,
}

impl Mutation {
    fn random() -> Mutation {
        let r = random::<f32>();
        if r < 0.25 {
            Mutation::Add
        } else if r < 0.5 {
            Mutation::Delete
        } else if r < 0.75 {
            Mutation::Edit
        } else {
            Mutation::EditGradual
        }
    }
}

impl BodyPlan {
    pub fn random_plant(brain: &mut BrainPlan) -> BodyPlan {
        let mut genes = Vec::new();

        let mut leaf = BuildGene::random();
        leaf.node_kind = 1;
        let mut egg = BuildGene::random();
        egg.node_kind = 0;

        let leaf_id = BuildId::new();
        let egg_id = BuildId::new();

        let leaf_gene = Gene::Build((leaf, leaf_id.clone()));
        let egg_gene = Gene::Build((egg, egg_id.clone()));

        brain.mutate_gene(Mutation::Add, &leaf_gene);
        brain.mutate_gene(Mutation::Add, &egg_gene);

        genes.push(leaf_gene);
        genes.push(egg_gene);

        let mut ret = BodyPlan { genes };
        ret.mutate(brain);
        ret
    }
    pub fn mutate(&mut self, brain: &mut BrainPlan) {
        let mutation_count = random_range(1, 4);
        for _ in 0..mutation_count {
            let (i, mutation) = match self.genes.len() {
                0 => (0, Mutation::Add),
                _ => (random_range(0, self.genes.len()), Mutation::random()),
            };

            match mutation {
                Mutation::Add => {
                    let new_gene = Gene::random();
                    brain.mutate_gene(mutation, &new_gene);
                    self.genes.insert(i, new_gene);
                }
                Mutation::Delete => {
                    let rem_gene = self.genes.remove(i);
                    brain.mutate_gene(mutation, &rem_gene);
                }
                Mutation::Edit => {
                    self.genes[i].mutate_one();
                    brain.mutate_gene(mutation, &self.genes[i]);
                }
                Mutation::EditGradual => {
                    self.genes[i].mutate_one_gradual();
                    brain.mutate_gene(mutation, &self.genes[i]);
                }
            };
        }

        self.make_valid();
    }
    pub fn get(&self, index: usize) -> Option<&Gene> {
        self.genes.get(index)
    }
    /// will only return Build and Repeat genes
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
    /// will only return Build genes, obviously not Repeat, and Up is removed due to get_next
    fn get_next_non_repeat(&self, index: usize) -> Option<usize> {
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
    pub fn get_build(&self, index: usize) -> Option<(usize, &(BuildGene, BuildId))> {
        self.get(index).and_then(|real_gene| {
            match real_gene {
                // if its repeat, try to find first non repeat gene (that's what it's repeating)
                Gene::Repeat => {
                    self.get_next_non_repeat(index)
                        .map(|build_index| {
                            self.get(build_index).map(|gene| {
                                // gauranteed to be Build because of get_next_non_repeat
                                let Gene::Build(gene) = gene else { unreachable!() };
                                (build_index, gene)
                            })
                        })
                        .flatten()
                }
                Gene::Build(gene) => Some((index, gene)),
                Gene::Up => unreachable!(),
            }
        })
    }
    fn make_valid(&mut self) {
        let has_build = self
            .genes
            .iter()
            .any(|gene| matches!(gene, Gene::Build { .. }));
        if !has_build {
            self.genes
                .insert(0, Gene::Build((BuildGene::random(), BuildId::new())));
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

impl Display for BodyPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut depth = 0;
        for gene in &self.genes {
            let mut tabs = depth;
            let s = match gene {
                Gene::Build(gene) => {
                    depth += 1;
                    format!("Build {} {{", gene.0)
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
