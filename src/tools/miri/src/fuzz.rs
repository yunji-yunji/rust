use petgraph::Graph;
use petgraph::dot::{Dot, Config};
use petgraph::algo::kosaraju_scc;
use petgraph::algo::toposort;
use petgraph::prelude::NodeIndex;
use petgraph::visit::Dfs;

use rustc_middle::ty::TyCtxt;
use rustc_middle::mir::*;

use std::fs::File;
use std::io::{BufReader, BufRead};
use std::collections::HashMap;

pub fn default_g () -> Graph<usize, String> {
    let my_g = Graph::<usize, String>::new();
    my_g
}

pub fn is_cycle(orig: Graph<usize, String>, scc:Vec<NodeIndex>) -> bool {
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
    println!("Test if {:?} has cycle" , scc);
    match toposort(&new, None){
        Ok(_order) => {
            println!("no cycle");
            return false;
        },
        Err(err) => {
            println!("cycle {:?}",err);
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

pub fn get_single_latch(scc: &mut Vec<NodeIndex>, 
    header: NodeIndex, 
    g: &mut Graph<usize, String>,
    scc_info_stk: &mut HashMap<NodeIndex, Vec<SccInfo>>,
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

pub fn break_down_and_mark(scc: &mut Vec<NodeIndex>, scc_id: &mut i32, 
    g: &mut Graph<usize, String>, 
    scc_info_stk: &mut HashMap<NodeIndex, Vec<SccInfo>>,
    arr: &mut Vec<NodeIndex>) {
    
    let loop_header;
    let single_latch;

    println!("====================== Transform & Mark ======================");
    // 1. mark header
    let headers = find_all_headers(scc.clone(), g);
    let new_node;
    if headers.len() ==1 {
        println!("[1] if there is a single header {:?}", headers);
        loop_header = headers[0];
    } else {
        println!("[2] if there are multiple headers {:?}", headers);
        new_node = g.add_node(777);
        loop_header = new_node;
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
    }

    let scc_info = SccInfo {
        _id: *scc_id, 
        _n_type: 'H', 
    };
    scc_info_stk.get_mut(&loop_header).map(|stk| stk.push(scc_info));

    // 2. mark latch
    single_latch = get_single_latch(scc, loop_header, g, scc_info_stk, arr);
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

pub fn my_app <'tcx>(_tcx: TyCtxt<'tcx>, _body: &Body<'_>) 
-> (Graph<usize, String>, Graph<usize, String>, Vec<NodeIndex>) {
    println!("\n------------ TEST graph ----------");
    let mut g = Graph::<usize, String>::new();
    let mut backup_g = Graph::<usize, String>::new();
    let mut arr0 :Vec<NodeIndex> = vec!();

    let mut scc_info_stk : HashMap<NodeIndex, Vec<SccInfo>> = HashMap::new();
    // ===================== create dummy graph
    let case = 1;
    let num_node;
    if case ==1 {
        num_node = 7;
    } else {
        num_node = 14;
    }
    for i in 0..num_node {
        let node1 = g.add_node(i);
        scc_info_stk.insert(node1, vec!());
        let _node2 = backup_g.add_node(i);
        arr0.push(node1);
    }
    println!("{:?} {:?}", arr0, arr0.len());
    println!("Initial stack info hash map {:?}\n\n", scc_info_stk);

    if case == 1 {
        g.update_edge(arr0[0], arr0[1], String::from("1"));
        g.update_edge(arr0[1], arr0[2], String::from("2"));
        g.update_edge(arr0[2], arr0[3], String::from("3"));
        g.update_edge(arr0[2], arr0[4], String::from("4"));
        g.update_edge(arr0[3], arr0[4], String::from("5"));
        g.update_edge(arr0[4], arr0[3], String::from("6"));
        g.update_edge(arr0[4], arr0[5], String::from("7"));
        g.update_edge(arr0[3], arr0[5], String::from("8"));
        g.update_edge(arr0[5], arr0[6], String::from("9"));
        g.update_edge(arr0[3], arr0[3], String::from("10"));
    
    } else {
        // big graph
        g.update_edge(arr0[0], arr0[1], String::from("1"));
        g.update_edge(arr0[1], arr0[2], String::from("2"));
        g.update_edge(arr0[2], arr0[3], String::from("3"));
        g.update_edge(arr0[3], arr0[5], String::from("4"));
        g.update_edge(arr0[2], arr0[4], String::from("5"));
        g.update_edge(arr0[4], arr0[5], String::from("6"));
        g.update_edge(arr0[5], arr0[6], String::from("7"));
        g.update_edge(arr0[6], arr0[7], String::from("8"));
        g.update_edge(arr0[7], arr0[8], String::from("9"));
        g.update_edge(arr0[8], arr0[5], String::from("10"));
        g.update_edge(arr0[7], arr0[9], String::from("11"));
        g.update_edge(arr0[9], arr0[10], String::from("12"));
        g.update_edge(arr0[9], arr0[9], String::from("13"));
        g.update_edge(arr0[10], arr0[11], String::from("14"));
        g.update_edge(arr0[11], arr0[12], String::from("15"));
        g.update_edge(arr0[10], arr0[1], String::from("16"));
        g.update_edge(arr0[11], arr0[10], String::from("17"));
        g.update_edge(arr0[12], arr0[10], String::from("18"));
        g.update_edge(arr0[12], arr0[13], String::from("19"));
        g.update_edge(arr0[11], arr0[1], String::from("20"));
        // g.update_edge(arr0[9], arr0[11], String::from("21"));
    }
    println!("before transform graph\n{:?}", Dot::with_config(&g, &[Config::EdgeIndexLabel]));

    let mut scc_id : i32 = 1;
    let mut copy_graph = g.clone();
    loop {
        let mut stop  =true;
        let mut scc_list = kosaraju_scc(&copy_graph);
        println!("SCC ={:?}", scc_list.clone());
        for scc in &mut scc_list {
            let is_cycle = is_cycle(copy_graph.clone(), scc.clone());
            if is_cycle == true {
                stop = false;
                break_down_and_mark(scc, &mut scc_id, 
                &mut copy_graph, &mut scc_info_stk
                , &mut arr0);
            }
        }
        println!("after break down graph = \n{:?}", Dot::with_config(&copy_graph, &[Config::EdgeIndexLabel]));

        if stop==true {
            println!("\nBREAK!\n final SCC ={:?}\n\nSCC INFO STACK", scc_list.clone());
            for (n_idx, &ref stack) in scc_info_stk.iter() {
                println!("node: {:?} == {:?}", n_idx, stack);
            }
            break;
        }
    }

    println!("\nafter ALL transformation: graph\n{:?}", Dot::with_config(&copy_graph, &[Config::EdgeIndexLabel]));
    generate_path3(copy_graph.clone(), &mut scc_info_stk, arr0.clone());

    return (copy_graph, g, arr0);
}



// ======= Generate final path (Discard repeated component) ============== //
pub fn generate_path3(_g: Graph::<usize, String>,
    scc_info_stk: &mut HashMap<NodeIndex, Vec<SccInfo>>,
    arr: Vec<NodeIndex>) -> Vec<i32> {

    #[derive(Debug)]
    struct Ele {
        counts: HashMap<Vec<i32>, usize>,
        temp_path: Vec<Vec<i32>>,
        prefix: Vec<i32>,
    }

    let mut fin : Vec<i32>;
    let limit : usize= 3;
    let mut stk :Vec<Ele> = vec!()                                                                                                                                 ;
    let mut is_loop = false;

    // dummy path
    let case = 1;
    let path: Vec<i32>;
    if case ==1 {
        path = vec![
            // 0, 1, 2, 7,  3,3, 3,3, 3, 3,3, 3, 7, 4, 7, 3, 7, 3, 7, 3,3, 7, 4, 7, 3, 7, 3, 7, 4, 7, 4, 5, 6];
            // [, 1, 2, 7, 3, 7, 4, 7, 3, 7, 3, 7, 4, 7, 3, 7, 3, 7, 4, 7, 4, 5, 6]
            0, 1, 2, 7, 3, 3,3 , 3, 3, 3, 3,3, 8, 7, 4, 8, 7, 3, 8, 7, 3, 8, 7, 4, 8, 7, 3, 8, 7, 3, 8, 7, 4, 8, 7, 4, 5, 6];
    } else {
        path = vec![0, 1, 2, 3,
        5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7,
         9, 10, 14, 1,2,3,

         5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7,
         9, 10, 11, 14, 1, 2, 3, 
         
         5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7,
         9, 10, 11, 14, 1, 2, 3, 
         
         5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7,
         9, 10, 11, 14, 1, 2, 3,
         
         5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7,
         9, 10, 11, 14, 1, 2, 3,
         
         5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7,
         9, 10, 14,
         
         1,2,3,
         5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7,
        9, 9, 9, 9, 9, 9 ,9, 
        10, 11, 10, 11, 10,  11, 10, 11, 10, 11, 
        12, 13];


        // [0, 
        
        // 1, 2, 3, 
        // 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 
        // 9, 10, 14, 
        
        // 1, 2, 3, 
        // 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 
        // 9, 10, 11, 14, 
        
        // 1, 2, 3, 
        // 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 
        // 9, 10, 11, 14, 
        
        // 1, 2, 3, 
        // 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 
        // 9, 10, 14, 
        
        // 1, 2, 3, 
        // 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 
        // 9, 
        
        // 10, 11, 10, 11, 10, 11, 
        // 12, 13]



        // expected
        // 0 
        // 1 2 3 "3" 9 10 14 
        // 1 2 3 "3" 9 10 11 14
        // 1 2 3 "3" 9 10 11 14
        // 1 2 3 "3" 9 10 11 14
        // 1 2 3 "3" 9 10 14
        // 1 2 3 "3"
        // 9 
        // "4"
        // 12
        // 13
        // => 
        // 0 
        // 1 2 3 "3" 9 10 14 
        // 1 2 3 "3" 9 10 11 14
        // 1 2 3 "3" 9 10 11 14
        // 1 2 3 "3" 9 10 11 14
        // 1 2 3 "3" 9 10 14
        // 1 2 3 "3"
        // 9 
        // "4"
        // 12
        // 13

    }

    println!("============= Generate Path ================");
    println!("dummy path INFO: {:?} {:?} ", path.len(), path.clone());
    fin = vec!();
    fin.push(path[0]);

    for idx in 0..path.len()-1 {
        let s : usize = path[idx] as usize;   // bb_n in i32 (todo => usize)
        let t : usize = path[idx+1].try_into().unwrap();
        println!("---------{:?} -> {:?}--------", s, t);
        let mut recorded = false;

        // ============= Exiting edge ============= //
        let mut s_idx = 0;
        let mut t_idx = 0;

        while s_idx < scc_info_stk[&arr[s]].len() 
            && t_idx < scc_info_stk[&arr[t]].len() 
            && scc_info_stk[&arr[s]][s_idx]._id == scc_info_stk[&arr[t]][t_idx]._id {
                s_idx += 1;
                t_idx += 1;
        }

        while s_idx < scc_info_stk[&arr[s]].len() {
            if let Some(mut prev) = stk.pop() {
                prev.temp_path.push(prev.prefix.clone());
                
                let sccid : i32 = scc_info_stk[&arr[s]][s_idx]._id * -1;
                if let Some(last) = stk.last_mut() {
                    last.prefix.push(sccid.try_into().unwrap());
                    for p in prev.temp_path {
                        for pp in p {
                            last.prefix.push(pp);
                        }
                    }
                    last.prefix.push(sccid.try_into().unwrap());
                } else {
                    // fin.push(sccid.try_into().unwrap()); // for debugging
                    for p in prev.temp_path {
                        for pp in p {
                            fin.push(pp);
                        }
                    }
                    // fin.push(sccid.try_into().unwrap()); // for debugging
                }
            }
            println!("[1] Exit edge");
            for e in &stk {
                println!("  * {:?}", e);
            }
            s_idx += 1;
            is_loop=false;
        } 

        // ============= Normal & Back edge ============= //

        s_idx = 0;
        t_idx = 0;
        while s_idx < scc_info_stk[&arr[s]].len() 
        && t_idx < scc_info_stk[&arr[t]].len() 
        && scc_info_stk[&arr[s]][s_idx]._id == scc_info_stk[&arr[t]][t_idx]._id {
            is_loop=true;
            
            if s==t || (scc_info_stk[&arr[s]][s_idx]._n_type == 'L' && scc_info_stk[&arr[t]][t_idx]._n_type == 'H') {
                if let Some(last) = stk.last_mut() {
                    if recorded==false {
                        last.prefix.push(t as i32);
                        recorded=true;
                    }
                    let mut content :Vec<i32> = vec!();
                    let mut prefix_to_key :Vec<i32> = vec!();
                    let mut k:i32;
                    let mut i=0;
                    while i<last.prefix.len() {
                        k = last.prefix[i];
                        while k < 0 {
                            i += 1;
                            if last.prefix[i] < 0 { break;}
                            content.push(last.prefix[i].try_into().unwrap());
                        }
                        content.push(last.prefix[i].try_into().unwrap());
                        prefix_to_key.push(k.try_into().unwrap()); 
                        i += 1;
                    } 
                    println!("content {:?}", content);
                    let mut flag = true;
                    if let Some(val) = last.counts.get_mut(&prefix_to_key) {
                        *val += 1;
                        if *val >= limit { flag = false;}
                    } else {
                        last.counts.insert(prefix_to_key, 1);
                    }
                    if flag {
                        last.temp_path.push(content);
                    }
                    last.prefix = vec!();
                }

                println!("[2] back edge" );
                for e in &stk {
                    println!("  * {:?}", e);
                }
                if s==t {
                    t_idx = scc_info_stk[&arr[t]].len();
                    println!("[2-1] self loop back edge" );
                    break;
                }

            } 
            else {
                if recorded==false {
                    stk.last_mut().unwrap().prefix.push(t as i32);
                    recorded=true;
                }
                println!("[3] normal edge" );
                for e in &stk {
                    println!("  * {:?}", e);
                }
                
            }
            s_idx += 1;
            t_idx += 1;
        }

        // ============= Entering edge (Header node) ============= //
        while t_idx < scc_info_stk[&arr[t]].len() {
            is_loop = true;

            let tmp;
            if recorded {
                tmp = vec!(vec!());
            } else {        // in case it never met back edge, push header node
                tmp = vec!(vec!(t as i32));
            }
            let el = Ele {
                counts: HashMap::new(), 
                temp_path: tmp, 
                prefix: vec!(),
            };

            stk.push(el);
            t_idx += 1;

            println!("[4] Entering edge (Push)" );
            for e in &stk {
                println!("  * {:?}", e);
            }
        }

        if is_loop == false {
            fin.push(t.try_into().unwrap());
            println!("[5] Not loop" );
            for e in &stk {
                println!("  * {:?}", e);
            }
        }
    }
    println!("\n");

    println!("Before remove markers: {:?} {:?}", fin.len(), fin);
    let mut res : Vec<i32> = vec![];
    for f in fin {
        if f>=0 {
            res.push(f);
        }
    }
    println!("RES: {:?} {:?}", res.len(), res);
    return res;
}


// ======================================= old code ======================================= //
// #[derive(Eq, Hash, PartialEq)]
// #[derive(Copy, Clone)]
// struct Inp {
//     x: i32,
//     y: i32,
//     n: i32,

// }
// fn printInp(i : Inp) {
//     print!(" {} {} {} ", i.x, i.y, i.n);
// }

// fn _save_test(i: Inp, path : &str, id : &mut i32) {
//     print!("TEST added to corpus = ");
//     printInp(i);

//     let mut name = format!("/home/y23kim/rust/corpus_dir2/n_inp{}", id);
//     while Path::new(&name).exists() {
//         *id += 1;
//         name = format!("/home/y23kim/rust/corpus_dir2/n_inp{}", *id);
//     }
//     let mut fi = fs::OpenOptions::new().append(true).create(true).open(name.clone()).expect("Fail to write yunji");

//     let d = format!("{}\n{}\n{}\n", i.x, i.y, i.n);
//     fi.write_all(d.as_bytes()).expect("yunji: fail to write");
// }

// fn evaluate_path(test: Inp, test_trace: Vec<usize>, m : &mut HashMap<Inp, Vec<usize>>) -> bool{
fn evaluate_path(test_path: Vec<usize>, final_paths : &mut Vec<Vec<usize>>) -> bool{

    // for path in m.values() {
    for path in &mut *final_paths {
        if path.to_vec() == test_path {
            println!("NOT interesting!");
            return false
        }
    }

    final_paths.push(test_path.clone());
    println!("interesting!");
    return true
}

pub fn generate_path(g: &mut Graph::<usize, String>, _new_g: &mut Graph::<usize, String>, 
    start : &mut usize, arr: Vec<NodeIndex>) -> Vec<i32> {
    let mut tmp: Vec<i32> = vec![];

    // ========================= parse step.rs result => paths ========================= //
    let mut paths: Vec<Vec<i32>> = vec![];
    let f = File::open("/home/y23kim/rust/fuzzer/output/res1").unwrap();
    let reader = BufReader::new(f);
    for line in reader.lines() {
        let data = line.unwrap();

        let numbers: Vec<i32> = data
            .split_whitespace()
            .map(|s| s.parse().expect("parse error"))
            .collect();

        // let mut fin: Vec<Vec<i32>> = vec![];
        let mut prev1: i32 = -1;

        for i in *start..numbers.len() {
            let n: i32 = numbers[i];

            if prev1 != n {
                if n == 0 {
                    paths.push(tmp.clone());
                    tmp = vec![];
                }
                tmp.push(n);
                prev1 = n;
            }
        }

        paths.push(tmp.clone());
        // i dont use this start
        *start = numbers.len();
    }

    println!("paths = {:?} {:?}", paths.len(), paths);

    let back = String::from("Back");
    let enter = String::from("Enter");
    let exit = String::from("Exit");

    // ========================= Generate final path (Discard repeated component) ========================= //
    let mut final_paths: Vec<Vec<usize>> = vec!();
    let mut fin : Vec<usize>;
    let limit : usize= 3;
    let mut stk :Vec<usize> = vec!();
    stk.push(0);
    
    let mut _flag = false;
    // println!("check tmp = {:?} {}", tmp, tmp.len());
    println!("Paths INFO {:?} {:?}", paths.len(), paths.clone());

    for i in 1..paths.len()-1 {
        let path = &paths[i];
        println!("=============================");
        println!("path INFO: {:?} {:?} ", path.len(), path.clone());
        println!("stack {:?}", stk);
        // println!("previous fin: {:?} {:?}", fin.clone().len(), fin.clone());
        // final_paths.push(fin.clone());

        fin = vec!();
        fin.push(path[0] as usize);
        let mut skip_cnt = 0;
        let mut record = true;
        for idx in 0..path.len()-1 {

            let s : usize = path[idx] as usize;   // bb_n in i32 (todo => usize)
            let t : usize = path[idx+1].try_into().unwrap();

            let Some(edge_idx) = g.find_edge(arr[s], arr[t]) else { 
                println!("cannot find edge in generate path");
                break;
            };

            println!("{:?} {:?}: {:? }from {:?} to {:?} => {:?}", idx, path.len(),stk, arr[s], arr[t], edge_idx );
            
            let Some(edge_weight) = g.edge_weight(edge_idx) else { 
                println!("cannot find weight");
                break; };

            // version 2
            // ========================= Discard repeated component ========================= //
            if edge_weight.find(&back).is_some() {    // exit edge
           // if back.eq(edge_weight) {           // back edge

                if record {
                // if *stk.last().unwrap() < limit {
                    fin.push((t as i32).try_into().unwrap());
                }
                let a :usize = 1;
                *stk.last_mut().unwrap() += a;
                // if it's over limit, bool = true
                if *stk.last_mut().unwrap() >= limit {
                    record = false;
                }

            // } else if enter.eq(edge_weight){    // entering edge
            } else if edge_weight.find(&enter).is_some() {    // entering edge
                    println!("out Entering! push to stk {:?}, {:?} {:?}", stk, *stk.last().unwrap(), limit);

                    stk.push(0);

                    if record {

                // if *stk.last().unwrap() < limit {

                    // if stk.len()>=3 && stk[stk.len() -2] >= limit {

                    // } else {
                    // fin.push(s);
                    fin.push((t as i32).try_into().unwrap());
                    // fin.push(t as i32);
                    // }
                } else {
                    println!("FIX Entering! push to stk {:?}, {:?}", stk, *stk.last().unwrap());
                    skip_cnt += 1;
                }
            } 
            else if edge_weight.find(&exit).is_some() {    // exit edge
            // else if exit.eq(edge_weight) {    // exiting edge
                // if stk.len()>=2 && stk[stk.len() -1] >=limit {
                // }else{

                // }
                // if skip_cnt != 0 {
                //     skip_cnt -= 1;
                // } else {
                // }
                stk.pop();
                record = true;
                for el in &stk {
                    if *el >= limit {
                        record = false;
                    }
                }
                

                if record {

                // if *stk.last().unwrap() < limit {
                    // fin.pop();
                    // fin.push(s);
                    fin.push((t as i32).try_into().unwrap());
                    // fin.push(t as i32);
                }
            } 
            else {        
                // inside large scc
                // if *stk.last().unwrap() < limit {
                if record{
                    fin.push((t as i32).try_into().unwrap());
                    // fin.push(t as i32);
                }
            }
        }
        // ========================= Save unique path only ========================= //
        
        println!("fin: {:?} {:?} {:?}", fin.len(), fin, skip_cnt);
        evaluate_path(fin, &mut final_paths);
        // final_paths.push(fin.clone());
    };
    
    println!("final paths: {:?} {:?}", final_paths.len(), final_paths.clone());

    // let mut file = fs::OpenOptions::new().append(true).create(true).open("/home/y23kim/rust/output_dir/final_path").expect("Fail to write yunji in fuzz.rs");
    // file.write_all(stk.as_bytes()).expect("yunji: Fail to write in fuzz.rs.");

    // for (idx, val) in tmp.clone().iter().enumerate() {
    //     let edge_idx = g.find_edge(arr[idx], arr[idx+1]);
    //     println!("edge index {:?}", edge_idx);
    // }
    
    return tmp
}

