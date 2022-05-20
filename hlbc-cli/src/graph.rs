use std::fmt;
use std::fmt::{Display, Formatter};

use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use petgraph::graphmap::{DiGraphMap, GraphMap};
use petgraph::visit::{
    EdgeRef, GraphProp, IntoEdgeReferences, IntoNodeReferences, NodeIndexable, NodeRef,
};

use hlbc::opcodes::Opcode;
use hlbc::types::{Function, RefFun, RefFunPointee};
use hlbc::Bytecode;

use crate::iter_ops;
use crate::utils::find_calls;

pub fn build_graph(code: &Bytecode, f: RefFun) -> DiGraphMap<RefFun, ()> {
    let mut g = DiGraphMap::new();
    match f.resolve(code).unwrap() {
        RefFunPointee::Fun(f) => {
            g.add_node(f.findex);
            build_graph_rec(code, &mut g, f, 20);
        }
        RefFunPointee::Native(n) => {
            g.add_node(n.findex);
        }
    }
    g
}

fn build_graph_rec(code: &Bytecode, g: &mut DiGraphMap<RefFun, ()>, f: &Function, depth: usize) {
    if depth == 0 {
        return;
    }
    for fun in find_calls(f) {
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

static TYPE: [&str; 2] = ["graph", "digraph"];
static EDGE: [&str; 2] = ["--", "->"];
static INDENT: &str = "    ";

pub struct GraphDisplay<'a> {
    g: &'a DiGraphMap<RefFun, ()>,
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

pub trait CodeDisplay {
    fn display<'a>(&'a self, code: &'a Bytecode) -> GraphDisplay<'a>;
}

impl CodeDisplay for DiGraphMap<RefFun, ()> {
    fn display<'a>(&'a self, code: &'a Bytecode) -> GraphDisplay<'a> {
        GraphDisplay { g: self, code }
    }
}
