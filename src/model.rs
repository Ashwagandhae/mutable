use nannou::prelude::*;
mod world;
use world::World;
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

pub struct Model {
    pub camera: Camera,
    pub input_state: InputState,
    pub world: World,
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
        }
    }
    pub fn update(&mut self, event: Event) {
        match event {
            Event::WindowEvent {
                simple: Some(event),
                ..
            } => match event {
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
                    Key::Left => self.left = false,
                    Key::Right => self.right = false,
                    Key::Equals => self.plus = false,
                    Key::Minus => self.minus = false,
                    _ => (),
                },
                MouseMoved(pos) => self.mouse_pos = pos,
                MousePressed(_) => self.mouse_pressed = true,
                MouseReleased(_) => self.mouse_pressed = false,
                _ => (),
            },
            _ => (),
        }
    }
}
impl Model {
    pub fn new() -> Model {
        Model {
            camera: Camera {
                pos: Point2::new(0.0, 0.0),
                zoom: 2.0,
            },
            world: World::new(),
            input_state: InputState::new(),
        }
    }
    pub fn view(&self, app: &App, frame: Frame) {
        let draw = app.draw();
        draw.background().color(BLACK);
        for node in &self.world.nodes {
            // adjust for camera zoom and pos
            let pos = self.camera.world_to_view(node.pos);
            if !pos.is_finite() {
                continue;
            }
            let radius = node.radius * self.camera.zoom;
            draw.ellipse()
                .color(rgb(150 as u8, 150, 200))
                .xy(pos)
                .radius(radius);
        }
        for edge in &self.world.edges {
            // adjust for camera zoom and pos
            let pos_1 = self.camera.world_to_view(self.world.nodes[edge.node_1].pos);
            let pos_2 = self.camera.world_to_view(self.world.nodes[edge.node_2].pos);
            if !pos_1.is_finite() || !pos_1.is_finite() {
                continue;
            }

            let radius = 3.0 * self.camera.zoom;
            draw.line()
                .color(rgb(150 as u8, 200, 150))
                .start(pos_1)
                .end(pos_2)
                .weight(radius);
        }
        for muscle in &self.world.muscles {
            // adjust for camera zoom and pos
            let pos_1 = self
                .camera
                .world_to_view(self.world.nodes[muscle.node_1].pos);
            let pos_2 = self
                .camera
                .world_to_view(self.world.nodes[muscle.node_2].pos);
            if !pos_1.is_finite() || !pos_1.is_finite() {
                continue;
            }

            let radius = 3.0 * self.camera.zoom;
            draw.line()
                .color(rgb(200 as u8, 150, 150))
                .start(pos_1)
                .end(pos_2)
                .weight(radius);
        }
        draw.to_frame(app, &frame).unwrap();
    }
    pub fn update(&mut self, _app: &App, update: Update) {
        self.world.update(update.since_last, update.since_start);
    }
    pub fn event(&mut self, _app: &App, event: Event) {
        const CAMERA_MOVE: f32 = 2.0;
        const CAMERA_ZOOM: f32 = 1.02;
        self.input_state.update(event);
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
            let mut nearest_node = 0;
            let mut nearest_dist = 100000000.0;
            for (i, node) in self.world.nodes.iter().enumerate() {
                let dist = node.pos.distance(mouse_pos);
                if dist < nearest_dist {
                    nearest_dist = dist;
                    nearest_node = i;
                }
            }
            // move node towards mouse pos
            let node = &mut self.world.nodes[nearest_node];
            let dist = node.pos.distance(mouse_pos);
            if dist > 0.0 {
                node.pos = mouse_pos;
            }
        }
    }
}
