use super::collection::Collection;
use super::node::Node;
use super::MAX_NODE_RADIUS;
use itertools::iproduct;
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
    fn update(&mut self, nodes: &Collection<Node>) {
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
    pub fn collide(
        &mut self,
        nodes: &mut Collection<Node>,
        mut collide_fn: impl FnMut(&mut Node, &mut Node),
    ) {
        self.update(nodes);
        for (x, y) in iproduct!(0..self.grid_size.0, 0..self.grid_size.1) {
            let current_cell = &self.grid[y * self.grid_size.1 + x];
            for (dx, dy) in [
                // (-1, -1) exclude because its covered by the last cell's (1, 1).
                // (0, -1) exclude because its covered by the last cell's (0, 1).
                // there's no need to check it ever because the first cell ignores out of border cells.
                // (1, -1),
                // (-1, 0) exclude because its covered by the last cell's (1, 0).
                // (0, 0) exclude self because special handling
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ] {
                let y = y as i32 + dy;
                let x = x as i32 + dx;
                if y < 0 || x < 0 || y >= self.grid_size.1 as i32 || x >= self.grid_size.0 as i32 {
                    continue;
                }
                let other_cell = &self.grid[y as usize * self.grid_size.1 + x as usize];
                for (id_1, id_2) in iproduct!(current_cell.iter(), other_cell.iter()) {
                    let (Some(node_1), Some(node_2)) = nodes.get_2_index_mut(*id_1, *id_2) else {unreachable!()};
                    collide_fn(node_1, node_2);
                }
            }
            // self special handling
            for i in 0..current_cell.len() {
                for j in i + 1..current_cell.len() {
                    let (id_1, id_2) = (current_cell[i], current_cell[j]);
                    let (Some(node_1), Some(node_2)) = nodes.get_2_index_mut(id_1, id_2) else {unreachable!()};
                    collide_fn(node_1, node_2);
                }
            }
        }
    }
}
use rayon::prelude::*;

impl Collider {
    pub fn par_collide(
        &mut self,
        nodes: &mut Collection<Node>,
        collide_fn: fn(&mut Node, &mut Node),
    ) {
        self.update(nodes);
        let nodes_slice = sync_mut_vec::UnsafeMutSlice::new(nodes.get_mut_slice());
        let even_rows_iter = (0..self.grid_size.1 - 1).into_par_iter().step_by(2);
        let odd_rows_iter = (1..self.grid_size.1).into_par_iter().step_by(2);
        let collide = |y| {
            for x in 0..self.grid_size.0 {
                let current_cell: &Vec<usize> = &self.grid[y * self.grid_size.1 + x];
                for (dx, dy) in [(1, 0), (-1, 1), (0, 1), (1, 1)] {
                    let y = y as i32 + dy;
                    let x = x as i32 + dx;
                    if y < 0
                        || x < 0
                        || y >= self.grid_size.1 as i32
                        || x >= self.grid_size.0 as i32
                    {
                        continue;
                    }
                    let other_cell = &self.grid[y as usize * self.grid_size.1 + x as usize];
                    for (id_1, id_2) in iproduct!(current_cell.iter(), other_cell.iter()) {
                        let Some(node_1) = (unsafe { nodes_slice.get_mut(*id_1) }) else {unreachable!()};
                        let Some(node_2) = (unsafe { nodes_slice.get_mut(*id_2) }) else {unreachable!()};
                        collide_fn(node_1, node_2);
                    }
                }
                // self special handling
                for i in 0..current_cell.len() {
                    for j in i + 1..current_cell.len() {
                        let (id_1, id_2) = (current_cell[i], current_cell[j]);
                        let Some(node_1) = (unsafe { nodes_slice.get_mut(id_1) }) else {unreachable!()};
                        let Some(node_2) = (unsafe { nodes_slice.get_mut(id_2) }) else {unreachable!()};
                        collide_fn(node_1, node_2);
                    }
                }
            }
        };
        even_rows_iter.for_each(collide);
        odd_rows_iter.for_each(collide);
    }
}

mod sync_mut_vec {
    use std::cell::UnsafeCell;

    pub struct UnsafeMutSlice<'a, T> {
        slice: UnsafeCell<&'a mut [T]>,
    }
    impl<'a, T> UnsafeMutSlice<'a, T> {
        /// # Safety
        /// The caller must ensure that no two threads modify the same index at the same time.
        pub unsafe fn get_mut(&self, index: usize) -> &mut T {
            let slice = unsafe { &mut *self.slice.get() };
            &mut slice[index]
        }
        pub fn new(slice: &'a mut [T]) -> Self {
            Self {
                slice: UnsafeCell::new(slice),
            }
        }
    }

    unsafe impl<'a, T> Sync for UnsafeMutSlice<'a, T> {}
}
