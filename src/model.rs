use nannou::prelude::*;
mod world;
use world::World;

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
    pub skip_toggled: bool,
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
                    Key::Space => self.skip_toggled = !self.skip_toggled,
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
                pos: Point2::new(100.0, 100.0),
                zoom: 2.0,
            },
            world: World::new(),
            input_state: InputState::new(),
        }
    }
    pub fn within_view(&self, pos: Point2) -> bool {
        const LOWER: f32 = WINDOW_SIZE as f32 / -2.0;
        const UPPER: f32 = WINDOW_SIZE as f32 / 2.0;
        pos.x > LOWER && pos.x < UPPER && pos.y > LOWER && pos.y < UPPER
    }
    pub fn view(&self, app: &App, frame: Frame) {
        let draw = app.draw();
        draw.background().color(BLACK);
        draw.rect()
            .color(rgb(50u8, 50, 50))
            .xy(self.camera.world_to_view(self.world.size / 2.))
            .wh(self.world.size * self.camera.zoom);
        for node in self.world.nodes.iter() {
            // adjust for camera zoom and pos
            let pos = self.camera.world_to_view(node.pos);
            if !pos.is_finite() {
                continue;
            }
            if !self.within_view(pos) {
                continue;
            }
            let radius = node.radius * self.camera.zoom;
            draw.ellipse()
                .color(rgb(150u8, 150, 200))
                .xy(pos)
                .radius(radius);
        }
        for bone in self.world.bones.iter() {
            // adjust for camera zoom and pos
            let pos_1 = self
                .camera
                .world_to_view(self.world.nodes.get(bone.node_1).unwrap().pos);
            let pos_2 = self
                .camera
                .world_to_view(self.world.nodes.get(bone.node_2).unwrap().pos);
            if !pos_1.is_finite() || !pos_1.is_finite() {
                continue;
            }
            if !self.within_view(pos_1) && !self.within_view(pos_2) {
                continue;
            }

            let radius = 3.0 * self.camera.zoom;
            draw.line()
                .color(rgb(150u8, 200, 150))
                .start(pos_1)
                .end(pos_2)
                .weight(radius);
        }
        for muscle in self.world.muscles.iter() {
            // adjust for camera zoom and pos
            let pos_1 = self
                .camera
                .world_to_view(self.world.nodes.get(muscle.node_1).unwrap().pos);
            let pos_2 = self
                .camera
                .world_to_view(self.world.nodes.get(muscle.node_2).unwrap().pos);
            if !pos_1.is_finite() || !pos_1.is_finite() {
                continue;
            }
            if !self.within_view(pos_1) && !self.within_view(pos_2) {
                continue;
            }

            let radius = 3.0 * self.camera.zoom;
            draw.line()
                .color(rgb(200u8, 150, 150))
                .start(pos_1)
                .end(pos_2)
                .weight(radius);
        }
        draw.to_frame(app, &frame).unwrap();
    }
    pub fn update(&mut self, _app: &App, _update: Update) {
        if self.input_state.skip_toggled {
            self.world.skip(100);
        } else {
            self.world.update();
        }
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
            let node = &mut self.world.nodes.get_mut(nearest_node).unwrap();
            let dist = node.pos.distance(mouse_pos);
            if dist > 0.0 {
                node.pos = mouse_pos;
            }
        }
    }
}
