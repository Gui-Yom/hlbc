use std::collections::HashMap;
use std::ops::Deref;

use eframe::egui::style::Margin;
use eframe::egui::{Area, Color32, DragValue, Frame, Id, ScrollArea, Stroke, Ui, Vec2, Widget};
use eframe::epaint::CubicBezierShape;

use hlbc::analysis::graph::petgraph::visit::EdgeRef;
use hlbc::analysis::graph::petgraph::visit::IntoEdgeReferences;
use hlbc::analysis::graph::{call_graph, Callgraph};
use hlbc::fmt::EnhancedFmt;
use hlbc::types::RefFun;

use crate::AppCtxHandle;

#[derive(Default)]
pub struct CallgraphView {
    max_depth: usize,
    graph: Option<Callgraph>,

    // Cache variables
    graph_fun: RefFun,
    graph_depth: usize,

    // Graph area
    pan: Vec2,
}

impl CallgraphView {
    fn title(&self) -> &str {
        "Callgraph"
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        // Update cached graph
        /*
        if let Some(sel) = ctx.selected() {
            if sel != self.graph_fun || self.graph_depth != self.max_depth {
                self.graph = Some(call_graph(ctx.code().deref(), sel, self.max_depth));
                self.graph_fun = sel;
                self.graph_depth = self.max_depth;
            }
        } else {
            self.graph = None;
            self.graph_fun = RefFun(0);
        }*/

        let margin = Margin::same(4.0);

        Frame::none().inner_margin(margin).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Max depth : ");
                DragValue::new(&mut self.max_depth)
                    .clamp_range(0..=20)
                    .ui(ui);
            });

            if let Some(cg) = &self.graph {
                //ui.code(display_graph(cg, &ctx.code).to_string());
                let start = ui.next_widget_position().to_vec2();
                ScrollArea::both()
                    .id_source("graph_area")
                    .auto_shrink([false, false])
                    .show_viewport(ui, |ui, rect| {
                        let mut nodes_pos = HashMap::new();
                        for n in cg.nodes() {
                            let pos = ui.next_widget_position();
                            nodes_pos.insert(
                                n,
                                Area::new(Id::new(n))
                                    .default_pos(pos)
                                    .drag_bounds(rect.translate(start))
                                    .show(ui.ctx(), |ui| {
                                        Frame::window(ui.style().as_ref()).show(ui, |ui| {
                                            ui.code(
                                                n.display_header::<EnhancedFmt>(ctx.code().deref())
                                                    .to_string(),
                                            )
                                        })
                                    })
                                    .response
                                    .rect,
                            );
                        }
                        for e in cg.edge_references() {
                            // Paint a nice bezier curve as the link between nodes
                            let s = nodes_pos.get(&e.source()).unwrap().center_bottom();
                            let t = nodes_pos.get(&e.target()).unwrap().center_top();
                            let scale = ((t.x - s.x) / 2.0).max(30.0);
                            let ctrl1 = s + Vec2::new(0.0, scale);
                            let ctrl2 = t - Vec2::new(0.0, scale);
                            let bezier = CubicBezierShape::from_points_stroke(
                                [s, ctrl1, ctrl2, t],
                                false,
                                Color32::TRANSPARENT,
                                Stroke::new(3.0, Color32::LIGHT_GRAY),
                            );
                            ui.painter_at(rect).add(bezier);
                        }
                    });
            } else {
                ui.label("Select a function in the Functions view to view its bytecode");
            }
        });
    }
}
