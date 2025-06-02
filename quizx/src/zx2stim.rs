use crate::hash_graph::Graph;
use crate::graph::GraphLike;
use crate::graph::VType::{X, Z, H};
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

pub fn zx_to_stim(graph: &Graph) -> String {
    let mut stim_lines = Vec::new();
    let mut nodes_by_row: BTreeMap<OrderedFloat<f64>, Vec<usize>> = BTreeMap::new();

    // Group nodes by row
    for node in graph.vertices() {
        let row = OrderedFloat(graph.vertex_data(node).row);
        nodes_by_row.entry(row).or_default().push(node);
    }

    // Loop over rows in order
    for (_row, nodes) in nodes_by_row {
        for node in nodes {
            // This is where you would match on node type and write Stim gates
            stim_lines.push(format!("# node {}", node));
            stim_lines.push(format!("Type: {:?}", graph.vertex_type(node)));
        }
        stim_lines.push("TICK".to_string());
    }

    stim_lines.join("\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_loader::load_graph;

    #[test]
    fn test_zx2stim()-> () {
        let mut graph = Graph::new();
        let v1 = graph.add_vertex(X);
        let v2 = graph.add_vertex(Z);
        graph.add_edge(v1, v2);

        print!("{}",zx_to_stim(&graph));
    }

    
    #[test]
    fn better_test_zx2stim() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("test_files")
        .join("xx_stab.zxg");
        let graph = load_graph(path.to_str().unwrap());
        println!("{}", zx_to_stim(&graph));
    }
}