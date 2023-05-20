use std::time::Duration;

use nannou::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Angle(f32);
/// angle is between 0 and 2PI
impl Angle {
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.0.cos(), self.0.sin())
    }
    pub fn clockwise_diff(&self, other: Angle) -> Angle {
        let diff = self.0 - other.0;
        if diff < 0. {
            Angle(diff + 2. * PI)
        } else {
            Angle(diff)
        }
    }
    pub fn counter_clockwise_diff(&self, other: Angle) -> Angle {
        let diff = other.0 - self.0;
        if diff < 0. {
            Angle(diff + 2. * PI)
        } else {
            Angle(diff)
        }
    }
    // converts an angle from -PI to PI to 0 to 2PI
    pub fn from_pi_pi_range(angle: f32) -> Angle {
        if angle < 0. {
            Angle(angle + 2. * PI)
        } else {
            Angle(angle)
        }
    }
}

impl std::ops::AddAssign for Angle {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = (self.0 + rhs.0) % (2. * PI);
    }
}

impl std::ops::SubAssign for Angle {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = (self.0 - rhs.0) % (2. * PI);
    }
}

impl std::ops::Add for Angle {
    type Output = Angle;
    fn add(self, rhs: Self) -> Self::Output {
        Angle((self.0 + rhs.0) % (2. * PI))
    }
}

impl std::ops::Sub for Angle {
    type Output = Angle;
    fn sub(self, rhs: Self) -> Self::Output {
        Angle((self.0 - rhs.0) % (2. * PI))
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub pos: Point2,
    pub radius: f32,
    pub vel: Vec2,
    pub accel: Vec2,
}
impl Node {
    pub fn new(pos: Point2, radius: f32) -> Node {
        Node {
            pos,
            radius,
            vel: Vec2::new(0., 0.),
            accel: Vec2::new(0., 0.),
        }
    }
    pub fn update(&mut self) {
        // verlet
        self.vel = self.vel * 0.9;
        self.vel += self.accel;
        self.pos += self.vel;
        self.accel = Vec2::new(0., 0.);
    }
}
#[derive(Debug, Clone)]
pub struct Edge {
    pub node_1: usize,
    pub node_2: usize,
    pub dist: f32,
}
impl Edge {
    pub fn new(node_1: usize, node_2: usize, dist: f32) -> Edge {
        Edge {
            node_1,
            node_2,
            dist,
        }
    }
    pub fn update(&mut self, nodes: &mut [Node]) {
        let node_1 = &nodes[self.node_1];
        let node_2 = &nodes[self.node_2];
        // move towards dist
        let dist = node_1.pos.distance(node_2.pos);
        let dist_diff = dist - self.dist;
        let pos_change = (dist_diff / 2.0) * (node_2.pos - node_1.pos).normalize();

        let vel = (node_1.vel + node_2.vel) / 2.;
        let facing = (node_2.pos - node_1.pos).normalize().perp();
        let stroke_amp = vel.dot(facing);
        let mut friction = -facing * stroke_amp * 1.;

        // apply
        if !friction.is_finite() {
            friction = Vec2::new(0., 0.);
        }
        nodes[self.node_1].pos += pos_change;
        nodes[self.node_2].pos -= pos_change;
        nodes[self.node_1].accel += friction;
        nodes[self.node_2].accel += friction;
    }
}
#[derive(Debug, Clone)]
pub struct Muscle {
    pub joint_node: usize,
    pub node_1: usize,
    pub node_2: usize,
    pub angle: Angle,
    pub angle_fn: fn(f32) -> f32,
    pub strength: f32,
}
impl Muscle {
    pub fn new(
        joint_node: usize,
        node_1: usize,
        node_2: usize,
        angle: Angle,
        angle_fn: fn(f32) -> f32,
        strength: f32,
    ) -> Muscle {
        Muscle {
            joint_node,
            node_1,
            node_2,
            angle,
            angle_fn,
            strength,
        }
    }
    pub fn update(&mut self, nodes: &mut [Node], time: Duration) {
        let joint_node = &nodes[self.joint_node];
        let node_1 = &nodes[self.node_1];
        let node_2 = &nodes[self.node_2];

        // update angle by slow sin wave
        // let real_angle = self.angle.0 + (4. * time.as_secs_f32()).sin() * 0.5;
        let real_angle = (self.angle_fn)(time.as_secs_f32()) + self.angle.0;

        let angle_diff = Angle::from_pi_pi_range(
            (node_1.pos - joint_node.pos).angle_between(node_2.pos - joint_node.pos),
        )
        .0 - real_angle;

        // move towards each other
        let accel_change_1 =
            (node_1.pos - joint_node.pos).perp().normalize() * angle_diff * self.strength;
        let accel_change_2 =
            -(node_2.pos - joint_node.pos).perp().normalize() * angle_diff * self.strength;

        // apply
        nodes[self.node_1].accel += accel_change_1;
        nodes[self.node_2].accel += accel_change_2;
    }
}
#[derive(Debug, Clone)]
pub struct World {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub muscles: Vec<Muscle>,
}

pub fn make_random_trees() -> (Vec<Node>, Vec<Edge>, Vec<Muscle>) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut muscles = Vec::new();
    for _ in 0..50 {
        nodes.push(Node::new(
            Point2::new(random_range(-100., 100.), random_range(-100., 100.)),
            5.,
        ));
        let mut start_index = nodes.len() - 1;
        for _ in 0..1 {
            let mut new_nodes = Vec::new();
            for (i, parent) in nodes[start_index..].iter().enumerate() {
                let parent_id = start_index + i;
                for j in 0..2 {
                    let new_child = Node::new(
                        Point2::new(parent.pos.x + 20., parent.pos.y + random_range(-10., 10.)),
                        5.,
                    );
                    new_nodes.push(new_child);
                    let node_id = nodes.len() + new_nodes.len() - 1;
                    edges.push(Edge::new(parent_id, node_id, 15.));
                    if j > 0 && random::<f32>() > 0. {
                        let angle_between = (new_nodes[j - 1].pos - parent.pos)
                            .angle_between(new_nodes[j].pos - parent.pos);
                        let (node_1, node_2) = if angle_between < 0. {
                            (node_id, node_id - 1)
                        } else {
                            (node_id - 1, node_id)
                        };

                        muscles.push(Muscle::new(
                            parent_id,
                            node_1,
                            node_2,
                            Angle(random_range(0., 3. * PI)),
                            |x| x,
                            0.1,
                        ));
                    }
                }
            }
            start_index = nodes.len();
            nodes.extend(new_nodes.clone());
        }
    }
    (nodes, edges, muscles)
}

impl World {
    pub fn new() -> World {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut muscles = Vec::new();

        let head = Node::new(Point2::new(0., 0.), 5.);
        let body = Node::new(Point2::new(0., 20.), 5.);
        let subtail = Node::new(Point2::new(0., 40.), 5.);
        let tail = Node::new(Point2::new(0., 60.), 5.);
        nodes.push(head);
        nodes.push(body);
        nodes.push(subtail);
        nodes.push(tail);

        let head_body_edge = Edge::new(0, 1, 15.);
        let body_tail_edge = Edge::new(1, 2, 15.);
        let tail_subtail_edge = Edge::new(2, 3, 15.);
        edges.push(head_body_edge);
        edges.push(body_tail_edge);
        edges.push(tail_subtail_edge);

        let top_fn = |x: f32| (4. * x).sin() * 1.;
        let bottom_fn = |x: f32| (4. * (x + PI / 3.)).sin() * 1.;
        let top_muscle = Muscle::new(1, 0, 2, Angle(PI), top_fn, 0.25);
        let bottom_muscle = Muscle::new(2, 1, 3, Angle(PI), bottom_fn, 0.25);

        muscles.push(top_muscle);
        muscles.push(bottom_muscle);

        // let (nodes, edges, muscles) = make_random_trees();

        World {
            nodes,
            edges,
            muscles,
        }
    }
    pub fn update(&mut self, _delta_time: Duration, time: Duration) {
        // update nodes
        for node in &mut self.nodes {
            node.update();
        }
        // update edges
        for edge in &mut self.edges {
            edge.update(&mut self.nodes[..]);
        }
        // update muscles
        for muscle in &mut self.muscles {
            muscle.update(&mut self.nodes[..], time);
        }
        // collide nodes
        for i in 0..self.nodes.len() {
            for j in i + 1..self.nodes.len() {
                let node1 = &self.nodes[i];
                let node2 = &self.nodes[j];
                let dist = node1.pos.distance(node2.pos);
                let min_dist = node1.radius + node2.radius;
                if dist < min_dist {
                    // move them away from each other
                    let diff = node1.pos - node2.pos;
                    let diff = diff.normalize() * (min_dist - dist) / 2.0;
                    self.nodes[i].pos += diff;
                    self.nodes[j].pos -= diff;
                }
            }
        }
    }
}
