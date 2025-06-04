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
            let vtype = graph.vertex_type(node);
            let neighbors: Vec<_> = graph.neighbors(node).collect();
            let qubit = graph.qubit(node);
            let row = graph.row(node);
            // Wow this dbg!() macro is so much better and more concise
            dbg!(node, qubit, row, &neighbors);
            match neighbors.len() {
                1 => {
                    // One neighbor means either a Z / X measurement or a |0> / |+> preparation,
                    // depending on where the neighbor is in terms of TICKs
                    let neighbor = neighbors[0];
                    let neighbor_row = graph.row(neighbor);
                    dbg!(row, neighbor_row);
                    if vtype == B {
                        continue;
                    }
                    if vtype == Z {
                            stim_lines.push(format!("H {}", qubit));
                        }
                    if neighbor_row < row {
                        stim_lines.push(format!("MR {}", qubit));
                    }
                    // No need to handle the qubit allocation case beyond the H gate,
                    // since stim dynamically allocates |0> qubits
                }
                2 => {
                    // Do nothing, phaseless one-input one-output spiders are identities
                    // Different for e.g. two-input
                    // or two-output nodes, but stim does not support those as a native
                    // instruction anyways
                }
                3 => {
                    // This is going to be a CNOT
                    // Skip if current node is target, 
                    // will be handled in iteration over control node
                    if vtype == X {
                        continue
                    }
                    for neighbor in neighbors {
                        if graph.qubit(neighbor) != qubit {
                            stim_lines.push
                            (format!("CNOT {} {}", qubit, graph.qubit(neighbor)));
                        }
                    }


                }
                _ => {
                    stim_lines
                    .push
                    (format!("# Unsupported node degree {}: {}", 
                    neighbors.len(), node));
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
        .join("steane_style_steane_2_rounds.zxg");
        let graph = load_graph(path.to_str().unwrap());
        let svg = graph_to_svg(&graph, &[]);
        let outpath = "test_write.svg";
        let _ = std::fs::write(test_file(outpath), svg);
        println!("{}", zx_to_stim(&graph));
    }
}