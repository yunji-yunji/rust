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
            my_g.update_edge(arr[source.index()], arr[target.index()], String::from(""));
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

pub fn generate_path(g: &mut Graph::<usize, String>, _new_g: &mut Graph::<usize, String>, start : &mut usize, _arr: Vec<NodeIndex>) -> Vec<i32> {
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


    let sccs = kosaraju_scc(&g.clone());
    println!("in below func SCC ={:?}", g);

    for edge in g.clone().raw_edges() {
        println!("edge = {:?} {:?} {:?} source in {:?}",
                 edge, edge.source(), edge.target(), node_in_which_scc(edge.source(), sccs.clone()));
        let s = node_in_which_scc(edge.source(), sccs.clone());
        let t = node_in_which_scc(edge.target(), sccs.clone());
        // if len == 1 : NL
        // if len > 1 : Loop(L)


        if s == t {
            if edge.source() > edge.target() {
                let w = String::from("back edge");
                println!("Found back edge {:?} {:?}", edge.source(), edge.target());
                g.update_edge(edge.source(), edge.target(), w);
            }
        } else { // s != t
            if sccs[s].len() > sccs[t].len() { // source is bigger => exiting
                let w = String::from("exiting edge");
                println!("Found exiting edge {:?}", edge);
                g.update_edge(edge.source(), edge.target(), w);
            } else if sccs[s].len() < sccs[t].len() { // target is bigger => exiting
                let w = String::from("entering edge");
                println!("Found entering edge {:?}", edge);
                g.update_edge(edge.source(), edge.target(), w);
            } else {
                println!("NOTHING {:?}", edge);
            }
        }

/*
        // s == NL && t == L : entering edge (-1)
        // let a:EdgeWeightsMut<>
        if (sccs[s].len() == 1) && (sccs[t].len() != 1){
            let w = String::from("entering edge");
            println!("Found entering edge {:?}", edge);
            // g.remove_edge(ei);
            g.update_edge(edge.source(), edge.target(), w);
            // new_g.update_edge(edge.source(), edge.target(), w);
        }

        // s == L && t == NL : outgoing edge (+1)
        if (sccs[s].len() > 1) && (sccs[t].len() == 1){
            let w = String::from("exiting edge");
            println!("Found exiting edge {:?}", edge);
            g.update_edge(edge.source(), edge.target(), w);
        }
        // s == last node in L && t == first node in L (0)
        if (s == sccs[s].len() > 1) && (sccs[t].len() == 1){
            let w = String::from("exiting edge");
            println!("Found outgoing edge {:?}", edge);
            g.update_edge(edge.source(), edge.target(), w);
        }
*/

    }

    println!("new graph{:?}", Dot::with_config(&g.clone(), &[Config::EdgeIndexLabel]));
    for edge in g.clone().raw_edges() {
        println!("new edge = {:?}", edge);
    }
    // println!("new graph{:?}", Dot::with_config(&new_g.clone(), &[Config::EdgeIndexLabel]));
    // println!("{:?}", Dot::(g));
    // println!("{:?}", Dot::with_config(g, &[Config::EdgeIndexLabel]));

    // for scc in sccs {
    //     if scc.len() == 1 {
    //         mark_bb[g[scc[0]]] = -1;
    //
    //     }
    // }

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
                // println!("{:?} {:?} {:?}", min, max, g[node_idx]);

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
