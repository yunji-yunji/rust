use petgraph::Graph;
use petgraph::dot::{Dot, Config};
use petgraph::algo::kosaraju_scc;
use petgraph::prelude::NodeIndex;

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

pub fn default_g () -> Graph<usize, String> {
    let my_g = Graph::<usize, String>::new();
    my_g
}

pub fn my_app <'tcx>(_tcx: TyCtxt<'tcx>, body: &Body<'_>) -> (Graph<usize, String>, Graph<usize, String>, Vec<NodeIndex>) {

    let mut my_g = Graph::<usize, String>::new();
    let mut new_g = Graph::<usize, String>::new();

    // let mut edges = Vec::new();
    let mut cnt: usize = 0;
    let mut arr = vec![];
    // let mut arr = [] * body.basic_blocks.len();
    for _tmp in body.basic_blocks.iter() {
        // println!("basicblcok = {:?}", tmp);
        let node1 = my_g.add_node(cnt);
        let _node2 = new_g.add_node(cnt);
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

        }
    }

    println!("{:?}", Dot::with_config(&my_g, &[Config::EdgeIndexLabel]));

    // using graph..
    // is scc?
    let res = kosaraju_scc(&my_g);
    println!("SCC ={:?}", res);


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

pub fn generate_path(g: &mut Graph::<usize, String>, _new_g: &mut Graph::<usize, String>, 
    start : &mut usize, arr: Vec<NodeIndex>) -> Vec<i32> {
    let mut tmp: Vec<i32> = vec![];
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

    println!("paths = {:?}", paths);
    let sccs = kosaraju_scc(&g.clone());
    println!("in below func SCC ={:?}", g);

    for edge in g.clone().raw_edges() {
        println!("edge = {:?} {:?} {:?} source in {:?}",
                 edge, edge.source(), edge.target(), node_in_which_scc(edge.source(), sccs.clone()));
        let s = node_in_which_scc(edge.source(), sccs.clone());
        let t = node_in_which_scc(edge.target(), sccs.clone());

        if s == t {
            if edge.source() > edge.target() {
                let w = String::from("Back");
                println!("Found back edge {:?} {:?}", edge.source(), edge.target());
                g.update_edge(edge.source(), edge.target(), w);
            }
        } else { // s != t
            if sccs[s].len() > sccs[t].len() { // source is bigger => exiting
                let w = String::from("Exit");
                println!("Found exiting edge {:?}", edge);
                g.update_edge(edge.source(), edge.target(), w);
            } else if sccs[s].len() < sccs[t].len() { // target is bigger => exiting
                let w = String::from("Enter");
                println!("Found entering edge {:?}", edge);
                g.update_edge(edge.source(), edge.target(), w);
            } else { // s len == t len NL -> NL
                // println!("NOTHING {:?}", edge);
                let w = String::from("NoLoop");
                println!("Found no loop single node SCC {:?}", edge);
                g.update_edge(edge.source(), edge.target(), w);
            }
        }

    }

    println!("new graph{:?}", Dot::with_config(&g.clone(), &[Config::EdgeIndexLabel]));
    for edge in g.clone().raw_edges() {
        println!("new edge = {:?}", edge);
    }


    // enter 0 -> 1
    // exit 1 -> 0
    // back 8 -> 1
    // my path=[0,
    //     1, 2, 3, 4, 7, 8,
    //     1, 2, 5, 6, 7, 8,
    //     1, 2, 3, 4, 7, 8,
    //     1, 2, 5, 6, 7, 8,
    //     1, 2, 3, 4, 7, 8,
    //     1, 9, 11,
    //
    //
    // 0, 1, 2, 5, 6, 7, 8, 1, 2, 5, 6, 7, 8, 1, 2, 3, 4, 7, 8, 1, 2, 5, 6, 7, 8, 1, 2, 3, 4, 7, 8, 1, 2, 5, 6, 7, 8, 1, 2, 3, 4, 7, 8, 1, 2, 5, 6, 7, 8, 1, 2, 3, 4, 7, 8, 1, 2, 5, 6, 7, 8, 1, 2, 3, 4, 7, 8, 1, 2, 5, 6, 7, 8, 1, 2, 3, 4, 7, 8, 1, 9, 10]
    // [[NodeIndex(11)],
    //     [NodeIndex(10)],
    //     [NodeIndex(9)],
    //     [NodeIndex(1), NodeIndex(2), NodeIndex(5), NodeIndex(6), NodeIndex(7), NodeIndex(8), NodeIndex(3), NodeIndex(4)],
    //     [NodeIndex(0)]]


    let mut fin : Vec<usize> = vec!();
    let limit : usize= 3;
    // let mut cnt = 0;
    let mut stk :Vec<usize> = vec!();
    stk.push(0);
    
    let mut _flag = false;
    println!("check tmp = {:?} {}", tmp, tmp.len());
    let mut start_idx = 0;
    // let _res = loop {
    for i in 1..paths.len()-1 {
        let path = &paths[i];
    // for path in &paths {
        println!("start of one execution..");
        println!("fin path {:?}", fin);
        println!("stack {:?}", stk);
        println!("INFO: {:?} {:?} {:?} {:?}", start_idx, path.clone(), paths.clone(), paths.len());

        stk = vec!();
        // stk = [0];
        stk.push(0);
        fin = vec!();
        for idx in 0..path.len()-2 {
        // if flag {
        //     println!("start ids = {:?} {:?}", start_idx, tmp.len());
        //     return tmp

        // }

        // for idx in start_idx..tmp.len()-2 {
            // if idx + 1 == tmp.len() -1 {
            // if idx > 500{
            //     println!("{:?} {:?} bb", idx, tmp.len());
            //     flag = true;
            // }

            // if idx > 500{
            //     flag = true;
            // }

            let s : usize = path[idx] as usize;   // bb_n in i32 (todo => usize)
            let t : usize = path[idx+1].try_into().unwrap();

            let Some(edge_idx) = g.find_edge(arr[s], arr[t]) else { 
                start_idx = idx + 1;
                println!("end of one execution..");
                println!("fin path {:?}", fin);
                println!("stack {:?}", stk);
                println!("break here? next edge: {:?} {:?} {:?}", start_idx, arr[s], arr[t]);

                stk = vec!();
                // stk = [0];
                stk.push(0);
                fin = vec!();
                break;
            };


            println!("{:?} {:?} {:?}: from {:?} to {:?} => {:?}", idx, start_idx, path.len(), arr[s], arr[t], edge_idx);
            
            let back = String::from("Back");
            let enter = String::from("Enter");
            let exit = String::from("Exit");
            // let noloop = String::from("NoLoop");
            let Some(edge_weight) = g.edge_weight(edge_idx) else { 
                println!("break here?2");
                break; };
            // match edge_weight {
            //     back => cnt +=1,
            //     enter => cnt +=1,
            //     exit => cnt +=1,
            //     // _ => cnt += 3
            // }



            // version 1
    /*
            if *stk.last().unwrap() < limit {
                println!("edge weight {:?} -> {:?} : {:?} stk {:?}", s, t, edge_weight, stk);
                if back.eq(edge_weight) {           // back edge
                    // top += 1;
                    let a :usize = 1;
                    *stk.last_mut().unwrap() += a;
                    fin.push(t);

                } else if enter.eq(edge_weight){    // entering edge
                    println!("push only here stk {:?}", stk);

                    stk.push(0);
                    fin.push(s);
                    fin.push(t);
                } 
                // else if exit.eq(edge_weight) {    // exiting edge

                // } 
                else {        
                                        // inside large scc
                    fin.push(t);

                }
            } else {
                if exit.eq(edge_weight) {
                    fin.push(t);
                    stk.pop();
                }
            }
    */

            // version 2
            if back.eq(edge_weight) {           // back edge

                if *stk.last().unwrap() < limit {
                    fin.push(t);
                }
                let a :usize = 1;
                *stk.last_mut().unwrap() += a;

            } else if enter.eq(edge_weight){    // entering edge
                println!("push only here stk {:?}", stk);

                stk.push(0);
                if *stk.last().unwrap() < limit {

                    fin.push(s);
                    fin.push(t);
                }
            } 
            else if exit.eq(edge_weight) {    // exiting edge
                stk.pop();
                if *stk.last().unwrap() < limit {
                    // fin.push(s);
                    fin.push(t);
                }
            } 
            else {        
                // inside large scc
                if *stk.last().unwrap() < limit {
                    fin.push(t);
                }

            }

        }



    };


    // let mut file = fs::OpenOptions::new().append(true).create(true).open("/home/y23kim/rust/output_dir/final_path").expect("Fail to write yunji in fuzz.rs");
    // file.write_all(stk.as_bytes()).expect("yunji: Fail to write in fuzz.rs.");

    // for (idx, val) in tmp.clone().iter().enumerate() {
    //     let edge_idx = g.find_edge(arr[idx], arr[idx+1]);
    //     println!("edge index {:?}", edge_idx);
    // }
    
    return tmp
}

