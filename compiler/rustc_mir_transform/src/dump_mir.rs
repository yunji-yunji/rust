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
    // for edge in g.clone().raw_edges() {
    //     println!("{:?}", edge);
    // }
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

pub fn transform_to_single_header(scc: &mut Vec<NodeIndex>,
                        headers: Vec<NodeIndex>,
                        g: &mut Graph<usize, String>,
                        scc_info_stk: &mut FxHashMap<NodeIndex, Vec<SccInfo>>,
                        arr: &mut Vec<NodeIndex>)
                        -> NodeIndex {
    let new_node = g.add_node(777);
    arr.push(new_node);
    scc_info_stk.insert(new_node, vec!());

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
    }
    scc.push(new_node);
    return new_node;
}

pub fn transform_to_single_latch(scc: &mut Vec<NodeIndex>,
                        header: NodeIndex,
                        g: &mut Graph<usize, String>,
                        scc_info_stk: &mut FxHashMap<NodeIndex, Vec<SccInfo>>,
                        arr: &mut Vec<NodeIndex>)
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
    _tcx: TyCtxt<'tcx>, _body: &mut Body<'tcx>,
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
        loop_header = transform_to_single_header(scc, headers, g, scc_info_stk, arr);
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