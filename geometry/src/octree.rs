use morton_encoding::*;

use crate::geometry::aabb::Aabb;
use crate::geometry::{Point3d};
use std::cmp::Ordering;
use rayon::prelude::*;
use std::ops::Deref;
use crate::geometry::triangle::Triangle;
use std::marker;
use std::marker::PhantomData;

const OCTREE_MAX_NUM_CHILDREN: usize = 2;
const OCTREE_MAX_LEAF_CAPACITY: usize = 8;

trait OrderByKey {
	fn get_key(&self) -> u64;
}
trait Bounded<T: OrderByKey> {
	fn get_centroid(&self) -> T;
	fn get_bounding_box(&self) -> Aabb;
}

struct OctreeMortonItem {
	idx: usize,
	morton: u64,
}

struct OctreeLeafNode {
	parent: Option<usize>,
	bb: Aabb,
	items_idx: [Option<usize>; OCTREE_MAX_LEAF_CAPACITY]
}
struct OctreeInnerNode {
	parent: Option<usize>,
	bb: Aabb,
	children_idx: [Option<usize>; OCTREE_MAX_NUM_CHILDREN],
}
enum OctreeNode {
	Leaf(OctreeLeafNode),
	Inner(OctreeInnerNode),
}
impl OctreeNode {
	fn get_bb (&self) -> Aabb {
		match self {
			OctreeNode::Inner(n) => n.bb,
			OctreeNode::Leaf(n) => n.bb,
		}
	}
	fn set_bb (&mut self, bb: Aabb) {
		match self {
			OctreeNode::Inner(n) => n.bb = bb,
			OctreeNode::Leaf(n) => n.bb = bb,
		}
	}
	// fn get_children (&self) -> [Option<usize>; OCTREE_MAX_NUM_CHILDREN] {
	// 	match self {
	// 		OctreeNode::Inner(n) => n.children,
	// 		OctreeNode::Leaf(n) => [None; OCTREE_MAX_NUM_CHILDREN],
	// 	}
	// }
	fn set_child (&mut self, child_idx: usize, val: usize) {
		match self {
			OctreeNode::Inner(n) => n.children_idx[child_idx] = Some(val),
			OctreeNode::Leaf(n) => panic!(),
		}
	}
	// fn get_items (&self) -> [Option<usize>; OCTREE_MAX_NUM_CHILDREN] {
	// 	match self {
	// 		OctreeNode::Inner(n) => [None; OCTREE_MAX_NUM_CHILDREN],
	// 		OctreeNode::Leaf(n) => n.items,
	// 	}
	// }
}
impl OctreeLeafNode {
	fn new() -> OctreeLeafNode {
		OctreeLeafNode {
			parent: None,
			bb: Aabb::new(),
			items_idx: [None; OCTREE_MAX_LEAF_CAPACITY],
		}
	}
}
impl OctreeInnerNode {
	fn new() -> OctreeInnerNode {
		OctreeInnerNode {
			parent: None,
			bb: Aabb::new(),
			children_idx: [None; OCTREE_MAX_NUM_CHILDREN],
		}
	}
}

struct Octree<T, P> {
	nodes: Vec<OctreeNode>,
	max_leaf_capacity: usize,
	_primitive: marker::PhantomData<P>,
	_centroid: marker::PhantomData<T>,
}

impl<T: OrderByKey, P: Bounded<T>> Octree<T, P> {
	pub fn new(primitives: &[P], max_leaf_capacity: usize) -> Octree<T, P> {
		let min_num_nodes = Octree::<T, P>::get_min_num_nodes(primitives.len(), max_leaf_capacity);
		let mut octree = Octree::<T, P> {
			nodes: Vec::with_capacity(min_num_nodes),
			max_leaf_capacity,
			_primitive: PhantomData,
			_centroid: PhantomData,
		};
		
		let mut indexed_morton: Vec<OctreeMortonItem> = primitives
			.iter()
			.enumerate()
			.map(|(idx, prim)| {
				OctreeMortonItem {
					idx,
					morton: prim.get_centroid().get_key(),
				}
			})
			.collect();
		
		indexed_morton.sort_by_key(|a| a.morton);
		
		octree.build(primitives, &mut indexed_morton, None);
		octree
	}
	
	fn get_min_num_nodes(num_elems: usize, max_node_capacity: usize) -> usize { //TODO change radix8 to radix2
		let depth = (num_elems as f32).log2().ceil() / (max_node_capacity as f32).log2().ceil();
		let num_nodes = (8.0_f32.powf(depth + 1.0) - 1.0) / 7.0;
		num_nodes as usize
	}
	
	fn split(elems: &mut [OctreeMortonItem]) -> (&mut [OctreeMortonItem], &mut [OctreeMortonItem])
	{
		let (below, above) = elems.split_at_mut((elems.len() / 2) as usize);
		(below, above)
	}
	
	pub fn build(&mut self, primitives: &[P], elems: &mut [OctreeMortonItem], parent_idx: Option<usize>) -> (usize, Aabb) {
		if elems.len() <= self.max_leaf_capacity {
			let leaf_idx = self.push_leaf(primitives, elems, parent_idx);
			let leaf_bb = self.nodes[leaf_idx].get_bb();
			return (leaf_idx, leaf_bb);
		}
		else {
			let inner_idx = self.push_inner(parent_idx);
			
			let (left, right) = Octree::<T, P>::split(elems);
			
			let mut inner_bb = Aabb::new();
			if left.len() > 0 {
				let (child_idx, child_bb) = self.build(primitives, left, Some(inner_idx));
				self.nodes[inner_idx].set_child(0, child_idx);
				inner_bb += child_bb;
			}
			if right.len() > 0 {
				let (child_idx, child_bb) = self.build(primitives, right, Some(inner_idx));
				self.nodes[inner_idx].set_child(1, child_idx);
				inner_bb += child_bb;
			}
			
			self.nodes[inner_idx].set_bb(inner_bb);
			(inner_idx, inner_bb)
		}
	}
	
	// returns the index of the created leaf node
	fn push_leaf(&mut self, primitives: &[P], elems: &[OctreeMortonItem], parent_idx: Option<usize>) -> usize {
		let mut leaf = OctreeLeafNode::new();//TODO
		leaf.parent = parent_idx;
		elems.iter().enumerate().for_each(|(idx, item)| {
			leaf.items_idx[idx] = Some(item.idx);
			leaf.bb += primitives[item.idx].get_bounding_box();
		});
		self.nodes.push(OctreeNode::Leaf(leaf));
		self.nodes.len() - 1
	}
	fn push_inner(&mut self, parent_idx: Option<usize>) -> usize {
		let mut inner = OctreeInnerNode::new();
		inner.parent = parent_idx;
		self.nodes.push(OctreeNode::Inner(inner));
		self.nodes.len() - 1
	}
	// pub fn find(&self, f: FnOnce()) -> Option<[Option<usize>; 8]> {
	// 	None
	// }
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::geometry::TraceablePrimitive;
	
	const MAX_NODE_CAPACITY: usize = 1;
	
	impl OrderByKey for Point3d {
		
		fn get_key(&self) -> u64 {
			let x: u16 = self.x as u16;
			let y: u16 = self.y as u16;
			let z: u16 = self.z as u16;
			morton_encode([x, y, z])
		}
	}
	impl Bounded<Point3d> for Triangle {
		fn get_centroid(&self) -> Point3d {
			TraceablePrimitive::get_centroid(self)
		}
		
		fn get_bounding_box(&self) -> Aabb {
			TraceablePrimitive::get_bounding_box(self)
		}
	}
	
	#[test]
	fn test1() {
		let mut triangles: Vec<Triangle> = Vec::new();
		triangles.push(
			Triangle::new(
				Point3d::from_coords(1.0, 1.0, 1.0),
				Point3d::from_coords(2.0, 2.0, 2.0),
				Point3d::from_coords(3.0, 3.0, 3.0),
			)
		);
		let octree = Octree::new(&*triangles, MAX_NODE_CAPACITY);
		
		octree.nodes.iter().for_each(|n| println!("{}", n.get_bb()));
	}
}