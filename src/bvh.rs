use crate::aabb::AABB;
use crate::pt_renderer::{Intersect, Ray};
use crate::triangle::RTTriangle;

pub struct BVHNode {
    aabb: AABB,
    pub start_i: usize,
    pub end_i: usize,
    left_child_i: Option<usize>,
    right_child_i: Option<usize>,
}

impl BVHNode {
    fn new(triangles: &[RTTriangle], start_i: usize, end_i: usize) -> BVHNode {
        let mut node = BVHNode {
            aabb: triangles[start_i].aabb(),
            start_i, end_i,
            left_child_i: None, right_child_i: None
        };
        for tri in triangles[(start_i + 1)..end_i].iter() {
            node.aabb.add_aabb(&tri.aabb());
        }
        node
    }

    fn size(&self) -> usize {
        self.end_i - self.start_i
    }

    pub fn is_leaf(&self) -> bool {
        self.left_child_i.is_none()
    }
}

impl Intersect<'a, f32> for BVHNode {
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        self.aabb.intersect(ray)
    }
}

const MAX_LEAF_SIZE: usize = 8;

pub struct BVH {
    nodes: Vec<BVHNode>,
}

impl BVH {
    pub fn empty() -> BVH {
        BVH { nodes: Vec::new() }
    }

    pub fn build_object_median(triangles: &mut Vec<RTTriangle>) -> BVH {
        let mut nodes = Vec::with_capacity(f32::log2(triangles.len() as f32) as usize);
        nodes.push(BVHNode::new(triangles, 0, triangles.len()));
        let mut split_stack = vec![0usize];

        while let Some(node_i) = split_stack.pop() {
            let start_i = nodes[node_i].start_i;
            let end_i = nodes[node_i].end_i;
            let axis_i = nodes[node_i].aabb.longest_edge_i();
            triangles[start_i..end_i]
                .sort_unstable_by(|ref tri1, ref tri2| {
                    let c1 = tri1.center()[axis_i];
                    let c2 = tri2.center()[axis_i];
                    c1.partial_cmp(&c2).unwrap()
                });
            let mid_i = (start_i + end_i) / 2;

            let left_child = BVHNode::new(triangles, start_i, mid_i);
            nodes[node_i].left_child_i = Some(nodes.len());
            if left_child.size() > MAX_LEAF_SIZE {
                split_stack.push(nodes.len());
            }
            nodes.push(left_child);

            let right_child = BVHNode::new(triangles, mid_i, end_i);
            nodes[node_i].right_child_i = Some(nodes.len());
            if right_child.size() > MAX_LEAF_SIZE {
                split_stack.push(nodes.len());
            }
            nodes.push(right_child);
        }
        nodes.shrink_to_fit();
        BVH { nodes }
    }

    pub fn get_children(&self, node: &BVHNode) -> Option<(&BVHNode, &BVHNode)> {
        if let Some(left_i) = node.left_child_i {
            Some((&self.nodes[left_i], &self.nodes[node.right_child_i.unwrap()]))
        } else {
            None
        }
    }

    pub fn root(&self) -> &BVHNode {
        &self.nodes[0]
    }
}
