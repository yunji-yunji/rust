//! This pass removes storage markers if they won't be emitted during codegen.

use crate::MirPass;
// use rustc_middle::mir::*;
use rustc_middle::ty::TyCtxt;

// use rustc_data_structures::fx::FxHashMap;
// // use rustc_middle::mir::Body;
// // use rustc_middle::mir::*; // visit
// use rustc_index::vec::IndexVec;
// // use rustc_middle::mir::MirPass;
use rustc_middle::mir::Body;
// use rustc_middle::mir::{BasicBlock, Body, TerminatorKind, BasicBlockData, };

// // use rustc_ast::ast::{InlineAsmOptions, InlineAsmTemplatePiece};
// use std::string::String;
// // use std::fs::File;
// // use std::io::Write;


pub struct LoopUnroll();
impl<'tcx> MirPass<'tcx> for LoopUnroll {
    fn is_enabled(&self, _sess: &rustc_session::Session) -> bool {
        // sess.instrument_coverage()
        // sess.mir_opt_level() > 0
        // sess.mir_opt_level() == 2
        true
    }
    // #[instrument(skip(self, tcx, body))]
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {

        // let target_name:String = "fuzz_target".parse().unwrap();
        // if let Ok(val) = std::env::var("TARGET_NAME") {
        //     target_name = val;
        // } else {
        //     target_name = "fuzz_target".parse().unwrap();
        // }
        //
        // let mut target_name = String()ToString("");
        // if let Ok(extra_flags) = env::var("TARGET_NAME") {
        //     target_name = extra_flags;
        //     // for flag in extra_flags.split_whitespace() {
        //     //     // program.args.push(flag.into());
        //     //     target_name = flag.`into();
        //     // }
        // }

        // let mut start: String = "off".parse().unwrap();
        // if let Ok(val) = std::env::var("START") {
        //     start = val;
        // }

        let def_id = body.source.def_id();
        println!("[MIRPASS] def_id [{:?}] def str {:?}", def_id, &tcx.def_path_str(def_id));
        // if &tcx.def_path_str(def_id) == "fuzz_target" {
        // if tcx.def_path_str(def_id).contains(&target_name) {
        // }
        /*

        let tmp = tcx.def_path_str(def_id);
        let name = format!("/home/y23kim/rust/scc_info/{:?}_{:?}.json", def_id.krate, def_id.index);
        println!("[mirpass] Create file = {:?}, defID {:?}", name.clone(), tmp);

        let scc_info_stk: FxHashMap<usize, Vec<SccInfo>> = Default::default();
        let json = serde_json::to_string_pretty(&scc_info_stk).expect("write json yj");
        let mut file = File::create(name.clone()).expect("yjyj cannot open file");
        // let mut file = File::create(format!("/home/y23kim/rust/scc_info/{:?}.json", tmp)).expect("yjyj cannot open file");
        // let mut file = File::create(format!("/home/y23kim/rust/scc_info/{:?}_{:?}.json", def_id.krate, def_id.index))
        //     .expect("yjyj cannot open file");
        file.write_all(json.as_bytes()).expect("cannot write file");

        println!("mirpass after LOOP");

         */

        // if tmp.contains("move") {
        //     println!("[mirpass] def id {:?}, {:?}, def name {:?}", def_id.index, def_id.krate, tmp);
        // }
        // let mut file = File::open(name);
        // if tcx.def_path_str(def_id).contains(&target_name) {
        // if tcx.def_path_str(def_id) == target_name {
        // println!(" run {:?} function", target_name);

        // let bbs=body.basic_blocks_mut();
        // insert_dummy_block(body);
        // if start.equals('on'.borrow()) {
        // if start == String::from("on") && tmp.contains("move") {
            // let name = format!("~/rust/scc_info/{}.json", tmp);
            // let mut index_map: Vec<NodeIndex> = vec!();
            // let mut scc_info_stk: FxHashMap<NodeIndex, Vec<SccInfo>> = Default::default();
        // let mut scc_info_stk: FxHashMap<usize, Vec<SccInfo>> = Default::default();
        /*

                    let g = mir_to_petgraph(tcx,
                                            body,
                                            &mut index_map, &mut scc_info_stk);
                    // print_bbs(body.clone().basic_blocks, "Initial MIR");

                    let mut scc_id: i32 = 1;
                    let mut copy_graph = g.clone();
                    loop {
                        let mut stop = true;
                        let mut scc_list = kosaraju_scc(&copy_graph);
                        println!("SCC = {:?}", scc_list.clone());
                        for scc in &mut scc_list {
                            let is_cycle = is_cycle(copy_graph.clone(), scc.clone());
                            if is_cycle == true {
                                stop = false;
                                break_down_and_mark(tcx, body,
                                                    scc, &mut scc_id, &mut copy_graph, &mut scc_info_stk, &mut index_map);
                            }
                        }
                        /*
                        // let serialized_data = serde_json::to_string(&scc_info_stk).;
                        // let mut file = File::create(PathBuf::from(name.clone())).expect("create file");
                        // file.write_all(serialized_data.as_bytes())?;
                        // println!("after break down graph = \n{:?}", Dot::with_config(&copy_graph, &[Config::EdgeIndexLabel]));


                         */
                        if stop {
                            // println!("\nBREAK!\n final SCC ={:?}\n\nSCC INFO STACK", scc_list.clone());
                            // for (n_idx, &ref stack) in scc_info_stk.iter() {
                            //     println!("node: {:?} == {:?}", n_idx, stack);
                            // }
                            break;
                        }
                    } // loop end



         */

        // }
        // }
    }
}

