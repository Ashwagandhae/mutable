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
    node_kind: u8 = 0..4,
    node_lifespan: u32 = 256..32_768,

    bone_length: f32 = 5.0..30.0,

    has_muscle: u8 = 0..2,
        muscle_angle: f32 = 0.0..TAU,
        muscle_strength: f32 = 0.5..2.0,
        muscle_has_movement: u8 = 0..2,
            muscle_freq: f32 = 0.1..2.0,
            muscle_amp: f32 = 0.0..1.5,
            muscle_shift: f32 = 0.0..PI,

    starting_energy: f32 = 0.0..10.0,
    child_goto_index: usize = 0..20,
});
make_gene_struct!(pub EggGene {
    starting_energy: f32 = 0.0..20.0,
    child_distance: f32 = 5.0..25.0,
});
make_gene_struct!(pub SkipGene {
    goto_index: usize = 0..20,
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
        cost += self.node_radius.powi(3) / 50.0; // up to 20
        cost += self.bone_length.max(self.node_radius) / 20.0; // up to 1.5
        if self.has_muscle == 1 {
            cost += self.muscle_strength; // up to 1
        }
        cost += self.node_lifespan as f32 / 16_384.0; // up to 2
        cost += self.starting_energy;
        cost
    }
}
#[derive(Debug, Clone)]
pub enum Gene {
    Build(BuildGene),
    Egg(EggGene),
    Skip(SkipGene),
    Stop,
    Junk,
}

impl Gene {
    pub fn random() -> Gene {
        let r = random::<f32>();
        if r < 0.3 {
            Gene::Build(BuildGene::random())
        } else if r < 0.6 {
            Gene::Egg(EggGene::random())
        } else if r < 0.8 {
            Gene::Skip(SkipGene::random())
        } else if r < 0.9 {
            Gene::Stop
        } else {
            Gene::Junk
        }
    }

    pub fn mutate_one(&mut self) {
        match self {
            Gene::Build(gene) => gene.mutate_one(),
            Gene::Egg(gene) => gene.mutate_one(),
            Gene::Skip(gene) => gene.mutate_one(),
            Gene::Stop => {}
            Gene::Junk => {}
        }
    }
    pub fn mutate_one_gradual(&mut self) {
        match self {
            Gene::Build(gene) => gene.mutate_one_gradual(),
            Gene::Egg(gene) => gene.mutate_one_gradual(),
            Gene::Skip(gene) => gene.mutate_one_gradual(),
            Gene::Stop => {}
            Gene::Junk => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct Genome {
    genes: Vec<Gene>,
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
                for gene in self.genes.iter_mut() {
                    if let Gene::Skip(SkipGene { ref mut goto_index })
                    | Gene::Build(BuildGene {
                        child_goto_index: ref mut goto_index,
                        ..
                    }) = gene
                    {
                        if *goto_index > i {
                            *goto_index -= 1;
                        }
                    }
                }
                self.genes.remove(i);
            } else {
                let i = random_range(0, self.genes.len());
                for gene in self.genes.iter_mut() {
                    if let Gene::Skip(SkipGene { ref mut goto_index })
                    | Gene::Build(BuildGene {
                        child_goto_index: ref mut goto_index,
                        ..
                    }) = gene
                    {
                        if *goto_index >= i {
                            *goto_index += 1;
                        }
                    }
                }
                self.genes.insert(i, Gene::random());
            }
        }

        self.make_valid();
    }
    pub fn get(&self, index: usize) -> &Gene {
        &self.genes[index % self.genes.len()]
    }
    fn make_valid(&mut self) {
        let mut has_build = false;
        let len = self.genes.len();
        for gene in self.genes[..].iter_mut() {
            if let Gene::Build { .. } = gene {
                has_build = true;
            }
            if let Gene::Skip(SkipGene { ref mut goto_index })
            | Gene::Build(BuildGene {
                child_goto_index: ref mut goto_index,
                ..
            }) = gene
            {
                *goto_index %= len;
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
