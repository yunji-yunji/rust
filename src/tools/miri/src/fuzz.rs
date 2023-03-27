use petgraph::Graph;
use petgraph::dot::{Dot, Config};
use petgraph::algo::kosaraju_scc;
use petgraph::prelude::NodeIndex;
use petgraph::Incoming;

use petgraph::algo::toposort;

// use petgraph::visit::DfsPostOrder;
// use petgraph::Directed;
use petgraph::visit::EdgeRef;

// use rustc_data_structures::graph::implementation::NodeIndex;
// use petgraph::graph::NodeIndex;
use rustc_middle::ty::TyCtxt;
use rustc_middle::mir::*;

use std::fs::File;
use std::io::{BufReader, BufRead};
// use std::string;

pub fn _a() {
    // let mut my_g = Graph::new("".to_string(), vec![], vec![]);
    // let mut _my_g = Graph::new("aa".to_string(), vec![], vec![]);
    let mut _my_g = Graph::<i32, i32>::new();

}

fn _vect_difference(v1: &Vec<NodeIndex>, v2: &Vec<NodeIndex>) -> Vec<NodeIndex> {
    v1.iter().filter(|&x| !v2.contains(x)).cloned().collect()
}

pub fn default_g () -> Graph<usize, String> {
    let my_g = Graph::<usize, String>::new();
    my_g
}

pub fn has_cycle2(orig: Graph<usize, String>, scc:Vec<NodeIndex>) -> bool {

    // if scc.len()
    let mut new = Graph::<usize, String>::new();
    let mut i = 0;
    for _node in orig.clone().raw_nodes() {
        // for _node_idx in &scc {
        let _node1 = new.add_node(i);
        i += 1;
    }
    for edge in orig.clone().raw_edges() {
        // println!("edge = {:?} {:?} {:?} source in {:?}",
                // edge, edge.source(), edge.target(), node_in_which_scc(edge.source(), sccs.clone()));
        let s = edge.source();
        let t = edge.target();
        // match orig.edge_endpoints(edge) {
            // Ok(s, t) => {
        println!("{:?} {:?} ", s, t);
        if scc.contains(&s) && scc.contains(&t) {
            new.update_edge(s, t, String::from("random"));
            // new.update_edge(arr0[0], arr0[1], String::from("1"));

        }
            // }
        // }
        // let s = node_in_which_scc(edge.source(), sccs.clone());
        // let t = node_in_which_scc(edge.target(), sccs.clone());
    
    }
    println!("new graph\n{:?}", Dot::with_config(&new, &[Config::EdgeIndexLabel]));
    match toposort(&new, None){
        Ok(_order) => {
            println!("no cycle");
            return false;
        },
        Err(_err) => {
            println!("cycle");
            return true;
            // g.node_weight(err.node_id()).map(|weight|
                // println!("Error graph has cycle at node {}", weight));
        }
    }
}

pub fn _has_cycle(mut orig: Graph<usize, String>, rmv:Vec<NodeIndex>) -> bool {

    // let mut orig = Graph::<usize, String>::new();
    for node in rmv {
        orig.remove_node(node);
    }
    println!("removed graph\n{:?}", Dot::with_config(&orig, &[Config::EdgeIndexLabel]));

    match toposort(&orig, None){
        Ok(_order) => {
            println!("no cycle");
            return false;
        },
        Err(_err) => {
            println!("cycle");
            return true;
            // g.node_weight(err.node_id()).map(|weight|
                // println!("Error graph has cycle at node {}", weight));
        }
    }
    // return 
}

pub fn my_app <'tcx>(_tcx: TyCtxt<'tcx>, body: &Body<'_>) 
-> (Graph<usize, String>, Graph<usize, String>, Vec<NodeIndex>) {


    // ====================== = test = ======================

    println!("\n------------ temporary graph ----------");
    let mut g = Graph::<usize, String>::new();
    // g.extend_with_edges(&[
    //     (1, 2, "1"), (0, 2, "0"), (0, 3, "3")
    // ]);
    // let mut cnt0: usize = 0;
    let mut arr1 = vec![];
    let mut arr0 :Vec<NodeIndex> = vec!();
    let mut stk_info : Vec<Vec<i32>> = vec!();
    for i in 0..14 {
        let node1 = g.add_node(i);
        // let _node2 = tmpg.add_node(i);
        arr0.push(node1);
        arr1.push(i);
        stk_info.push(vec!());
        // cnt0 = cnt0 + 1;
    }
      
    g.update_edge(arr0[0], arr0[1], String::from("1"));
    g.update_edge(arr0[1], arr0[2], String::from("2"));
    g.update_edge(arr0[2], arr0[3], String::from(" 3"));
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
    // let mut tmpg = g.clone();
    // let mut k = 0;

    // g.remove_node(arr0[0]);
    // g.remove_node(arr0[13]);
    println!("g\n{:?}", Dot::with_config(&g, &[Config::EdgeIndexLabel]));
    let mut scc_id = 0;
    loop {

        let mut len1flag = false;
        // let mut is_cycle;
        
        let scc_list = kosaraju_scc(&g);
        println!("test SCC = {:?}", scc_list);
        for scc in &scc_list {
            // change this to if scc has cycle
            let mut arr2 = vec![];
            for s in scc.clone() {
                arr2.push(s.index());
            }
            let target_node = scc[0];

            // let rmv = vect_difference(arr1.clone(), arr2);
            // let rmv = vect_difference(&arr0, scc);
            // let rmv = elementwise_subtraction(arr1.clone(), arr2);
            // println!("remove list {:?}", rmv);
            let is_cycle = has_cycle2(g.clone(), scc.clone());
            println!("tartget node {:?}, scc {:?} cycle? {:?} ", target_node, scc, is_cycle);
            if scc.len() != 1 {
                // true when len is not 1
                // if len is 1, false
                // no cycle = false
                // cycle = true
                len1flag = true;
            }
            for i in 0..scc.len() {
                // if scc.len() != 1 {
                // if that scc doesn't have a cycle
                if is_cycle == true {
                    stk_info[scc[i].index()].push(scc_id);
                    println!("{:?} {:?} ", scc[i].index(),scc_id);
                }

                // }
                // println!("edge, leng = {:?} {:?}", scc[i], scc.len());
                let Some(edge_idx) = g.find_edge(scc[i], target_node) else { 
                    // println!("cannot find edge in test");
                    continue;
                };
                println!("Remove edge index {:?} {:?} {:?}", scc[i], target_node, edge_idx);
                g.remove_edge(edge_idx);

            }
            if is_cycle == true {
                scc_id += 1;
            }
   

            // for e in g.edges(target_node) {
            //     println!("loop forming? {:?}", e);

            // }

        }
                println!("============================");

        if len1flag == false {
            // continue;
            // or
            println!("break here");
            println!("final SCC ={:?}", scc_list);
            println!("stack info ={:?}", stk_info);

            break;
        }
    /*

        println!("\n------------ first toposort ----------");
        match toposort(&g, None) {
            Ok(order) => {
                for i in order {
                    g.node_weight(i).map(|weight| {
                        print!("{}, ", weight);
                        // weight
                    });
                }
            },
            Err(err) => {
                g.node_weight(err.node_id()).map(|weight| println!("Error graph has cycle at node {}", weight));
            }
        }

        println!("\n------------ DFSPostOrder  ----------");

        let mut prev_idx = Default::default();
        let mut src;
        let mut tar;
        for start in g.node_indices() {
            let mut dfs = DfsPostOrder::new(&g, start);
            let mut dfs2 = DfsPostOrder::new(&tmpg, start);
            print!("[{}] ", start.index());

            while let Some(visited) = dfs2.next(&tmpg) {
                print!(" {}", visited.index());
            }
            println!();


            let Some(target_edge_idx) = dfs.next(&g) else {
                println!("yj: there is no target edge index.!");
                break;
            };
            if prev_idx != target_edge_idx {
                src = prev_idx;
                tar = target_edge_idx;
                print!("=> {:?} {:?}\n", src, tar);
                if prev_idx != Default::default() {
                    let Some(edge_idx) = g.find_edge(src, tar) else { 
                        println!("yj: cannot find edge");
                        break;
                    };
                    println!("Remove edge index {:?}\n", edge_idx);
                    g.remove_edge(edge_idx);
                    tmpg.remove_edge(edge_idx);
                    break;

                }
   
                prev_idx = target_edge_idx;
                // break;
            }

            // while let Some(visited) = dfs.next(&g) {
            //     print!(" {}", visited.index());
            // }
    
            // println!();
        }

        */
        // k = k+1;

        // if k>3 {
        //     break;
        // }
        // return ( g, g, arr0);
        // return ();
        // g.remo
        // // remove this back edge
        // if edge.target() == en {
        //     let Some(edge_idx) = copy_g.find_edge(edge.source(), edge.target()) else { 
        //         println!("cannot find edge 22");
        //         break;
        //     };
        //     println!("Remove edge index {:?}", edge_idx);
        //     copy_g.remove_edge(edge_idx);
        // }
        // break;
    }

    // return;
    println!("================== test end ========================");
    // ====================== = test = ======================


    let mut my_g = Graph::<usize, String>::new();
    let new_g = Graph::<usize, String>::new();
    let mut copy_g = Graph::<usize, String>::new();

    // let mut edges = Vec::new();
    let mut cnt: usize = 0;
    let mut arr = vec![];
    // let mut arr = [] * body.basic_blocks.len();
    for _tmp in body.basic_blocks.iter() {
        // println!("basicblcok = {:?}", tmp);
        let node1 = my_g.add_node(cnt);
        let _node2 = copy_g.add_node(cnt);
        arr.push(node1);
        cnt = cnt + 1;
    }



    // println!("array! = {:?}", arr);
    for (source, _) in body.basic_blocks.iter_enumerated() {
        // let def_id = body.source.def_id();
        // let def_name = format!("{}_{}", def_id.krate.index(), def_id.index.index(),);

        let terminator = body[source].terminator();
        let labels = terminator.kind.fmt_successor_labels();

        for (target, _label) in terminator.successors().zip(labels) {

            my_g.update_edge(arr[source.index()], arr[target.index()], String::from(""));
            copy_g.update_edge(arr[source.index()], arr[target.index()], String::from(""));

        }
    }

    // println!("{:?}", Dot::with_config(&my_g, &[Config::EdgeIndexLabel]));

    // ========================= find SCC ========================= //
    let mut sccs = kosaraju_scc(&my_g);
    println!("SCC ={:?}", sccs);



    loop {
        let mut back = String::from("Back");
        let mut enter = String::from("Enter");
        let mut exit = String::from("Exit");

        // println!("copy_g\n{:?}", Dot::with_config(&copy_g, &[Config::EdgeIndexLabel]));
        let mut len1flag = false;
        sccs = kosaraju_scc(&copy_g);
        for scc in &sccs {
            if scc.len() != 1 {
                len1flag = true;
            }
            // all false -> all == 1 -> stop loop
        }
        // println!("in SCC ={:?}", sccs.clone());

        match toposort(&copy_g, None) {
            Ok(order) => {
                for i in order {
                    copy_g.node_weight(i).map(|_weight| {
                        // print!("{}, ", weight);
                        // weight
                    });
                }
            },
            Err(err) => {
                copy_g.node_weight(err.node_id()).map(|weight| println!("Error graph has cycle at node {}", weight));
            }
        }
        println!("\n------------ after toposort ----------");
        // DFS ORDER
        // for start in copy_g.node_indices() {
        //     let mut dfs = DfsPostOrder::new(&copy_g, start);
        //     print!("[{}] ", start.index());

        //     while let Some(visited) = dfs.next(&copy_g) {
        //         print!(" {}", visited.index());
        //     }

        //     println!();
        // }
        // let dfs = DfsPostOrder(&copy_g);
        // println!("in dfs posrt order ={:?}", dfs);

        let mut en = Default::default();

        for edge in my_g.clone().raw_edges() {
            // println!("edge = {:?} {:?} {:?} source in {:?}",
                    // edge, edge.source(), edge.target(), node_in_which_scc(edge.source(), sccs.clone()));
            let s = node_in_which_scc(edge.source(), sccs.clone());
            let t = node_in_which_scc(edge.target(), sccs.clone());
            if s == t {
                if (edge.source() > edge.target() ) && (edge.target() == en) {
                    // if (edge.source() > edge.target()) and edge.target(){
                    // let w = String::from("Back");
                    println!("Found back edge {:?} {:?}", edge.source(), edge.target());
                    my_g.update_edge(edge.source(), edge.target(), back.clone());
                    back = back.clone() + &String::from(" Back");
                    
                    let mut enter_s = Default::default();
                    let mut enter_t = Default::default();
                    // find entering edge of back edge's target node
                    // let tmp = my_g.edges(edge.target()); 
                    for targ_edge in my_g.edges_directed(edge.target(), Incoming) {
                        // println!("tmp = {:?} {:?}", targ_edge, targ_edge.source());
                        // if (a.source() == edge.source()) || (a.source() > a.target()) {
                        if targ_edge.source() > targ_edge.target() {
                            // println!("do nothing");
                        } else {
                            enter_s = targ_edge.source();
                            enter_t = targ_edge.target();
                            // println!("this is entering edge. Mark! {:?} {:?}", enter_s, enter_t);
                        }
                        // among these, 
                        // exclude itself, exclude another backedge
                        // mark entering
                    }
                    // println!("updated enter node {:?}",enter);
                    my_g.update_edge(enter_s, enter_t, enter.clone());
                    enter = enter.clone() + &String::from(" Enter");


                    // remove this back edge
                    if edge.target() == en {
                        let Some(edge_idx) = copy_g.find_edge(edge.source(), edge.target()) else { 
                            println!("cannot find edge 22");
                            break;
                        };
                        println!("Remove edge index {:?}", edge_idx);
                        copy_g.remove_edge(edge_idx);
                    }
                }
            } else { // s != t
                if sccs[s].len() > sccs[t].len() { // source is bigger => exiting
                    // let w = String::from("Exit");
                    // println!("Found exiting edge {:?} {:?}", edge.source(), edge.target());
                    my_g.update_edge(edge.source(), edge.target(), exit.clone());
                    exit = exit.clone() + &String::from(" Exit");
                    // println!("next exit {:?}", exit);

                } else if sccs[s].len() < sccs[t].len() { // target is bigger => entering
                    // let w = String::from("Enter");
                    en = edge.target();
                    // println!("Found entering edge but do nothing {:?} {:?}", edge.source(), edge.target());
                    // my_g.update_edge(edge.source(), edge.target(), enter.clone());
                    // enter = enter.clone() + &String::from(" Enter");
                } else { // s len == t len NL -> NL
                    // println!("NOTHING {:?}", edge);
                    // let w = String::from("NoLoop");
                    // println!("Found no loop single node SCC {:?}", edge);
                    // my_g.update_edge(edge.source(), edge.target(), w);
                }
            }

        }
        for edge in my_g.clone().raw_edges() {
            println!("{:?}", edge);
        }
        if len1flag == false {
            // continue;
            // or
            println!("break here");
            break;
        }

    }

    // ========================= Mark Edges (weights in graph) ========================= //
    

    /*
    for edge in my_g.clone().raw_edges() {
        println!("edge = {:?} {:?} {:?} source in {:?}",
                 edge, edge.source(), edge.target(), node_in_which_scc(edge.source(), sccs.clone()));
        let s = node_in_which_scc(edge.source(), sccs.clone());
        let t = node_in_which_scc(edge.target(), sccs.clone());

        if s == t {
            if edge.source() > edge.target() {
                // let w = String::from("Back");
                println!("Found back edge {:?} {:?}", edge.source(), edge.target());
                my_g.update_edge(edge.source(), edge.target(), back.clone());
            }
        } else { // s != t
            if sccs[s].len() > sccs[t].len() { // source is bigger => exiting
                // let w = String::from("Exit");
                println!("Found exiting edge {:?}", edge);
                my_g.update_edge(edge.source(), edge.target(), exit.clone());
            } else if sccs[s].len() < sccs[t].len() { // target is bigger => exiting
                // let w = String::from("Enter");
                println!("Found entering edge {:?}", edge);
                my_g.update_edge(edge.source(), edge.target(), enter.clone());
            } else { // s len == t len NL -> NL
                // println!("NOTHING {:?}", edge);
                let w = String::from("NoLoop");
                println!("Found no loop single node SCC {:?}", edge);
                my_g.update_edge(edge.source(), edge.target(), w);
            }
        }

    }
*/
    // println!("<<<<new graph>>>> {:?}", Dot::with_config(&my_g.clone(), &[Config::EdgeIndexLabel]));
    println!("## NEW GRAPH ##");
    for edge in my_g.clone().raw_edges() {
        println!("{:?}", edge);
    }
    // my_g = clone
    return (my_g, new_g, arr);
}

fn node_in_which_scc(n_idx: NodeIndex, sccs: Vec<Vec<NodeIndex>>) -> usize {
    for i in 0..sccs.len() {
        if sccs[i].contains(&n_idx) {
            return i;
        }
    }
    return 0;
}

// use std::fs::File;
// use std::fs;
// use std::path::Path;
// use std::io::{self, Write, BufReader, BufRead, Error};
// use std::iter::Map;
// use std::collections::HashMap;

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

// fn save_test(i: Inp, path : &str, id : &mut i32) {
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
    let f = File::open("/home/y23kim/rust/output_dir/result3").unwrap();
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

        stk = vec!();
        stk.push(0);
        fin = vec!();
        fin.push(path[0] as usize);
        let mut skip_cnt = 0;
        let mut record = true;
        for idx in 0..path.len()-1 {

            let s : usize = path[idx] as usize;   // bb_n in i32 (todo => usize)
            let t : usize = path[idx+1].try_into().unwrap();

            let Some(edge_idx) = g.find_edge(arr[s], arr[t]) else { 
                println!("cannot find edge");
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
                    fin.push(t);
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
                    fin.push(t);
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
                    fin.push(t);
                }
            } 
            else {        
                // inside large scc
                // if *stk.last().unwrap() < limit {
                if record{
                    fin.push(t);
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

