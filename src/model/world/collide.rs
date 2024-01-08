use super::collection::GenId;
use super::collection::{Collection, CollectionView};
use super::node::Node;
use super::MAX_NODE_RADIUS;
use itertools::{iproduct, Itertools};
use nannou::prelude::*;

pub const CELL_SIZE: f32 = MAX_NODE_RADIUS * 2.0;
#[derive(Debug, Clone)]
pub struct Collider {
    pub grid: Vec<Vec<GenId>>,
    pub world_size: Vec2,
    pub grid_size: (usize, usize),
}
impl Collider {
    pub fn new(size: Vec2) -> Collider {
        let mut grid = Vec::new();
        let cell_size = CELL_SIZE;
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
    pub fn node_to_grid_pos(&self, pos: Point2) -> (usize, usize) {
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
    // def find_ray_voxel_intersects(ray: Ray, amount: int = 10):
    //     voxel_pos = pos_to_voxel(ray.pos)
    //     x = voxel_pos.x
    //     y = voxel_pos.y
    //     yield voxel_pos
    //     step_x = get_one_zero_neg_one(ray.dir.x)
    //     step_y = get_one_zero_neg_one(ray.dir.y)

    //     x_barrier = (x + (1 if step_x > 1 else 0)) * GRID_SIZE
    //     y_barrier = (y + (1 if step_y > 1 else 0)) * GRID_SIZE

    //     t_max_x = (x_barrier - ray.pos.x) / ray.dir.x if ray.dir.x != 0 else 100000000
    //     t_max_y = (y_barrier - ray.pos.y) / ray.dir.y if ray.dir.y != 0 else 100000000

    //     t_delta_x = GRID_SIZE / abs(ray.dir.x) if ray.dir.x != 0 else 100000000
    //     t_delta_y = GRID_SIZE / abs(ray.dir.y) if ray.dir.y != 0 else 100000000

    //     while True:
    //         if t_max_x < t_max_y:
    //             t_max_x += t_delta_x
    //             x += step_x
    //         else:
    //             t_max_y += t_delta_y
    //             y += step_y
    //         yield Vec(x, y)
    //         x_maxed = x == max_x
    //         y_maxed = y == max_y
    //         if x_maxed and y_maxed:
    //              print(x, max_x)
    //              print(y, max_y)
    //              break
    // pub fn ray_collides_iter<'a>(
    //     &'a self,
    //     nodes: &'a Collection<Node>,
    //     origin: Point2,
    //     dir: Vec2,
    // ) -> impl Iterator<Item = &Node> + 'a {
    //     self.ray_cells_iter(origin, dir)
    //         .flat_map(move |(x, y)| self.grid[y * self.grid_size.1 + x].iter())
    //         .filter_map(move |index| nodes.get(*index))
    // }
    pub fn ray_collides_iter<'a>(
        &'a self,
        nodes: &'a Collection<Node>,
        origin: Point2,
        dir: Vec2,
    ) -> impl Iterator<Item = &Node> + 'a {
        self.ray_cells_padded_iter(origin, dir)
            .filter_map(move |index| nodes.get(*index))
    }

    /// returns the cells that the ray passes through, excluding the first cell
    pub fn ray_cells_iter(
        &self,
        origin: Point2,
        dir: Vec2,
    ) -> impl Iterator<Item = (usize, usize)> {
        // TODO fix imperfect cells
        let (x, y) = self.node_to_grid_pos(origin);
        let mut x = x as i32;
        let mut y = y as i32;
        let (end_x, end_y) = self.node_to_grid_pos(origin + dir);
        let end_x = end_x as i32;
        let end_y = end_y as i32;
        let step_x = if dir.x > 0. { 1 } else { -1 };
        let step_y = if dir.y > 0. { 1 } else { -1 };
        let x_barrier = (x + (if step_x > 1 { 1 } else { 0 })) as f32 * CELL_SIZE;
        let y_barrier = (y + (if step_y > 1 { 1 } else { 0 })) as f32 * CELL_SIZE;
        let mut t_max_x = ((x_barrier - origin.x) / dir.x).abs();
        let mut t_max_y = ((y_barrier - origin.y) / dir.y).abs();
        let t_delta_x = CELL_SIZE / dir.x.abs();
        let t_delta_y = CELL_SIZE / dir.y.abs();

        // dbg!(&t_max_x, &t_max_y, &t_delta_x, &t_delta_y);

        let mut finished = false;

        let grid_size = self.grid_size;

        std::iter::from_fn(move || {
            if finished {
                return None;
            }
            if t_max_x < t_max_y {
                t_max_x += t_delta_x;
                x += step_x;
            } else {
                t_max_y += t_delta_y;
                y += step_y;
            }
            if ((x >= end_x && step_x >= 0) || (x <= end_x && step_x <= 0))
                && ((y >= end_y && step_y >= 0) || (y <= end_y && step_y <= 0))
            {
                finished = true;
            }
            if !(0..grid_size.0 as i32).contains(&x) || !(0..grid_size.1 as i32).contains(&y) {
                return None;
            }
            return Some((x as usize, y as usize));
        })
    }

    /// returns the cells that the ray passes through, including the first cell, and padding all cells by 1
    pub fn ray_cells_padded_iter(&self, origin: Point2, dir: Vec2) -> impl Iterator<Item = &GenId> {
        // fully pad the first cell
        let (first_x, first_y) = self.node_to_grid_pos(origin);
        let first_cells = self.neighbor_cells(
            first_x,
            first_y,
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
        );
        let cells = self.ray_cells_iter(origin, dir).tuple_windows().flat_map(
            |((old_x, old_y), (new_x, new_y))| {
                let diff = (new_x as i32 - old_x as i32, new_y as i32 - old_y as i32);
                match diff {
                    (1, 0) => self.neighbor_cells(old_x, old_y, &[(2, -1), (2, 0), (2, 1)]),
                    (-1, 0) => self.neighbor_cells(old_x, old_y, &[(-2, -1), (-2, 0), (-2, 1)]),
                    (0, 1) => self.neighbor_cells(old_x, old_y, &[(-1, 2), (0, 2), (1, 2)]),
                    (0, -1) => self.neighbor_cells(old_x, old_y, &[(-1, -2), (0, -2), (1, -2)]),
                    _ => panic!("invalid diff {:?}", diff),
                }
            },
        );

        first_cells.chain(cells).flatten()
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
