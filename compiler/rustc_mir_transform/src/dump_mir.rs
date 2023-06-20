//! This pass just dumps MIR at a specified point.

use std::fs::File;
use std::io;

use crate::MirPass;
// use rustc_middle::mir::write_mir_pretty;
// use rustc_middle::mir::Body;
use rustc_middle::ty::TyCtxt;
use rustc_session::config::{OutFileName, OutputType};

// use std::collections::HashMap;
use rustc_data_structures::fx::FxHashMap;

use petgraph::Graph;
use petgraph::dot::{Dot, Config};
use petgraph::algo::kosaraju_scc;
use petgraph::algo::toposort;
use petgraph::prelude::NodeIndex;
use petgraph::visit::Dfs;

pub struct Marker(pub &'static str);

use rustc_middle::mir::*; // visit
use rustc_ast::ast::{InlineAsmOptions, InlineAsmTemplatePiece};
// use rustc_middle::mir::patch::MirPatch;
// use rustc_middle::

impl<'tcx> MirPass<'tcx> for Marker {
    fn name(&self) -> &'static str {
        self.0
    }

    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        println!("YuNJI: in mir dump");
        let def_id = body.source.def_id();
        if &tcx.def_path_str(def_id) == "fuzz_target" {
            println!("YuNJI: FUZZ target");
            // dummy_mir(tcx, body);
            // println!(" before add test {:?} {:?}", body.basic_blocks.len(), body.basic_blocks);
            println!(" before add test {:?}", body.basic_blocks.len());
            for i in 0..body.basic_blocks.len() {
                println!("   * {:?}: {:?}", i, body.basic_blocks[i.into()].terminator);
            }
            test_add(tcx, body);
            // println!("after");
            println!(" after add test {:?}", body.basic_blocks.len());
            for i in 0..body.basic_blocks.len() {
                println!("   * {:?}: {:?}", i, body.basic_blocks[i.into()].terminator);
            }
            // * 0: Some(Terminator { source_info: SourceInfo { span: ../hello_world/src/main9.rs:27:5: 41:6 (#0), scope: scope[1] }, kind: goto -> 1 })
            // * 1: Some(Terminator { source_info: SourceInfo { span: ../hello_world/src/main9.rs:27:11: 27:21 (#57), scope: scope[1] }, kind: switchInt(move _4) -> [0: 5, otherwise: 2] })
            // * 2: Some(Terminator { source_info: SourceInfo { span: ../hello_world/src/main9.rs:29:12: 29:17 (#58), scope: scope[2] }, kind: switchInt(move _10) -> [0: 4, otherwise: 3] })
            // * 3: Some(Terminator { source_info: SourceInfo { span: ../hello_world/src/main9.rs:31:13: 31:21 (#0), scope: scope[2] }, kind: goto -> 6 })
            // * 4: Some(Terminator { source_info: SourceInfo { span: ../hello_world/src/main9.rs:35:13: 35:21 (#0), scope: scope[2] }, kind: goto -> 6 })
            // * 5: Some(Terminator { source_info: SourceInfo { span: ../hello_world/src/main9.rs:44:2: 44:2 (#0), scope: scope[0] }, kind: return })
            // * 6: Some(Terminator { source_info: SourceInfo { span: no-location (#0), scope: scope[1] }, kind: goto -> 1 })
            // * 7: Some(Terminator { source_info: SourceInfo { span: ../hello_world/src/main9.rs:27:11: 27:21 (#57), scope: scope[1] }, kind: switchInt(_4) -> [0: 5, otherwise: 4] })

            let mut index_map: Vec<NodeIndex> = vec!();
            let mut scc_info_stk: FxHashMap<NodeIndex, Vec<SccInfo>> = Default::default();
            let g = mir_to_petgraph(tcx, body, &mut index_map);

            let mut scc_id: i32 = 1;
            let mut copy_graph = g.clone();
            loop {
                let mut stop = true;
                let mut scc_list = kosaraju_scc(&copy_graph);
                println!("SCC ={:?}", scc_list.clone());
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
        }
        // insert_bb(tcx, body);


        // find_scc(tcx, body);

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
// use rustc_index::vec::IndexVec;
// /// Ensures that the `otherwise` branch leads to an unreachable bb, returning `None` if so and a new
// /// bb to use as the new target if not.
// fn ensure_otherwise_unreachable<'tcx>(
//     bbs: &mut IndexVec<usize, BasicBlockData<'tcx>>,
//     targets: &SwitchTargets,
// ) -> Option<BasicBlockData<'tcx>> {
//     let otherwise = targets.otherwise();
//     let bb = &bbs[otherwise.into()];
//     if bb.terminator().kind == TerminatorKind::Unreachable
//         && bb.statements.iter().all(|s| matches!(&s.kind, StatementKind::StorageDead(_)))
//     {
//         return None;
//     }
//
//     let mut new_block = BasicBlockData::new(Some(Terminator {
//         source_info: bb.terminator().source_info,
//         kind: TerminatorKind::Unreachable,
//     }));
//     new_block.is_cleanup = bb.is_cleanup;
//     Some(new_block)
// }

fn test_add<'tcx>(_tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
    let bbs = body.basic_blocks_mut();
    // basicblock-> statements, "terminator", is_cleanup
    // terminator-> source_info, kind
    // kind (TerminatorKind) -> Goto | SwitchInt
    // SwitchInt -> discr, targets
    // discr -> Operand<'tcx>
    // targets = SwitchTargets -> values(SmallVec<[u128; 1]>), targets(SmallVec<[BasicBlock; 2]>)
    //     /// new(iterator(1.1), otherwise(1.2))
    //
    //
    // index of basic block to copy...

    // 1. new targets (switchTargets)
        // 1.1. iterator
        // 2.1. otherwise
    // 2. discr(parent's)
    // 3. source_info
    // 4. terminator
    // 5. basicblcok
    let i = 1;
    let copy = BasicBlock::from_usize(i);
    let new_dest = BasicBlock::from_usize(4);
    // let go_1 m
    let TerminatorKind::SwitchInt {
        discr: copy_op,
        targets: copy_targets
    } = &bbs[copy].terminator().kind else {
        println!("faile to declare");
        unreachable!()
    };




    let copy_op = match copy_op {
        Operand::Move(x) => Operand::Copy(*x),
        Operand::Copy(x) => Operand::Copy(*x),
        Operand::Constant(x) => Operand::Constant(x.clone()),
    };
    // let copy_ty = copy_op.ty(body.local_decls(), tcx);
    // let statements_before = bbs[copy].statements.len();
    // let copy_end = Location { block: copy, statement_index: statements_before};
    // let mut patch = MirPatch::new(body);

    // 1.1.
    let new_targets = copy_targets.iter().map(|(value, block)| {
        if value ==0 {
            println!("check target 1 {:?} {:?} {:?}", value, block, bbs[block].terminator().kind);
            (value, copy)
        } else if value==1 {
            println!("check target 2 {:?} {:?} {:?}", value, block, bbs[block].terminator().kind);
            (value, block)

        }
        // let TerminatorKind::SwitchInt {targets, ..} = &bbs[block].terminator().kind else {
        //     unreachable!()
        //     // return
        // };
        else {
            // (value, targets.target_for_value(value))
            println!("check target 3 {:?} {:?} {:?}", value, block, bbs[block].terminator().kind);
            (value, block)
        }
    });

    // 1.
    // let TerminatorKind::SwitchInt {
    //     targets: otherwise_t,
    //     discr: o_op,
    // } = &copy.kind else {
    //     // return None;
    //     println!("error 1");
    //     unreachable!()
    // };

    let eq_targets = SwitchTargets::new(new_targets, new_dest);

    let eq_b = BasicBlockData::new(Some(Terminator {
        source_info: bbs[copy].terminator().source_info,
        kind: TerminatorKind::SwitchInt {
            discr: copy_op,
            targets: eq_targets,
        },
    }));
    bbs.push(eq_b);

    let root = BasicBlock::from_usize(0);
    // method 1
    let goto_new = TerminatorKind::Goto {
        target: BasicBlock::from_usize(7),
        // target: root,
    };

    // method 2
    // if let TerminatorKind::Goto {target} = &bbs[root].terminator().kind {
    //     *target =  BasicBlock::from_usize(7);
    //     // target = eq_b;
    // } else {
    //     println!("faile to declare");
    //     unreachable!()
    // };

    let b2change = bbs.get_mut(root).expect("get root");
    // let mut new_term_for_b = b2change.terminator.as_ref().expect("no ter").clone();
    let mut new_term_for_b = b2change.terminator.as_mut().expect("n t");
    new_term_for_b.kind = goto_new;

    println!("after change root {:?}", BasicBlock::from_usize(0));



    // bbs[root].terminator().kind = goto_new;

    // source_info = self.source_info()
    // let si = body.basic_blocks[0].statement.source_info;
    // body.source_info(body.span);

    /*
    // for bb in body.basic_blocks.indices() {
        for bb in bbs.indices() {
        println!("processing block {:?}", bb);
        //
        // let Some(discriminant_ty) = get_switched_on_type(&body.basic_blocks[bb], tcx, body) else {
        //     continue;
        // };
        //
        // let layout = tcx.layout_of(tcx.param_env(body.source.def_id()).and(discriminant_ty));
        //
        // let allowed_variants = if let Ok(layout) = layout {
        //     variant_discriminants(&layout, discriminant_ty, tcx)
        // } else {
        //     continue;
        // };
        //
        // trace!("allowed_variants = {:?}", allowed_variants);

        if let TerminatorKind::SwitchInt { targets, .. } =
            &mut bbs[bb].terminator_mut().kind
        {
            let mut new_targets = SwitchTargets::new(
                // targets.iter().filter(|(val, _)| body.basic_blocks.as_ref().contains(val)),
                // targets.iter().filter(|(val, _)| body.basic_blocks.as_mut().clone().contains(val)),
                targets.iter(),
                targets.otherwise(),
            );

            if let Some(updated) = ensure_otherwise_unreachable(bbs, &new_targets) {
                let new_otherwise = bbs.push(updated);
                *new_targets.all_targets_mut().last_mut().unwrap() = new_otherwise;
            }

            if let TerminatorKind::SwitchInt { targets, .. } =
                &mut bbs[bb].terminator_mut().kind
            {
                *targets = new_targets;
            } else {
                // unreachable!()
                println!("unreachable1");
            }
        } else {
            println!("unreachable2");
            // unreachable!()
        }
        let original_last_block = bbs.get_mut(BasicBlock::from_usize(1)).expect("No last block!");
        let new_terminator = original_last_block.terminator.as_ref().expect("no terminator").clone();


        // new_terminator.kind = asm_terminator_kind;
        let new_bb = BasicBlockData {
            statements: vec![],
            terminator: Some(new_terminator.to_owned()),
            is_cleanup: false,
        };
        bbs.push(new_bb);

    }
*/

}


fn _dummy_mir<'tcx>(_tcx: TyCtxt<'tcx>, _body: &mut Body<'tcx>) {
    // let bbs = body.basic_blocks_mut();
    /*
    let _switch_block_idx = 1;
    let block_to_copy_terminator = bbs.get_mut(BasicBlock::from_usize(1)).expect("no bb");
    let block_to_copy_terminator = block_to_copy_terminator.terminator.as_ref().expect("no terminator").clone();
    let mut new_bb = BasicBlockData {
        statements: vec![],
        terminator: Some(Terminator {
            source_info: SourceInfo::dummy(),
            kind: TerminatorKind::Unreachable,
        }),
        is_cleanup: false,
    };

    let modified_terminator = match block_to_copy_terminator.kind {
        TerminatorKind::SwitchInt { discr, ref targets, ..} => {
            let modified_targets = targets.iter().map(|target| {
                // let modified_targets = targets.iter().map(|&target| {
                if target.0==2 {
                    4
                } else {
                    1
                }
            }).collect();
            TerminatorKind::SwitchInt {discr, targets: modified_targets}
        },
        _ => panic!("Error switch"),
    };
    new_bb.terminator = Some(Terminator {
        source_info: SourceInfo::dummy(),
        kind: modified_terminator,
    });

    bbs.push(new_bb);
    println!("after change {:?} {:?}", new_bb, bbs.clone());



    // copy "As switch" terminator
    // i will copy #1. (->2, ->5)
    let block_to_copy_terminator = bbs.get_mut(BasicBlock::from_usize(1)).expect("no bb");
    let _store_original_terminator = block_to_copy_terminator.terminator.as_ref().expect("no terminator").clone();
    let get_terminator_and_modify = block_to_copy_terminator.terminator.as_mut().expect("no terminator");

    // new terminator kind
    let v = SmallVec(0);
    let t : SmallVec<BasicBlock> = SmallVec(BasicBlock::from_usize(1), BasicBlock::from_usize(4));
    let new_targets = SwitchTargets {
        values: v,
        targets: t,
    };
    let new_op : Operand<'tcx> = Operand::Copy(Place::Local(Local::new(0)));
    let new_terminator_kind = TerminatorKind::SwitchInt {
        discr: new_op,
        targets: new_targets,
    };
    get_terminator_and_modify.kind = new_terminator_kind;
    // targets: vec![BasicBlock::new(11), BasicBlock::new(14)],
    // let mut k = get_terminator_and_modify.kind.as_switch();
    // // let Some(kk) = get_terminator_and_modify.kind.as_switch() else {
    // //     println!("error");
    // //     return;
    // // };
    // let kkk = k.unwrap().1.all_targets_mut();
    // println1
    // for t in &mut kkk {
    //     println!("all targets1: {:?}", t);
    //     *t = BasicBlock::from_usize(1);
    // }
    println!("after change {:?} {:?}", kkk, bbs.clone());
*/

    // let goto7 = TerminatorKind::Goto {
    //     target: BasicBlock::from_usize(7),
    // // };
    // let bb1 = bbs.get_mut(BasicBlock::from_usize(1)).expect("no bb in mir");
    // // let mut t_bb_store = bb1.terminator.as_ref().expect("no terminator").clone();
    // let t_bb_store = bb1.terminator.as_mut().expect("no terminator");
    // // t_bb_store.kind);
    // let tmp = t_bb_store.kind.as_switch().unwrap().1;
    // for t in tmp.all_targets() {
    //     println!("all targets: {:?}", t);
    //     // *t = BasicBlock::from_usize(1);
    // }
    // let mut t_bb_store = bb1.terminator.as_mut().expect("no terminator").clone();

    // let bb0 = bbs.get_mut(BasicBlock::from_usize(0)).expect("no bb in mir");
    // let mut t_bb0 = bb0.terminator.as_mut().expect("no terminator");
    // t_bb0.kind = goto7;



    // let a = [BasicBlock::from_usize(4)];
    // let mut iter = a.iter();
    // let target = SwitchTargets(iter, BasicBlock::from_usize(2));
    //
    // let op = Operand {
    //     Copy, Move, Constant
    // }
    // let asm_terminator_kind = TerminatorKind::SwitchInt {
    //     discr: ,
    //     targets: target,
    // };

    // [0: 11, otherwise: 2]
    // let _len = bbs.len();
    // let original_last_block = bbs.get_mut(BasicBlock::from_usize(0)).expect("No last block!");

    // let h = bbs.get_mut(BasicBlock::from_usize(2)).expect("no bb in mir");
    // let temr = h.terminator.as_ref().expect("no terminator").clone();
    // println!("RUn dummy mir \nterm info {:?}", temr.kind);


    // let mut new_terminator = original_last_block.terminator.as_ref().expect("no terminator").clone();
    // let mut new_terminator = original_last_block.terminator.as_mut().expect("no terminator");
    // let original_last_block_terminator = original_last_block.terminator_mut();
    // let original_last_block_terminator = original_last_block.terminator();
    // new_terminator.kind = asm_terminator_kind;

    // let new_bb = BasicBlockData {
    //     statements: vec![],
    //     terminator: Some(t_bb_store.to_owned()),
    //     is_cleanup: false,
    // };
    //
    // bbs.push(new_bb);

}

fn _insert_bb<'tcx>(tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
    // let bbs = mir.basic_blocks_mut();
    let bbs = body.basic_blocks_mut();
    // let bbs = tcx.body().basic_blocks_mut();
    // let bbs = tcx.promoted.basic_blocks_mut();
    // let bbs = tcx.hir().body().basic_blocks_mut();

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
    // let mut new_terminator = original_last_block.terminator.as_mut().expect("no terminator");
    let original_last_block_terminator = original_last_block.terminator_mut();
    // let original_last_block_terminator = original_last_block.terminator();
    new_terminator.kind = asm_terminator_kind;

    let new_bb = BasicBlockData {
        statements: vec![],
        terminator: Some(original_last_block_terminator.to_owned()),
        is_cleanup: false,
    };

    bbs.push(new_bb);

}

fn mir_to_petgraph<'tcx>(_tcx: TyCtxt<'tcx>, body: &Body<'tcx>, arr: &mut Vec<NodeIndex>)
    -> Graph::<usize, String>{
    let mut g = Graph::<usize, String>::new();

    let mut cnt: usize = 0;
    for _ in body.basic_blocks.iter() {
        let node = g.add_node(cnt);
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

    println!("## Generated PETGRAPH ##");
    for edge in g.clone().raw_edges() {
        println!("{:?}", edge);
    }
    for n in g.clone().raw_nodes() {
        println!("{:?}", n);
    }
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

pub fn transform_to_single_header<'tcx>(scc: &mut Vec<NodeIndex>,
                        headers: Vec<NodeIndex>,
                        g: &mut Graph<usize, String>,
                        scc_info_stk: &mut FxHashMap<NodeIndex, Vec<SccInfo>>,
                        arr: &mut Vec<NodeIndex>,
                                  _tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>)
                        -> NodeIndex {
    let new_node = g.add_node(777);
    scc.push(new_node);
    arr.push(new_node);
    scc_info_stk.insert(new_node, vec!());

    // add to petgraph

    // add to mir body
    let bbs = body.basic_blocks_mut();

    for header in headers {
        let predecessors = get_predecessors_of(header, g);
        for pred in predecessors {
            // redirect
            let Some(edge_to_remove) = g.find_edge(pred, header) else {
                continue;
            };
            g.remove_edge(edge_to_remove);
            g.update_edge(pred, new_node, String::from("HEADER"));
            g.update_edge(new_node, header, String::from("HEADER"));
        }

        let h = bbs.get_mut(BasicBlock::from_usize(header.index())).expect("no bb in mir");
        println!("header basicblock {:?}", h);
    }



    // let bbs = body.basic_blocks_mut();
    // // let template_piece = InlineAsmTemplatePiece::String(String::from("new header"));
    // // let template2 = [template_piece];
    // // let template = tcx.arena.alloc_from_iter(template2);
    //
    // let asm_terminator_kind = TerminatorKind::Goto {
    //     target: bbs.next_index()
    // };
    //
    // let len = bbs.len();
    // let prev_last_b = bbs.get_mut(BasicBlockData::from_usize(len-1)).expect("no last block");
    //
    // let mut new_terminator = prev_last_b.terminator.as_ref().expect("no terminator").clone();
    // new_terminator.kind = asm_terminator_kind;
    //
    // let prev_last_b_terminator = prev_last_b.terminator_mut();
    //
    // let new_bb = BasicBlockData {
    //     statements: vec![],
    //     terminator: Some(prev_last_b_terminator.to_owned()),
    //     is_cleanup: false,
    // };
    // bbs.push(new_bb);


    return new_node;
}

pub fn transform_to_single_latch(scc: &mut Vec<NodeIndex>,
                        header: NodeIndex,
                        g: &mut Graph<usize, String>,
                        scc_info_stk: &mut FxHashMap<NodeIndex, Vec<SccInfo>>,
                        arr: &mut Vec<NodeIndex>
                                 )
                        -> NodeIndex {
    println!("SCC in get back edges {:?}", scc);

    let mut back_edges :Vec<(NodeIndex, NodeIndex)> = vec!();
    let mut inner_edges :Vec<(NodeIndex, NodeIndex)> = vec!();
    let mut remove = false;

    for edge in g.clone().raw_edges() {

        let mut test_g = g.clone();
        println!("header, source, target {:?} {:?} {:?}", header, edge.source(), edge.target());
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
                println!("back_edges  {:?}", back_edges);
                remove = true;
            }
        }
    }
    if remove == false {
        // remove both
        println!("there is no proper outer back edges, instead both can be latches {:?}", inner_edges);
        back_edges = inner_edges;
    }

    // single LATCH
    let single_latch;
    let new_node;
    if back_edges.len() > 1 {
        new_node = g.add_node(999);
        single_latch = new_node;
        arr.push(new_node);
        scc_info_stk.insert(new_node, vec!());
        for back_edge in back_edges {
            // redirect
            let latch = back_edge.0;
            let Some(edge_to_remove) = g.find_edge(latch, header) else {
                continue;
            };
            g.remove_edge(edge_to_remove);
            g.update_edge(latch, new_node, String::from("LATCH"));
            g.update_edge(new_node, header, String::from("LATCH"));
        }

    } else {
        single_latch = back_edges[0].0;
    }
    scc.push(single_latch);

    // 24 [0, 1, 2, 7, 3, 3, 3, 8, 7, 4, 8, 7, 3, 8, 7, 3, 8, 7, 4, 8, 7, 4, 5, 6]
    // 21 [0, 1, 2, 7, 3, 3, 3, 8, 7, 4, 8, 7, 3, 8, 7, 4, 8, 7, 4, 5, 6]

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

#[derive(Debug)]
pub struct SccInfo {
    _id: i32,
    _n_type: char,
}

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
    let scc_info = SccInfo {
        _id: *scc_id,
        _n_type: 'H',
    };
    scc_info_stk.get_mut(&loop_header).map(|stk| stk.push(scc_info));

    // 2. mark latch
    single_latch = transform_to_single_latch(scc, loop_header, g, scc_info_stk, arr);
    if scc.len() != 1 {
        // only if it is not a self loop, mark as Latch
        let scc_info = SccInfo {
            _id: *scc_id,
            _n_type: 'L',
        };
        scc_info_stk.get_mut(&single_latch).map(|stk| stk.push(scc_info));
    }

    // 3. mark 'X'
    for node in scc.clone() {
        if node != loop_header && node != single_latch {
            let scc_info = SccInfo {
                _id: *scc_id,
                _n_type: 'X',
            };
            scc_info_stk.get_mut(&node).map(|stk| stk.push(scc_info));
        }
    }

    println!("====================== Break Down ======================");
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