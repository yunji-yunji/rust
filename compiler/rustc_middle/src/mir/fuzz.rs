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
use std::cmp;

pub fn _a() {
    // let mut my_g = Graph::new("".to_string(), vec![], vec![]);
    // let mut _my_g = Graph::new("aa".to_string(), vec![], vec![]);
    let mut _my_g = Graph::<i32, i32>::new();

}

pub fn default_g () -> Graph<usize, ()> {
    let my_g = Graph::<usize, ()>::new();
    my_g
}

pub fn my_app <'tcx>(_tcx: TyCtxt<'tcx>, body: &Body<'_>) -> (Graph<usize, ()>, Vec<NodeIndex>) {

    let mut my_g = Graph::<usize, ()>::new();

    // let mut edges = Vec::new();
    let mut cnt: usize = 0;
    let mut arr = vec![];
    // let mut arr = [] * body.basic_blocks.len();
    for _tmp in body.basic_blocks.iter() {
        // println!("basicblcok = {:?}", tmp);
        let node = my_g.add_node(cnt);
        arr.push(node);
        cnt = cnt + 1;
    }
    // println!("array! = {:?}", arr);
    for (source, _) in body.basic_blocks.iter_enumerated() {
        // let def_id = body.source.def_id();
        // let def_name = format!("{}_{}", def_id.krate.index(), def_id.index.index(),);

        let terminator = body[source].terminator();
        let labels = terminator.kind.fmt_successor_labels();

        for (target, _label) in terminator.successors().zip(labels) {
            // let mut s;

            // for node in my_g.raw_nodes() {
            //     if node.weight == source.index() {
            //         s = node;
            //         // let s = my_g.add_node(source.index());
            //         break;
            //     }
            // }
            //
            // println!("{:?} {:?} {:?}", node, node.weight, node.weight.index());

            // // let a = NodeIndex();
            // let k :u32 = source.index();
            // let a = NodeIndex(k);
            // let b = NodeIndex(target.index());
            // println!("node index from to check = {:?} {:?}", a, b);
            // for node in my_g.raw_nodes() {
            //     println!("{:?}", node);
            //
            // }

            //// add node
            // let s = my_g.add_node(source.index());
            // let t = my_g.add_node(target.index());

            // let s  = i32::from(source.index());
            // let t  = i32::from(target.index());
            // let k : i32 = 3;
            my_g.update_edge(arr[source.index()], arr[target.index()], ());
            // my_g.add_edge(s.into(), t.into(), ());
            // my_g.add_edge(source.index().into(), target.index().into(), ());
            // my_g.extend_with_edges(&[(a, b)]);
            // my_g.extend_with_edges(&[(s, t)]);

        }
    }
    // println!("nodes={:?}", my_g.raw_nodes());
    // for node in my_g.raw_nodes() {
    //     println!("{:?} {:?} {:?}", node, node.weight, node.weight.index());
    // }
    println!("{:?}", Dot::with_config(&my_g, &[Config::EdgeNoLabel]));

    // using graph..
    // is scc?
    let res = kosaraju_scc(&my_g);
    println!("SCC ={:?}", res);


    return (my_g, arr);
}

pub fn generate_path(g: Graph::<usize, ()>, start : &mut usize, _arr: Vec<NodeIndex>) -> Vec<i32> {
    let mut tmp: Vec<i32> = vec![];
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
                tmp.push(n);
                prev1 = n;
            }
        }
        *start = numbers.len();
    }


    // let res = kosaraju_scc(_g);
    // println!("SCC ={:?}", res);

    // SCC =[[NodeIndex(11)], [NodeIndex(10)], [NodeIndex(9)], [NodeIndex(1), NodeIndex(2), NodeIndex(5), NodeIndex(6), NodeIndex(7), NodeIndex(8), NodeIndex(3), NodeIndex(4)], [NodeIndex(0)]]
    // yunji: in run_thread!
    // seed = 0 0 2 trace_seed = Some([0, 1, 2, 3, 4, 7, 8, 1, 2, 5, 6, 7, 8, 1, 2, 3, 4, 7, 8, 1, 2, 5, 6, 7, 8, 1, 2, 3, 4, 7, 8, 1, 9, 11])
    // yunji: after run threads

    // let arr_len = arr.len();
    let mut mark_bb = [0; 100];

    let sccs = kosaraju_scc(&g);
    println!("in below func SCC ={:?}", g);
    for scc in sccs {
        // for node_idx in scc {
        //     mark_bb[g[node_idx]]= -1;
        // }
        if scc.len() == 1 {
            mark_bb[g[scc[0]]] = -1;
        } else {
            let mut min = 10000;
            let mut max = 0;
            // let mut prev = g[scc[0]];
            for node_idx in scc {
                println!("{:?} {:?} {:?}", min, max, g[node_idx]);

                mark_bb[g[node_idx]] = -100;
                min = cmp::min(min, g[node_idx]);
                max = cmp::max(max, g[node_idx]);
                // prev = g[node_idx];
            }
            // let Some(minv) = g.node_weight(scc.iter()).min();
            // let Some(maxv) = scc.iter().max();
            // println!("{:?} {:?}", minv, maxv);
            println!("{:?} {:?}", min, max);

            mark_bb[min] = 88;
            mark_bb[max] = 999;

            // for node_idx in scc.clone() {
            //     // let Some(node) = node_idx;
            //     let node = g[node_idx];
            //     println!("{:?}", node);
            //     // match node {
            //     //     min =>  mark_bb[node]= -100,
            //     //     max =>  mark_bb[node]= 10,
            //     //     _ =>  mark_bb[node]= 7,
            //     // }
            //     // if node == minv {
            //     //     mark_bb.push(0);
            //     // } else if node == maxv {
            //     //     mark_bb.push(10);
            //     // } else {
            //     //     mark_bb.push(7);
            //     // }
            // }
        }

        // if scc.len() > 1 {
        //     println!("scc = {:?}", r);
        //     for rr in r {
        //         // let Some(nw) = my_g.node_weight_mut(rr);
        //         let nw = my_g[rr];
        //         println!("{:?}", nw);
        //     }
        // }
    }
    println!("mark bb  {:?}", mark_bb);

    // 88 : start of scc
    // 999 : source node of back edge
    // -100 : inside of scc
    // -1 : NO loop basic block
    // 0 : empty

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

    let mut fin = vec!();
    let limit = 3;
    let mut cnt = 0;
    for n in tmp.clone() {
        let a = n as usize;
        if mark_bb[a] == 999 {
            cnt += 1;
        }
        if cnt == limit {
            fin.push(a);
        }


//        if mark_bb[n] == 999 {
//             cnt += 1;
//         } else {
//             fin.push(n);
//         }
    }
    println!("fin path {:?}", fin);

    return tmp
}

// fn is_in_cycle() -> bool {
//
// }
