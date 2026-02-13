use crate::ast::*;

pub fn analyze(mut graph: Graph) -> Graph {
    if graph.viewport.is_none() {
        graph.viewport = Some(Viewport::default());
    }
    graph
}
