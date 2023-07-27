use std::collections::VecDeque;
// use petgraph::Graph;
// use petgraph::dot::{Dot, Config};
// use petgraph::algo::kosaraju_scc;
// use petgraph::algo::toposort;
// use petgraph::prelude::NodeIndex;
// use petgraph::visit::Dfs;
//
// use rustc_middle::ty::TyCtxt;
// use rustc_middle::mir::*;
//
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufRead};
// use std::io::{self, Write, BufReader, BufRead, Error};

// use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct Inp {
    x: i32,
    y: i32,
    n: i32,
}

fn _mutate_seed(seed: Inp, flag: bool) -> Inp {

    if flag { // previous one is interesting
        Inp {x: seed.x + 1, y: seed.y, n: seed.n + 1}
    } else {
        // Inp {x: seed.x - 1, y: seed.y + 1, n: seed.n + 1}
        Inp {x: seed.x, y: seed.y + 1, n: seed.n + 1}
    }
}

pub fn _get_seeds(input_dir : String,q:&mut VecDeque<Inp>/* s_l : &mut Vec<Inp>*/) {
    // let corpus_dir : &str = ;
    println!("input dir={:?}", input_dir);
    let path = fs::read_dir(input_dir).unwrap();
    for file in path {
        let mut arr : Vec<i32> = vec!();
        // println!("yunji Debug = {:?}", file);

        let input = File::open(file.unwrap().path()).expect("yunji : Unable to open file");
        let buffered = BufReader::new(input);
        for line in buffered.lines() {
            if let Ok(num) = line {
                // println!("yunji Debug = {:?}", num);
                let tmp: i32 = num.parse().unwrap();
                arr.push(tmp);
            }
        }

        let seed = Inp {
            x: arr[0],
            y: arr[1],
            n: arr[2],
        };
        q.push_back(seed);
        println!("seed={:?}", seed);
        // s_l.push(seed);
    }
}

//
// pub fn my_app <'tcx>(_tcx: TyCtxt<'tcx>, _body: &Body<'_>)
// -> (Graph<usize, String>, Graph<usize, String>, Vec<NodeIndex>) {
//     println!("\n------------ TEST graph ----------");
//     let mut g = Graph::<usize, String>::new();
//     let mut backup_g = Graph::<usize, String>::new();
//     let mut arr0 :Vec<NodeIndex> = vec!();
//
//     let mut scc_info_stk : HashMap<NodeIndex, Vec<SccInfo>> = HashMap::new();
//     // ===================== create dummy graph
//     let case = 1;
//     let num_node;
//     if case ==1 {
//         num_node = 7;
//     } else {
//         num_node = 14;
//     }
//     for i in 0..num_node {
//         let node1 = g.add_node(i);
//         scc_info_stk.insert(node1, vec!());
//         let _node2 = backup_g.add_node(i);
//         arr0.push(node1);
//     }
//     println!("{:?} {:?}", arr0, arr0.len());
//     println!("Initial stack info hash map {:?}\n\n", scc_info_stk);
//
//     if case == 1 {
//         g.update_edge(arr0[0], arr0[1], String::from("1"));
//         g.update_edge(arr0[1], arr0[2], String::from("2"));
//         g.update_edge(arr0[2], arr0[3], String::from("3"));
//         g.update_edge(arr0[2], arr0[4], String::from("4"));
//         g.update_edge(arr0[3], arr0[4], String::from("5"));
//         g.update_edge(arr0[4], arr0[3], String::from("6"));
//         g.update_edge(arr0[4], arr0[5], String::from("7"));
//         g.update_edge(arr0[3], arr0[5], String::from("8"));
//         g.update_edge(arr0[5], arr0[6], String::from("9"));
//         g.update_edge(arr0[3], arr0[3], String::from("10"));
//
//     } else {
//         // big graph
//         g.update_edge(arr0[0], arr0[1], String::from("1"));
//         g.update_edge(arr0[1], arr0[2], String::from("2"));
//         g.update_edge(arr0[2], arr0[3], String::from("3"));
//         g.update_edge(arr0[3], arr0[5], String::from("4"));
//         g.update_edge(arr0[2], arr0[4], String::from("5"));
//         g.update_edge(arr0[4], arr0[5], String::from("6"));
//         g.update_edge(arr0[5], arr0[6], String::from("7"));
//         g.update_edge(arr0[6], arr0[7], String::from("8"));
//         g.update_edge(arr0[7], arr0[8], String::from("9"));
//         g.update_edge(arr0[8], arr0[5], String::from("10"));
//         g.update_edge(arr0[7], arr0[9], String::from("11"));
//         g.update_edge(arr0[9], arr0[10], String::from("12"));
//         g.update_edge(arr0[9], arr0[9], String::from("13"));
//         g.update_edge(arr0[10], arr0[11], String::from("14"));
//         g.update_edge(arr0[11], arr0[12], String::from("15"));
//         g.update_edge(arr0[10], arr0[1], String::from("16"));
//         g.update_edge(arr0[11], arr0[10], String::from("17"));
//         g.update_edge(arr0[12], arr0[10], String::from("18"));
//         g.update_edge(arr0[12], arr0[13], String::from("19"));
//         g.update_edge(arr0[11], arr0[1], String::from("20"));
//         // g.update_edge(arr0[9], arr0[11], String::from("21"));
//     }
//     println!("before transform graph\n{:?}", Dot::with_config(&g, &[Config::EdgeIndexLabel]));
//
//     let mut scc_id : i32 = 1;
//     let mut copy_graph = g.clone();
//     loop {
//         let mut stop  =true;
//         let mut scc_list = kosaraju_scc(&copy_graph);
//         println!("SCC ={:?}", scc_list.clone());
//         for scc in &mut scc_list {
//             let is_cycle = is_cycle(copy_graph.clone(), scc.clone());
//             if is_cycle == true {
//                 stop = false;
//                 break_down_and_mark(scc, &mut scc_id,
//                 &mut copy_graph, &mut scc_info_stk
//                 , &mut arr0);
//             }
//         }
//         println!("after break down graph = \n{:?}", Dot::with_config(&copy_graph, &[Config::EdgeIndexLabel]));
//
//         if stop==true {
//             println!("\nBREAK!\n final SCC ={:?}\n\nSCC INFO STACK", scc_list.clone());
//             for (n_idx, &ref stack) in scc_info_stk.iter() {
//                 println!("node: {:?} == {:?}", n_idx, stack);
//             }
//             break;
//         }
//     }
//
//     println!("\nafter ALL transformation: graph\n{:?}", Dot::with_config(&copy_graph, &[Config::EdgeIndexLabel]));
//     generate_path3(copy_graph.clone(), &mut scc_info_stk, arr0.clone());
//
//     return (copy_graph, g, arr0);
// }
//

//
//


// fn _evaluate_path(test_path: Vec<usize>, final_paths : &mut Vec<Vec<usize>>) -> bool{
//
//     // for path in m.values() {
//     for path in &mut *final_paths {
//         if path.to_vec() == test_path {
//             println!("NOT interesting!");
//             return false
//         }
//     }
//
//     final_paths.push(test_path.clone());
//     println!("interesting!");
//     return true
// }
//
//
//
