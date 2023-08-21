//! This pass removes storage markers if they won't be emitted during codegen.

use crate::MirPass;
use rustc_middle::mir::*;
use rustc_middle::ty::TyCtxt;

// use std::collections::HashMap;
use rustc_data_structures::fx::FxHashMap;
use petgraph::Graph;
// use petgraph::dot::{Dot, Config};
use petgraph::algo::kosaraju_scc;
use petgraph::algo::toposort;
use petgraph::prelude::NodeIndex;
use petgraph::visit::Dfs;
// use rustc_middle::mir::Body;

// use rustc_middle::mir::*; // visit
use rustc_index::vec::IndexVec;
// use rustc_middle::mir::MirPass;
use rustc_middle::mir::{BasicBlock, Body, TerminatorKind, BasicBlockData, };

// use rustc_ast::ast::{InlineAsmOptions, InlineAsmTemplatePiece};

pub struct LoopUnroll();
impl<'tcx> MirPass<'tcx> for LoopUnroll {
    fn is_enabled(&self, sess: &rustc_session::Session) -> bool {
        // sess.instrument_coverage()
        // sess.mir_opt_level() > 0
        sess.mir_opt_level() == 0
        // true
    }
    // #[instrument(skip(self, tcx, body))]
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let def_id = body.source.def_id();
        if &tcx.def_path_str(def_id) == "fuzz_target" {
            println!("RUn roop_unroll.rs file, run fuzz_target function");

            // let bbs=body.basic_blocks_mut();
            // insert_dummy_block(body);

            let mut index_map: Vec<NodeIndex> = vec!();
            let mut scc_info_stk: FxHashMap<NodeIndex, Vec<SccInfo>> = Default::default();

            let g = mir_to_petgraph(tcx, body, &mut index_map, &mut scc_info_stk);
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
                // println!("after break down graph = \n{:?}", Dot::with_config(&copy_graph, &[Config::EdgeIndexLabel]));

                if stop {
                    // println!("\nBREAK!\n final SCC ={:?}\n\nSCC INFO STACK", scc_list.clone());
                    // for (n_idx, &ref stack) in scc_info_stk.iter() {
                    //     println!("node: {:?} == {:?}", n_idx, stack);
                    // }
                    break;
                }
            }
            println!("End of LOOP");
        }
    }
}


fn mir_to_petgraph<'tcx>(_tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>, arr: &mut Vec<NodeIndex>, scc_info_stk: &mut FxHashMap<NodeIndex, Vec<SccInfo>>)
                         -> Graph::<usize, String>{
    let mut g = Graph::<usize, String>::new();

    let mut cnt: usize = 0;
    for _ in body.basic_blocks.iter() {
        let node = g.add_node(cnt);
        scc_info_stk.insert(node, vec!());
        // node.index() should be index of IndexVector
        let index = body.scc_info.push(vec![]);
        // println!("mir to petgraph {:?} == {:?}, {:?}", node.index(), index, body.scc_info);
        assert_eq!(node.index(), index);

        arr.push(node);
        cnt = cnt + 1;
    }

    for (source, _) in body.basic_blocks.iter_enumerated() {
        // let def_id = body.source.def_id();
        // let def_name = format!("{}_{}", def_id.krate.index(), def_id.index.index(),);
        let terminator = body[source].terminator();
        let labels = terminator.kind.fmt_successor_labels();
        for (target, _label) in terminator.successors().zip(labels) {
            g.update_edge(arr[source.index()], arr[target.index()], String::from(""));
        }
    }
    printpg(g.clone(), "Initial");

    g
}

fn is_cycle(orig: Graph<usize, String>, scc:Vec<NodeIndex>) -> bool {
    let mut new = Graph::<usize, String>::new();

    let mut i = 0;
    for _node in orig.clone().raw_nodes() {
        let _node1 = new.add_node(i);
        i += 1;
    }

    for edge in orig.clone().raw_edges() {
        let s = edge.source();
        let t = edge.target();
        if scc.contains(&s) && scc.contains(&t) {
            new.update_edge(s, t, String::from("random"));
        }

    }

    // println!("Test if {:?} has cycle" , scc);
    match toposort(&new, None){
        Ok(_order) => {
            // println!("no cycle");
            return false;
        },
        Err(err) => {
            println!("yes cycle {:?}",err);
            return true;
        }
    }
}



pub fn find_all_headers(scc:Vec<NodeIndex>, g:&Graph<usize, String>) -> Vec<NodeIndex> {
    let mut headers :Vec<NodeIndex> = vec!();
    for edge in g.clone().raw_edges() {
        if !scc.contains(&edge.source()) && scc.contains(&edge.target()) {
            if !headers.contains(&edge.target()) {
                headers.push(edge.target());
                println!("Header = {:? } scc= {:?}", edge.target(), scc);
            }
        }
    }
    /// Assumption: only a single header possible
    assert_eq!(headers.len(), 1);
    return headers;
}

pub fn print_bbs<'tcx>(bbs: BasicBlocks<'tcx>, title: &str) {
    println!("\n\n=====  {} ({:?})  =====", title, bbs.len());
    for i in 0..bbs.len() {
        let tmp = bbs[i.into()].terminator.as_ref().expect("Error in print bbs").clone();
        // println!("  * {:?}: span[{:?}]  kind[{:?}]", i, tmp.source_info.span, tmp.kind);
        println!("  * {:?}: [{:?}]  [{:?}]", i, tmp.kind, bbs[i.into()].statements);
    }
}

/// TODO: [fix] not necessarily mutable
pub fn print_bbs_mut<'tcx>(bbs: &mut IndexVec<BasicBlock, BasicBlockData<'tcx>>, title: &str) {
    println!("\n\n=====  {} ({:?})  =====", title, bbs.len());
    for i in 0..bbs.len() {
        let tmp = bbs[i.into()].terminator.as_ref().expect("Error in print bbs").clone();
        // println!("  * {:?}: kind[{:?}]  span[{:?}]", i, tmp.kind, tmp.source_info.span);
        println!("  * {:?}: [{:?}]  [{:?}]", i, tmp.kind, bbs[i.into()].statements);
        // println!("  * {:?}: [{:?}]", i, tmp.kind);
    }
}

pub fn printpg(g: Graph<usize, String>, title: &str) {
    println!("\n\n===== PetGraph ({})   =====", title);
    for edge in g.clone().raw_edges() {
        println!("{:?}", edge);
    }
    // for node in g.clone().raw_nodes() {
    //     println!("{:?}", node);
    // }
}



pub fn change_target_goto<'tcx>(body: &mut Body<'tcx>, change_bb: BasicBlock, new_t: BasicBlock) {
    let bbs = body.basic_blocks_mut();
    let bb = bbs.get_mut(change_bb).expect("get bb to be changed.");
    let terminator = bb.terminator.as_mut().expect("terminator");
    /// method 1
    let new_goto_kind = TerminatorKind::Goto {
        target: new_t,
    };
    terminator.kind = new_goto_kind;


    /// method 2
    /*
        if let Some(_) = bb_terminator.kind.as_goto() {
            let new_goto_kind = TerminatorKind::Goto {
                target: new_t,
            };
            bb_terminator.kind = new_goto_kind;
        } else {
            println!("Input Basic block is not Goto Type!");
            unreachable!()
        }
    */

    println!("[T KIND] after change GOTO target = {:?}", terminator);
}

pub fn change_target_switchint<'tcx>(body: &mut Body<'tcx>,
                                     orig_bb: BasicBlock, new_t: BasicBlock, header_idx: usize) {

    let terminator = &mut body.basic_blocks_mut().get_mut(orig_bb).expect("").terminator_mut().kind;
    let TerminatorKind::SwitchInt {
        discr: old_op,
        targets: old_targets
    } = &terminator else {
        println!("Terminator kind of change_bb should be SwitchInt");
        unreachable!()
    };

    /// 1. check otherwise
    let new_otherwise;
    if old_targets.otherwise().index() == header_idx {
        new_otherwise = new_t;
    } else {
        new_otherwise = old_targets.otherwise();
    }

    /// 1. check targets
    let new_targets = old_targets.iter().map(|(value, target)| {
        if target.index() == header_idx {
            (value, new_t)
        } else {
            (value, target)
        }
    });

    let new_switch_targets =  SwitchTargets::new(new_targets, new_otherwise);
    let new_switchint_kind = TerminatorKind::SwitchInt {
        discr: old_op.to_copy(),
        targets: new_switch_targets,
    };

    *terminator = new_switchint_kind;

    println!("[T KIND] after change switch int {:?}", terminator);

}


/// copy, modify target, insert
pub fn insert_latch<'tcx>(body: &mut Body<'tcx>, header: BasicBlock) {
    let bbs = body.basic_blocks_mut();
    let bbd = BasicBlockData::new(Some(Terminator {
        source_info: bbs[header].terminator().source_info,
        kind: TerminatorKind::Goto {
            target: header,
        },
    }));
    // println!("copy header source info {:?}", bbd.terminator.clone().unwrap().source_info );
    bbs.push(bbd);
}

pub fn transform_to_single_header<'tcx>(scc: &mut Vec<NodeIndex>,
                                        headers: Vec<NodeIndex>,
                                        g: &mut Graph<usize, String>,
                                        scc_info_stk: &mut FxHashMap<NodeIndex, Vec<SccInfo>>,
                                        arr: &mut Vec<NodeIndex>,
                                        _tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>)
                                        -> NodeIndex {
    /// add to petgraph
    let new_node = g.add_node(777);
    scc.push(new_node);
    arr.push(new_node);
    scc_info_stk.insert(new_node, vec!());
    let index = body.scc_info.push(vec![]);
    println!("add new single header node {:?} == {:?}, {:?}", new_node.index(), index, body.scc_info);
    assert_eq!(new_node.index(), index);

    let bbs = body.basic_blocks_mut();

    /// copy type MUST be SwitchInt
    /// TODO: fix
    // insert_switchint(bbs, BasicBlock::from_usize(7),BasicBlock::from_usize(headers[0].index()), BasicBlock::from_usize(headers[1].index()) );

    for header in headers {

        let h = bbs.get_mut(BasicBlock::from_usize(header.index())).expect("no bb in mir");
        println!("header {:?}", h);

        let predecessors = get_predecessors_of(header, g);
        for pred in predecessors {
            // redirect
            let Some(edge_to_remove) = g.find_edge(pred, header) else {
                continue;
            };
            g.remove_edge(edge_to_remove);
            g.update_edge(pred, new_node, String::from("HEADER"));
            g.update_edge(new_node, header, String::from("HEADER"));
            println!("$$ first predecessors {:?} {:?} {:?}", pred, new_node, header);

            // MIR
            // TODO: [fix] remove temp
            //     decide_and_change_target(bbs,
            //                              BasicBlock::from_usize(pred.index()),
            //                              BasicBlock::from_usize(8),
            //                              BasicBlock::from_usize(8), BasicBlock::from_usize(8), true);
        }

        print_bbs_mut(bbs, "Get single hedaer");
    }

    // printpg(g.clone(), "header function");

    return new_node;
}


pub fn transform_to_single_latch<'tcx>(scc: &mut Vec<NodeIndex>,
                                       header: NodeIndex,
                                       g: &mut Graph<usize, String>,
                                       scc_info_stk: &mut FxHashMap<NodeIndex, Vec<SccInfo>>,
                                       arr: &mut Vec<NodeIndex>,
                                       body: &mut Body<'tcx>)
                                       -> NodeIndex {
    println!("SCC in get back edges {:?}", scc);

    let mut back_edges :Vec<(NodeIndex, NodeIndex)> = vec!();
    let mut inner_edges :Vec<(NodeIndex, NodeIndex)> = vec!();
    let mut remove = false;

    for edge in g.clone().raw_edges() {

        let mut test_g = g.clone();
        // println!("header, source, target {:?} {:?} {:?}", header.index(), edge.source().index(), edge.target().index());
        if scc.contains(&edge.source()) && scc.contains(&edge.target())
            && edge.target() == header {
            let Some(edge_idx) = test_g.find_edge(edge.source(), edge.target()) else {
                continue;
            };

            // assume
            test_g.remove_edge(edge_idx);
            // println!("remove edge {:?} -> {:?}", edge.source(), edge.target());

            let mut dfs_res = vec!();
            let mut dfs = Dfs::new(&test_g, edge.source());

            while let Some(visited) = dfs.next(&test_g) {
                dfs_res.push(visited.index());
            }
            // println!("dfs_res {:?}", dfs_res);

            if dfs_res.contains(&edge.target().index()) {
                // self loop is included here
                println!("still can reach {:?}", edge.target().index());
                inner_edges.push((edge.source(), edge.target()));
            } else {
                // if i cannot reach
                back_edges.push((edge.source(), edge.target()));
                println!("back_edges  = {:?}", back_edges);
                remove = true;
            }
        }
    }
    if remove == false {
        // remove both
        println!("there is no proper outer back edges, instead both can be latches {:?}", inner_edges);
        back_edges = inner_edges;
    }
    // let bbs = body.basic_blocks_mut();

    /// ============================== add dummy back edge to 8 to make multiple latches
    /// copy block not needed
    /// copy source_info from HEADER and create goto block to header
    // println!("copy header source info {:?}", BasicBlock::from_usize(header.index()) );
    // insert_latch(bbs,BasicBlock::from_usize(header.index()));
    insert_latch(body,BasicBlock::from_usize(header.index()));
    // let new_latch_idx = bbs.len() - 1;
    let new_latch_idx = body.basic_blocks.len() - 1;

    /// change_bb MUST BE SwtichInt kind
    /// TODO: remove temporarily
    // change_targets_switchint(bbs,BasicBlock::from_usize(7),
    //                          BasicBlock::from_usize(6),BasicBlock::from_usize(9), false);


    /// single LATCH
    let single_latch;
    let new_node;
    if back_edges.len() > 1 {
        println!("####### back edges {:?}", back_edges);
        new_node = g.add_node(999);
        single_latch = new_node;
        arr.push(new_node);
        scc_info_stk.insert(new_node, vec!());

        let index = body.scc_info.push(vec![]);
        println!("add new single latch node {:?} == {:?}, {:?}", new_node.index(), index, body.scc_info);
        assert_eq!(new_node.index(), index);

        /// ====================== create new basic block (add new sinlge latchnode)
        // pick a random latch to be copied
        // copy any latch(any random goto type) (assume latch is go to)
        // target should be header
        // insert_goto(bbs, BasicBlock::from_usize(back_edges[0].0.index()),BasicBlock::from_usize(header.index()));

        /// TEMP: test function.
        // change_target_switchint(body,
        //                         BasicBlock::from_usize(6),
        //                         BasicBlock::from_usize(new_latch_idx),
        //                         BasicBlock::from_usize(header.index()));

        /// for all latches
        for back_edge in back_edges {
            // redirect
            let latch = back_edge.0;
            let Some(edge_to_remove) = g.find_edge(latch, header) else {
                continue;
            };
            g.remove_edge(edge_to_remove);
            g.update_edge(latch, new_node, String::from("LATCH"));
            g.update_edge(new_node, header, String::from("LATCH"));

            // ==================== MIR part
            /// FIXED: decide if bb kind is goto or switchInt
            /// change edge latch->header to latch->new_single_latch
            // change_target_goto(body, BasicBlock::from_usize(latch.index().into()), BasicBlock::from_usize(new_latch_idx)),

            println!("tmp tmp {:?}", new_latch_idx);
            let terminator = body.basic_blocks_mut()
                .get_mut(latch.index().into()).expect("basic block data")
                .terminator.as_mut().expect("terminator");
            match terminator.kind {
                TerminatorKind::Goto {..} => {
                    change_target_goto(body,
                                       BasicBlock::from_usize(latch.index().into()),
                                       BasicBlock::from_usize(new_latch_idx),
                    );
                },
                TerminatorKind::SwitchInt {..} => {
                    change_target_switchint(body,
                                            BasicBlock::from_usize(latch.index().into()),
                                            BasicBlock::from_usize(new_latch_idx),
                                            header.index());
                },
                _ => panic!("[T KIND] terminator kind of latch is not Goto or SwitchInt"),
            }
        }


        print_bbs(body.clone().basic_blocks, "In get single latch");

    } else {
        single_latch = back_edges[0].0;
    }
    scc.push(single_latch);

    return single_latch;
}

pub fn get_predecessors_of(header: NodeIndex, g:&Graph<usize, String>) ->Vec<NodeIndex> {
    let mut preds :Vec<NodeIndex> = vec!();

    for edge in g.clone().raw_edges() {

        if edge.target() == header && edge.source() != header {
            preds.push(edge.source());
            println!("{:?}", edge);
        }
    }
    return preds;

}

use rustc_middle::mir::SccInfo;

pub fn break_down_and_mark<'tcx>(
    tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>,
    scc: &mut Vec<NodeIndex>, scc_id: &mut i32,
    g: &mut Graph<usize, String>,
    scc_info_stk: &mut FxHashMap<NodeIndex, Vec<SccInfo>>,
    arr: &mut Vec<NodeIndex>) {


    let loop_header;
    let single_latch;

    println!("====================== Transform & Mark ======================");
    // 1. mark header
    let headers = find_all_headers(scc.clone(), g);
    if headers.len() ==1 {
        println!("[1] if there is a single header {:?}", headers);
        loop_header = headers[0];
    } else {
        println!("[2] if there are multiple headers {:?}", headers);
        loop_header = transform_to_single_header(scc, headers, g, scc_info_stk, arr, tcx, body);
        // transform_mir_header(tcx, body)
    }
    // H: 1, L: 2, X: 3
    let scc_info = SccInfo::new(*scc_id as usize, NodeType::Header);
    let scc_info2 = SccInfo::new(*scc_id as usize, NodeType::Header);
    scc_info_stk.get_mut(&loop_header).map(|stk| stk.push(scc_info));
    body.scc_info[loop_header.index()].push(scc_info2);

    // 2. mark latch
    single_latch = transform_to_single_latch(scc, loop_header, g, scc_info_stk, arr, body);
    if scc.len() != 1 {
        // only if it is not a self loop, mark as Latch
        // let scc_info = SccInfo {
        //     _id: *scc_id,
        //     _n_type: 2,
        // };
        let scc_info = SccInfo::new(*scc_id as usize, NodeType::Latch);
        let scc_info2 = SccInfo::new(*scc_id as usize, NodeType::Latch);
        scc_info_stk.get_mut(&single_latch).map(|stk| stk.push(scc_info));
        body.scc_info[single_latch.index()].push(scc_info2);
    }

    // 3. mark 'X'
    for node in scc.clone() {
        if node != loop_header && node != single_latch {
            let scc_info = SccInfo::new(*scc_id as usize, NodeType::Normal);
            let scc_info2 = SccInfo::new(*scc_id as usize, NodeType::Normal);
            scc_info_stk.get_mut(&node).map(|stk| stk.push(scc_info));
            body.scc_info[node.index()].push(scc_info2);
        }
    }

    println!("\n====================== Break Down ======================");
    let Some(edge_idx) = g.find_edge(single_latch, loop_header) else {
        println!("cannot find edge in mark and break down");
        return;
    };
    println!("remove single latch = {:?} -> header = {:?}", single_latch, loop_header);
    g.remove_edge(edge_idx);

    *scc_id += 1;
}
