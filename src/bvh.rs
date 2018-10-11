use std::ops::Index;

use cgmath::Point3;

use crate::aabb::AABB;
use crate::pt_renderer::{Intersect, Ray};
use crate::stats;
use crate::triangle::RTTriangle;

const MAX_LEAF_SIZE: usize = 8;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum SplitMode {
    Object,
    Spatial,
    SAH,
}

pub struct BVHNode {
    aabb: AABB,
    pub start_i: usize,
    pub end_i: usize,
    left_child_i: Option<usize>,
    right_child_i: Option<usize>,
}

impl BVHNode {
    fn new(triangles: &Triangles) -> BVHNode {
        let start_i = triangles.start_i;
        let end_i = start_i + triangles.len();
        BVHNode {
            aabb: triangles.aabb.clone(),
            start_i,
            end_i,
            left_child_i: None,
            right_child_i: None,
        }
    }

    fn n_tris(&self) -> usize {
        self.end_i - self.start_i
    }

    pub fn is_leaf(&self) -> bool {
        self.left_child_i.is_none()
    }
}

impl Intersect<'_, f32> for BVHNode {
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        self.aabb.intersect(ray)
    }
}

struct Triangles<'a> {
    triangles: &'a [RTTriangle],
    centers: &'a [Point3<f32>],
    indices: &'a mut [usize],
    aabb: AABB,
    /// Node contains indices [start_i, start_i + len) from the main indices array
    start_i: usize,
    /// Axis along which the indices have been sorted
    sorted_axis: usize,
}

impl<'a> Triangles<'a> {
    fn new(
        triangles: &'a [RTTriangle],
        centers: &'a [Point3<f32>],
        indices: &'a mut [usize],
        start_i: usize,
    ) -> Triangles<'a> {
        let mut aabb = AABB::empty();
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

    fn last(&self) -> &RTTriangle {
        let &i = self.indices.last().unwrap();
        &self.triangles[i]
    }
}

impl Index<usize> for Triangles<'_> {
    type Output = RTTriangle;

    fn index(&self, i: usize) -> &RTTriangle {
        let i = self.indices[i];
        &self.triangles[i]
    }
}

pub struct BVH {
    nodes: Vec<BVHNode>,
}

impl BVH {
    pub fn empty() -> BVH {
        BVH { nodes: Vec::new() }
    }

    pub fn build(triangles: &[RTTriangle], split_mode: SplitMode) -> (BVH, Vec<usize>) {
        stats::start_bvh();
        let centers: Vec<Point3<f32>> = triangles.iter().map(|ref tri| tri.center()).collect();
        let mut permutation: Vec<usize> = (0..triangles.len()).collect();
        let tris = Triangles::new(triangles, &centers, &mut permutation, 0);
        let mut nodes = Vec::with_capacity(f32::log2(triangles.len() as f32) as usize);
        nodes.push(BVHNode::new(&tris));
        let mut split_stack = vec![(0usize, tris)];

        while let Some((node_i, mut tris)) = split_stack.pop() {
            let mid_offset = match split_mode {
                SplitMode::Object => object_split(&mut tris),
                SplitMode::Spatial => spatial_split(&mut tris),
                SplitMode::SAH => sah_split(&mut tris),
            };
            let (t1, t2) = if let Some(offset) = mid_offset {
                tris.split(offset)
            } else {
                continue;
            };

            let left_child = BVHNode::new(&t1);
            nodes[node_i].left_child_i = Some(nodes.len());
            if left_child.n_tris() > MAX_LEAF_SIZE {
                split_stack.push((nodes.len(), t1));
            }
            nodes.push(left_child);

            let right_child = BVHNode::new(&t2);
            nodes[node_i].right_child_i = Some(nodes.len());
            if right_child.n_tris() > MAX_LEAF_SIZE {
                split_stack.push((nodes.len(), t2));
            }
            nodes.push(right_child);
        }
        nodes.shrink_to_fit();
        let bvh = BVH { nodes };
        stats::stop_bvh(&bvh);
        (bvh, permutation)
    }

    pub fn get_children(&self, node: &BVHNode) -> Option<(&BVHNode, &BVHNode)> {
        if let Some(left_i) = node.left_child_i {
            Some((
                &self.nodes[left_i],
                &self.nodes[node.right_child_i.unwrap()],
            ))
        } else {
            None
        }
    }

    pub fn root(&self) -> &BVHNode {
        &self.nodes[0]
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    pub fn n_tris(&self) -> usize {
        self.root().n_tris()
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
    let mut min_score = std::f32::MAX;
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
        let mut left_bb = AABB::empty();
        // Go through the possible splits
        for i in 0..triangles.len() {
            left_bb.add_aabb(&triangles[i].aabb());
            let right_bb = &right_bbs[right_bbs.len() - 1 - i];
            let score = i as f32 * left_bb.area() + (triangles.len() - i) as f32 * right_bb.area();
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
