//! Contains types and functions used to compute an extended set of
//! maximal common subsequences represented as a graph
//! 
//! This modules contains types and functions that are used to 
//! compute an extended set of maximal common subsequences of
//! multiple sequences, represented as a directed acyclic graph
//! 
//! The graph can be seen as an (not deterministic) automaton with 
//! epsilon-transitions and no loops.

mod xmcs2;
pub use xmcs2::xmcs2;
pub(self) use xmcs2::xmcs2_raw;
mod xmcsk;
pub use xmcsk::xmcsk;

use std::collections::HashSet;

/// Struct used to store a graph representing a set of sequences.
pub struct Dag<'a, T> {
    /// Array of nodes
    nodes: Vec<Node<'a, T>>,
    /// Index of the first node
    start: usize,
    /// Minimum size of a subsequence
    len: usize
}

#[derive(Debug)]
#[derive(Clone)]
struct Node<'a, T> {
    max_length: usize,
    min_length: usize,
    //cur_length: usize,
    inner: NodeType<'a, T>
}

#[derive(Debug)]
#[derive(Clone)]
enum NodeType<'a, T> {
    Empty,
    End {
        suffix: &'a [T]
    },
    Split {
        child1: usize,
        child2: usize
    },
    Element {
        value: T,
        child: usize,
    }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq, Hash)]
struct Position(usize, usize, usize);

impl<T> Dag<'_, T> 
where
    T: Copy 
{
    /// Extract one of the longest subsequence
    /// 
    /// Use information computed previously to return a longest common 
    /// subsequence of the sequences used to build this value.
    /// Returns `None` if there is no common subsequence of length more
    /// than `len`.1
    pub fn extract_lcs(&self) -> Option<Vec<T>> {
        let start = &self.nodes[self.start];
        if start.max_length == 0 {
            return None;
        }
        let mut res = Vec::with_capacity(start.max_length);
        self.extract_lcs_impl(start, &mut res);
        Some(res)
    }

    fn extract_lcs_impl(
        &self,
        current: &Node<T>,
        buffer: &mut Vec<T>
    ) {
        match current.inner {
            NodeType::Empty => (),
            NodeType::End { suffix } => 
                buffer.extend_from_slice(suffix),
            NodeType::Element { value, child } => {
                buffer.push(value);
                self.extract_lcs_impl(&self.nodes[child], buffer);
            }
            NodeType::Split { child1, child2 } => {
                let node1 = &self.nodes[child1];
                let node2 = &self.nodes[child2];
                if node1.max_length > node2.max_length {
                    self.extract_lcs_impl(node1, buffer)
                } else {
                    self.extract_lcs_impl(node2, buffer)
                }
            }
        }
    }

    pub fn to_set(&self) -> HashSet<T> {
        todo!();
    }

    /// Construct a graph representing the empty set
    pub fn empty(len: usize) -> Self {
        let mut nodes = Vec::new();
        nodes.push(Node {
            max_length: 0,
            min_length: 0,
            inner: NodeType::Empty
        });

        Self {
            nodes,
            start: 0,
            len
        }
    }
}

impl<'a, T> Dag<'a, T> {
    /// Construct a graph representing a singleton containing
    /// one string
    pub fn singleton(len: usize, seq: &'a [T]) -> Dag<'a, T> {
        let mut nodes = Vec::new();
        nodes.push(Node {
            max_length: 0,
            min_length: 0,
            inner: NodeType::Empty
        });
        nodes.push(Node {
            max_length: seq.len(),
            min_length: seq.len(),
            inner: NodeType::End {
                suffix: seq
            }
        });

        Self {
            nodes,
            start: 1,
            len
        }
    }
}

use std::fmt::Display;

use std::io::{
    Write,
    Result
};

impl<T> Dag<'_, T> 
where
    T: Display
{
    /// Outputs code to be used with the [dot] tool to produce 
    /// a visualisation of the current graph
    /// 
    /// # Errors
    /// Forwards errors from writing into `w`.
    /// 
    /// [dot]: (https://graphviz.org/)
    pub fn format_graph(&self, w: &mut impl Write) -> Result<()> {
        writeln!(w, "digraph xmcs {{")?;
        writeln!(w, "rankdir=LR;")?;

        for (i, node) in self.nodes.iter().enumerate() {
            write!(w, "node{} [", i)?;
            match &node.inner {
                NodeType::Empty =>
                    write!(w, "shape=circle, color=red, ")?,
                NodeType::End { suffix } => {
                    write!(w, "shape=none, fontcolor=green, label=\"")?;
                    for e in *suffix {
                        write!(w, "{}", e)?;
                    }
                    write!(w, "\"")?;
                }
                _ => write!(w, "shape=point, ")?
            }
            if i == self.start {
                write!(w, "style=bold, shape=circle, label=\"start\"")?;
            }
            writeln!(w, "];")?;
        }

        for (i, node) in self.nodes.iter().enumerate() {
            match &node.inner {
                NodeType::Element { child, value }
                    => writeln!(w, "node{} -> node{} [label={}, color=blue, fontcolor=red];", i, child, value)?,
                NodeType::Split { child1, child2 } 
                    => writeln!(w, "node{} -> {{ node{}, node{} }};", i, child1, child2)?,
                _ => ()
            }
        }

        writeln!(w, "}}")
    }
}

impl<'a, T> Node<'a, T> {
    // Change the index of the children of this node
    // so that they are still valid if all nodes
    // are shifted by `index` in the array of nodes.
    // This is useful to insert a subgraph into another graph.
    fn with_base_index(self, index: usize) -> Node<'a, T> {
        let node_type = match self.inner {
            NodeType::Element { value, child } => 
                NodeType::Element { value: value, child: child + index},
            NodeType::Split { child1, child2 } =>
                NodeType::Split { 
                    child1: child1 + index,
                    child2: child2 + index 
                },
            node_type => node_type
        };

        Self {
            max_length: self.max_length,
            min_length: self.min_length,
            inner: node_type
        }
    }
}