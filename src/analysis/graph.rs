use std::fmt;
use std::fmt::{Display, Formatter};

use petgraph::graphmap::DiGraphMap;
use petgraph::visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences, NodeIndexable, NodeRef};

use crate::analysis::{find_calls, is_std_fn};
use crate::types::{Function, RefFun, RefFunPointee};
use crate::Bytecode;

type Callgraph = DiGraphMap<RefFun, ()>;

pub fn call_graph(code: &Bytecode, f: RefFun, max_depth: usize) -> Callgraph {
    let mut g = Callgraph::new();
    match f.resolve(code).unwrap() {
        RefFunPointee::Fun(f) => {
            g.add_node(f.findex);
            build_graph_rec(code, &mut g, f, max_depth);
        }
        RefFunPointee::Native(n) => {
            g.add_node(n.findex);
        }
    }
    g
}

fn build_graph_rec(code: &Bytecode, g: &mut Callgraph, f: &Function, depth: usize) {
    if depth == 0 {
        return;
    }
    for fun in find_calls(f) {
        if !is_std_fn(code, fun) {
            match fun.resolve(code).unwrap() {
                RefFunPointee::Fun(fun) => {
                    if !g.contains_node(fun.findex) {
                        g.add_node(fun.findex);
                        build_graph_rec(code, g, fun, depth - 1);
                    }
                    g.add_edge(f.findex, fun.findex, ());
                }
                RefFunPointee::Native(n) => {
                    if !g.contains_node(n.findex) {
                        g.add_node(n.findex);
                    }
                    g.add_edge(f.findex, n.findex, ());
                }
            }
        }
    }
}

static TYPE: [&str; 2] = ["graph", "digraph"];
static EDGE: [&str; 2] = ["--", "->"];
static INDENT: &str = "    ";

pub struct GraphDisplay<'a> {
    g: &'a Callgraph,
    code: &'a Bytecode,
}

impl Display for GraphDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {{", TYPE[self.g.is_directed() as usize])?;

        // output all labels
        for node in self.g.node_references() {
            writeln!(
                f,
                "{}{} [ label = \"{}\" ]",
                INDENT,
                self.g.to_index(node.id()),
                node.weight().display_header(self.code)
            )?;
        }
        // output all edges
        for edge in self.g.edge_references() {
            writeln!(
                f,
                "{}{} {} {} [ label = \"\" ]",
                INDENT,
                self.g.to_index(edge.source()),
                EDGE[self.g.is_directed() as usize],
                self.g.to_index(edge.target()),
            )?;
        }

        writeln!(f, "}}")?;
        Ok(())
    }
}

/// Generate dot language
pub fn display_graph<'a>(g: &'a Callgraph, code: &'a Bytecode) -> GraphDisplay<'a> {
    GraphDisplay { g, code }
}
