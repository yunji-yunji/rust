//! This pass removes storage markers if they won't be emitted during codegen.

use crate::MirPass;
use rustc_middle::mir::*;
use rustc_middle::ty::TyCtxt;

// use std::collections::HashMap;
use rustc_data_structures::fx::FxHashMap;
use petgraph::Graph;
use petgraph::dot::{Dot, Config};
use petgraph::algo::kosaraju_scc;
use petgraph::algo::toposort;
use petgraph::prelude::NodeIndex;
use petgraph::visit::Dfs;
// use rustc_middle::mir::Body;

// use rustc_middle::mir::*; // visit
use rustc_index::vec::IndexVec;
// use rustc_middle::mir::MirPass;
use rustc_middle::mir::{BasicBlock, Body, TerminatorKind, BasicBlockData, };

use rustc_ast::ast::{InlineAsmOptions, InlineAsmTemplatePiece};

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
        println!("RUn roop_unroll.rs file");
        let def_id = body.source.def_id();
        if &tcx.def_path_str(def_id) == "fuzz_target" {
            println!("YuNJI: FUZZ target");

            // let bbs=body.basic_blocks_mut();
            // insert_dummy_block(body);

            let mut index_map: Vec<NodeIndex> = vec!();
            let mut scc_info_stk: FxHashMap<NodeIndex, Vec<SccInfo>> = Default::default();

            let g = mir_to_petgraph(tcx, body, &mut index_map, &mut scc_info_stk);
            print_bbs(body.clone().basic_blocks, "Initial MIR");

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
                println!("after break down graph = \n{:?}", Dot::with_config(&copy_graph, &[Config::EdgeIndexLabel]));

                if stop {
                    println!("\nBREAK!\n final SCC ={:?}\n\nSCC INFO STACK", scc_list.clone());
                    for (n_idx, &ref stack) in scc_info_stk.iter() {
                        println!("node: {:?} == {:?}", n_idx, stack);
                    }
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
        println!("mir to petgraph {:?} == {:?}, {:?}", node.index(), index, body.scc_info);
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
            println!("no cycle");
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
    return headers;
}


fn _insert_dummy_block<'tcx>(body: &mut Body<'tcx>) {
    /// basicblock-> statements, "terminator", is_cleanup
    /// terminator-> source_info, kind
    /// kind (TerminatorKind) -> Goto | SwitchInt
    /// SwitchInt -> discr, targets
    /// discr -> Operand<'tcx>
    /// targets = SwitchTargets -> new(target_iter, otherwise)
    /// target_iter -> values,targets
    /// values -> SmallVec<[u128; 1]>
    /// targets -> SmallVec<[BasicBlock; 2]>

    let _bbs = body.basic_blocks_mut();

    // 1. add Dummy basic block(node 7) branch to b1 and b4
    // TODO: fix
    // insert_switchint(bbs, BasicBlock::from_usize(1), BasicBlock::from_usize(1), BasicBlock::from_usize(4));

    // 2. change edge 0->1 to 0->7(new node)
    // TODO: remove all dummy insert things..
    // change_target_goto(bbs, BasicBlock::from_usize(0), BasicBlock::from_usize(7));

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
    for node in g.clone().raw_nodes() {
        println!("{:?}", node);
    }
}

/// insert inline assembly kind basic block
fn _insert_inline_asm<'tcx>(tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
    let bbs = body.basic_blocks_mut();

    let template_piece = InlineAsmTemplatePiece::String(String::from("yunji mir pass test"));
    let template = [template_piece];
    // let template = tcx.arena.alloc(&template);
    // let template = tcx.arena.alloc([template_piece]);
    let template = tcx.arena.alloc_from_iter(template);

    let asm_terminator_kind = TerminatorKind::InlineAsm {
        template,
        operands: vec![],
        options: InlineAsmOptions::empty(),
        line_spans: &[],
        destination: Some(bbs.next_index()),
        unwind: UnwindAction::Unreachable,
        // cleanup: None,
    };

    let len = bbs.len();
    let original_last_block = bbs.get_mut(BasicBlock::from_usize(len-1)).expect("No last block!");

    let mut new_terminator = original_last_block.terminator.as_ref().expect("no terminator").clone();
    let original_last_block_terminator = original_last_block.terminator_mut();
    new_terminator.kind = asm_terminator_kind;

    let new_bb = BasicBlockData {
        statements: vec![],
        terminator: Some(original_last_block_terminator.to_owned()),
        is_cleanup: false,
    };

    bbs.push(new_bb);
}



pub fn change_target_goto<'tcx>(body: &mut Body<'tcx>, change_bb: BasicBlock, new_t: BasicBlock) {
    let bbs = body.basic_blocks_mut();
    let bb = bbs.get_mut(change_bb).expect("get bb to be changed.");
    let bb_terminator = bb.terminator.as_mut().expect("terminator");

    /// method 1
    let new_goto_kind = TerminatorKind::Goto {
        target: new_t,
    };
    bb_terminator.kind = new_goto_kind;


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

    println!("after change GOTO target = {:?}", bb_terminator);
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

pub fn _change_targets_switchint<'tcx>(bbs: &mut IndexVec<BasicBlock, BasicBlockData<'tcx>>, change_bb: BasicBlock, new_t1: BasicBlock, new_t2:BasicBlock, keep_otherwise: bool) {

    let bb = bbs.get_mut(change_bb).expect("Get basic block to be changed");
    let bb_terminator = bb.terminator.as_mut().expect("Get terminator of the BB");
    let TerminatorKind::SwitchInt {
        discr: copy_op,
        targets: copy_targets
    } = &bb_terminator.kind else {
        println!("Terminator kind of change_bb should be SwitchInt");
        unreachable!()
    };

    /// otherwise
    let otherwise;
    if keep_otherwise {
        otherwise = copy_targets.otherwise();
    } else {
        otherwise = new_t2;
    }

    /// change targets : condition can be changed
    let new_targets = copy_targets.iter().map(|(value, block)| {
        /// TODO: for in header part condition 1, for other part condition 2
        // if block==BasicBlock::from_usize(header.index()) {
        if value==0{
            (value, new_t1)
        } else {
            (value, block)
        }
    });
    let switch_targets = SwitchTargets::new(new_targets, otherwise);

    /// new terminator kind
    let new_target_terminator_kind = TerminatorKind::SwitchInt {
        discr: copy_op.to_copy(),
        targets: switch_targets,
    };

    // a update terminator kind
    bb_terminator.kind = new_target_terminator_kind;
}


/// copy, modify targets, insert
pub fn _insert_switchint<'tcx>(bbs: &mut IndexVec<BasicBlock, BasicBlockData<'tcx>>, copy: BasicBlock, t1: BasicBlock, t2:BasicBlock) {

    let TerminatorKind::SwitchInt {
        discr: copy_op,
        targets: copy_targets
    } = &bbs[copy].terminator().kind else {
        println!("Terminator kind of copy is not SwitchInt");
        unreachable!()
    };

    /// SwitchTargets
    let new_targets = copy_targets.iter().map(|(value, block)| {
        if value==0 {
            println!("value is 0 {:?} {:?} {:?}", value, block, bbs[block].terminator().kind);
            (value, t1)
        } else {
            (value, block)
        }
    });
    let new_switch_targets = SwitchTargets::new(new_targets, t2);

    /// discr
    let copy_op = match copy_op {
        Operand::Move(x) => Operand::Copy(*x),
        Operand::Copy(x) => Operand::Copy(*x),
        Operand::Constant(x) => Operand::Constant(x.clone()),
    };

    /// BasicBlockData with SwitchInt kind
    let new_bbd = BasicBlockData::new(Some(Terminator {
        source_info: bbs[copy].terminator().source_info,
        kind: TerminatorKind::SwitchInt {
            discr: copy_op,
            targets: new_switch_targets,
        },
    }));

    bbs.push(new_bbd);
}


pub fn _decide_and_change_target<'tcx>(bbs: &mut IndexVec<BasicBlock, BasicBlockData<'tcx>>,
                                       change_bb: BasicBlock,
                                       _t_goto: BasicBlock,
                                       _new_t1: BasicBlock, _new_t2:BasicBlock, _keep_otherwise: bool) {
    // let bb = bbs.get_mut(change_bb).expect("get bb to be changed.");
    /// bb == bbs[change_bb]
    let bb_terminator = bbs[change_bb].terminator.as_ref().expect("terminator kind check only").clone();

    if let Some(_) = bb_terminator.kind.as_goto() {
        println!("Input Basic block is Goto Type!");
        // TODO: [fix] remove temporarily
        // change_target_goto(bbs, change_bb, t_goto);
    } else {
        // TODO: [fix] assume if it's not goto, it is SwitchInt
        println!("Input Basic block is not Goto Type! It is SwitchInt");
        // TODO: [fix] remove temporarily
        // change_targets_switchint(bbs, change_bb, new_t1, new_t2, keep_otherwise);
    }
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
                                       body: &mut Body<'tcx>   )
                                       -> NodeIndex {
    println!("SCC in get back edges {:?}", scc);

    let mut back_edges :Vec<(NodeIndex, NodeIndex)> = vec!();
    let mut inner_edges :Vec<(NodeIndex, NodeIndex)> = vec!();
    let mut remove = false;

    for edge in g.clone().raw_edges() {

        let mut test_g = g.clone();
        println!("header, source, target {:?} {:?} {:?}", header.index(), edge.source().index(), edge.target().index());
        if scc.contains(&edge.source()) && scc.contains(&edge.target())
            && edge.target() == header {
            let Some(edge_idx) = test_g.find_edge(edge.source(), edge.target()) else {
                continue;
            };

            // assume
            test_g.remove_edge(edge_idx);
            println!("remove edge {:?} -> {:?}", edge.source(), edge.target());

            let mut dfs_res = vec!();
            let mut dfs = Dfs::new(&test_g, edge.source());

            while let Some(visited) = dfs.next(&test_g) {
                dfs_res.push(visited.index());
            }
            println!("dfs_res {:?}", dfs_res);

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
        /// TODO: fix
        /// remove temporarily
        // insert_goto(bbs, BasicBlock::from_usize(back_edges[0].0.index()),BasicBlock::from_usize(header.index()));

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
            // TODO: decide if bb kind is goto or switchInt
            // now assuming it's goto
            // change edge latch->header to latch->new_single_latch
            // TODO: remove dummy things
            /// Assumption: one edge of the branch is backedge and other one is normal edge -> not possible
            /// Assumption: every latch's terminator type is GOTO
            change_target_goto(body, BasicBlock::from_usize(latch.index().into()), BasicBlock::from_usize(new_latch_idx));
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

/// TODO: need to check
// pub fn _mir_get_predecessors_of(header: NodeIndex, g:&Graph<usize, String>) ->Vec<NodeIndex> {
// get predecessors using mir
// let mut predecessors2: Vec<BasicBlock>= Vec::new();
// for (bb, bb_data) in bbs.iter_enumerated() {
//     let Terminator {kind, ..} = bb_data.terminator();
//         if let Some(tmp) = kind.as_goto() { // i should consider both "goto" and "switchInt"
//         // if kind.as_goto() == h{
//         //     if bbs[tmp] == h.clone() {
//             if tmp == BasicBlock::from_usize(header.index().clone()) {
//                 predecessors2.push(bb);
//             }
//         // if let Some(TerminatorKind::Goto {target}) = kind.as_goto() {
//         //     if target == h {
//         }
// }
// println!("predecessors {:?}", predecessors2);
// }

use rustc_middle::mir::SccInfo;
//
// #[derive(Debug, Clone)]
// pub struct SccInfo {
//     _id: i32,
//     _n_type: usize,
// }

// pub enum SccInfo {
//     ID(usize),
//     NodeType(usize),    // H: 1, L: 2, X: 3
// }
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
    let scc_info = SccInfo::new(*scc_id as usize, 1);
    let scc_info2 = SccInfo::new(*scc_id as usize, 1);
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
        let scc_info = SccInfo::new(*scc_id as usize, 2);
        let scc_info2 = SccInfo::new(*scc_id as usize, 2);
        scc_info_stk.get_mut(&single_latch).map(|stk| stk.push(scc_info));
        body.scc_info[single_latch.index()].push(scc_info2);
    }

    // 3. mark 'X'
    for node in scc.clone() {
        if node != loop_header && node != single_latch {
            let scc_info = SccInfo::new(*scc_id as usize, 3);
            let scc_info2 = SccInfo::new(*scc_id as usize, 3);
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

// ========================================

// rustc_middle::mir::traversal::Preorder
use rustc_middle::mir::traversal::preorder;

fn _find_scc<'tcx>(_tcx: TyCtxt<'tcx>, body: &Body<'tcx> /*body: &mut Body<'tcx>*/) {

    let mut res = preorder(body);
    // for p in res.iter() {
    //     println!("preorder res {:?}", p.Item);

    // }
    let mut v = vec!();
    while let Some(visited) = res.next() {
        v.push(visited.0);
    }
    println!("{:?}", v);
}