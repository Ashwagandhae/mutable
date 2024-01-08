use std::fmt::{Display, Formatter};

use super::brain::BrainPlan;
use super::gene::BodyPlan;
use super::tag::Tag;
use nannou::prelude::random;

#[derive(Debug, Clone)]
pub struct Genome {
    pub body: BodyPlan,
    pub brain: BrainPlan,
    pub tag: Tag,
}

impl Genome {
    pub fn random_plant() -> Genome {
        let mut brain = BrainPlan::new();
        let body = BodyPlan::random_plant(&mut brain);
        let tag = Tag::random();
        Genome { body, brain, tag }
    }
    // pub fn random_animal() -> Genome {
    //     let mut brain = BrainPlan::new();
    //     let body = BodyPlan::random_animal(&mut brain);
    //     let tag = Tag::random();
    //     Genome { body, brain, tag }
    // }
    pub fn mutate(&mut self) {
        let r = random::<f32>();
        self.tag.mutate();
        if r < 0.5 {
            self.body.mutate(&mut self.brain);
            self.brain.mutate();
        } else if r < 0.75 {
            self.body.mutate(&mut self.brain);
        } else {
            self.brain.mutate();
        }
    }
}

impl Display for Genome {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Genome:\nBodyPlan:\n{}\nBrainPlan:\n{}",
            self.body, self.brain
        )
    }
}
