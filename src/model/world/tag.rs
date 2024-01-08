use nannou::rand::random_range;
#[derive(Debug, Clone, PartialEq)]
pub struct Tag([f32; 16]);

impl Tag {
    pub fn random() -> Tag {
        Tag(rand::random())
    }

    pub fn mutate(&mut self) {
        let index = random_range(0, self.0.len());
        self.0[index] = rand::random();
    }

    pub fn distance(&self, other: &Tag) -> f32 {
        let mut distance = 0.;
        for i in 0..self.0.len() {
            distance += (self.0[i] - other.0[i]).powi(2);
        }
        distance.sqrt()
    }
    pub fn zero() -> Tag {
        Tag([0.; 16])
    }
}

use std::ops::{Add, Div, Mul, Sub};

impl Add for Tag {
    type Output = Tag;
    fn add(self, other: Tag) -> Tag {
        let mut new = self.clone();
        for i in 0..self.0.len() {
            new.0[i] = self.0[i] + other.0[i];
        }
        new
    }
}

impl Sub for Tag {
    type Output = Tag;
    fn sub(self, other: Tag) -> Tag {
        let mut new = self.clone();
        for i in 0..self.0.len() {
            new.0[i] = self.0[i] - other.0[i];
        }
        new
    }
}

impl Mul<f32> for Tag {
    type Output = Tag;
    fn mul(self, other: f32) -> Tag {
        let mut new = self.clone();
        for i in 0..self.0.len() {
            new.0[i] = self.0[i] * other;
        }
        new
    }
}

impl Div<f32> for Tag {
    type Output = Tag;
    fn div(self, other: f32) -> Tag {
        let mut new = self.clone();
        for i in 0..self.0.len() {
            new.0[i] = self.0[i] / other;
        }
        new
    }
}
