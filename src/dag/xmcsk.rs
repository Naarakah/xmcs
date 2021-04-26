//! Compute a dag representing an extended set of maximal common
//!  subsequences of k sequences
//! 

use super::{
    Dag,
    Node,
    NodeType,
    Position
};

use std::collections::HashMap;
use std::cmp::{min, max};

/// Intermediate structure used to compute an extended set of
/// common subsequences of a sequence and a graph representing
/// a set of subsequences
struct Builder<'a, T> {
    /// Array of nodes
    nodes: Vec<Node<'a, T>>,
    /// Used to remember if we already computed the result for a given node
    memo: HashMap<Position, Option<usize>>,
    /// Graph representing a set of sequences
    base_graph: Vec<Node<'a, T>>
}

impl<'a, T> Builder<'a, T>
where
    T: Eq + Copy
{
    pub(super) fn add_sequence(xmcs: Dag<'a, T>, sequence: &'a [T]) 
        -> Dag<'a, T> 
    {   
        let len = xmcs.len;
        let start = xmcs.start;

        let mut res = Builder {
            nodes: Vec::new(),
            memo: HashMap::new(),
            base_graph: xmcs.nodes
        };

        let start = res.compute(len, start, sequence);

        if start.is_none() {
            assert_eq!(true, res.nodes.is_empty());

            res.nodes.push(Node {
                max_length: 0,
                min_length: 0,
                inner: NodeType::Empty
            });                
        }
        
        Dag {
            nodes: res.nodes,
            start: start.unwrap_or(0),
            len
        }
    }

    // Compute (or retrieve if already calculated) the part of the graph
    // representing all the subsequences possible from the given minimum
    // length of subsequence `len`, the subgraph starting at `node_index`
    // and the sequence `seq`
    fn compute(
        &mut self, 
        len: usize,
        current: usize,
        seq: &'a [T]
    ) -> Option<usize>
    {
        let node = &self.base_graph[current];
        let l1 = node.max_length;
        let l2 = seq.len();
        let pos = Position(len, l1, l2);

        // TODO: maybe swap the next two conditions ?
        // there may be a lot of case where len > l1 or len > l2
        // and it is probably faster to test it than to retrieve
        // an element from the hashmap
        // TODO: benchmark

        // Value already computed once, return its index in the array
        if let Some(&index) = self.memo.get(&pos) {
            return index;
        }

        // Empty set. Empty node is always at index 0
        if len > l1 || len > l2 {
            return self.insert_empty_at(pos);
        }

        match node.inner {
            NodeType::Empty => 
                self.insert_empty_at(pos),

            // Use the algorithm for two sequences
            NodeType::End { suffix } => {
                let (subgraph, start) = super::xmcs2_raw(len, suffix, seq);
                self.insert_subgraph_at(pos, subgraph, start)
            },

            NodeType::Split { child1, child2 } => {
                let index1 = self.compute(len, child1, seq);
                let index2 = self.compute(len, child2, seq);

                self.compute_split_node(index1, index2, pos)
            },

            // `seq` is empty but we have enough elements: end here
            NodeType::Element { .. } if l2 == 0 && len == 0 => {
                let node = Node {
                    max_length: 0,
                    min_length: 0,
                    inner: NodeType::End {
                        suffix: seq
                    }
                };

                self.insert_node_at(pos, node)
            },

            // `seq` is empty and not enough elements: empty set
            NodeType::Element {..} if l2 == 0 =>
                self.insert_empty_at(pos),

            // Matching elements (safety: `seq` is not empty here)
            NodeType::Element { value, child } if value == seq[0] => {
                let len = len.saturating_sub(1); // Stop at 0
                let index = self.compute(len, child, &seq[1..]);

                self.compute_common_element_node(index, value, pos)
            },

            // Mismatching elements
            NodeType::Element { child, .. } => {
                let index1 = self.compute(len, current, &seq[1..]);
                let index2 = self.compute(len, child, seq);

                self.compute_split_node(index1, index2, pos)
            }
        }
    }

    // Insert `node` into the graph and return its index
    fn insert_node_at(&mut self, position: Position, node: Node<'a, T>) 
        -> Option<usize>
    {
        let index = Some(self.nodes.len());
        self.nodes.push(node);
        self.memo.insert(position, index);
        index
    }

    /// Register that a position points to an existing node
    /// 
    /// Panics if that node does not exists
    fn points_to_node(&mut self, position: Position, index: usize)
        -> Option<usize> 
    {
        assert!(index < self.nodes.len());
        self.memo.insert(position, Some(index));
        Some(index)
    }

    /// Register that a node is empty
    fn insert_empty_at(&mut self, position: Position) 
        -> Option<usize> 
    {
        self.memo.insert(position, None);
        None
    }

    /// Insert another graph into `self`, shifting all the
    /// indices to keep correct references to children
    /// return the index of the first node of the inserted
    /// subgraph or `None` if the subgraph was empty.
    #[inline(always)]
    fn insert_subgraph_at(
        &mut self,
        position: Position, 
        other: Vec<Node<'a, T>>,
        start: Option<usize>
    ) -> Option<usize>
    {
        match start {
            None => 
                self.insert_empty_at(position),
            Some(start) => {
                let index = self.nodes.len();
                let nodes = other
                    .into_iter()
                    .map(|node| node.with_base_index(index));
                self.nodes.extend(nodes);
                Some(start + index)
            }
        }
    }

    #[inline(always)]
    fn compute_split_node(
        &mut self,
        index1: Option<usize>,
        index2: Option<usize>,
        position: Position
    ) -> Option<usize> {
        match (index1, index2) {
            // Both children empty => node is empty
            (None, None) => 
                self.insert_empty_at(position),
            
            // Only one child non-empty => node is equal to that child
            (Some(idx), None) | (None, Some(idx)) =>
                self.points_to_node(position, idx),
            
            // (Optimization)
            // The children are identical => point to that node
            (Some(idx1), Some(idx2)) if idx1 == idx2 => self.points_to_node(position, idx1),

            // Else the node is a split node
            (Some(idx1), Some(idx2)) => {
                let node1 = &self.nodes[idx1];
                let node2 = &self.nodes[idx2];

                // (Optimization)
                // If we are in a case like this:
                //       1 __ ...
                //      /  \
                // Self     \
                //      \___ 2 ...
                // Then node 1 has the same children, we can use it
                // instead of inserting a new node.
                if node1.is_split_with_child(idx2) {
                    return self.points_to_node(position, idx1);
                }
                if node2.is_split_with_child(idx1) {
                    return self.points_to_node(position, idx2);
                }

                let max_length = max(node1.max_length, node2.max_length);
                let min_length = min(node1.min_length, node2.min_length);

                let node = Node {
                    max_length,
                    min_length,
                    inner: NodeType::Split {
                        child1: idx1,
                        child2: idx2,
                    }
                };
        
                self.insert_node_at(position, node)
            }
        }
    }

    #[inline(always)]
    fn compute_common_element_node(
        &mut self,
        index: Option<usize>,
        element: T,
        position: Position
    ) -> Option<usize> {

        match index {
            None => 
                self.insert_empty_at(position),

            Some(idx) => {
                let node = &self.nodes[idx];

                let max_length = node.max_length + 1;
                let min_length = node.min_length + 1;
        
                let node = Node {
                    max_length,
                    min_length,
                    inner: NodeType::Element {
                        value: element,
                        child: idx
                    }
                };

                self.insert_node_at(position, node)        
            }
        }
    }
}

pub fn xmcsk<'a, T>(len: usize, sequences: &[&'a [T]]) 
    -> Dag<'a, T> 
where
    T: Eq + Copy
{   
    match sequences {
        &[] => Dag::empty(len),
        &[s] => Dag::singleton(len, s),
        // In theory this case is not necessary
        //&[s1, s2] => super::xmcs2(len, s1, s2),
        &[ ref seqs @ .., s] => {
            let graph = xmcsk(len, seqs);
            Builder::add_sequence(graph, s)
        }
    }
}
