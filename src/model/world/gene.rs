use super::bone::Bone;
use super::collection::GenId;
use super::math::Angle;
use super::muscle::Muscle;
use super::node::{Node, NodeKind};
use nannou::prelude::*;

use super::MAX_NODE_RADIUS;
#[derive(Debug, Clone)]
pub struct BoneBuild {
    pub length: f32,
}
// TODO make everything 1
impl BoneBuild {
    pub fn build(&self, node_1: GenId, node_2: GenId) -> Bone {
        Bone::new(node_1, node_2, self.length)
    }
    pub fn energy_cost(&self) -> f32 {
        (self.length - 5.0) / 20.0 + 0.25
    }
}
#[derive(Debug, Clone)]
pub struct NodeBuild {
    pub radius: f32,
    pub energy_weight: f32,
    pub kind: NodeKind,
}
impl NodeBuild {
    pub fn build(
        &self,
        pos: Point2,
        gene_index: usize,
        energy: f32,
        parent_id: Option<GenId>,
    ) -> Node {
        Node::new(
            pos,
            self.radius,
            energy,
            self.energy_weight,
            self.kind.clone(),
            gene_index,
            parent_id,
        )
    }
    pub fn energy_cost(&self) -> f32 {
        (self.radius - 5.0) / 5.0 + 0.25
    }
}
#[derive(Debug, Clone)]
pub struct MuscleBuild {
    pub angle: Angle,
    pub strength: f32,
}
impl MuscleBuild {
    pub fn build(&self, node_1: GenId, node_2: GenId, joint_node: GenId) -> Muscle {
        Muscle::new(joint_node, node_1, node_2, self.angle, self.strength)
    }
    pub fn energy_cost(&self) -> f32 {
        self.strength + 0.25
    }
}

#[derive(Debug, Clone)]
pub enum Gene {
    Build {
        node: NodeBuild,
        bone: BoneBuild,
        muscle: Option<MuscleBuild>,
        starting_energy: f32,
        child_skip: usize,
    },
    Egg {
        starting_energy: f32,
    },
    Skip(usize),
    Junk,
}

impl Gene {
    pub fn random() -> Gene {
        let r = random::<f32>();
        if r < 0.25 {
            Gene::random_build()
        } else if r < 0.5 {
            Gene::Skip(random_range(0, 20))
        } else if r < 0.75 {
            Gene::Egg {
                starting_energy: random_range(0., 10.),
            }
        } else {
            Gene::Junk
        }
    }
    pub fn random_build() -> Gene {
        let bone = BoneBuild {
            length: random_range(5., 25.),
        };
        let node = NodeBuild {
            radius: random_range(5., MAX_NODE_RADIUS),
            energy_weight: random_range(1., 10.),
            kind: {
                let r = random::<f32>();
                if r < 0.5 {
                    NodeKind::Body
                } else {
                    NodeKind::Leaf
                }
            },
        };
        let muscle = if random::<f32>() > 0.5 {
            Some(MuscleBuild {
                angle: Angle(random_range(0., 2. * PI)),
                strength: random_range(0., 1.),
            })
        } else {
            None
        };
        let starting_energy = random_range(0., 10.);
        let child_skip = random_range(0, 20);
        Gene::Build {
            bone,
            node,
            muscle,
            starting_energy,
            child_skip,
        }
    }
    pub fn mutate(&mut self) {
        match self {
            Gene::Build {
                bone,
                node,
                muscle,
                starting_energy,
                child_skip,
            } => {
                let r = random::<f32>();
                if r < 0.33 {
                    bone.length = random_range(5., 25.);
                } else if r < 0.66 {
                    node.radius = random_range(5., MAX_NODE_RADIUS);
                    node.energy_weight = random_range(1., 10.);
                    node.kind = {
                        let r = random::<f32>();
                        if r < 0.5 {
                            NodeKind::Body
                        } else {
                            NodeKind::Leaf
                        }
                    };
                } else if r < 0.75 {
                    if random::<f32>() > 0.5 {
                        *muscle = Some(MuscleBuild {
                            angle: Angle(random_range(0., 2. * PI)),
                            strength: random_range(0., 1.),
                        });
                    } else {
                        *muscle = None;
                    }
                } else if r < 0.9 {
                    *starting_energy = random_range(0., 10.);
                } else {
                    *child_skip = random_range(0, 20);
                }
            }
            Gene::Skip(child_skip) => {
                *child_skip = random_range(0, 20);
            }
            Gene::Egg { starting_energy } => {
                *starting_energy = random_range(0., 10.);
            }
            Gene::Junk => {}
        }
    }
    pub fn build_energy_cost(&self) -> f32 {
        match self {
            Gene::Build {
                bone, node, muscle, ..
            } => {
                bone.energy_cost()
                    + node.energy_cost()
                    + muscle.as_ref().map_or(0., |m| m.energy_cost())
            }
            Gene::Egg { .. } => 1.,
            Gene::Skip(..) => 0.,
            Gene::Junk => 0.,
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
            self.genes.push(Gene::random_build());
        }
    }
}
