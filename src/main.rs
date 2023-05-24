use nannou::prelude::*;
mod model;
use model::Model;
use model::WINDOW_SIZE;

fn main() {
    nannou::app(model)
        .update(update)
        .event(event)
        .simple_window(view)
        .size(WINDOW_SIZE, WINDOW_SIZE)
        .run();
}

fn model(_app: &App) -> Model {
    Model::new()
}

fn update(app: &App, model: &mut Model, update: Update) {
    model.update(app, update);
}

fn event(app: &App, model: &mut Model, event: Event) {
    model.event(app, event);
}

fn view(app: &App, model: &Model, frame: Frame) {
    model.view(app, frame);
}
