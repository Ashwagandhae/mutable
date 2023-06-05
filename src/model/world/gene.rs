use super::bone::Bone;
use super::collection::GenId;
use super::math::Angle;
use super::muscle::Muscle;
use super::node::{self, Node, NodeKind};
use nannou::prelude::*;

mod macros;
use macros::{count_fields, make_gene_struct, replace_expr};

use super::MAX_NODE_RADIUS;

make_gene_struct!(pub BuildGene {
    node_radius: f32 = 3.0..MAX_NODE_RADIUS,
    node_energy_weight: f32 = 1.0..10.0,
    node_kind: u8 = 0..3,
    node_lifespan: u32 = 256..8192,

    bone_length: f32 = 5.0..25.0,

    has_muscle: u8 = 0..2,
    muscle_angle: f32 = 0.0..TAU,
    muscle_strength: f32 = 0.0..1.0,

    starting_energy: f32 = 0.0..10.0,
    child_skip: usize = 0..20,
});
make_gene_struct!(pub EggGene {
    starting_energy: f32 = 0.0..20.0,
    child_distance: f32 = 5.0..25.0,
});
make_gene_struct!(pub SkipGene {
    skip: usize = 0..20,
});

impl BuildGene {
    pub fn build_node(
        &self,
        pos: Point2,
        gene_index: usize,
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
        Some(Muscle::new(joint_id, node_1, node_2, angle, strength))
    }

    pub fn energy_cost(&self) -> f32 {
        let mut cost = 0.0;
        cost += self.node_radius.powi(2) / 20.0; // up to 5
        cost += self.bone_length.max(self.node_radius) / 10.0; // up to 2.5
        if self.has_muscle == 1 {
            cost += self.muscle_strength; // up to 1
        }
        cost += self.node_lifespan as f32 / 4096.0; // up to 2
        cost += self.starting_energy;
        cost
    }
}

#[derive(Debug, Clone)]
pub enum Gene {
    Build(BuildGene),
    Egg(EggGene),
    Skip(SkipGene),
    Junk,
}

impl Gene {
    pub fn random() -> Gene {
        let r = random::<f32>();
        if r < 0.5 {
            Gene::Build(BuildGene::random())
        } else if r < 0.75 {
            Gene::Egg(EggGene::random())
        } else if r < 0.9 {
            Gene::Skip(SkipGene::random())
        } else {
            Gene::Junk
        }
    }

    pub fn mutate(&mut self) {
        match self {
            Gene::Build(gene) => gene.mutate_one(),
            Gene::Egg(gene) => gene.mutate_one(),
            Gene::Skip(gene) => gene.mutate_one(),
            Gene::Junk => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct Genome {
    pub genes: Vec<Gene>,
}

impl Genome {
    pub fn random() -> Genome {
        let mut genes = Vec::new();
        for _ in 0..random_range(1, 20) {
            genes.push(Gene::random());
        }
        let mut ret = Genome { genes };
        ret.make_valid();
        ret
    }
    pub fn mutate(&mut self) {
        let r = random::<f32>();
        if r < 0.5 {
            let i = random_range(0, self.genes.len());
            self.genes[i].mutate();
        } else if r < 0.75 {
            let i = random_range(0, self.genes.len());
            self.genes.remove(i);
            self.make_valid();
        } else {
            let i = random_range(0, self.genes.len());
            self.genes.insert(i, Gene::random());
        }
    }
    pub fn get(&self, index: usize) -> &Gene {
        &self.genes[index % self.genes.len()]
    }
    fn make_valid(&mut self) {
        // check if there are still build genes
        let mut has_build = false;
        for gene in &self.genes {
            if let Gene::Build { .. } = gene {
                has_build = true;
                break;
            }
        }
        if !has_build {
            self.genes.push(Gene::Build(BuildGene::random()));
        }
    }
    pub fn get_start_gene(&self) -> (usize, &Gene) {
        self.genes
            .iter()
            .enumerate()
            .cycle()
            .find(|(_, gene)| matches!(gene, Gene::Build { .. }))
            .unwrap()
    }
}
