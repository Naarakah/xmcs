//! Module used to format graphs to visualize them with the
//! graphviz tools
//!
//! The functions in this modules are not really optimised
//! I don't care, since the graphviz tool is way slower to
//! render the graph anyways.

use super::{Dag, NodeType};

use std::fmt::Display;
use std::io::{Result, Write};

impl<T> Dag<'_, T>
where
    T: Display,
{
    /// Outputs code to be used with the [dot] tool to produce
    /// a visualisation of the current graph
    ///
    /// # Errors
    /// Forwards errors from writing into `w`.
    ///
    /// [dot]: (https://graphviz.org/)
    pub fn format_graph(&self, w: &mut impl Write) -> Result<()> {
        writeln!(w, "digraph xMCS {{")?;
        writeln!(w, "\trankdir = LR;")?;
        writeln!(w, "\tpad = 1;")?;
        writeln!(w, "\tnewrank = yes;")?;

        writeln!(w, "\tstart [shape = none, height = 0, width = 0];")?;

        writeln!(w, "\tsubgraph cluster_info {{")?;
        writeln!(w, "\t\trank = same;")?;
        writeln!(w, "\t\tnode [shape = box];")?;
        writeln!(
            w,
            r#"{}min_len [label = "Minimum subsequence length: {}"];"#,
            "\t\t", self.len
        )?;
        writeln!(
            w,
            r#"{}states [label = "{} states"];"#,
            "\t\t",
            self.nodes.len()
        )?;
        writeln!(
            w,
            r#"{}opt [label = "Merge optimisations enabled"];"#,
            "\t\t",
        )?;
        //writeln!(w, "\t\tmin_len -> states -> opt -> start [style = invis];")?;
        writeln!(w, "\t}}")?;

        self.write_clusters(w)?;

        self.write_edges(w)?;

        writeln!(w, "}}")
    }

    fn write_clusters(&self, w: &mut impl Write) -> Result<()> {
        let depths = self.compute_depths();

        /* write!(w, "\t")?;
        for (depth, _) in depths.iter().enumerate().skip(1).rev() {
            write!(w, "depth_{} -> ", depth)?;
        }
        writeln!(w, "depth_0;")?; */

        for (depth, nodes) in depths.iter().enumerate() {
            /* writeln!(w, "\t\trank = same;")?;
            writeln!(w, "\t\tnode_{};", nodes[0])?;
            writeln!(w, "\t\tdepth_{};", depth)?;
            writeln!(w, "\t}}")?; */
            writeln!(w, r#"{}node [shape = point, label = ""];"#, "\t")?;

            if depth == 0 {
                writeln!(w, "\t{{")?;
                writeln!(w, "\t\tnode [shape = none, fontcolor = green];")?;
                for &idx in nodes {
                    let node = &self.nodes[idx];
                    match node.inner {
                        NodeType::End { suffix } => {
                            write!(w, "\t\t")?;
                            write_seq(w, suffix)?;
                            write!(w, r#" [label = ""#)?;
                            write_seq(w, suffix)?;
                            writeln!(w, r#""];"#)?;
                        }
                        _ => (),
                    }
                }
                writeln!(w, "\t}}")?;
            } else {
                for &idx in nodes {
                    writeln!(w, "\tnode_{};", idx)?;
                }
            }

            /*
            if depth == 0 {
                writeln!(w, "\tsubgraph cluster_subseq {{")?;
                writeln!(w, "\t\trank = sink;")?;
                writeln!(w, r#"{}label = "Subsequences";"#, "\t\t")?;
                for &idx in nodes {
                    let node = &self.nodes[idx];
                    write!(w, "\t\t")?;
                    match node.inner {
                        NodeType::End { suffix } => {
                            write_seq(w, suffix)?;
                            write!(w, r#" [label = ""#)?;
                            write_seq(w, suffix)?;
                            write!(w, r#"", shape = none, fontcolor = green]"#)?;
                        }
                        _ => (),
                    }
                    writeln!(w, ";")?;
                }
                writeln!(w, "\t}}")?;
            } else if depth > 1 {
                writeln!(w, "\t{{")?;
                writeln!(w, "\t\tlabelloc = b;")?;
                writeln!(w, "\t\tstyle = dashed;")?;
                writeln!(w, r#"{}label = "len = {}";"#, "\t\t", depth - 1)?;
                for &idx in nodes {
                    let node = &self.nodes[idx];
                    match node.inner {
                        NodeType::Element {..} => writeln!(w, "\t\tnode_{}", idx)?,
                        _ => ()
                    }
                }
                writeln!(w, "\t}}")?;
            } */
        }

        Ok(())
    }

    fn write_edges(&self, w: &mut impl Write) -> Result<()> {
        writeln!(w, "\tstart -> node_{} [dir = back, arrowhead = none, arrowtail = crow, arrowsize = 2, color = green];", self.start)?;

        for (i, node) in self.nodes.iter().enumerate() {
            match &node.inner {
                NodeType::Element { child, value } => {
                    write!(w, "\tnode_{} -> ", i)?;
                    if let NodeType::End { suffix } = self.nodes[*child].inner {
                        write_seq(w, suffix)?;
                        write!(w, " [arrowhead = dot, ")?;
                    } else {
                        write!(w, "node_{} [", child)?;
                    }
                    writeln!(
                        w,
                        "label = {}, weight = 2, color = blue, fontcolor = red];",
                        value
                    )?;
                }
                NodeType::Split { child1, child2 } => {
                    write!(w, "\tnode_{} -> ", i)?;
                    if let NodeType::End { suffix } = self.nodes[*child1].inner {
                        write_seq(w, suffix)?;
                        write!(w, " [arrowhead = dot]")?;
                    } else {
                        write!(w, "node_{}", child1)?;
                    }
                    writeln!(w, ";")?;

                    write!(w, "\tnode_{} -> ", i)?;
                    if let NodeType::End { suffix } = self.nodes[*child2].inner {
                        write_seq(w, suffix)?;
                        write!(w, " [arrowhead = dot]")?;
                    } else {
                        write!(w, "node_{}", child2)?;
                    }
                    writeln!(w, ";")?;
                }
                _ => (),
            }
        }

        Ok(())
    }

    fn compute_depths(&self) -> Vec<Vec<usize>> {
        use priority_queue::PriorityQueue;

        let mut queue = PriorityQueue::with_capacity(self.nodes.len());
        queue.push(self.start, self.len);

        let mut res = vec![vec![]; self.len + 2];
        let mut visited = vec![false; self.nodes.len()];

        while let Some((idx, depth)) = queue.pop() {
            let node = &self.nodes[idx];
            visited[idx] = true;

            if let NodeType::End { .. } = node.inner {
                res[0].push(idx);
            } else {
                res[depth + 1].push(idx);
            }

            match node.inner {
                NodeType::Split { child1, child2 } => {
                    if !visited[child1] {
                        queue.push_increase(child1, depth);
                    }
                    if !visited[child2] {
                        queue.push_increase(child2, depth);
                    }
                }
                NodeType::Element { child, .. } => {
                    queue.push_increase(child, depth.saturating_sub(1));
                }
                _ => (),
            }
        }

        res
    }
}

fn write_seq(w: &mut impl Write, seq: &[impl Display]) -> Result<()> {
    for e in seq {
        write!(w, "{}", e)?;
    }

    Ok(())
}
