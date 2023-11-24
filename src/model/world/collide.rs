use super::collection::CollectionView;
use super::collection::GenId;
use super::node::Node;
use super::MAX_NODE_RADIUS;
use itertools::{iproduct, Itertools};
use nannou::prelude::*;
#[derive(Debug, Clone)]
pub struct Collider {
    pub grid: Vec<Vec<GenId>>,
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
    fn node_to_grid_pos(&self, pos: Point2) -> (usize, usize) {
        let x = ((pos.x / self.world_size.x * self.grid_size.0 as f32) as usize)
            .clamp(0, self.grid_size.0 - 1);
        let y = ((pos.y / self.world_size.y * self.grid_size.1 as f32) as usize)
            .clamp(0, self.grid_size.1 - 1);
        (x, y)
    }
    fn node_to_grid_index(&self, pos: Point2) -> usize {
        let (x, y) = self.node_to_grid_pos(pos);
        y * self.grid_size.1 + x
    }
    fn update(&mut self, nodes: &CollectionView<Node>) {
        for cell in &mut self.grid {
            cell.clear();
        }
        for (id, node) in nodes.iter_with_ids() {
            let index = self.node_to_grid_index(node.pos());
            self.grid[index].push(id);
        }
    }
    fn neighbor_cells<'a>(
        &'a self,
        x: usize,
        y: usize,
        deltas: &'a [(i32, i32)],
    ) -> impl Iterator<Item = &Vec<GenId>> + 'a {
        deltas.iter().filter_map(move |(dx, dy)| {
            let y = y as i32 + dy;
            let x = x as i32 + dx;
            if y < 0 || x < 0 || y >= self.grid_size.1 as i32 || x >= self.grid_size.0 as i32 {
                return None;
            }
            Some(&self.grid[y as usize * self.grid_size.1 + x as usize])
        })
    }
    fn collide_cells<'a>(
        &'a self,
        x: usize,
        y: usize,
        deltas: &'a [(i32, i32)],
    ) -> impl Iterator<Item = (GenId, GenId)> + 'a {
        let current_cell = &self.grid[y * self.grid_size.1 + x];
        let neighbor_collide = self
            .neighbor_cells(x, y, deltas)
            .map(|other_cell| iproduct!(current_cell.iter(), other_cell.iter()))
            .flatten()
            .map(|(index_1, index_2)| (*index_1, *index_2));
        let self_collide = (0..current_cell.len())
            .tuple_combinations()
            .map(|(i, j)| (current_cell[i], current_cell[j]));
        neighbor_collide.chain(self_collide)
    }
    #[allow(dead_code)]
    pub fn collide(
        &mut self,
        nodes: &mut CollectionView<Node>,
        mut collide_fn: impl FnMut(&mut Node, &mut Node),
    ) {
        self.update(nodes);
        for (x, y) in iproduct!(0..self.grid_size.0, 0..self.grid_size.1) {
            self.collide_cells(x, y, &[
                // (-1, -1) exclude because its covered by the last cell's (1, 1).
                // (0, -1) exclude because its covered by the last cell's (0, 1).
                // there's no need to check it ever because the first cell ignores out of border cells.
                // (1, -1),
                // (-1, 0) exclude because its covered by the last cell's (1, 0).
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ]).for_each(|(id_1, id_2)| {
                let (Some(node_1), Some(node_2)) = nodes.get_2_mut(id_1, id_2) else {unreachable!()};
                collide_fn(node_1, node_2);
            })
        }
    }
    pub fn pos_collides_iter<'a>(
        &'a self,
        nodes: &'a CollectionView<Node>,
        pos: Point2,
    ) -> impl Iterator<Item = &Node> + 'a {
        let (x, y) = self.node_to_grid_pos(pos);
        self.neighbor_cells(
            x,
            y,
            &[
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
                (0, 0),
            ],
        )
        .flatten()
        .filter_map(move |index| nodes.get(*index))
    }
}
use rayon::prelude::*;

impl Collider {
    pub fn par_collide(
        &mut self,
        nodes: &mut CollectionView<Node>,
        collide_fn: fn(&mut Node, &mut Node),
    ) {
        self.update(nodes);
        let nodes_slice = super::sync_mut::UnsafeMutSlice::new(nodes.get_mut_slice());
        let even_rows_iter = (0..self.grid_size.1).into_par_iter().step_by(2);
        let odd_rows_iter = (1..self.grid_size.1).into_par_iter().step_by(2);
        let collide = |y| {
            for x in 0..self.grid_size.0 {
                self.collide_cells(x, y, &[(1, 0), (-1, 1), (0, 1), (1, 1)])
                    .for_each(|(id_1, id_2)| {
                        let Some(node_1) = (unsafe { nodes_slice.get(id_1.index) }) else {return}; // TODO revert this
                        let Some(node_2) = (unsafe { nodes_slice.get(id_2.index) }) else {return};
                        collide_fn(node_1, node_2);
                    });
            }
        };
        even_rows_iter.for_each(collide);
        odd_rows_iter.for_each(collide);
    }
}
