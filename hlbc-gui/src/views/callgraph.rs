use eframe::egui::style::Margin;
use eframe::egui::{DragValue, Frame, Ui, Widget};
use egui_dock::Tab;

use hlbc::analysis::graph::{call_graph, display_graph, Callgraph};
use hlbc::types::RefFun;

use crate::AppCtx;

#[derive(Default)]
pub struct CallgraphTab {
    max_depth: usize,
    graph: Option<Callgraph>,

    // Cache variables
    graph_fun: RefFun,
    graph_depth: usize,
}

impl Tab<AppCtx> for CallgraphTab {
    fn title(&self) -> &str {
        "Callgraph"
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut AppCtx) {
        // Update cached graph
        if let Some(sel) = ctx.selected_fn {
            if sel != self.graph_fun || self.graph_depth != self.max_depth {
                self.graph = Some(call_graph(&ctx.code, sel, self.max_depth));
                self.graph_fun = sel;
                self.graph_depth = self.max_depth;
            }
        } else {
            self.graph = None;
            self.graph_fun = RefFun(0);
        }

        let margin = Margin::same(4.0);

        Frame::none().inner_margin(margin).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Max depth : ");
                DragValue::new(&mut self.max_depth)
                    .clamp_range(0..=20)
                    .ui(ui);
            });
            if let Some(cg) = &self.graph {
                ui.code(display_graph(cg, &ctx.code).to_string());
            } else {
                ui.label("Select a function in the Functions view to view its bytecode");
            }
        });
    }
}
