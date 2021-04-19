//! Compute a directed acyclic graph representing an extended set 
//! of maximal common subsequences of two sequences.
//! 


use super::{
    Dag,
    Node,
    NodeType,
    Position
};

use std::collections::HashMap;

use crate::substr::SubString as SubSeq;
use std::cmp::{min, max};

/// Intermediate structure used to compute the xMCS of two
/// sequences as a directed acyclic graph
struct Builder<'a, T> {
    /// Array of nodes
    nodes: Vec<Node<'a, T>>,
    /// Used to remember if we already computed the result for a given node
    memo: HashMap<Position, Option<usize>>,
}

impl<'a, T> Builder<'a, T>
where 
    T: Eq + Copy
{
    /// Compute a dag that represent a set of maximal common subsequences.
    /// 
    /// This function computes a directed acyclic graph (dag) used to 
    /// represent an extended set of maximal common subsequences of 
    /// length at least `len` of the two sequences `s1` and `s2`.
    pub(super) fn build(len: usize, s1: &'a [T], s2: &'a [T]) 
        -> Dag<'a, T>
    {
        let (mut graph, start) = Self::build_raw(len, s1, s2);

        // Set of common subsequences is empty
        if start.is_none() {
            // If there is no subsequence, the graph should be empty
            assert_eq!(0, graph.len());

            graph.push(Node { 
                min_length: 0,
                max_length: 0,
                inner: NodeType::Empty 
            });
        }

        Dag {
            nodes: graph,
            start: start.unwrap_or(0),
            len,
        }
    }

    fn build_raw(len: usize, s1: &'a [T], s2: &'a [T]) 
        -> (Vec<Node<'a, T>>, Option<usize>)
    {
        let n = max(s1.len(), s2.len());
        let delta = n - len;
        let subseq = SubSeq::new(s1, s2, delta);

        let mut res = Self {
            nodes: Vec::new(),
            memo: HashMap::new()
        };

        let start = res.compute(len, s1, s2, &subseq);
        (res.nodes, start)
    }

    /// Recursively compute the graph
    fn compute(
        &mut self, 
        len: usize,
        s1: &'a [T],
        s2: &'a [T],
        substr: &SubSeq)
    -> Option<usize> 
    {
        let l1 = s1.len();
        let l2 = s2.len();
        let pos = Position(len, l1, l2);

        // Value already computed once, return its index in the array
        if let Some(&index) = self.memo.get(&pos) {
            return index;
        }

        // Empty set. Empty node is always at index 0
        if len > l1 || len > l2 {
            self.insert_empty_at(pos);
            return None;
        }

        if substr.is_substring_from_end(l1, l2) {
            // One sequence a substring of the other        
            self.compute_subseq_node(s1, l1, s2, l2, pos)

        } else if s1[0] == s2[0] {
            // Matching elements
            // s1 and s2 always contains at least one element
            // otherwise one is a subsequence of the other.

            let len = len.saturating_sub(1);
            let index = self.compute(len, &s1[1..], &s2[1..], substr);

            self.compute_common_element_node(s1[0], index, pos)

        } else {
            // Mismatching elements
        
            let index1 = self.compute(len, &s1[1..], s2, substr);
            let index2 = self.compute(len, s1, &s2[1..], substr);

            self.compute_split_node(index1, index2, pos)
        }
    }

    /// Insert a node into the dag, remember to what parameters it correspond
    /// and returns its index.
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

    /// Compute the next node in the case one of the sequence is a
    /// subsequence of the other.
    #[inline(always)]
    fn compute_subseq_node(
        &mut self,
        s1: &'a [T], l1: usize, 
        s2: &'a [T], l2: usize,
        position: Position
    ) -> Option<usize> 
    {      
        let (length, suffix) = 
            if l1 < l2 {
                (l1, s1)
            } else {
                (l2, s2)
        }; 

        let node = Node {
            max_length: length,
            min_length: length,
            inner: NodeType::End { suffix }
        };

        self.insert_node_at(position, node)
    }

    /// Compute the next node in the case where the two sequences have an 
    /// element in common.
    #[inline(always)]
    fn compute_common_element_node(
        &mut self,
        element: T,
        index: Option<usize>,
        position: Position
    ) -> Option<usize>
    {
        match index {
            // Child is empty, the new node is also empty.
            None => self.insert_empty_at(position),

            // Else the new node references the child.
            Some(i) => {
                let node = &self.nodes[i];
        
                let max_length = node.max_length + 1;
                let min_length = node.min_length + 1;
                
                let node = Node {
                    max_length,
                    min_length,
                    inner: NodeType::Element {
                        value: element,
                        child: i
                    }
                };
        
                self.insert_node_at(position, node)
            }
        }
    }

    #[inline(always)]
    fn compute_split_node(
        &mut self,
        index1: Option<usize>,
        index2: Option<usize>,
        position: Position
    ) -> Option<usize>
    {
        match (index1, index2) {
            // Both children are empty nodes, the new node is also empty.
            (None, None) => 
                self.insert_empty_at(position),

            // One of the children is an empty node, 
            // the new node can point to the other.
            (None, Some(i)) | (Some(i), None) => 
                self.points_to_node(position, i),

            // Else the new node references the two children.
            (Some(i1), Some(i2)) => {
                let node1 = &self.nodes[i1];
                let node2 = &self.nodes[i2];
        
                let max_length = max(node1.max_length, node2.max_length);
                let min_length = min(node1.min_length, node2.min_length);
        
                let node = Node {
                    max_length,
                    min_length,
                    inner: NodeType::Split {
                        child1: i1,
                        child2: i2,
                    }
                };
        
                self.insert_node_at(position, node)
            }
        }
    }
}

/// Compute a graph representing an extended set of maximal
/// common subsequences of length at least `len` of two sequences
/// `s1` and `s2`.
pub fn xmcs2<'a, T>(len: usize, s1: &'a [T], s2: &'a [T])
    -> Dag<'a, T>
where
    T: Eq + Copy
{
    Builder::build(len, s1, s2)
}

pub(super) fn xmcs2_raw<'a, T>(len: usize, s1: &'a [T], s2: &'a [T])
    -> (Vec<Node<'a, T>>, Option<usize>)
where
    T: Eq + Copy
{
    Builder::build_raw(len, s1, s2)
}
