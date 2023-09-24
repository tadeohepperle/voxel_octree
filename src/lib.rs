use std::{collections::VecDeque, ops::Deref};

use slab::Slab;

use pos::PosU8;

mod pos;

type Ptr = usize;

const EMPTY_PTR: Ptr = usize::MAX;

/// `V` is the type of voxel stored in the tree.
///
/// `HALF_WIDTH` is half of the cube side width of the tree.
/// E.g. if the tree should span a volume of 256x256x256, then the `HALF_WIDTH` = 128.
#[derive(Debug, Clone)]
pub struct Octree<V, const HALF_WIDTH: u8> {
    // the first element of the octree is always at ptr = 0 in the slab
    nodes: Slab<Node>,
    leafs: Slab<V>,
}

#[derive(Debug, Clone, Copy)]
pub enum Node {
    Full(Ptr),
    Mixed(
        /// each index is one of 8 space partitions
        /// -x-y-z: 0
        /// -x-y+z: 1
        /// -x+y-z: 2
        /// -x+y+z: 3
        /// +x-y-z: 4
        /// +x-y+z: 5
        /// +x+y-z: 6
        /// +x+y+z: 7
        [Ptr; 8],
        // non_empty_ptrs: usize,
    ),
}

impl Node {
    pub fn empty() -> Self {
        Node::Mixed([EMPTY_PTR; 8])
    }

    pub fn new_from_ptr(ptr: Ptr, ptr_index: usize) -> Self {
        let mut ptrs = [EMPTY_PTR; 8];
        ptrs[ptr_index] = ptr;
        Node::Mixed(ptrs)
    }
}

impl<V, const HALF_WIDTH: u8> Octree<V, HALF_WIDTH>
where
    V: Copy + PartialEq + std::fmt::Debug,
{
    pub fn new() -> Self {
        let mut nodes = Slab::<Node>::new();
        let root_ptr = nodes.insert(Node::Mixed([EMPTY_PTR; 8]));
        assert_eq!(root_ptr, 0);
        Octree {
            nodes,
            leafs: Slab::<V>::new(),
        }
    }

    /// pos is modified to be the new pos in the now half sized child node
    #[inline]
    fn oct_index(pos: &mut PosU8, half_width: u8) -> usize {
        let idx = match (pos.x < half_width, pos.y < half_width, pos.z < half_width) {
            (true, true, true) => 0,
            (true, true, false) => {
                pos.z -= half_width;
                1
            }
            (true, false, true) => {
                pos.y -= half_width;
                2
            }
            (true, false, false) => {
                pos.y -= half_width;
                pos.z -= half_width;
                3
            }
            (false, true, true) => {
                pos.x -= half_width;
                4
            }
            (false, true, false) => {
                pos.x -= half_width;
                pos.z -= half_width;
                5
            }
            (false, false, true) => {
                pos.x -= half_width;
                pos.y -= half_width;
                6
            }
            (false, false, false) => {
                pos.x -= half_width;
                pos.y -= half_width;
                pos.z -= half_width;
                7
            }
        };
        println!("calculate oct_index: pos: {pos:?} {half_width:?} -> {idx}");
        return idx;
    }

    pub fn get(&mut self, mut pos: PosU8) -> Option<V> {
        let mut node_ptr: usize = 0;
        let mut half_width: u8 = HALF_WIDTH;
        loop {
            let node = self.nodes[node_ptr];
            match node {
                Node::Full(leaf_ptr) => {
                    return Some(self.leafs[leaf_ptr]);
                }
                Node::Mixed(ptrs) => {
                    let idx = Self::oct_index(&mut pos, half_width);
                    // ptr points to node or leaf
                    let ptr = ptrs[idx];
                    if ptr == EMPTY_PTR {
                        return None;
                    } else if half_width == 1 {
                        // points to leaf
                        return Some(self.leafs[ptr]);
                    } else {
                        // points to node
                        half_width /= 2;
                        node_ptr = ptr;
                    }
                }
            }
        }
    }

    // pub fn go_down_inserting(&mut self, node: &mut InnerNode, pos: &mut pos) {}

    // fn insert_at_empty_ptr(
    //     &mut self,
    //     node_ptr: usize,
    //     oct_idx: usize,
    //     pos: PosU8,
    //     node_half_size: u8,
    // ) {
    //     assert!(self.nodes[node_ptr])
    // }

    /// should be optimized to return false very quickly in 99% of cases.
    fn insertion_would_make_node_full(
        &self,
        ptrs: &[usize; 8],
        insert_idx: usize,
        insert_val: &V,
        mut insert_pos: PosU8,
        mut node_half_width: u8,
    ) -> bool {
        // dbg!(ptrs, insert_idx, insert_val, insert_pos, node_half_width);
        if node_half_width == 1 {
            // Full leafs, except the one to be inserted need to be val_to_insert:
            ptrs.iter().enumerate().all(|(i, ptr)| {
                // skip the oct_idx_for_insert, then the pointer must point to a leaf with the same value as the one to be inserted.
                i == insert_idx || (*ptr != EMPTY_PTR && self.leafs[*ptr] == *insert_val)
            })
        } else {
            let mut child_node_where_insert_would_happen_would_be_full = || {
                let insert_node_ptr = ptrs[insert_idx];
                if insert_node_ptr == EMPTY_PTR {
                    return false;
                }
                let insert_node = self.nodes[insert_node_ptr];
                match &insert_node {
                    Node::Full(leaf_ptr) => &self.leafs[*leaf_ptr] == insert_val,
                    Node::Mixed(ptrs) => {
                        node_half_width /= 2;
                        let oct_idx_for_insert = Self::oct_index(&mut insert_pos, node_half_width);
                        self.insertion_would_make_node_full(
                            ptrs,
                            oct_idx_for_insert,
                            insert_val,
                            insert_pos,
                            node_half_width,
                        )
                    }
                }
            };

            let other_child_nodes_full_with_val_to_insert = || {
                ptrs.iter().enumerate().all(|(i, ptr)| {
                    // skip the oct_idx_for_insert, then child node ptr must be non empty and be "Full" node that points to leaf with val_to_insert value.
                    if i == insert_idx {
                        true
                    } else if *ptr == EMPTY_PTR {
                        false
                    } else {
                        let Node::Full(leaf_ptr) = &self.nodes[*ptr] else {
                            return false;
                        };
                        &self.leafs[*leaf_ptr] == insert_val
                    }
                })
            };

            child_node_where_insert_would_happen_would_be_full()
                && other_child_nodes_full_with_val_to_insert()
        }
    }

    fn delete_mixed_child_nodes(&mut self, ptrs: &[usize; 8], node_half_width: u8) {
        if node_half_width == 1 {
            for ptr in ptrs {
                if *ptr != EMPTY_PTR {
                    self.leafs.remove(*ptr);
                }
            }
        } else {
            for ptr in ptrs {
                if *ptr != EMPTY_PTR {
                    let node = self.nodes.remove(*ptr);
                    match node {
                        Node::Full(leaf_ptr) => {
                            self.leafs.remove(leaf_ptr);
                        }
                        Node::Mixed(ptrs) => {
                            self.delete_mixed_child_nodes(&ptrs, node_half_width / 2)
                        }
                    }
                }
            }
        }
    }

    /// returns pointer to child nodes
    fn insert_mixed_child_nodes_for_full_node_split(
        &mut self,
        majority_val: V,
        insert_idx: usize,
        insert_val: V,
        mut insert_pos: PosU8,
        node_half_with: u8,
    ) -> [usize; 8] {
        let mut ptrs = [0; 8];
        if node_half_with == 1 {
            // insert 8 leafs:
            for i in 0..8 {
                let val = if i == insert_idx {
                    insert_val
                } else {
                    majority_val
                };
                println!("inserted leaf because full_node_split: {insert_val:?}");
                ptrs[i] = self.leafs.insert(val);
            }
        } else {
            // insert 7 Full nodes and recursively insert a mixed node until leaf is reached:
            for i in 0..8 {
                let node = if i == insert_idx {
                    let child_insert_idx = Self::oct_index(&mut insert_pos, node_half_with);
                    let ptrs = self.insert_mixed_child_nodes_for_full_node_split(
                        majority_val,
                        child_insert_idx,
                        insert_val,
                        insert_pos,
                        node_half_with / 2,
                    );
                    Node::Mixed(ptrs)
                } else {
                    println!("inserted FullNode leaf from full_node_split: {insert_val:?}");
                    let leaf = self.leafs.insert(majority_val);
                    Node::Full(leaf)
                };
                ptrs[i] = self.nodes.insert(node);
            }
        }
        ptrs
    }

    // pub fn delete_node_recursively(&mut self, ptr: usize, nod)

    /// returns pointer to inserted node
    fn insert_nodes_below_empty_ptr(
        &mut self,
        mut pos: PosU8,
        val: V,
        node_half_width: u8,
    ) -> usize {
        // dbg!(("insert_nodes_below_empty_ptr", pos, val, node_half_width));
        if node_half_width == 0 {
            println!("insert leaf insert_nodes_below_empty_ptr {pos:?} {val:?}");
            let leaf_ptr = self.leafs.insert(val);
            dbg!(leaf_ptr);
            return leaf_ptr;
        } else {
            let oct_idx = Self::oct_index(&mut pos, node_half_width);
            let child_ptr = self.insert_nodes_below_empty_ptr(pos, val, node_half_width / 2);
            let node = Node::new_from_ptr(child_ptr, oct_idx);
            let node_ptr = self.nodes.insert(node);
            node_ptr
        }
        // let oct_idx = Self::oct_index(&mut pos, node_half_width);
        // let ptr = if node_half_width == 1 {
        //     let leaf_ptr = self.leafs.insert(val);
        //     leaf_ptr
        // } else {
        //     let child_node_ptr = self.insert_nodes_below_empty_ptr(pos, val, node_half_width / 2);
        //     child_node_ptr
        // };
        // let node = Node::new_from_ptr(ptr, oct_idx);
        // let node_ptr = self.nodes.insert(node);
        // node_ptr
    }

    pub fn insert(&mut self, mut pos: PosU8, val: V) {
        let original_pos = pos;
        let mut node_ptr: usize = 0;
        let mut half_width: u8 = HALF_WIDTH;
        loop {
            let node = self.nodes[node_ptr];
            match node {
                Node::Full(leaf_ptr) => {
                    let full_val = self.leafs[leaf_ptr];
                    if full_val != val {
                        let insert_idx = Self::oct_index(&mut pos, half_width);
                        // create child nodes resulting from split:
                        let child_node_ptrs = self.insert_mixed_child_nodes_for_full_node_split(
                            full_val, insert_idx, val, pos, half_width,
                        );
                        // remove the leaf:
                        self.leafs.remove(leaf_ptr);
                        // replace the current node with a Mixed Node.
                        self.nodes[node_ptr] = Node::Mixed(child_node_ptrs)
                    } else {
                        // ignore, full_val and val are the same, no edit needed
                    }
                    return;
                }
                Node::Mixed(mut ptrs) => {
                    let idx = Self::oct_index(&mut pos, half_width);

                    let node_would_be_full =
                        self.insertion_would_make_node_full(&ptrs, idx, &val, pos, half_width);

                    if node_would_be_full {
                        // recursively delete Full child nodes,
                        self.delete_mixed_child_nodes(&ptrs, half_width);
                        // replace the current node with a Full Node.
                        println!("inserted leaf because node_would_be_full: {original_pos:?},{pos:?} {val:?}");
                        let leaf_ptr = self.leafs.insert(val);
                        self.nodes[node_ptr] = Node::Full(leaf_ptr);
                        return;
                    } else {
                        let ptr = ptrs[idx];
                        if ptr == EMPTY_PTR {
                            // insert nodes into the tree until reaching the leaf level.
                            let inserted_node_ptr =
                                self.insert_nodes_below_empty_ptr(pos, val, half_width / 2);
                            // update the node pointer in this node
                            ptrs[idx] = inserted_node_ptr;
                            self.nodes[node_ptr] = Node::Mixed(ptrs);
                            return;
                        } else if half_width == 1 {
                            // edit leaf node
                            let leaf = &mut self.leafs[ptr];
                            let _old_val = std::mem::replace(leaf, val);
                            println!("edit leaf: {_old_val:?} -> {val:?}");
                            return;
                        } else {
                            // go one level deeper. Go to next loop iteration.
                            half_width /= 2;
                            node_ptr = ptr;
                        }
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, pos: PosU8) -> ! {
        todo!()
    }

    pub fn get_mut(&mut self, pos: PosU8) -> ! {
        todo!()
    }

    pub fn to_string(&self) -> String {
        const INDENT: &str = "   ";
        let mut lines: Vec<String> = vec![];

        // prefix, ptr, half_width, ident
        let mut frontier: VecDeque<(String, usize, u8, usize)> =
            vec![("".to_string(), 0, HALF_WIDTH, 0)].into();

        while let Some((prefix, ptr, half_width, indent)) = frontier.pop_back() {
            let node = self.nodes[ptr];
            lines.push(format!(
                "{}{prefix}Node {ptr} ({half_width}):",
                INDENT.repeat(indent)
            ));

            match node {
                Node::Full(leaf_ptr) => {
                    let leaf = self.leafs[leaf_ptr];
                    lines.push(format!("{}All: {leaf:?}", INDENT.repeat(indent + 1)));
                }
                Node::Mixed(ptrs) => {
                    let mut empties: Vec<usize> = vec![];
                    for (i, child_ptr) in ptrs.into_iter().enumerate() {
                        if child_ptr == EMPTY_PTR {
                            empties.push(i);
                        } else if half_width == 1 {
                            let leaf = self.leafs[child_ptr];
                            // dbg!(child_ptr, self.leafs[child_ptr]);
                            lines.push(format!("{}{i}: Leaf: {leaf:?}", INDENT.repeat(indent + 1)));
                        } else {
                            frontier.push_back((
                                format!("{i}: "),
                                child_ptr,
                                half_width / 2,
                                indent + 1,
                            ))
                        }
                    }
                    if !empties.is_empty() {
                        lines.push(format!(
                            "{}{}: Empty",
                            INDENT.repeat(indent + 1),
                            empties
                                .iter()
                                .cloned()
                                .map(|e| format!("{e}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }
            }
        }
        lines.join("\n")
    }

    pub fn print(&self) {
        let s = self.to_string();
        println!("{s}");
    }
}

// pub struct OctreeInnerAndLeafIter {}

// pub struct OctreeCoarseIter {}

// pub struct OctreeCourseIterItem{

// }

// impl Iterator for OctreeCoarseIter {
//     type Item = ();

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }

/*

Octree should store different data in leaves than in

*/

#[cfg(test)]
pub mod test {
    use rand::{thread_rng, Rng};

    use crate::{pos, pos::PosU8};

    use super::Octree;

    #[test]
    pub fn octree_leaf_node_count() {
        // create a 16x16x16 octree:
        let mut octree = Octree::<&'static str, 16>::new();
        octree.insert(PosU8 { x: 0, y: 0, z: 0 }, "Hello");
        octree.insert(PosU8 { x: 0, y: 1, z: 0 }, "Hello");
        octree.insert(PosU8 { x: 0, y: 0, z: 1 }, "Hello");

        assert_eq!(octree.leafs.len(), 3);
        octree.insert(PosU8 { x: 0, y: 1, z: 0 }, "Moin");
        octree.insert(PosU8 { x: 0, y: 0, z: 1 }, "Hello");

        assert_eq!(octree.leafs.len(), 3);
        octree.insert(PosU8 { x: 0, y: 1, z: 0 }, "Hello");

        octree.insert(PosU8 { x: 0, y: 1, z: 1 }, "Hello");
        octree.insert(PosU8 { x: 1, y: 0, z: 0 }, "Hello");
        octree.insert(PosU8 { x: 1, y: 1, z: 0 }, "Hello");
        octree.insert(PosU8 { x: 1, y: 0, z: 1 }, "Hello");
        assert_eq!(octree.leafs.len(), 7);
        octree.insert(PosU8 { x: 1, y: 1, z: 1 }, "Hello");
        // octree should now have just a leaf, because an all node was created:
        assert_eq!(octree.leafs.len(), 1);

        octree.insert(PosU8 { x: 1, y: 1, z: 1 }, "lol");
        // should have 8 leafs again:
        assert_eq!(octree.leafs.len(), 8);
    }
    #[test]
    pub fn octree_leaf_node_count_2() {
        // create a 16x16x16 octree:

        let mut octree = Octree::<&'static str, 16>::new();

        // fill one 8x8 cube:
        for x in 8..16 {
            for y in 0..8 {
                for z in 8..16 {
                    octree.insert(PosU8 { x, y, z }, "Hello");
                }
            }
        }
        assert_eq!(octree.leafs.len(), 1);
        // set one field of the cube differently:
        octree.insert(pos!(13, 5, 9), "Ok");

        assert_eq!(octree.leafs.len(), 7 + 7 + 7 + 1); // just 22 nodes to store an 8x8x8 chunk which has 512 values in it

        assert_eq!(octree.get(pos!(13, 5, 9)), Some("Ok"));
        assert_eq!(octree.get(pos!(0, 1, 2)), None);
        assert_eq!(octree.get(pos!(8, 5, 9)), Some("Hello"));
    }

    #[test]
    pub fn insert_and_get() {
        // create a 16x16x16 octree:
        let mut octree = Octree::<u32, 16>::new();

        for _ in 0..100 {
            let mut rng = thread_rng();
            let x: u8 = rng.gen_range(0..32);
            let y: u8 = rng.gen_range(0..32);
            let z: u8 = rng.gen_range(0..32);
            let r: u32 = rng.gen();
            let pos = pos!(x, y, z);
            octree.insert(pos, r);
            assert_eq!(octree.get(pos), Some(r))
        }
    }
}
