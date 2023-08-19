use std::fmt::{Display, Formatter};

use super::brain::BrainPlan;
use super::gene::BodyPlan;

#[derive(Debug, Clone)]
pub struct Genome {
    pub body: BodyPlan,
    pub brain: BrainPlan,
}

impl Genome {
    pub fn random_plant() -> Genome {
        let mut brain = BrainPlan::new();
        let body = BodyPlan::random_plant(&mut brain);
        Genome { body, brain }
    }
    pub fn mutate(&mut self) {
        self.body.mutate(&mut self.brain);
        self.brain.mutate();
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
