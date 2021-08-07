use std::ops::{Index, Range};

use cgmath::Point3;

use crate::aabb::Aabb;
use crate::consts;
use crate::float::*;
use crate::intersect::{Intersect, Ray};
use crate::stats;
use crate::triangle::Triangle;

const MAX_LEAF_SIZE: usize = 8;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum SplitMode {
    Object,
    Spatial,
    Sah,
}

enum Indices {
    Inner(u32, u32),
    Leaf(u32, u32),
}

#[repr(align(64))]
pub struct BvhNode {
    aabb: Aabb,
    indices: Indices,
}

impl BvhNode {
    fn new(triangles: &Triangles) -> BvhNode {
        let start_i = triangles.start_i as u32;
        let end_i = start_i + triangles.len() as u32;
        BvhNode {
            aabb: triangles.aabb.clone(),
            indices: Indices::Leaf(start_i, end_i),
        }
    }

    fn convert_to_inner(&mut self, left_child: usize, right_child: usize) {
        self.indices = Indices::Inner(left_child as u32, right_child as u32);
    }

    pub fn range(&self) -> Option<Range<usize>> {
        match self.indices {
            Indices::Leaf(start_i, end_i) => Some(start_i as usize..end_i as usize),
            Indices::Inner(_, _) => None,
        }
    }
}

impl Intersect<'_, Float> for BvhNode {
    fn intersect(&self, ray: &Ray) -> Option<Float> {
        self.aabb.intersect(ray)
    }
}

struct Triangles<'a> {
    triangles: &'a [Triangle],
    centers: &'a [Point3<Float>],
    indices: &'a mut [usize],
    aabb: Aabb,
    /// Node contains indices [start_i, start_i + len) from the main indices array
    start_i: usize,
    /// Axis along which the indices have been sorted
    sorted_axis: usize,
}

impl<'a> Triangles<'a> {
    fn new(
        triangles: &'a [Triangle],
        centers: &'a [Point3<Float>],
        indices: &'a mut [usize],
        start_i: usize,
    ) -> Triangles<'a> {
        let mut aabb = Aabb::empty();
        for &i in indices.iter() {
            let tri = &triangles[i];
            aabb.add_aabb(&tri.aabb());
        }
        Triangles {
            triangles,
            centers,
            indices,
            aabb,
            start_i,
            // Use bogus value since nothing has been sorted
            sorted_axis: 42,
        }
    }

    fn sort_longest_axis(&mut self) {
        let axis_i = self.aabb.longest_edge_i();
        self.sort(axis_i);
    }

    fn sort(&mut self, axis_i: usize) {
        // The indices are already sorted along the requested axis
        if axis_i == self.sorted_axis {
            return;
        }
        let centers = self.centers;
        self.indices.sort_unstable_by(|&i1, &i2| {
            let c1 = centers[i1][axis_i];
            let c2 = centers[i2][axis_i];
            c1.partial_cmp(&c2).unwrap()
        });
        self.sorted_axis = axis_i;
    }

    fn split(self, i: usize) -> (Triangles<'a>, Triangles<'a>) {
        let (i1, i2) = self.indices.split_at_mut(i);
        let mut node1 = Triangles::new(self.triangles, self.centers, i1, self.start_i);
        let mut node2 = Triangles::new(self.triangles, self.centers, i2, self.start_i + i);
        node1.sorted_axis = self.sorted_axis;
        node2.sorted_axis = self.sorted_axis;
        (node1, node2)
    }

    fn len(&self) -> usize {
        self.indices.len()
    }

    fn last(&self) -> &Triangle {
        let &i = self.indices.last().unwrap();
        &self.triangles[i]
    }
}

impl Index<usize> for Triangles<'_> {
    type Output = Triangle;

    fn index(&self, i: usize) -> &Triangle {
        let i = self.indices[i];
        &self.triangles[i]
    }
}

pub struct Bvh {
    nodes: Vec<BvhNode>,
}

impl Bvh {
    pub fn build(triangles: &[Triangle], split_mode: SplitMode) -> (Bvh, Vec<usize>) {
        assert!(
            !triangles.is_empty(),
            "Scene doesn't contain any triangles!"
        );
        assert!(
            triangles.len() <= 2usize.pow(32),
            "Scene can contain maximum of 2^32 triangles! This scene has {} triangles.",
            triangles.len()
        );
        stats::start_bvh();
        let centers: Vec<Point3<Float>> = triangles.iter().map(|tri| tri.center()).collect();
        let mut permutation: Vec<usize> = (0..triangles.len()).collect();
        let tris = Triangles::new(triangles, &centers, &mut permutation, 0);
        let mut nodes = Vec::with_capacity(Float::log2(triangles.len().to_float()) as usize);
        nodes.push(BvhNode::new(&tris));
        let mut split_stack = vec![(0usize, tris)];

        while let Some((node_i, mut tris)) = split_stack.pop() {
            let mid_offset = match split_mode {
                SplitMode::Object => object_split(&mut tris),
                SplitMode::Spatial => spatial_split(&mut tris),
                SplitMode::Sah => sah_split(&mut tris),
            };
            let (t1, t2) = if let Some(offset) = mid_offset {
                tris.split(offset)
            } else {
                continue;
            };

            let left_child = BvhNode::new(&t1);
            let left_child_i = nodes.len();
            if t1.len() > MAX_LEAF_SIZE {
                split_stack.push((nodes.len(), t1));
            }
            nodes.push(left_child);

            let right_child = BvhNode::new(&t2);
            let right_child_i = nodes.len();
            if t2.len() > MAX_LEAF_SIZE {
                split_stack.push((nodes.len(), t2));
            }
            nodes.push(right_child);
            nodes[node_i].convert_to_inner(left_child_i, right_child_i);
        }
        nodes.shrink_to_fit();
        let bvh = Bvh { nodes };
        stats::stop_bvh(&bvh, triangles.len());
        (bvh, permutation)
    }

    pub fn get_children(&self, node: &BvhNode) -> Option<(&BvhNode, &BvhNode)> {
        match node.indices {
            Indices::Leaf(_, _) => None,
            Indices::Inner(left_i, right_i) => {
                Some((&self.nodes[left_i as usize], &self.nodes[right_i as usize]))
            }
        }
    }

    pub fn root(&self) -> &BvhNode {
        &self.nodes[0]
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }
}

fn object_split(triangles: &mut Triangles) -> Option<usize> {
    triangles.sort_longest_axis();
    Some(triangles.len() / 2)
}

fn spatial_split(triangles: &mut Triangles) -> Option<usize> {
    let aabb = &triangles.aabb;
    let axis_i = aabb.longest_edge_i();
    let mid_val = aabb.center()[axis_i];
    triangles.sort(axis_i);
    let centers = triangles.centers;
    let i = triangles
        .indices
        .binary_search_by(|&i| {
            let c = centers[i][axis_i];
            c.partial_cmp(&mid_val).unwrap()
        })
        .unwrap_or_else(|e| e);
    // Use object median if all centers are on one side of the median
    if i == 0 || i == triangles.len() {
        object_split(triangles)
    } else {
        Some(i)
    }
}

fn sah_split(triangles: &mut Triangles) -> Option<usize> {
    let mut min_score = consts::MAX;
    let mut min_axis = 0;
    let mut min_i = 0;
    let sorted_axis = triangles.sorted_axis;
    for offset in 0..3 {
        // Check the sorted axis first
        let axis = (sorted_axis + offset) % 3;
        triangles.sort(axis);
        // Precompute all right side bbs
        let mut right_bbs = Vec::with_capacity(triangles.len());
        right_bbs.push(triangles.last().aabb());
        for i in 1..triangles.len() {
            let mut new_bb = right_bbs[i - 1].clone();
            new_bb.add_aabb(&triangles[triangles.len() - 1 - i].aabb());
            right_bbs.push(new_bb);
        }
        let mut left_bb = Aabb::empty();
        // Go through the possible splits
        for i in 0..triangles.len() {
            left_bb.add_aabb(&triangles[i].aabb());
            let right_bb = &right_bbs[right_bbs.len() - 1 - i];
            let n_left = i.to_float();
            let n_right = (triangles.len() - i).to_float();
            let score = n_left * left_bb.area() + n_right * right_bb.area();
            if score < min_score {
                min_score = score;
                min_axis = axis;
                min_i = i;
            }
        }
    }
    if min_i == 0 {
        None
    } else {
        triangles.sort(min_axis);
        Some(min_i)
    }
}
