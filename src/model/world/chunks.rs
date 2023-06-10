use itertools::iproduct;
use nannou::prelude::*;
use noise::{NoiseFn, SuperSimplex};

#[derive(Debug, Clone)]
pub struct Chunk {
    pub sun: f32,
    pub tide: Vec2,
}
pub const TIDE_MULT: f32 = 0.05;

#[derive(Debug, Clone)]
pub struct Chunks {
    pub grid: Vec<Chunk>,
    pub world_size: Vec2,
    pub grid_size: (usize, usize),
    pub noise: (SuperSimplex, SuperSimplex),
}
impl Chunks {
    pub fn new(size: Vec2, cell_size: f32) -> Self {
        let grid_width = (size.x / cell_size).ceil() as usize;
        let grid_height = (size.y / cell_size).ceil() as usize;

        let mut grid: Vec<_> = iproduct!(0..grid_height, 0..grid_width)
            .map(|_| Chunk {
                sun: 0.,
                tide: vec2(0., 0.),
            })
            .collect();
        for _ in 0..8 {
            // choose random sun position
            let sun_pos: Point2 = (random_range(0., size.x), random_range(0., size.y)).into();
            let sun_radius = size.x / 4.0;
            for (y, x) in iproduct!(0..grid_height, 0..grid_width) {
                let pos = vec2(x as f32 * cell_size, y as f32 * cell_size);
                // sun is from 0 to 1
                let sun = 1.0 - (sun_pos.distance(pos) / sun_radius).clamp(0.0, 1.0);
                grid[y * grid_width + x].sun += sun;
            }
        }
        // let middle = vec2(size.x / 2.0, size.y / 2.0);
        // for (y, x) in iproduct!(0..grid_height, 0..grid_width) {
        //     let pos = vec2(x as f32 * cell_size, y as f32 * cell_size);
        //     // make it go in a circle around the middle (so rotate 90 degrees)
        //     let tide = (pos - middle).rotate(PI / 2.0).normalize();
        //     grid[y * grid_width + x].tide = tide * TIDE_MULT;
        // }

        let noise = (
            SuperSimplex::new(random_range(0, 20)),
            SuperSimplex::new(random_range(20, 40)),
        );

        let mut ret = Chunks {
            grid,
            world_size: size,
            grid_size: (grid_width, grid_height),
            noise,
        };
        ret.update(0);
        ret
    }
    pub fn get(&self, pos: Vec2) -> &Chunk {
        let y = ((pos.y / self.world_size.y * self.grid_size.1 as f32) as usize)
            .clamp(0, self.grid_size.1 - 1);
        let x = ((pos.x / self.world_size.x * self.grid_size.0 as f32) as usize)
            .clamp(0, self.grid_size.0 - 1);
        &self.grid[y * self.grid_size.0 + x]
    }
    pub fn update(&mut self, tick: u64) {
        let cell_size = self.world_size.x / self.grid_size.0 as f32;
        let scale = 0.01;
        let time = tick as f64 * 0.005;
        for (x, y) in iproduct!(0..self.grid_size.0, 0..self.grid_size.1) {
            let pos = vec2(x as f32 * cell_size, y as f32 * cell_size) * scale;
            let tide = vec2(
                self.noise.0.get([pos.x as f64, pos.y as f64, time]) as f32,
                self.noise.1.get([pos.x as f64, pos.y as f64, time]) as f32,
            ) * TIDE_MULT;
            self.grid[y * self.grid_size.0 + x].tide = tide;
        }
    }
}
