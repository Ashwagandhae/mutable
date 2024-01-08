use super::world::tag::Tag;

use rand::Rng;

#[derive(Debug, Clone)]
pub struct Cluster<T> {
    pub center: Tag,
    pub points: Vec<(Tag, T)>,
}

pub fn cluster<T>(data: &[(Tag, T)]) -> Vec<Cluster<T>>
where
    T: Clone,
{
    k_means(data, 16, 100)
}

fn k_means<T>(data: &[(Tag, T)], k: usize, max_iterations: usize) -> Vec<Cluster<T>>
where
    T: Clone,
{
    let mut rng = rand::thread_rng();

    // Step 1: Initialize clusters using k-means++
    let mut clusters: Vec<Cluster<T>> = Vec::new();
    clusters.push(Cluster {
        center: data[rng.gen_range(0..data.len())].0.clone(),
        points: Vec::new(),
    });

    while clusters.len() < k {
        // For each data point, calculate the distance to the nearest existing cluster
        let distances: Vec<f32> = data
            .iter()
            .map(|(point, _)| {
                clusters
                    .iter()
                    .map(|cluster| point.distance(&cluster.center))
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap()
            })
            .collect();

        // Choose the next cluster center with probability proportional to the square of the distance
        let sum_distances: f32 = distances.iter().sum();
        let probability: Vec<f32> = distances
            .iter()
            .map(|d| d.powi(2) / sum_distances)
            .collect();

        let chosen_index = (0..data.len())
            .max_by(|&i, &j| probability[i].partial_cmp(&probability[j]).unwrap())
            .unwrap();

        clusters.push(Cluster {
            center: data[chosen_index].0.clone(),
            points: Vec::new(),
        });
    }

    // Step 2: Assign data points to clusters and update cluster centers
    for _iteration in 0..max_iterations {
        for cluster in &mut clusters {
            cluster.points.clear();
        }
        // Assign each data point to the nearest cluster
        for (point, owner) in data {
            let nearest_cluster_index = clusters
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.center
                        .distance(point)
                        .partial_cmp(&b.center.distance(point))
                        .unwrap()
                })
                .map(|(index, _)| index)
                .unwrap();

            clusters[nearest_cluster_index]
                .points
                .push((point.clone(), owner.clone()));
        }

        // Update cluster centers
        for cluster in &mut clusters {
            if !cluster.points.is_empty() {
                let sum: Tag = cluster
                    .points
                    .iter()
                    .fold(Tag::zero(), |sum, (point, _)| sum + point.clone());
                let new_center = sum / cluster.points.len() as f32;
                cluster.center = new_center;
            }
        }
    }

    clusters
}
