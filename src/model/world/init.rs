use super::bone::Bone;
use super::collection::Collection;
use super::gene::Genome;
use super::muscle::Muscle;
use super::node::Node;
use super::organism::Organism;
use nannou::prelude::*;

pub fn random_organisms(
    nodes: &mut Collection<Node>,
    _bones: &mut Collection<Bone>,
    _muscles: &mut Collection<Muscle>,
    organisms: &mut Collection<Organism>,
    size: Vec2,
) {
    for _ in 0..((size.x * size.y / 1500.) as usize) {
        let genome = Genome::random_plant();
        let pos = vec2(random_range(0., size.x), random_range(0., size.y));
        organisms.push(Organism::build(pos, genome, 20., nodes));
    }
}

// pub fn random_trees(
//     nodes: &mut Collection<Node>,
//     bones: &mut Collection<Bone>,
//     muscles: &mut Collection<Muscle>,
// ) {
//     for _ in 0..10000 {
//         nodes.push(Node::new(
//             Point2::new(random_range(0., 9000.), random_range(0., 9000.)),
//             5.,
//         ));
//         let mut start_index = nodes.len() - 1;
//         for _ in 0..1 {
//             let mut new_nodes = Vec::new();
//             for (i, parent) in nodes.get_vec()[start_index..]
//                 .iter()
//                 .filter_map(|x| x.as_ref())
//                 .enumerate()
//             {
//                 let parent_id = start_index + i;
//                 for j in 0..2 {
//                     let new_child = Node::new(
//                         Point2::new(
//                             parent.pos.x + random_range(-10., 10.),
//                             parent.pos.y + random_range(-10., 10.),
//                         ),
//                         5.,
//                     );
//                     new_nodes.push(new_child);
//                     let node_id = nodes.len() + new_nodes.len() - 1;
//                     bones.push(Bone::new(parent_id, node_id, 15.));
//                     if j > 0 && random::<f32>() > 0. {
//                         let node_1 = node_id;
//                         let node_2 = node_id - 1;

//                         let min_angle = PI / 4.;
//                         muscles.push(Muscle::new(
//                             parent_id,
//                             node_2,
//                             node_1,
//                             Angle(random_range(min_angle, 2. * PI - min_angle)),
//                             |x| (x as f32 / 10.).sin(),
//                             0.25,
//                         ));
//                     }
//                 }
//             }
//             start_index = nodes.len();
//             nodes.extend(&mut new_nodes.clone());
//         }
//     }
// }

// pub fn fish(
//     nodes: &mut Collection<Node>,
//     bones: &mut Collection<Bone>,
//     muscles: &mut Collection<Muscle>,
// ) {
//     let head = Node::new(Point2::new(0., 0.), 5.);
//     let body = Node::new(Point2::new(0., 20.), 5.);
//     let subtail = Node::new(Point2::new(0., 40.), 5.);
//     let tail = Node::new(Point2::new(0., 60.), 5.);
//     nodes.push(head);
//     nodes.push(body);
//     nodes.push(subtail);
//     nodes.push(tail);

//     let head_body_bone = Bone::new(0, 1, 15.);
//     let body_tail_bone = Bone::new(1, 2, 15.);
//     let tail_subtail_bone = Bone::new(2, 3, 15.);
//     bones.push(head_body_bone);
//     bones.push(body_tail_bone);
//     bones.push(tail_subtail_bone);

//     let top_fn = |x: u64| (x as f32 / 8.).sin() * 1.5;
//     let bottom_fn = |x: u64| (x as f32 / 8. + PI / 3.).sin() * 0.5;
//     let top_muscle = Muscle::new(1, 0, 2, Angle(PI), top_fn, 0.25);
//     let bottom_muscle = Muscle::new(2, 1, 3, Angle(PI), bottom_fn, 0.25);

//     muscles.push(top_muscle);
//     muscles.push(bottom_muscle);
// }
