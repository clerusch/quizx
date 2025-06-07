/// Although the extract module already exists, it only works on graphs
/// that have gflow, which particularly QEC graphs don't, due to their
/// mid-circuit measurements
/// Hence, this new zx2stim module for qec noise simulations/thresholds
use crate::hash_graph::Graph;
use crate::graph::GraphLike;
use crate::detection_webs::{detection_webs, Pauli};
use crate::graph::VType::{X, Z,B};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

/// New struct to avoid unnecessary dependencies
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct F64(pub f64);

// This is ok as long as we promise the F64s here will not be NaNs, which they shouldnt be 
// since they're only used for row/qubit parameters
impl Eq for F64 {}

impl Ord for F64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}
impl fmt::Display for F64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Converts a zx diagram to a stim circuit for qec simulations
pub fn zx_to_stim(graph: &mut Graph) -> String {
    let mut stim_lines = Vec::new();
    let mut nodes_by_row: BTreeMap<F64, Vec<usize>> = BTreeMap::new();
    //this is important, since we want to ensure we work with the bipartite form of the graph
    let webs = detection_webs(graph);

    // We need to convert the real number float values of qubit to positive integers for stim
    let mut qubit_values: Vec<F64> = graph
        .vertices()
        .map(|v| F64(graph.vertex_data(v).qubit))
        .collect();
    qubit_values.sort();
    qubit_values.dedup();

    let qubit_map: BTreeMap<F64, usize> = qubit_values
        .into_iter()
        .enumerate()
        .map(|(i, q)| (q, i))
        .collect();
    let mut rec_offsets = HashMap::new();
    let mut current_rec = 0;

    // Group nodes by row
    for node in graph.vertices() {
        let row = F64(graph.vertex_data(node).row);
        nodes_by_row.entry(row).or_default().push(node);
    }

    // Loop over rows in order
    for (row, nodes) in nodes_by_row {
        for node in nodes {
            // This is all assuming its a bipartite phaseless diagram
            let vtype = graph.vertex_type(node);
            let neighbors: Vec<_> = graph.neighbors(node).collect();
            let qubit = F64(graph.qubit(node));
            let qid = qubit_map[&qubit];
            // Wow this dbg!() macro is so much better and more concise
            // dbg!(node, qubit, row, &neighbors);
            // let phase = Some(graph.phase(node));
            match neighbors.len() {
                1 => {
                    // One neighbor means either a Z / X measurement or a |0> / |+> preparation,
                    // depending on where the neighbor is in terms of TICKs
                    let neighbor = neighbors[0];
                    let neighbor_row = F64(graph.row(neighbor));
                    // dbg!(row, neighbor_row);
                    if vtype == B {
                        continue;
                    }
                    if vtype == Z {
                            stim_lines.push(format!("H {} # mapped from qubit {}", qid, qubit));
                        }
                    if neighbor_row < row {
                        stim_lines.push(format!("MR {} # mapped from qubit {}", qid, qubit));
                        rec_offsets.insert(node, current_rec);
                        current_rec += 1;
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
                        let neighq = F64(graph.qubit(neighbor));
                        if  neighq != qubit {
                            stim_lines.push
                            (format!("CNOT {} {} # mapped from qubits {} {}",qid, qubit_map[&neighq], qubit, neighq));
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
    for web in webs {
        let mut rec_terms = Vec::new();
        for ((a, b), pauli) in &web.edge_operators {
            for &node in &[a,b] {
                
                let neighbors = graph.neighbors(*node);
                if neighbors.count() != 1 {
                    continue;
                }
                if !&web.edge_operators.contains_key(&(*a,*b)) {
                    continue;
                }
                let vtype = graph.vertex_type(*node);
                if vtype == B {continue}
                if *pauli == Pauli::X && vtype != Z ||
                   *pauli == Pauli::Z && vtype != X { continue}
                if let Some(&offset) = rec_offsets.get(&node) {
                    rec_terms.push(format!("rec[-{}]", current_rec-offset));
                }
            }
        }
        if !rec_terms.is_empty() {
            stim_lines.push(format!("DETECTOR {}", rec_terms.join(" ")));
        }
    }

    stim_lines.join("\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_loader::load_graph;
    use crate::graph_to_svg::graph_to_svg_with_pauliweb;
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
        let mut graph = load_graph(path.to_str().unwrap());
        let webs = detection_webs(&mut graph);
        for (i, web) in webs.iter().enumerate() {
            let svg = graph_to_svg_with_pauliweb(&graph, Some(web));
            let outpath = format!("test_write_{}.svg", i);
            let _ = std::fs::write(test_file(&outpath), svg);
        }
    }
}