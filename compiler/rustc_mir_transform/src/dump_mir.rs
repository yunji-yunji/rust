//! This pass just dumps MIR at a specified point.

use std::fs::File;
use std::io;
// use std::intrinsics::mir::BasicBlock;

use crate::MirPass;
use rustc_middle::ty::TyCtxt;
use rustc_session::config::{OutFileName, OutputType};
//
use rustc_middle::mir::write_mir_pretty;
use rustc_middle::mir::Body;


// // use std::collections::HashMap;
// use rustc_data_structures::fx::FxHashMap;
// use petgraph::Graph;
// use petgraph::dot::{Dot, Config};
// use petgraph::algo::kosaraju_scc;
// use petgraph::algo::toposort;
// use petgraph::prelude::NodeIndex;
// use petgraph::visit::Dfs;

pub struct Marker(pub &'static str);

// use rustc_middle::mir::*; // visit
// use rustc_ast::ast::{InlineAsmOptions, InlineAsmTemplatePiece};
// use rustc_index::vec::IndexVec;
// use rustc_middle::mir::patch::MirPatch;
// use rustc_middle::

impl<'tcx> MirPass<'tcx> for Marker {
    fn name(&self) -> &'static str {
        self.0
    }

    fn run_pass(&self, _tcx: TyCtxt<'tcx>, _body: &mut Body<'tcx>) {
        println!("yunji: dump_mir (do nothing)");
    //     let def_id = body.source.def_id();
    //     if &tcx.def_path_str(def_id) == "fuzz_target" {
    //         println!("YuNJI: FUZZ target");
    //
    //         // let bbs=body.basic_blocks_mut();
    //         // insert_dummy_block(body);
    //
    //         let mut index_map: Vec<NodeIndex> = vec!();
    //         let mut scc_info_stk: FxHashMap<NodeIndex, Vec<SccInfo>> = Default::default();
    //
    //         let g = mir_to_petgraph(tcx, body, &mut index_map, &mut scc_info_stk);
    //         print_bbs(body.clone().basic_blocks, "Initial MIR");
    //
    //         let mut scc_id: i32 = 1;
    //         let mut copy_graph = g.clone();
    //         loop {
    //             let mut stop = true;
    //             let mut scc_list = kosaraju_scc(&copy_graph);
    //             println!("SCC = {:?}", scc_list.clone());
    //             for scc in &mut scc_list {
    //                 let is_cycle = is_cycle(copy_graph.clone(), scc.clone());
    //                 if is_cycle == true {
    //                     stop = false;
    //                     break_down_and_mark(tcx, body,
    //                                         scc, &mut scc_id, &mut copy_graph, &mut scc_info_stk, &mut index_map);
    //                 }
    //             }
    //             println!("after break down graph = \n{:?}", Dot::with_config(&copy_graph, &[Config::EdgeIndexLabel]));
    //
    //             if stop {
    //                 println!("\nBREAK!\n final SCC ={:?}\n\nSCC INFO STACK", scc_list.clone());
    //                 for (n_idx, &ref stack) in scc_info_stk.iter() {
    //                     println!("node: {:?} == {:?}", n_idx, stack);
    //                 }
    //                 break;
    //             }
    //         }
    //         println!("End of LOOP");
    //     }
    }

}

pub fn emit_mir(tcx: TyCtxt<'_>) -> io::Result<()> {
    match tcx.output_filenames(()).path(OutputType::Mir) {
        OutFileName::Stdout => {
            let mut f = io::stdout();
            write_mir_pretty(tcx, None, &mut f)?;
        }
        OutFileName::Real(path) => {
            let mut f = io::BufWriter::new(File::create(&path)?);
            write_mir_pretty(tcx, None, &mut f)?;
        }
    }
    Ok(())
}
