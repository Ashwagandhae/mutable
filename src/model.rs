use itertools::iproduct;
use nannou::prelude::*;
mod cluster;
mod world;
use world::collection::GenId;
use world::node::{Node, NodeKind};
use world::World;

use self::cluster::Cluster;
use self::world::node::{LifeState, SenseCalculate, SenseKind};
use self::world::organism::Organism;

pub const WINDOW_SIZE: u32 = 800;

pub struct Camera {
    pub pos: Point2,
    pub zoom: f32,
}
impl Camera {
    pub fn world_to_view(&self, pos: Point2) -> Point2 {
        (pos - self.pos) * self.zoom
    }
    pub fn view_to_world(&self, pos: Point2) -> Point2 {
        pos / self.zoom + self.pos
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Scene {
    World,
    Cluster,
}
pub struct Model {
    pub input_state: InputState,
    pub camera: Camera,
    pub world: World,
    pub clusters: Option<Vec<Cluster<GenId>>>,
    pub scene: Scene,
}
#[derive(Clone)]
pub struct NodeInfo {
    pub node_id: GenId,
    pub organism_id: Option<GenId>,
}
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub plus: bool,
    pub minus: bool,
    pub mouse_pos: Point2,
    pub mouse_pressed: bool,
    pub skip_toggled: bool,
    pub dragged: Option<NodeInfo>,
    pub selected: Option<NodeInfo>,
}
impl InputState {
    pub fn new() -> InputState {
        InputState {
            up: false,
            down: false,
            left: false,
            right: false,
            plus: false,
            minus: false,
            mouse_pos: Point2::new(0.0, 0.0),
            mouse_pressed: false,
            skip_toggled: false,
            dragged: None,
            selected: None,
        }
    }
    pub fn update(&mut self, event: &Event) {
        if let Event::WindowEvent {
            simple: Some(event),
            ..
        } = event
        {
            match event {
                KeyPressed(key) => match key {
                    Key::Up => self.up = true,
                    Key::Down => self.down = true,
                    Key::Left => self.left = true,
                    Key::Right => self.right = true,
                    Key::Equals => self.plus = true,
                    Key::Minus => self.minus = true,
                    _ => (),
                },
                KeyReleased(key) => match key {
                    Key::Up => self.up = false,
                    Key::Down => self.down = false,
                    Key::Right => self.right = false,
                    Key::Left => self.left = false,
                    Key::Equals => self.plus = false,
                    Key::Minus => self.minus = false,
                    Key::Space => self.skip_toggled = !self.skip_toggled,
                    _ => (),
                },
                MouseMoved(pos) => self.mouse_pos = *pos,
                MousePressed(_) => self.mouse_pressed = true,
                MouseReleased(_) => self.mouse_pressed = false,
                _ => (),
            }
        }
    }
}
impl Model {
    pub fn new() -> Model {
        Model {
            camera: Camera {
                pos: Point2::new(100.0, 100.0),
                zoom: 2.0,
            },
            world: World::new(),
            input_state: InputState::new(),
            clusters: None,
            scene: Scene::World,
        }
    }
    pub fn within_view(&self, pos: Point2) -> bool {
        const LOWER: f32 = WINDOW_SIZE as f32 / -2.0;
        const UPPER: f32 = WINDOW_SIZE as f32 / 2.0;
        pos.x > LOWER && pos.x < UPPER && pos.y > LOWER && pos.y < UPPER
    }
    fn draw_node(&self, draw: &Draw, node: &Node) {
        let pos = self.camera.world_to_view(node.pos());

        if !self.within_view(pos) {
            return;
        }

        self.draw_node_at_pos(draw, node, pos, self.camera.zoom);
    }

    fn draw_node_at_pos(&self, draw: &Draw, node: &Node, pos: Point2, zoom: f32) {
        let radius = node.radius * zoom;
        let energy_mult = node.energy.max(0.) / node.max_energy() * 0.5;
        let color = match &node.life_state {
            LifeState::Alive { kind, .. } => match kind {
                NodeKind::Leaf => rgb(0.5 + energy_mult, 0.7 + energy_mult, 0.5 + energy_mult),
                NodeKind::Storage => rgb(0.7 + energy_mult, 0.5 + energy_mult, 0.5 + energy_mult),
                NodeKind::Mouth => rgb(0.7 + energy_mult, 0.5 + energy_mult, 0.7 + energy_mult),
                NodeKind::Egg => rgb(0.5 + energy_mult, 0.5 + energy_mult, 0.7 + energy_mult),
                NodeKind::Spike => rgb(0.7 + energy_mult, 0.7 + energy_mult, 0.5 + energy_mult),
                NodeKind::Shell => rgb(0.5 + energy_mult, 0.7 + energy_mult, 0.7 + energy_mult),
                NodeKind::Jet => rgb(0.7 + energy_mult, 0.6 + energy_mult, 0.5 + energy_mult),
            },
            LifeState::Dead { .. } => rgb(0.5, 0.5, 0.5),
        };
        draw.ellipse().color(color).xy(pos).radius(radius);
        if let LifeState::Alive {
            sense: Some((kind, sense)),
            ..
        } = &node.life_state
        {
            let color = match kind {
                SenseKind::Sun => rgb(1.0, 1.0, 0.0),
                SenseKind::Energy => rgb(1.0, 0.5, 0.0),
                SenseKind::Age => rgb(1.0, 0.0, 0.0),

                SenseKind::TideAngle => rgb(0.0, 0.0, 1.0),
                SenseKind::TideSpeed => rgb(0.0, 0.5, 1.0),

                SenseKind::CollideAngle => rgb(0.0, 1.0, 0.5),
                SenseKind::CollideKind => rgb(0.3, 0.7, 0.3),
                SenseKind::CollideRadius => rgb(0.0, 1.0, 0.2),
                SenseKind::CollideSpeed => rgb(0.0, 1.0, 0.0),

                SenseKind::Eye => rgb(1.0, 0.5, 1.0),
            };
            let sense_color = match sense {
                SenseCalculate::Calculate(sense) => {
                    rgb(0.5 + (-sense).max(0.) * 0.5, 0.5 + sense.max(0.) * 0.5, 0.)
                }
                SenseCalculate::Skip => rgb(0., 0., 1.),
            };
            draw.ellipse().color(color).xy(pos).radius(radius * 0.5);
            draw.ellipse()
                .color(sense_color)
                .xy(pos)
                .radius(radius * 0.25);
        }
    }
    pub fn view(&self, app: &App, frame: Frame) {
        let draw = app.draw();
        match self.scene {
            Scene::World => self.view_world(&draw, app),
            Scene::Cluster => self.view_cluster(&draw),
        }
        draw.to_frame(app, &frame).unwrap();
    }
    pub fn view_cluster(&self, draw: &Draw) {
        draw.background().color(BLACK);
        let Some(clusters) = &self.clusters else {return};

        // split background into clusters.len() squares
        let squares = (clusters.len() as f32).sqrt().ceil() as usize;
        let square_size = WINDOW_SIZE as f32 / squares as f32;
        for (i, cluster) in clusters.iter().enumerate() {
            let x = (i % squares) as f32 * square_size;
            let y = (i / squares) as f32 * square_size;
            let pos = vec2(x, y) + vec2(square_size, square_size) / 2.
                - vec2(WINDOW_SIZE as f32 / 2., WINDOW_SIZE as f32 / 2.);
            let organism = cluster
                .points
                .iter()
                .filter_map(|(tag, org_id)| self.world.organisms.get(*org_id).map(|org| (tag, org)))
                .min_by(|(tag_1, _), (tag_2, _)| {
                    tag_1
                        .distance(&cluster.center)
                        .partial_cmp(&tag_2.distance(&cluster.center))
                        .unwrap()
                });
            if let Some((_, organism)) = organism {
                self.draw_organism(draw, pos, organism);
            }
        }
    }

    pub fn draw_organism(&self, draw: &Draw, pos: Vec2, organism: &Organism) {
        let average_pos = organism
            .node_ids()
            .iter()
            .filter_map(|node_id| self.world.nodes.get(*node_id))
            .map(|node| node.pos())
            .fold(Vec2::new(0., 0.), |sum, pos| sum + pos)
            / organism.node_ids().len() as f32;
        for node_id in organism.node_ids().iter() {
            let Some(node) = self.world.nodes.get(*node_id) else {continue};
            let pos = pos + node.pos() - average_pos;
            self.draw_node_at_pos(draw, node, pos, 1.0);
        }
    }

    pub fn view_world(&self, draw: &Draw, app: &App) {
        draw.background().color(BLACK);

        if !self.input_state.skip_toggled {
            draw.rect()
                .color(rgb(50u8, 50, 50))
                .xy(self.camera.world_to_view(self.world.size / 2.))
                .wh(self.world.size * self.camera.zoom);

            let chunks = &self.world.chunks;
            for (x, y) in iproduct!(0..chunks.grid_size.0, 0..chunks.grid_size.1) {
                let chunk = &chunks.grid[y * chunks.grid_size.0 + x];
                let size = vec2(
                    self.world.size.x / chunks.grid_size.0 as f32,
                    self.world.size.y / chunks.grid_size.1 as f32,
                );
                let pos = vec2(x as f32, y as f32) * size + size / 2.;
                let pos = self.camera.world_to_view(pos);
                if !self.within_view(pos) {
                    continue;
                }
                let size = size * self.camera.zoom;
                let color = rgb(0.1 + chunk.sun * 0.15, 0.1 + chunk.sun * 0.15, 0.1);
                draw.rect().color(color).xy(pos).wh(size);
                // draw arrow for tide
                // let tide: Vec2 = chunk.tide;
                // let tide = tide * self.camera.zoom / TIDE_MULT * 20.;
                // let color = rgb(0.5 + chunk.sun * 0.5, 0.5 + chunk.sun * 0.5, 0.5);
                // draw.arrow()
                //     .color(color)
                //     .start(pos)
                //     .end(pos + tide)
                //     .weight(2.0 * self.camera.zoom);
            }

            for node in self.world.nodes.iter() {
                self.draw_node(&draw, node);
            }
            for bone in self.world.bones.iter() {
                let Some(node_1) = self.world.nodes.get(bone.parent_node) else {continue};
                let Some(node_2) = self.world.nodes.get(bone.child_node) else {continue};
                let pos_1 = self.camera.world_to_view(node_1.pos());
                let pos_2 = self.camera.world_to_view(node_2.pos());
                self.draw_bone(&draw, pos_1, pos_2, self.camera.zoom);
            }
            for muscle in self.world.muscles.iter() {
                let Some(node_1) = self.world.nodes.get(muscle.node_1) else {continue};
                let Some(node_2) = self.world.nodes.get(muscle.node_2) else {continue};
                let pos_1 = self.camera.world_to_view(node_1.pos());
                let pos_2 = self.camera.world_to_view(node_2.pos());
                self.draw_muscle(&draw, pos_1, pos_2, self.camera.zoom);
            }

            // mouse ray
            // let world_center = Vec2::new(1500., 1500.);
            // let mouse_pos = self.camera.view_to_world(self.input_state.mouse_pos);
            // let dir = mouse_pos - world_center;

            // let first_node = self
            //     .world
            //     .collider
            //     .ray_collides_iter(&self.world.nodes, world_center, dir)
            //     .find(|node| ray_collides_circle(world_center, dir, node.pos(), node.radius));
            // if let Some(node) = first_node {
            //     let pos = node.pos();
            //     let radius = node.radius * self.camera.zoom;
            //     draw.ellipse()
            //         .color(rgb(255u8, 0, 0))
            //         .xy(self.camera.world_to_view(pos))
            //         .radius(radius);
            // }

            // for (x, y) in std::iter::once(self.world.collider.node_to_grid_pos(world_center))
            //     .chain(self.world.collider.ray_cells_iter(world_center, dir))
            // {
            //     let pos = vec2(x as f32, y as f32) * world::collide::CELL_SIZE;
            //     let pos = self.camera.world_to_view(pos);
            //     let size =
            //         vec2(world::collide::CELL_SIZE, world::collide::CELL_SIZE) * self.camera.zoom;
            //     draw.rect()
            //         .color(rgba(255u8, 0, 0, 0))
            //         .xy(pos + size / 2.)
            //         .wh(size)
            //         .stroke(rgb(255u8, 0, 0))
            //         .stroke_weight(1.0);
            // }

            // // draw it
            // draw.line()
            //     .color(rgb(255u8, 255, 255))
            //     .start(self.camera.world_to_view(world_center))
            //     .end(self.camera.world_to_view(mouse_pos))
            //     .weight(1.0);
        }

        draw_gui(self, app, draw.clone());
    }
    pub fn draw_bone(&self, draw: &Draw, pos_1: Point2, pos_2: Point2, zoom: f32) {
        if !pos_1.is_finite() || !pos_1.is_finite() {
            return;
        }
        if !self.within_view(pos_1) && !self.within_view(pos_2) {
            return;
        }

        let radius = 3.0 * zoom;
        draw.line()
            .color(rgb(150u8, 200, 150))
            .start(pos_1)
            .end(pos_2)
            .weight(radius);
    }

    pub fn draw_muscle(&self, draw: &Draw, pos_1: Point2, pos_2: Point2, zoom: f32) {
        if !pos_1.is_finite() || !pos_1.is_finite() {
            return;
        }
        if !self.within_view(pos_1) && !self.within_view(pos_2) {
            return;
        }

        let radius = 3.0 * zoom;
        draw.line()
            .color(rgb(200u8, 150, 150))
            .start(pos_1)
            .end(pos_2)
            .weight(radius);
    }

    pub fn update(&mut self, _app: &App, _update: Update) {
        if self.input_state.skip_toggled {
            self.world.skip(512);
        } else {
            self.world.update();
        }
        match self.scene {
            Scene::Cluster => {
                if self.clusters.is_none() {
                    let mut data = Vec::new();
                    for (org_id, org) in self.world.organisms.iter_with_ids() {
                        data.push((org.genome.tag.clone(), org_id));
                    }
                    self.clusters = Some(cluster::cluster(&data));
                }
            }
            Scene::World => (),
        }
    }
    pub fn event(&mut self, _app: &App, event: Event) {
        const CAMERA_MOVE: f32 = 2.0;
        const CAMERA_ZOOM: f32 = 1.02;
        self.input_state.update(&event);

        match event {
            Event::WindowEvent {
                simple: Some(event),
                ..
            } => match event {
                KeyPressed(key) => match key {
                    Key::C => match self.scene {
                        Scene::Cluster => self.scene = Scene::World,
                        _ => {
                            self.clusters = None;
                            self.scene = Scene::Cluster;
                        }
                    },
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
        if self.input_state.up {
            self.camera.pos.y += CAMERA_MOVE / self.camera.zoom;
        }
        if self.input_state.down {
            self.camera.pos.y -= CAMERA_MOVE / self.camera.zoom;
        }
        if self.input_state.left {
            self.camera.pos.x -= CAMERA_MOVE / self.camera.zoom;
        }
        if self.input_state.right {
            self.camera.pos.x += CAMERA_MOVE / self.camera.zoom;
        }
        if self.input_state.plus {
            self.camera.zoom *= CAMERA_ZOOM;
        }
        if self.input_state.minus {
            self.camera.zoom /= CAMERA_ZOOM;
        }

        if self.input_state.mouse_pressed {
            // move node towards mouse pos
            let mouse_pos = self.camera.view_to_world(self.input_state.mouse_pos);
            // get nearest node
            self.input_state.dragged = self.input_state.dragged.clone().or_else(|| {
                let mut nearest_dist = 100000000.0;
                let mut nearest_node = None;
                let mut nearest_org = None;
                for (org_id, org) in self.world.organisms.iter_with_ids() {
                    for node_id in org.node_ids().iter() {
                        let Some(node) = &self.world.nodes.get(*node_id) else {continue};
                        let dist = node.pos().distance(mouse_pos);
                        if dist < nearest_dist {
                            nearest_dist = dist;
                            nearest_node = Some(*node_id);
                            nearest_org = Some(org_id);
                        }
                    }
                }
                if let Some(node_id) = nearest_node {
                    Some(NodeInfo {
                        node_id,
                        organism_id: nearest_org,
                    })
                } else {
                    None
                }
            });

            // move node towards mouse pos
            let Some(dragged) = &self.input_state.dragged else {return};
            match &mut self.world.nodes.get_mut(dragged.node_id) {
                Some(ref mut node) => {
                    let dist = node.pos().distance(mouse_pos);
                    if dist > 0.0 {
                        *node.pos_mut() = mouse_pos;
                    }
                }
                None => self.input_state.dragged = None,
            }
        } else {
            if let Some(dragged) = &self.input_state.dragged {
                let Some(node) = &mut self.world.nodes.get(dragged.node_id) else {return};
                println!("Node: {:#?}", node);
                if let Some(organism_id) = dragged.organism_id {
                    let Some(organism) = &mut self.world.organisms.get(organism_id) else {return};
                    println!("{}", organism.genome);
                    println!("{}", organism.brain);
                }
                self.input_state.selected = self.input_state.dragged.take();
            }
        }
    }
}

fn draw_gui(model: &Model, app: &App, draw: Draw) {
    let mut y = WINDOW_SIZE as f32 / 2.0 - 10.;
    let mut add_text = |text: &str| {
        draw.text(text)
            .color(WHITE)
            .font_size(16)
            .x_y(WINDOW_SIZE as f32 / -2.0 + 50.0, y)
            .width(100.0)
            .left_justify();
        y -= 20.0;
    };
    let texts = vec![
        format!("FPS: {}", app.fps() as u32),
        format!("Nodes: {}", model.world.nodes.len()),
        format!("Bones: {}", model.world.bones.len()),
        format!("Muscles: {}", model.world.muscles.len()),
    ];

    // draw rect behind
    draw.rect()
        .color(BLACK)
        .w_h(100.0, 20.0 * texts.len() as f32)
        .x_y(
            WINDOW_SIZE as f32 / -2.0 + 50.0,
            WINDOW_SIZE as f32 / 2.0 - 10.0 * texts.len() as f32,
        );
    for text in texts.iter() {
        add_text(text);
    }
}
