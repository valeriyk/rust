extern crate rayon;

use rayon::prelude::*;

use geometry::aabb::Aabb;
use geometry::ray::Ray3d;
use geometry::triangle::Triangle;
use geometry::{Point3d, TraceablePrimitive, Vector3d};
use morton_encoding::morton_encode;
use std::cmp::Ordering;
use std::fmt::Formatter;
use std::marker;
use std::marker::PhantomData;
use std::ops::{AddAssign, Deref};

const OCTREE_MAX_NUM_CHILDREN: usize = 8;


#[derive(Debug)]
struct OctreeItem {
    idx: usize,
    key: u64,
}

impl std::cmp::PartialEq for OctreeItem {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.key == other.key
    }

    fn ne(&self, other: &Self) -> bool {
        self.idx != other.idx || self.key != other.key
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct OctreeLeafNode<const N: usize> {
    bb: Aabb,
    items_idx: [Option<usize>; N],
}
#[derive(Copy, Clone, PartialEq, Debug)]
struct OctreeInnerNode {
    bb: Aabb,
    children_idx: [Option<usize>; OCTREE_MAX_NUM_CHILDREN],
}
#[derive(Copy, Clone, PartialEq, Debug)]
enum OctreeNode<const N: usize> {
    Leaf(OctreeLeafNode<N>),
    Inner(OctreeInnerNode),
}
impl<const N: usize> OctreeNode<N> {
    fn get_bb(&self) -> Aabb {
        match self {
            OctreeNode::Inner(n) => n.bb,
            OctreeNode::Leaf(n) => n.bb,
        }
    }
    fn set_bb(&mut self, bb: Aabb) {
        match self {
            OctreeNode::Inner(n) => n.bb = bb,
            OctreeNode::Leaf(n) => n.bb = bb,
        }
    }

    fn set_child(&mut self, child_idx: usize, val: usize) {
        match self {
            OctreeNode::Inner(n) => n.children_idx[child_idx] = Some(val),
            OctreeNode::Leaf(n) => panic!("Leaf node shall not have children!"),
        }
    }
}
impl<const N: usize> OctreeLeafNode<N> {
    fn new() -> OctreeLeafNode<N> {
        OctreeLeafNode {
            bb: Aabb::new(),
            items_idx: [None; N],
        }
    }
}
impl OctreeInnerNode {
    fn new() -> OctreeInnerNode {
        OctreeInnerNode {
            bb: Aabb::new(),
            children_idx: [None; OCTREE_MAX_NUM_CHILDREN],
        }
    }
}

pub struct Octree<'a, P, const N: usize> {
    nodes: Vec<OctreeNode<N>>,
    max_LEAF_CAPACTITY: usize,
    //_primitive: marker::PhantomData<P>,
    primitives: &'a [P],
}

impl<'a, P, const N: usize> Octree<'a, P, N>
where
    P: TraceablePrimitive + Copy,
{
    fn linearize_primitives(primitives: &'a [P]) -> Vec<OctreeItem> {
        let mut top_bb = primitives
            .iter()
            .fold(Aabb::new(), |acc, x| acc + x.get_bounding_box());
        let min: Point3d = top_bb.get_min();
        let max: Point3d = top_bb.get_max();
        let range = max - min;

        primitives
            .iter()
            .enumerate()
            .map(|(idx, prim)| {
                let positive = prim.get_centroid() - min;
                let x: u16 = (positive.x * u16::MAX as f32 / range.x) as u16;
                let y: u16 = (positive.y * u16::MAX as f32 / range.y) as u16;
                let z: u16 = (positive.z * u16::MAX as f32 / range.z) as u16;
                let key: u64 = morton_encode([x, y, z]);
                OctreeItem { idx, key }
            })
            .collect()
    }

    fn sort_primitives(primitives: &'a [P]) -> Vec<OctreeItem> {
        let mut indexed_keys = Octree::<'a, P, N>::linearize_primitives(primitives);
        indexed_keys.par_sort_by_key(|p| p.key);
        indexed_keys
    }

    pub fn new(primitives: &'a [P]) -> Octree<'a, P, N> {
        let min_num_nodes = Octree::<'a, P, N>::get_min_num_nodes(primitives.len(), N);

        let mut octree = Octree::<'a, P, N> {
            nodes: Vec::with_capacity(min_num_nodes),
            max_LEAF_CAPACTITY: N,
            primitives,
        };
        let mut indexed_keys = Octree::<'a, P, N>::sort_primitives(primitives);
        octree.build(primitives, &mut indexed_keys);
        octree
    }

    fn get_min_num_nodes(num_elems: usize, max_node_capacity: usize) -> usize {
        if max_node_capacity == 8 {
            let depth = (num_elems as f32).log2().ceil() / (max_node_capacity as f32).log2().ceil();
            ((8.0_f32.powf(depth + 1.0) - 1.0) / 7.0) as usize
        } else {
            num_elems.pow(2)
        }
    }

    fn split(elems: &mut [OctreeItem]) -> (&mut [OctreeItem], &mut [OctreeItem]) {
        if elems.len() < 1 {
            return (&mut [], &mut []);
        }
        let first = elems[0].key;
        let last = elems[elems.len() - 1].key;
        let leftmost_different_bit_mask = 0x80000000_00000000_u64 >> (first ^ last).leading_zeros();

        let split_at = elems
            .iter()
            .position(|x| (x.key & leftmost_different_bit_mask) != 0); // TODO binary search needed

        let (below, above) = match split_at {
            None => elems.split_at_mut((elems.len() / 2) as usize),
            Some(i) => elems.split_at_mut(i),
        };
        
        (below, above)
    }

    fn build(&mut self, primitives: &[P], elems: &mut [OctreeItem]) -> Option<usize> {
        let len = elems.len();

        return if len < 1 {
            None
        } else if len <= N {
            let leaf_idx = self.add_leaf(primitives, elems);
            Some(leaf_idx)
        } else {
            let inner_idx = self.add_inner();
            let mut inner_bb = Aabb::new();

            // We have more elements than can fit into a leaf node; split the slice into two sub-slices
            let (left, right) = Octree::<P, N>::split(elems);

            // We actually have more elements than can fit into two leaf nodes, split the slices again so that
            // we have four sub-slices
            if len > N * 2 {
                let (left_bot, left_top) = Octree::<P, N>::split(left);
                let (right_bot, right_top) = Octree::<P, N>::split(right);

                // We actually have more elements than can fit into four leaf nodes, split the slices again so that
                // we have eight sub-slices. We don't split them further because we have at most eight children for each
                // inner node
                if len > N * 4 {
                    let (left_bot_near, left_bot_far) = Octree::<P, N>::split(left_bot);
                    let (left_top_near, left_top_far) = Octree::<P, N>::split(left_top);
                    let (right_bot_near, right_bot_far) = Octree::<P, N>::split(right_bot);
                    let (right_top_near, right_top_far) = Octree::<P, N>::split(right_top);
                    let mut children_primitives: [&mut [OctreeItem]; 8] = [
                        left_bot_near,
                        left_bot_far,
                        left_top_near,
                        left_top_far,
                        right_bot_near,
                        right_bot_far,
                        right_top_near,
                        right_top_far,
                    ];

                    // Continue recursively until all the slices are split into chunks of eight elements or less so that
                    // they become stored in leaf nodes.
                    for i in 0..8 {
                        let child_idx = self.build(primitives, children_primitives[i]);
                        if child_idx != None {
                            let child_idx = child_idx.unwrap();
                            self.nodes[inner_idx].set_child(i, child_idx); // TODO fix naming
                            inner_bb += self.nodes[child_idx].get_bb();
                        }
                    }
                } else {
                    // The elements we have can be stored into four leaf nodes or less
                    let mut children_primitives: [&mut [OctreeItem]; 4] =
                        [left_bot, left_top, right_bot, right_top];
                    for i in 0..4 {
                        let child_idx = self.build(primitives, children_primitives[i]);
                        if child_idx != None {
                            let child_idx = child_idx.unwrap();
                            self.nodes[inner_idx].set_child(i, child_idx); // TODO fix naming
                            inner_bb += self.nodes[child_idx].get_bb();
                        }
                    }
                }
            } else {
                // The elements we have can be stored into two leaf nodes or less
                let mut children_primitives: [&mut [OctreeItem]; 2] = [left, right];
                for i in 0..2 {
                    let child_idx = self.build(primitives, children_primitives[i]);
                    if child_idx != None {
                        let child_idx = child_idx.unwrap();
                        self.nodes[inner_idx].set_child(i, child_idx); // TODO fix naming
                        inner_bb += self.nodes[child_idx].get_bb();
                    }
                }
            }

            self.nodes[inner_idx].set_bb(inner_bb);

            Some(inner_idx)
        };
    }

    // returns the index of the created leaf node
    fn add_leaf(&mut self, primitives: &[P], elems: &[OctreeItem]) -> usize {
        let mut leaf = OctreeLeafNode::<N>::new();
        elems.iter().enumerate().for_each(|(idx, item)| {
            leaf.items_idx[idx] = Some(item.idx);
            leaf.bb += primitives[item.idx].get_bounding_box();
        });
        self.nodes.push(OctreeNode::Leaf(leaf));
        self.nodes.len() - 1
    }

    fn add_inner(&mut self) -> usize {
        let mut inner = OctreeInnerNode::new();
        self.nodes.push(OctreeNode::Inner(inner));
        self.nodes.len() - 1
    }

    fn get_nearest_from_leaf(&self, leaf: &OctreeLeafNode<N>, ray: &Ray3d) -> Option<(usize, f32)> {
        let mut nearest: Option<(usize, f32)> = None;
        for i in 0..N {
            match leaf.items_idx[i] {
                None => break,
                Some(item) => {
                    let current = self.primitives[item].get_distance_to(ray);

                    if current == None {
                        continue;
                    }
                    let current = current.unwrap();

                    if nearest == None {
                        nearest = Some((item, current))
                    } else if current < nearest.unwrap().1 {
                        nearest = Some((item, current))
                    }
                }
            }
        }
        nearest
    }

    pub fn traverse(&self, ray: &Ray3d) -> Option<(usize, f32)> {
        let mut nearest_overall: Option<(usize, f32)> = None;

        let mut node_stack: Vec<usize> = Vec::new();
        node_stack.push(0);
        while node_stack.len() > 0 {
            let node_idx = node_stack.pop();
            if node_idx == None {
                break;
            }
            let current_node = &self.nodes[node_idx.unwrap()];
            //let distance_to_bb = current_node.get_bb().get_distance_to(ray_origin, ray_dir);

            match current_node {
                OctreeNode::Leaf(leaf) => {
                    let nearest_in_this_leaf = self.get_nearest_from_leaf(leaf, ray);
                    if nearest_in_this_leaf != None {
                        if nearest_overall != None {
                            // safe to unwrap here
                            if nearest_in_this_leaf.unwrap().1 < nearest_overall.unwrap().1 {
                                nearest_overall = nearest_in_this_leaf;
                            }
                        } else {
                            nearest_overall = nearest_in_this_leaf;
                        }
                    }
                }
                OctreeNode::Inner(inner) => {
                    // we're not in a leaf:
                    // 1. get distance to all the children's bounding boxes
                    // 2. ideally we could sort them by the distance (nearest goes first), but this
                    //    made performance worse
                    // 3. if a distance to any primitive is not known yet:
                    //    - push all children to the queue
                    //    - else push those children whose distance is smaller than already known distance
                    inner
                        .children_idx
                        .iter()
                        .filter_map(|&x| x)
                        .filter_map(|child_idx| {
                            let child_node = self.nodes[child_idx];
                            let dist: Option<f32> = child_node.get_bb().get_distance_to(ray);
                            match dist {
                                Some(d) => Some ((child_idx, dist.unwrap())),
                                None => None,
                            }
                        })
                        .filter(|(idx, dist)| {
                            (nearest_overall == None)
                                || (nearest_overall != None
                                    && (*dist < nearest_overall.unwrap().1))
                        })
                        .for_each(|(idx, _)| node_stack.push(idx));
                }
            }
        }
        nearest_overall
    }
}

impl<'a, P, const N: usize> std::fmt::Display for Octree<'a, P, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut node_stack: Vec<(usize, usize)> = Vec::new();
        node_stack.push((0, 0));

        while node_stack.len() > 0 {
            let (node_idx, depth) = node_stack.pop().unwrap();

            if depth == 0 {
                write!(f, "<{}>:", node_idx);
            } else if depth == 1 {
                write!(f, "|-<{}>:", node_idx);
            } else {
                for i in 0..depth - 1 {
                    write!(f, "|  ");
                }
                write!(f, "|-<{}>:", node_idx);
            }

            match &self.nodes[node_idx] {
                OctreeNode::Inner(inner) => {
                    for i in inner.children_idx {
                        if i != None {
                            node_stack.push((i.unwrap(), depth + 1));
                        }
                    }
                    writeln!(f, " {}", inner.bb);
                }
                OctreeNode::Leaf(leaf) => {
                    write!(f, " {}, items:[", leaf.bb);
                    for i in leaf.items_idx {
                        match i {
                            None => write!(f, "None,"),
                            Some(item) => write!(f, "{},", item),
                        };
                    }
                    writeln!(f, "]");
                }
            }
        }
        write!(f, "Done!")
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use geometry::TraceablePrimitive;

    const golden_ref: [Point3d; 9] = [
        Point3d {x: 0.0, y: 0.0, z: 0.0},
        Point3d {x: 1.0, y: 1.0, z: 1.0},
        Point3d {x: 1.0, y: 1.0, z: -1.0},
        Point3d {x: 1.0, y: -1.0, z: 1.0},
        Point3d {x: 1.0, y: -1.0, z: -1.0},
        Point3d {x: -1.0, y: 1.0, z: 1.0},
        Point3d {x: -1.0, y: 1.0, z: -1.0},
        Point3d {x: -1.0, y: -1.0, z: 1.0},
        Point3d {x: -1.0, y: -1.0, z: -1.0},
    ];

    #[test]
    fn t_octree_build_leaf_cap_1() {
        const LEAF_CAPACITY: usize = 1;
        let octree = Octree::<Point3d, LEAF_CAPACITY>::new(&golden_ref);

        //
        println!("{}", octree);
    }

    // #[test]
    // fn t_octree_build_leaf_cap_2() {
    //     const LEAF_CAPACITY: usize = 2;
    //     let octree = Octree::<Point3d, LEAF_CAPACITY>::new(&golden_ref);
    //
    //     let test_vec = vec![
    //         OctreeNode::Inner(OctreeInnerNode{ bb: Aabb::from_arrays([-1.0, -1.0, -1.0],[1.0, 1.0, 1.0]), children_idx: [Some(1), Some(6)] }), // 0
    //         OctreeNode::Inner(OctreeInnerNode{ bb: Aabb::from_arrays([-1.0, -1.0, -1.0],[0.0, 1.0, 1.0]), children_idx: [Some(2), Some(5)] }), // 1
    //         OctreeNode::Inner(OctreeInnerNode{ bb: Aabb::from_arrays([-1.0, -1.0, -1.0],[0.0, 0.0, 1.0]), children_idx: [Some(3), Some(4)] }), // 2
    //         OctreeNode::Leaf(OctreeLeafNode{ bb: Aabb::from_arrays([-1.0, -1.0, -1.0],[0.0, 0.0, 0.0]), items_idx: [Some(8), Some(0)] }), // 3
    //         OctreeNode::Leaf(OctreeLeafNode{ bb: Aabb::from_arrays([-1.0, -1.0, 1.0],[-1.0, -1.0, 1.0]), items_idx: [Some(7), None] }), // 4
    //         OctreeNode::Leaf(OctreeLeafNode{ bb: Aabb::from_arrays([-1.0, 1.0, -1.0],[-1.0, 1.0, 1.0]), items_idx: [Some(6), Some(5)] }), // 5
    //         OctreeNode::Inner(OctreeInnerNode{ bb: Aabb::from_arrays([1.0, -1.0, -1.0],[1.0, 1.0, 1.0]), children_idx: [Some(7), Some(8)] }), // 6
    //         OctreeNode::Leaf(OctreeLeafNode{ bb: Aabb::from_arrays([1.0, -1.0, -1.0],[1.0, -1.0, 1.0]), items_idx: [Some(4), Some(3)] }), // 7
    //         OctreeNode::Leaf(OctreeLeafNode{ bb: Aabb::from_arrays([1.0, 1.0, -1.0],[1.0, 1.0, 1.0]), items_idx: [Some(2), Some(1)] }), // 8
    //     ];
    //     for i in 0..test_vec.len() {
    //         assert_eq!(test_vec[i], octree.nodes[i]);
    //     }
    // }
    //
    // #[test]
    // fn t_octree_build_leaf_cap_3() {
    //     const LEAF_CAPACITY: usize = 3;
    //     let octree = Octree::<Point3d, LEAF_CAPACITY>::new(&golden_ref);
    //
    //     let test_vec = vec![
    //         OctreeNode::Inner(OctreeInnerNode{ bb: Aabb::from_arrays([-1.0, -1.0, -1.0],[1.0, 1.0, 1.0]), children_idx: [Some(1), Some(4)] }), // 0
    //         OctreeNode::Inner(OctreeInnerNode{ bb: Aabb::from_arrays([-1.0, -1.0, -1.0],[0.0, 1.0, 1.0]), children_idx: [Some(2), Some(3)] }), // 1
    //         OctreeNode::Leaf(OctreeLeafNode{ bb: Aabb::from_arrays([-1.0, -1.0, -1.0],[0.0, 0.0, 1.0]), items_idx: [Some(8), Some(0), Some(7)] }), // 2
    //         OctreeNode::Leaf(OctreeLeafNode{ bb: Aabb::from_arrays([-1.0, 1.0, -1.0],[-1.0, 1.0, 1.0]), items_idx: [Some(6), Some(5), None] }), // 3
    //         OctreeNode::Inner(OctreeInnerNode{ bb: Aabb::from_arrays([1.0, -1.0, -1.0],[1.0, 1.0, 1.0]), children_idx: [Some(5), Some(6)] }), // 4
    //         OctreeNode::Leaf(OctreeLeafNode{ bb: Aabb::from_arrays([1.0, -1.0, -1.0],[1.0, -1.0, 1.0]), items_idx: [Some(4), Some(3), None] }), // 5
    //         OctreeNode::Leaf(OctreeLeafNode{ bb: Aabb::from_arrays([1.0, 1.0, -1.0],[1.0, 1.0, 1.0]), items_idx: [Some(2), Some(1), None] }), // 6
    //     ];
    //     for i in 0..test_vec.len() {
    //         assert_eq!(test_vec[i], octree.nodes[i]);
    //     }
    // }
    //
    // #[test]
    // fn t_octree_build_leaf_cap_8() {
    //     const LEAF_CAPACITY: usize = 8;
    //     let octree = Octree::<Point3d, LEAF_CAPACITY>::new(&golden_ref);
    //
    //     let test_vec = vec![
    //         OctreeNode::Inner(OctreeInnerNode{ bb: Aabb::from_arrays([-1.0, -1.0, -1.0],[1.0, 1.0, 1.0]), children_idx: [Some(1), Some(2)] }), // 0
    //         OctreeNode::Leaf(OctreeLeafNode{
    //             bb: Aabb::from_arrays([-1.0, -1.0, -1.0],[0.0, 1.0, 1.0]),
    //             items_idx: [Some(8), Some(0), Some(7), Some(6), Some(5), None, None, None]
    //         }),
    //         OctreeNode::Leaf(OctreeLeafNode{
    //             bb: Aabb::from_arrays([1.0, -1.0, -1.0],[1.0, 1.0, 1.0]),
    //             items_idx: [Some(4), Some(3), Some(2), Some(1), None, None, None, None]
    //         }),
    //     ];
    //     for i in 0..test_vec.len() {
    //         assert_eq!(test_vec[i], octree.nodes[i]);
    //     }
    // }
    //
    // #[test]
    // fn t_octree_build_leaf_cap_9() {
    //     const LEAF_CAPACITY: usize = 9;
    //     let octree = Octree::<Point3d, LEAF_CAPACITY>::new(&golden_ref);
    //
    //     let test_vec = vec![
    //         OctreeNode::Leaf(OctreeLeafNode{
    //             bb: Aabb::from_arrays([-1.0, -1.0, -1.0],[1.0, 1.0, 1.0]),
    //             items_idx: [
    //                 Some(8),
    //                 Some(0),
    //                 Some(7),
    //                 Some(6),
    //                 Some(5),
    //                 Some(4),
    //                 Some(3),
    //                 Some(2),
    //                 Some(1),
    //             ]
    //         }),
    //     ];
    //     for i in 0..test_vec.len() {
    //         assert_eq!(test_vec[i], octree.nodes[i]);
    //     }
    // }
    //
    // #[test]
    // fn t_octree_build_leaf_cap_10() {
    //     const LEAF_CAPACITY: usize = 10;
    //     let octree = Octree::<Point3d, LEAF_CAPACITY>::new(&golden_ref);
    //
    //     let test_vec = vec![
    //         OctreeNode::Leaf(OctreeLeafNode{
    //             bb: Aabb::from_arrays([-1.0, -1.0, -1.0],[1.0, 1.0, 1.0]),
    //             items_idx: [
    //                 Some(8),
    //                 Some(0),
    //                 Some(7),
    //                 Some(6),
    //                 Some(5),
    //                 Some(4),
    //                 Some(3),
    //                 Some(2),
    //                 Some(1),
    //                 None,
    //             ]
    //         }),
    //     ];
    //     for i in 0..test_vec.len() {
    //         assert_eq!(test_vec[i], octree.nodes[i]);
    //     }
    //     println!("{}", octree);
    // }
    //
    // #[test]
    // fn t_linearize_primitives() {
    //     const LEAF_CAPACITY: usize = 1;
    //     let mut primitives: Vec<Point3d> = Vec::new();
    //     primitives.push(Point3d::from_coords(0.0, 0.0, 0.0));
    //     assert_eq!(
    //         Octree::<Point3d, LEAF_CAPACITY>::linearize_primitives(&primitives)[0],
    //         OctreeItem { idx: 0, key: 0 }
    //     );
    //     primitives.push(Point3d::from_coords(1.0, 1.0, 1.0));
    //     assert_eq!(
    //         Octree::<Point3d, LEAF_CAPACITY>::linearize_primitives(&primitives)[1],
    //         OctreeItem {
    //             idx: 1,
    //             key: 0x0000FFFFFFFFFFFF_u64
    //         }
    //     );
    //     primitives.push(Point3d::from_coords(1.0, 1.0, -1.0));
    //     assert_eq!(
    //         Octree::<Point3d, LEAF_CAPACITY>::linearize_primitives(&primitives)[2],
    //         OctreeItem {
    //             idx: 2,
    //             key: 0x0000DB6DB6DB6DB6_u64
    //         }
    //     );
    //     primitives.push(Point3d::from_coords(1.0, -1.0, 1.0));
    //     assert_eq!(
    //         Octree::<Point3d, LEAF_CAPACITY>::linearize_primitives(&primitives)[3],
    //         OctreeItem {
    //             idx: 3,
    //             key: 0x0000B6DB6DB6DB6D_u64
    //         }
    //     );
    //     primitives.push(Point3d::from_coords(1.0, -1.0, -1.0));
    //     assert_eq!(
    //         Octree::<Point3d, LEAF_CAPACITY>::linearize_primitives(&primitives)[4],
    //         OctreeItem {
    //             idx: 4,
    //             key: 0x0000924924924924_u64
    //         }
    //     );
    //     primitives.push(Point3d::from_coords(-1.0, 1.0, 1.0));
    //     assert_eq!(
    //         Octree::<Point3d, LEAF_CAPACITY>::linearize_primitives(&primitives)[5],
    //         OctreeItem {
    //             idx: 5,
    //             key: 0x00006DB6DB6DB6DB_u64
    //         }
    //     );
    //     primitives.push(Point3d::from_coords(
    //         -0.999938963,
    //         -0.999938963,
    //         -0.999938963,
    //     ));
    //     assert_eq!(
    //         Octree::<Point3d, LEAF_CAPACITY>::linearize_primitives(&primitives)[6],
    //         OctreeItem {
    //             idx: 6,
    //             key: 0x0000000000000007_u64
    //         }
    //     );
    //     primitives.push(Point3d::from_coords(0.999969482, 0.999969482, 0.999969482));
    //     assert_eq!(
    //         Octree::<Point3d, LEAF_CAPACITY>::linearize_primitives(&primitives)[7],
    //         OctreeItem {
    //             idx: 7,
    //             key: 0x0000FFFFFFFFFFF8_u64
    //         }
    //     );
    // }
    //
    // #[test]
    // fn t_sort_primitives() {
    //     const LEAF_CAPACITY: usize = 1;
    //     let mut primitives: Vec<Point3d> = Vec::new();
    //     primitives.push(Point3d::from_coords(0.0, 0.0, 0.0));
    //     primitives.push(Point3d::from_coords(1.0, 1.0, 1.0));
    //     primitives.push(Point3d::from_coords(1.0, 1.0, -1.0));
    //     primitives.push(Point3d::from_coords(1.0, -1.0, 1.0));
    //     primitives.push(Point3d::from_coords(1.0, -1.0, -1.0));
    //     primitives.push(Point3d::from_coords(-1.0, 1.0, 1.0));
    //     primitives.push(Point3d::from_coords(
    //         -0.999938963,
    //         -0.999938963,
    //         -0.999938963,
    //     ));
    //     primitives.push(Point3d::from_coords(0.999969482, 0.999969482, 0.999969482));
    //     assert_eq!(
    //         Octree::<Point3d, LEAF_CAPACITY>::sort_primitives(&primitives),
    //         vec![
    //             OctreeItem {
    //                 idx: 6,
    //                 key: 0x0000000000000007_u64
    //             },
    //             OctreeItem {
    //                 idx: 0,
    //                 key: 0x00001FFFFFFFFFFF_u64
    //             },
    //             OctreeItem {
    //                 idx: 5,
    //                 key: 0x00006DB6DB6DB6DB_u64
    //             },
    //             OctreeItem {
    //                 idx: 4,
    //                 key: 0x0000924924924924_u64
    //             },
    //             OctreeItem {
    //                 idx: 3,
    //                 key: 0x0000B6DB6DB6DB6D_u64
    //             },
    //             OctreeItem {
    //                 idx: 2,
    //                 key: 0x0000DB6DB6DB6DB6_u64
    //             },
    //             OctreeItem {
    //                 idx: 7,
    //                 key: 0x0000FFFFFFFFFFF8_u64
    //             },
    //             OctreeItem {
    //                 idx: 1,
    //                 key: 0x0000FFFFFFFFFFFF_u64
    //             },
    //         ]
    //     );
    // }
    //
    // #[test]
    // fn t_get_nearest_from_leaf() {
    //     const LEAF_CAPACITY: usize = 8;
    //     let octree = Octree::<Point3d, LEAF_CAPACITY>::new(&golden_ref);
    //
    //     let origin = Point3d::from_coords(0.0, 0.0, 5.0);
    //
    //     let leaf = OctreeLeafNode{
    //         bb: Aabb::from_arrays([-1.0, -1.0, -1.0],[0.0, 1.0, 1.0]),
    //         items_idx: [Some(8), Some(0), Some(7), Some(6), Some(5), None, None, None]
    //     };
    //     // for i in 0..golden_ref.len() {
    //     //     let ray_dir = Vector3d::from_points(origin, golden_ref[i]);
    //     //     let ray = Ray3d::from(origin, ray_dir);
    //     //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), Some((i, ray_dir.len())));
    //     // }
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[0]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), Some((0, ray_dir.len())));
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[1]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), None);
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[2]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), None);
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[3]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), None);
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[4]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), None);
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[5]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), Some((5, ray_dir.len())));
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[6]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), Some((6, ray_dir.len())));
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[7]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), Some((7, ray_dir.len())));
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[8]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), Some((8, ray_dir.len())));
    //
    //
    //     let leaf = OctreeLeafNode {
    //         bb: Aabb::from_arrays([1.0, -1.0, -1.0],[1.0, 1.0, 1.0]),
    //         items_idx: [Some(4), Some(3), Some(2), Some(1), None, None, None, None]
    //     };
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[0]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), None);
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[1]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), Some((1, ray_dir.len())));
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[2]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), Some((2, ray_dir.len())));
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[3]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), Some((3, ray_dir.len())));
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[4]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), Some((4, ray_dir.len())));
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[5]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), None);
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[6]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), None);
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[7]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), None);
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[8]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), None);
    //
    //     // now test the case when the ray goes through multiple points, must always return the nearest
    //     let origin = Point3d::from_coords(1.0, 1.0, 5.0);
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[1]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), Some((1, 4.0)));
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[2]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.get_nearest_from_leaf(&leaf, &ray), Some((1, 4.0)));
    //
    // }
    //
    // #[test]
    // fn t_traverse() {
    //     const LEAF_CAPACITY: usize = 8;
    //     let octree = Octree::<Point3d, LEAF_CAPACITY>::new(&golden_ref);
    //
    //     let origin = Point3d::from_coords(0.0, 0.0, 5.0);
    //     let ray_dir = Vector3d::from_points(origin, golden_ref[0]);
    //     let ray = Ray3d::from(origin, ray_dir);
    //     assert_eq!(octree.traverse(&ray), Some((0, ray_dir.len())));
    // }
}

