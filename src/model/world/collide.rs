use super::collection::Collection;
use super::node::Node;
use super::MAX_NODE_RADIUS;
use itertools::Itertools;
use nannou::prelude::*;

#[derive(Debug, Clone)]
pub struct Collider {
    pub grid: Vec<Vec<usize>>,
    pub world_size: Vec2,
    pub grid_size: (usize, usize),
}
impl Collider {
    pub fn new(size: Vec2) -> Collider {
        let mut grid = Vec::new();
        let cell_size = MAX_NODE_RADIUS * 2.0;
        let grid_width = (size.x / cell_size).ceil() as usize;
        let grid_height = (size.y / cell_size).ceil() as usize;
        for _ in 0..grid_height * grid_width {
            grid.push(Vec::new());
        }
        Collider {
            grid,
            world_size: size,
            grid_size: (grid_width, grid_height),
        }
    }
    pub fn update(&mut self, nodes: &Collection<Node>) {
        // clear
        for cell in &mut self.grid {
            cell.clear();
        }
        for (i, node) in nodes.iter_with_indices() {
            let y = ((node.pos.y / self.world_size.y * self.grid_size.1 as f32) as usize)
                .clamp(0, self.grid_size.1 - 1);
            let x = ((node.pos.x / self.world_size.x * self.grid_size.0 as f32) as usize)
                .clamp(0, self.grid_size.0 - 1);
            self.grid[y * self.grid_size.1 + x].push(i);
        }
    }
    pub fn collide(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.grid_size.1 - 2).flat_map(move |y| {
            (0..self.grid_size.0 - 2).flat_map(move |x| {
                (y..y + 3)
                    .flat_map(move |y| {
                        (x..x + 3).flat_map(move |x| self.grid[y * self.grid_size.1 + x][..].iter())
                    })
                    .copied()
                    .tuple_combinations()
            })
        })
    }
}
