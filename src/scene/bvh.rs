use cgmath::Point3;

use aabb::AABB;
use renderer::RTTriangle;

pub struct BVHNode {
    aabb: AABB,
    start_i: usize,
    end_i: usize,
    left_child_i: Option<usize>,
    right_child_i: Option<usize>,
}

impl BVHNode {
    fn new(indices: &[usize], aabbs: &[AABB], start_i: usize, end_i: usize) -> BVHNode {
        let mut node = BVHNode {
            aabb: AABB::default(),
            start_i, end_i,
            left_child_i: None, right_child_i: None
        };
        for i in indices[start_i..end_i].iter() {
            node.aabb.add_aabb(&aabbs[*i]);
        }
        node
    }

    fn size(&self) -> usize {
        self.end_i - self.start_i
    }
}

const MAX_LEAF_SIZE: usize = 6;

pub fn build_object_median(triangles: &mut Vec<RTTriangle>) -> Vec<BVHNode> {
    let mut indices: Vec<usize> = (0..triangles.len()).collect();
    let aabbs: Vec<AABB> = triangles.iter().map(|t| t.aabb()).collect();
    let centers: Vec<Point3<f32>> = triangles.into_iter().map(|t| t.center()).collect();
    let mut hierarcy = Vec::with_capacity(f32::log2(triangles.len() as f32) as usize);
    hierarcy.push(BVHNode::new(&indices, &aabbs, 0, triangles.len()));
    let mut split_stack = vec![0usize];

    while let Some(node_i) = split_stack.pop() {
        let start_i = hierarcy[node_i].start_i;
        let end_i = hierarcy[node_i].end_i;
        let axis_i = hierarcy[node_i].aabb.longest_edge_i();
        indices[start_i..end_i]
            .sort_unstable_by(|i1, i2| {
                let c1 = centers[*i1][axis_i];
                let c2 = centers[*i2][axis_i];
                c1.partial_cmp(&c2).unwrap()
            });
        let mid_i = (start_i + end_i) / 2;

        let left_child = BVHNode::new(&indices, &aabbs, start_i, mid_i);
        hierarcy[node_i].left_child_i = Some(hierarcy.len());
        if left_child.size() > MAX_LEAF_SIZE {
            split_stack.push(hierarcy.len());
        }
        hierarcy.push(left_child);

        let right_child = BVHNode::new(&indices, &aabbs, mid_i, end_i);
        hierarcy[node_i].right_child_i = Some(hierarcy.len());
        if right_child.size() > MAX_LEAF_SIZE {
            split_stack.push(hierarcy.len());
        }
        hierarcy.push(right_child);
    }

    // Make triangles ordering match indices ordering
    // TODO: This could be done better
    *triangles = indices.iter().map(|&i| triangles[i].clone()).collect();
    hierarcy.shrink_to_fit();
    hierarcy
}
