use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use hlbc::opcodes::Opcode;
use hlbc::types::Function;
use hlbc::Bytecode;

#[derive(Debug, Clone)]
pub enum ControlFlow {
    Branch(usize, usize),
    Jump(usize),
    Return,
}

#[derive(Debug, Clone)]
pub struct BasicBlock<'a> {
    start: usize,
    end: usize,
    ops: &'a [Opcode],
    cf: ControlFlow,
}

#[derive(Debug)]
pub struct BasicBlocks<'a>(HashMap<usize, BasicBlock<'a>>);

impl<'a> BasicBlocks<'a> {
    pub fn new(f: &'a Function) -> BasicBlocks<'a> {
        let mut start = 0;
        let mut blocks = HashMap::new();
        let mut targets = HashSet::new();

        for (i, o) in f.ops.iter().enumerate() {
            macro_rules! process_branch {
                ($offset:ident) => {
                    blocks.insert(
                        start,
                        BasicBlock {
                            start,
                            end: i,
                            ops: &f.ops[start..=i],
                            cf: ControlFlow::Branch(i + $offset as usize + 1, i + 1),
                        },
                    );
                    start = i + 1;
                    targets.insert(i + $offset as usize);
                };
            }

            match o {
                &Opcode::JTrue { cond, offset } => {
                    process_branch!(offset);
                }
                &Opcode::JFalse { cond, offset } => {
                    process_branch!(offset);
                }
                &Opcode::JNull { reg, offset } => {
                    process_branch!(offset);
                }
                &Opcode::JNotNull { reg, offset } => {
                    process_branch!(offset);
                }
                &Opcode::JSLt { a, b, offset } | &Opcode::JULt { a, b, offset } => {
                    process_branch!(offset);
                }
                &Opcode::JSGte { a, b, offset } | &Opcode::JUGte { a, b, offset } => {
                    process_branch!(offset);
                }
                &Opcode::JSGt { a, b, offset } => {
                    process_branch!(offset);
                }
                &Opcode::JSLte { a, b, offset } => {
                    process_branch!(offset);
                }
                &Opcode::JEq { a, b, offset } => {
                    process_branch!(offset);
                }
                &Opcode::JNotEq { a, b, offset } => {
                    process_branch!(offset);
                }
                &Opcode::JAlways { offset } => {
                    blocks.insert(
                        start,
                        BasicBlock {
                            start,
                            end: i,
                            ops: &f.ops[start..=i],
                            cf: ControlFlow::Jump(i + offset as usize + 1),
                        },
                    );
                    start = i + 1;
                    targets.insert(i + offset as usize);
                }
                &Opcode::Ret { ret } => {
                    blocks.insert(
                        start,
                        BasicBlock {
                            start,
                            end: i,
                            ops: &f.ops[start..=i],
                            cf: ControlFlow::Return,
                        },
                    );
                    start = i + 1;
                }
                _ => {}
            }
            if targets.remove(&i) && start <= i {
                blocks.insert(
                    start,
                    BasicBlock {
                        start,
                        end: i,
                        ops: &f.ops[start..=i],
                        cf: ControlFlow::Jump(i + 1),
                    },
                );
                start = i + 1;
            }
        }
        Self(blocks)
    }
}

impl BasicBlock<'_> {
    pub fn display<'a>(&'a self, ctx: &'a Bytecode, f: &'a Function) -> impl Display + 'a {
        fmtools::fmt! {
            for (i, o) in self.ops.iter().enumerate() {
                {o.display(ctx, f, (self.start + i) as i32, 0)}"\n"
            }
        }
    }
}

#[cfg(feature = "alt-graph")]
mod graph {
    use std::cmp::Ordering;
    use std::fmt::Display;
    use std::hash::{Hash, Hasher};

    use petgraph::graphmap::DiGraphMap;
    use petgraph::visit::{
        EdgeRef, IntoEdgeReferences, IntoNodeReferences, NodeIndexable, NodeRef,
    };

    use hlbc::types::Function;
    use hlbc::Bytecode;

    use crate::alt::bb::{BasicBlock, BasicBlocks, ControlFlow};

    impl BasicBlocks<'_> {
        pub fn make_graph(&self) -> BlockGraph {
            let mut graph = DiGraphMap::with_capacity(self.0.len() + 2, self.0.len() * 2);

            graph.add_node(GraphNode::Start);
            graph.add_node(GraphNode::End);
            for bb in self.0.values() {
                graph.add_node(GraphNode::Block(bb));
            }
            graph.add_edge(
                GraphNode::Start,
                GraphNode::Block(self.0.get(&0).unwrap()),
                "",
            );
            for bb in self.0.values() {
                match &bb.cf {
                    ControlFlow::Branch(a, b) => {
                        graph.add_edge(
                            GraphNode::Block(bb),
                            GraphNode::Block(self.0.get(a).unwrap()),
                            "true",
                        );
                        graph.add_edge(
                            GraphNode::Block(bb),
                            GraphNode::Block(self.0.get(b).unwrap()),
                            "false",
                        );
                    }
                    ControlFlow::Jump(a) => {
                        graph.add_edge(
                            GraphNode::Block(bb),
                            GraphNode::Block(self.0.get(a).unwrap()),
                            "",
                        );
                    }
                    ControlFlow::Return => {
                        graph.add_edge(GraphNode::Block(bb), GraphNode::End, "");
                    }
                }
            }
            BlockGraph(graph)
        }
    }

    pub struct BlockGraph<'ops, 'block>(DiGraphMap<GraphNode<'ops, 'block>, &'static str>);

    impl BlockGraph<'_, '_> {
        pub fn display<'a>(&'a self, ctx: &'a Bytecode, f: &'a Function) -> impl Display + 'a {
            const INDENT: &str = "    ";

            fmtools::fmt! {
                "digraph {\n"
                {INDENT}"fontname=\"Jetbrains Mono,Fira Code,monospace\"\n"
                {INDENT}"node [fontname=\"Jetbrains Mono,Fira Code,monospace\" style=filled fillcolor=\"#f8f8f8\"]\n"
                {INDENT}"edge [fontname=\"Jetbrains Mono,Fira Code,monospace\"]\n"
                // output all labels
                for node in self.0.node_references() {
                    {INDENT}{self.0.to_index(node.id())}" [ label = \""{node.weight().display(ctx, f)}"\" fontsize=13 shape=box color=\"#b20400\" fillcolor=\"#edd6d5\" ]\n"
                }
                // output all edges
                for edge in self.0.edge_references() {
                    {INDENT}{self.0.to_index(edge.source())}" -> "{self.0.to_index(edge.target())}" [ label = \""{edge.weight()}"\" ]\n"
                }
                "}"
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum GraphNode<'ops, 'block> {
        Start,
        End,
        Block(&'block BasicBlock<'ops>),
    }

    impl PartialEq for GraphNode<'_, '_> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (GraphNode::Start, GraphNode::Start) => true,
                (GraphNode::End, GraphNode::End) => true,
                (&GraphNode::Block(a), &GraphNode::Block(b)) => {
                    std::ptr::eq(a as *const BasicBlock, b as *const BasicBlock)
                }
                _ => false,
            }
        }
    }

    impl Eq for GraphNode<'_, '_> {}

    impl PartialOrd for GraphNode<'_, '_> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            if self == other {
                return Some(Ordering::Equal);
            }
            match (self, other) {
                (GraphNode::Start, GraphNode::End) => Some(Ordering::Less),
                (GraphNode::Start, GraphNode::Block(_)) => Some(Ordering::Less),
                (GraphNode::End, GraphNode::Start) => Some(Ordering::Greater),
                (GraphNode::End, GraphNode::Block(_)) => Some(Ordering::Greater),
                (&GraphNode::Block(a), &GraphNode::Block(b)) => a.start.partial_cmp(&b.start),
                (GraphNode::Block(_), GraphNode::Start) => Some(Ordering::Greater),
                (GraphNode::Block(_), GraphNode::End) => Some(Ordering::Less),
                _ => None,
            }
        }
    }

    impl Ord for GraphNode<'_, '_> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    impl Hash for GraphNode<'_, '_> {
        fn hash<H: Hasher>(&self, state: &mut H) {
            match self {
                GraphNode::Start => {
                    1.hash(state);
                }
                GraphNode::End => {
                    2.hash(state);
                }
                &GraphNode::Block(addr) => {
                    3.hash(state);
                    (addr as *const BasicBlock).hash(state);
                }
            }
        }
    }

    impl GraphNode<'_, '_> {
        pub fn display<'a>(&'a self, ctx: &'a Bytecode, f: &'a Function) -> impl Display + 'a {
            fmtools::fmt! { move
                match self {
                    GraphNode::Start => "start",
                    GraphNode::End => "end",
                    GraphNode::Block(bb) => {{bb.display(ctx, f)}},
                }
            }
        }
    }
}

#[cfg(all(test, feature = "alt-graph"))]
mod tests {
    use std::io::Cursor;

    use hlbc::Bytecode;

    use crate::alt::bb::BasicBlocks;

    #[test]
    fn simple() {
        let ctx =
            Bytecode::load(&mut Cursor::new(include_bytes!("../../../data/Branch.hl"))).unwrap();
        let f = &ctx.functions[*ctx.fnames.get("main").unwrap()];
        println!("{}", BasicBlocks::new(f).make_graph().display(&ctx, f));
    }

    #[test]
    fn nested() {
        let ctx = Bytecode::load(&mut Cursor::new(include_bytes!(
            "../../../data/BranchNested.hl"
        )))
        .unwrap();
        let f = &ctx.functions[*ctx.fnames.get("main").unwrap()];
        println!("{}", BasicBlocks::new(f).make_graph().display(&ctx, f));
    }
}
