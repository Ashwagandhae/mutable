struct BoneJoint {
    // Define the properties of the bone joint
    width: f32,
    height: f32,
    drag_coefficient: f32,
}

struct Fluid {
    // Define the properties of the fluid
    density: f32,
    viscosity: f32,
}

struct Node {
    // Define the properties of a node
    position: (f32, f32),
    velocity: (f32, f32),
    mass: f32,
}

struct Simulation {
    // Define the simulation properties
    gravity: (f32, f32),
    fluid: Fluid,
    bone_joint: BoneJoint,
    nodes: Vec<Node>,
}

impl BoneJoint {
    fn compute_water_resistance(&self, fluid: &Fluid, relative_velocity: (f32, f32)) -> (f32, f32) {
        let cross_sectional_area = self.width * self.height;
        let drag_force_x = 0.5
            * fluid.density
            * cross_sectional_area
            * self.drag_coefficient
            * relative_velocity.0.powi(2);
        let drag_force_y = 0.5
            * fluid.density
            * cross_sectional_area
            * self.drag_coefficient
            * relative_velocity.1.powi(2);
        (drag_force_x, drag_force_y)
    }
}

impl Simulation {
    fn update_node(&mut self, node_index: usize, time_step: f32) {
        let node = &mut self.nodes[node_index];

        // Apply gravitational force
        let gravitational_force = (self.gravity.0 * node.mass, self.gravity.1 * node.mass);
        let acceleration = (
            gravitational_force.0 / node.mass,
            gravitational_force.1 / node.mass,
        );
        node.velocity.0 += acceleration.0 * time_step;
        node.velocity.1 += acceleration.1 * time_step;

        // Compute water resistance
        let relative_velocity = node.velocity;
        let water_resistance = self
            .bone_joint
            .compute_water_resistance(&self.fluid, relative_velocity);

        // Apply water resistance force
        let drag_force = (water_resistance.0 * -1.0, water_resistance.1 * -1.0);
        node.velocity.0 += drag_force.0 / node.mass * time_step;
        node.velocity.1 += drag_force.1 / node.mass * time_step;

        // Update node position
        node.position.0 += node.velocity.0 * time_step;
        node.position.1 += node.velocity.1 * time_step;
    }

    fn update(&mut self, time_step: f32) {
        for i in 0..self.nodes.len() {
            self.update_node(i, time_step);
        }
    }
}

fn main() {
    let bone_joint = BoneJoint {
        width: 1.0,
        height: 1.5,
        drag_coefficient: 0.7,
    };

    let fluid = Fluid {
        density: 1000.0,
        viscosity: 0.001,
    };

    let mut simulation = Simulation {
        gravity: (0.0, -9.8),
        fluid,
        bone_joint,
        nodes: vec![
            Node {
                position: (0.0, 0.0),
                velocity: (0.0, 0.0),
                mass: 1.0,
            },
            Node {
                position: (0.0, 5.0),
                velocity: (2.0, 0.0),
                mass: 1.0,
            },
        ],
    };

    let time_step = 0.1;
    for _ in 0..10 {
        simulation.update(time_step);
        for node in &simulation.nodes {
            println!("Node Position: ({}, {})", node.position.0, node.position.1);
        }
        println!("--------");
    }
}
