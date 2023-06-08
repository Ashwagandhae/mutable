use itertools::iproduct;
use nannou::prelude::*;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub sun: f32,
}

#[derive(Debug, Clone)]
pub struct Chunks {
    pub grid: Vec<Chunk>,
    pub world_size: Vec2,
    pub grid_size: (usize, usize),
}
impl Chunks {
    pub fn new(size: Vec2, cell_size: f32) -> Self {
        let grid_width = (size.x / cell_size).ceil() as usize;
        let grid_height = (size.y / cell_size).ceil() as usize;

        // let sun_pos: Point2 = (size.x / 2.0, size.y / 2.0).into();
        // let sun_radius = size.x / 3.0;
        // for (y, x) in iproduct!(0..grid_height, 0..grid_width) {
        //     let pos = vec2(x as f32 * cell_size, y as f32 * cell_size);
        //     // sun is from 0 to 1
        //     let sun = 1.0 - (sun_pos.distance(pos) / sun_radius).clamp(0.0, 1.0);
        //     grid.push(Chunk { sun });
        // }
        let mut grid: Vec<_> = iproduct!(0..grid_height, 0..grid_width)
            .map(|_| Chunk { sun: 0. })
            .collect();
        for _ in 0..4 {
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
        Chunks {
            grid,
            world_size: size,
            grid_size: (grid_width, grid_height),
        }
    }
    pub fn get(&self, pos: Vec2) -> &Chunk {
        let y = ((pos.y / self.world_size.y * self.grid_size.1 as f32) as usize)
            .clamp(0, self.grid_size.1 - 1);
        let x = ((pos.x / self.world_size.x * self.grid_size.0 as f32) as usize)
            .clamp(0, self.grid_size.0 - 1);
        &self.grid[y * self.grid_size.0 + x]
    }
}
