//! Utilities to generate a callgraph and generate dot graphs

use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};

use petgraph::graphmap::DiGraphMap;
use petgraph::visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences, NodeIndexable, NodeRef};

use crate::analysis::{find_last_closure_assign, is_std_fn};
use crate::types::{Function, RefFun, RefFunPointee};
use crate::{Bytecode, Opcode, Type};

pub enum Call {
    // Called with Call0, Call1, ...
    Direct,
    // Called a closure
    Closure,
}

type Callgraph = DiGraphMap<RefFun, Call>;
// Function argument number to function ptr
type RegCtx = HashMap<usize, RefFun>;

pub fn find_calls<'a>(
    code: &'a Bytecode,
    f: &'a Function,
    reg_ctx: &'a RegCtx,
) -> impl Iterator<Item = (Call, RefFun, RegCtx)> + 'a {
    macro_rules! build_ctx {
        ($i:ident; $args:expr) => {{
            let mut tmp = RegCtx::new();
            for (p, arg) in $args.into_iter().enumerate() {
                if matches!(f.regs[arg.0 as usize].resolve(&code.types), Type::Fun(_)) {
                    if let Some(value) = find_last_closure_assign(code, f, *arg, $i)
                        .or_else(|| reg_ctx.get(&(arg.0 as usize)).copied())
                    {
                        tmp.insert(p, value);
                    }
                }
            }
            tmp
        }};
    }

    f.ops.iter().enumerate().filter_map(|(i, o)| match o {
        Opcode::Call0 { fun, .. } => Some((Call::Direct, *fun, RegCtx::new())),
        Opcode::Call1 { fun, arg0, .. } => Some((Call::Direct, *fun, build_ctx!(i; [arg0]))),
        Opcode::Call2 {
            fun, arg0, arg1, ..
        } => Some((Call::Direct, *fun, build_ctx!(i; [arg0, arg1]))),
        Opcode::Call3 {
            fun,
            arg0,
            arg1,
            arg2,
            ..
        } => Some((Call::Direct, *fun, build_ctx!(i; [arg0, arg1, arg2]))),
        Opcode::Call4 {
            fun,
            arg0,
            arg1,
            arg2,
            arg3,
            ..
        } => Some((Call::Direct, *fun, build_ctx!(i; [arg0, arg1, arg2, arg3]))),
        Opcode::CallN { fun, args, .. } => Some((Call::Direct, *fun, build_ctx!(i; args))),
        Opcode::CallClosure { fun, args, .. } => find_last_closure_assign(code, f, *fun, i)
            .map(|f| (Call::Closure, f, build_ctx!(i; args)))
            .or_else(|| {
                reg_ctx
                    .get(&(fun.0 as usize))
                    .map(|param| (Call::Closure, *param, build_ctx!(i; args)))
            }),
        Opcode::CallMethod { field, args, .. } => f.regs[args[0].0 as usize]
            .resolve(&code.types)
            .get_type_obj()
            .map(|o| (Call::Direct, o.protos[field.0].findex, build_ctx!(i; args))),
        Opcode::CallThis { field, args, .. } => f.regs[0]
            .resolve(&code.types)
            .get_type_obj()
            .map(|o| (Call::Direct, o.protos[field.0].findex, build_ctx!(i; args))),
        _ => None,
    })
}

pub fn call_graph(code: &Bytecode, f: RefFun, max_depth: usize) -> Callgraph {
    let mut g = Callgraph::new();
    match f.resolve(code).unwrap() {
        RefFunPointee::Fun(f) => {
            g.add_node(f.findex);
            build_graph_rec(code, &mut g, f, &RegCtx::new(), max_depth);
        }
        RefFunPointee::Native(n) => {
            g.add_node(n.findex);
        }
    }
    g
}

fn build_graph_rec(code: &Bytecode, g: &mut Callgraph, f: &Function, ctx: &RegCtx, depth: usize) {
    if depth == 0 {
        return;
    }
    for (call, fun, ctx) in find_calls(code, f, ctx) {
        if !is_std_fn(code, fun) {
            match fun.resolve(code).unwrap() {
                RefFunPointee::Fun(fun) => {
                    if !g.contains_node(fun.findex) {
                        g.add_node(fun.findex);
                        //println!("call to {} with args: {:?}", fun.display_header(code), ctx);
                        build_graph_rec(code, g, fun, &ctx, depth - 1);
                    }
                    g.add_edge(f.findex, fun.findex, call);
                }
                RefFunPointee::Native(n) => {
                    if !g.contains_node(n.findex) {
                        g.add_node(n.findex);
                    }
                    g.add_edge(f.findex, n.findex, call);
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

        writeln!(f, "{}fontname=\"Helvetica,Arial,sans-serif\"", INDENT)?;
        writeln!(
            f,
            "{}node [fontname=\"Helvetica,Arial,sans-serif\" style=filled fillcolor=\"#f8f8f8\"]",
            INDENT
        )?;
        writeln!(
            f,
            "{}edge [fontname=\"Helvetica,Arial,sans-serif\"]",
            INDENT
        )?;

        // output all labels
        for node in self.g.node_references() {
            writeln!(
                f,
                "{}{} [ label = \"{}\" fontsize=18 shape=box color=\"#b20400\" fillcolor=\"#edd6d5\" ]",
                INDENT,
                self.g.to_index(node.id()),
                node.weight().display_header(self.code)
            )?;
        }
        // output all edges
        for edge in self.g.edge_references() {
            writeln!(
                f,
                "{}{} {} {} [ label = \"{}\" ]",
                INDENT,
                self.g.to_index(edge.source()),
                EDGE[self.g.is_directed() as usize],
                self.g.to_index(edge.target()),
                match edge.weight() {
                    Call::Direct => "",
                    Call::Closure => "closure",
                }
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
