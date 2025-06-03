/// Although the extract module already exists, it only works on graphs
/// that have gflow, which particularly QEC graphs don't, due to their
/// mid-circuit measurements
/// Hence, this new zx2stim module for qec noise simulations/thresholds
use crate::hash_graph::Graph;
use crate::graph::GraphLike;
use crate::graph::VType::{X, Z, H, B};
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
            // This is all assuming its a bipartite phaseless diagram
            // stim_lines.push(format!("# node {}", node));
            // stim_lines.push(format!("Type: {:?}", graph.vertex_type(node)));
            let vtype = graph.vertex_type(node);
            let neighbors: Vec<_> = graph.neighbors(node).collect();
            let qubit = graph.qubit(node);
            let row = graph.row(node);
            log::debug!("Currently on node {}, qubit {}, row {}, and have neighbors {:?}", node, qubit, row, neighbors);
            match neighbors.len() {
                1 => {
                    let neighbor = neighbors[0];
                    let neighbor_row = graph.row(neighbor);
                    log::debug!("node row: {}\n neighbor row: {}",row, neighbor_row);
                    if vtype == B {
                        continue;
                    }
                    if vtype == Z {
                            stim_lines.push(format!("H {}", qubit));
                        }
                    if neighbor_row < row {
                        stim_lines.push(format!("MR {}", qubit))
                    }
                }
                2 => {}
                3 => {}
                _ => {
                    stim_lines
                    .push
                    (format!("# Unsupported node degree {}: {}", 
                    neighbors.len(), node))
                }
            }
            
        }
        stim_lines.push("TICK".to_string());
    }

    stim_lines.join("\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_loader::load_graph;
    use crate::graph_to_svg::graph_to_svg;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            env_logger::builder()
                .is_test(true)
                .filter_level(log::LevelFilter::Debug)
                .try_init()
                .ok();
        });
    }
    fn test_file(name: &str) -> String {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../")
            .join("test_files")
            .join(name)
            .to_str()
            .unwrap()
            .to_string()
    }

    // #[test]
    // fn test_zx2stim()-> () {
    //     init_logger();
    //     let mut graph = Graph::new();
    //     let v1 = graph.add_vertex(X);
    //     let v2 = graph.add_vertex(Z);
    //     graph.add_edge(v1, v2);

    //     println!("{}",zx_to_stim(&graph));
    // }

    
    #[test]
    fn better_test_zx2stim() {
        init_logger();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("test_files")
        .join("simple_meas.zxg");
        let graph = load_graph(path.to_str().unwrap());
        let svg = graph_to_svg(&graph, &[]);
        let outpath = "test_write.svg";
        let _ = std::fs::write(test_file(outpath), svg);
        println!("{}", zx_to_stim(&graph));
    }
}