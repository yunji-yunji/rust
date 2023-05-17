use petgraph::Graph;
use petgraph::dot::{Dot, Config};
use petgraph::algo::kosaraju_scc;
use petgraph::prelude::NodeIndex;
// use std::collections::HashSet;
// use petgraph::prelude::EdgeIndex;
// use petgraph::Incoming;

use petgraph::algo::toposort;
// use petgraph::visit::DfsPostOrder;
use petgraph::visit::Dfs;
// use petgraph::Directed;
// use petgraph::visit::EdgeRef;

// use rustc_data_structures::graph::implementation::NodeIndex;
// use petgraph::graph::NodeIndex;
use rustc_middle::ty::TyCtxt;
use rustc_middle::mir::*;

use std::fs::File;
use std::io::{BufReader, BufRead};



pub fn default_g () -> Graph<usize, String> {
    let my_g = Graph::<usize, String>::new();
    my_g
}

pub fn has_cycle2(orig: Graph<usize, String>, scc:Vec<NodeIndex>) -> bool {
    let mut new = Graph::<usize, String>::new();
    let mut i = 0;

    for _node in orig.clone().raw_nodes() {
        let _node1 = new.add_node(i);
        i += 1;
    }
    for edge in orig.clone().raw_edges() {
        let s = edge.source();
        let t = edge.target();
        // println!("{:?} {:?} ", s, t);
        if scc.contains(&s) && scc.contains(&t) {
            new.update_edge(s, t, String::from("random"));
        }
    
    }
    println!("in has cycle {:?}", scc);
    println!("new graph\n{:?}", Dot::with_config(&new, &[Config::EdgeIndexLabel]));
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


// pub fn find_all_headers(scc:Vec<NodeIndex>, g:&Graph<usize, String>) -> HashSet<NodeIndex> {
pub fn find_all_headers(scc:Vec<NodeIndex>, g:&Graph<usize, String>) -> Vec<NodeIndex> {
    // let mut headers :HashSet<NodeIndex> = HashSet::new();
    let mut headers :Vec<NodeIndex> = vec!();
    // TODO: headers should be set
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


pub fn get_all_back_edges(scc:Vec<NodeIndex>, header: NodeIndex, g:&mut Graph<usize, String>) 
->Vec<(NodeIndex, NodeIndex)> {
    // println!("IN get all back edges(");
    let mut back_edges :Vec<(NodeIndex, NodeIndex)> = vec!();
    let mut inner_edges :Vec<(NodeIndex, NodeIndex)> = vec!();
    let mut remove = false;
    println!("SCC in get back edges {:?}", scc);

    for edge in g.clone().raw_edges() {

        let mut test_g = g.clone();
        // g.remove_edge(edge_idx);
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
            // let mut dfs = DfsPostOrder::new(&test_g, edge.source());
            let mut dfs = Dfs::new(&test_g, edge.source());

            while let Some(visited) = dfs.next(&test_g) {
                // print!(" {}", visited.index());
                dfs_res.push(visited.index());
            }
            println!("dfs_res {:?}", dfs_res);
            
            // TO DO: fix

            // // if self loop
            // if (edge.source()== edge.target()) {
            //     back_edges.push((edge.source(), edge.target()));

            // }
            if dfs_res.contains(&edge.target().index()) {
                // self loop included here
                // i can reach it even after i removed it
                println!("I can reach  {:?}", edge.target().index());
                inner_edges.push((edge.source(), edge.target()));
                // inner_edges.push(edge_idx);
            } else {
                // i cannot reach it
                back_edges.push((edge.source(), edge.target()));
                // back_edges.push(edge_idx);
                remove = true;
            }
            // test_g.update_edge(arr0[0], arr0[1], String::from("1"));

        }
    }
    if remove == false {
        // cannot remove anything..
        // remove both
        println!("thereis no proper back edges, instead remove both {:?}", inner_edges);
        back_edges = inner_edges;
    }
    return back_edges;

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


pub fn _break_down_outer_once(scc:Vec<NodeIndex>, _scc_id: &mut i32, g: &mut Graph<usize, String>) {
    println!("====================== IN break_down_outer_once ==================");
    let back_edges :Vec<(NodeIndex, NodeIndex)>;
    // let back_edges :Vec<EdgeIndex>;
    let headers = find_all_headers(scc.clone(), g);
    let new_node;
    if headers.len() ==1 {
        back_edges = get_all_back_edges(scc, headers[0], g);
    } else {
        new_node = g.add_node(777);

        for header in headers {
            let predecessors = get_predecessors_of(header, g);
            for pred in predecessors {
                // redirect
                // let edge_to_remove = g.find_edge(pred, header);
                let Some(edge_to_remove) = g.find_edge(pred, header) else { 
                    continue;
                };
                g.remove_edge(edge_to_remove);
                // println!("after remove backedges\n{:?}", Dot::with_config(g, [Config::EdgeIndexLabel]));
                // println!("gra\n{:?}", Dot::with_config(g, &[Config::EdgeIndexLabel]));
        
                // g.remove_edge(edge_idx);
                g.update_edge(pred, new_node, String::from("REDIR"));
                g.update_edge(new_node, header, String::from("REDIR"));
            }
        }
        back_edges = get_all_back_edges(scc, new_node, g);
    }

    // find and remove backedges correctly
    for back_edge in back_edges {
    // for edge in g.clone().raw_edges() {
        let Some(edge_idx) = g.find_edge(back_edge.0, back_edge.1) else { 
            continue;
        };
        // println!("remove {:?} {:?}", edge_idx.source(), edge_idx.target());
        println!("remove {:?} {:?}", edge_idx, back_edge);
        g.remove_edge(edge_idx);
    // }
    }

    // FIX: cannot remove it
    // TODO: remove new node
    // g.remove_node()
    println!("done in break_down_outer_once");
}

#[derive(Debug)]
pub struct SccInfo {
    _id: i32,
    _n_type: char,
    _n_info: usize,
}

pub fn break_down_and_mark(scc: &Vec<NodeIndex>, scc_id: &mut i32, 
    g: &mut Graph<usize, String>, 
    scc_info_stk: &mut HashMap<NodeIndex, Vec<SccInfo>>,
    arr: &mut Vec<NodeIndex>) {
    
    // for i in 0..scc.len() {
    //     stk_info[scc[i].index()].push(scc_id);
    //     // stk_info array(vector) -> dictionary(key == node index, value = stack)
    //     // if not exists(== if newly added node) : not add stack 
    // }
    
    println!("====================== Break down and Mark ==================");
    let back_edges :Vec<(NodeIndex, NodeIndex)>;
    // let back_edges :Vec<EdgeIndex>;
    let headers = find_all_headers(scc.clone(), g);

    let loop_header;
    let new_node;
    if headers.len() ==1 {
        println!("case 1 {:?}", headers);
        loop_header = headers[0];
        back_edges = get_all_back_edges(scc.clone(), headers[0], g);
        // if scc.len() == 1 {
            // self loop
            // -> no backedges
            // back_edges = vec!();

        let scc_info = SccInfo {
            _id: *scc_id, 
            _n_type: 'H', 
            _n_info: back_edges.len(),
            // _n_info: 1,
        };
        scc_info_stk.get_mut(&headers[0]).map(|stk| stk.push(scc_info));

    } else {
        println!("case 2 {:?}", headers);
        new_node = g.add_node(777);
        loop_header = new_node;
        arr.push(new_node);
        scc_info_stk.insert(new_node, vec!());

        for header in headers {
            let predecessors = get_predecessors_of(header, g);
            for pred in predecessors {
                // redirect
                // let edge_to_remove = g.find_edge(pred, header);
                let Some(edge_to_remove) = g.find_edge(pred, header) else { 
                    continue;
                };
                g.remove_edge(edge_to_remove);
                g.update_edge(pred, new_node, String::from("REDIR"));
                g.update_edge(new_node, header, String::from("REDIR"));
            }
        }
        // scc = kosaraju_scc(&*g);
        let mut new_scc = scc.clone();
        new_scc.push(new_node);

        back_edges = get_all_back_edges(new_scc.clone(), new_node, g);
        println!("all back edges = {:?}", back_edges);
        let scc_info = SccInfo {
            _id: *scc_id, 
            _n_type: 'H', 
            _n_info: back_edges.len(),
        };
        scc_info_stk.get_mut(&new_node).map(|stk| stk.push(scc_info));

    }

    // find and remove backedges correctly
    let mut l_idx =0;
    let mut l_list = vec!();
    for back_edge in back_edges {
        l_list.push(back_edge.0);
        let scc_info;
        if scc.len() != 1 {
        //     scc_info = SccInfo {
        //         _id: *scc_id, 
        //         _n_type: 'S', 
        //         _n_info: l_idx,
        //     };
        // } else {
            scc_info = SccInfo {
                _id: *scc_id, 
                _n_type: 'L', 
                _n_info: l_idx,
            };
            scc_info_stk.get_mut(&back_edge.0).map(|stk| stk.push(scc_info));
            l_idx += 1;
        }
        
    // for edge in g.clone().raw_edges() {
        let Some(edge_idx) = g.find_edge(back_edge.0, back_edge.1) else { 
            continue;
        };
        // println!("remove {:?} {:?}", edge_idx.source(), edge_idx.target());
        println!("remove {:?} {:?}", edge_idx, back_edge);
        g.remove_edge(edge_idx);
    // }
    }

    for node in scc.clone() {
        if node != loop_header && !l_list.contains(&node) {
            let scc_info = SccInfo {
                _id: *scc_id, 
                _n_type: 'X', 
                _n_info: 9999,
            };
            scc_info_stk.get_mut(&node).map(|stk| stk.push(scc_info));
            
        }
    }

    // FIX: cannot remove it
    // TODO: remove new node
    // g.remove_node()
    *scc_id += 1;

    println!("done in break_down_outer_once");
}

pub fn my_app <'tcx>(_tcx: TyCtxt<'tcx>, _body: &Body<'_>) 
-> (Graph<usize, String>, Graph<usize, String>, Vec<NodeIndex>) {
    println!("\n------------ TEST graph ----------");
    let mut g = Graph::<usize, String>::new();
    let mut backup_g = Graph::<usize, String>::new();

    let mut check_done :Vec<bool> = vec!();

    let mut arr0 :Vec<NodeIndex> = vec!();
    // let mut stk_info : Vec<Vec<i32>> = vec!();
    // let mut stk_info : Vec<Vec<i32>> = vec!();
    let mut scc_info_stk : HashMap<NodeIndex, Vec<SccInfo>> = HashMap::new();

    // ===================== create graph
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
        // if i==12 {
        //     tmp_node = node1;
        // }
        let _node2 = backup_g.add_node(i);
        arr0.push(node1);
        check_done.push(false);
        // stk_info.push(vec!());
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
        // g.update_edge(arr0[9], arr0[9], String::from("13"));
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

    // =================== NEW version =================== //
    let mut scc_id : i32 = 0;
    let mut copy_graph = g.clone();
    loop {
        let mut stop  =true;
        let scc_list = kosaraju_scc(&copy_graph);
        println!("SCC ={:?}", scc_list.clone());
        for scc in &scc_list {
            let is_cycle = has_cycle2(copy_graph.clone(), scc.clone());
            if is_cycle == true {
                stop = false;
                // _break_down_outer_once(scc.clone(), &mut scc_id, &mut copy_graph);
                break_down_and_mark(scc, &mut scc_id, 
                &mut copy_graph, &mut scc_info_stk
                , &mut arr0);
            }
        }
        println!("after break down graph = \n{:?}", Dot::with_config(&copy_graph, &[Config::EdgeIndexLabel]));
        // for node in copy_graph.clone().raw_nodes() {
        //     println!("node check {:?}", node);
        // }

        // stack info =[[], [0], [0], [0], [0], [0, 3], [0, 3], [0, 3], [0, 3], [0, 2], [0, 1, 4], [0, 1, 4], [0, 1], []]
        // 
        if stop==true {
            println!("\nBREAK! final SCC ={:?}\n\n", scc_list.clone());
            println!("stack info ={:?}\n\n", scc_info_stk);
            for (n_idx, &ref stack) in scc_info_stk.iter() {
                println!("node: {:?} == {:?}", n_idx, stack);
            }
            break;
        }
    }

    println!("after stack info hash map {:?}\n\n", scc_info_stk);

    // generate_path2(backup_g.clone(), stk_info, arr0.clone());
    generate_path3(copy_graph.clone(), &mut scc_info_stk, arr0.clone());

    println!("================== test end ========================");


    return (copy_graph, g, arr0);
    // return (my_g, new_g, arr);
}



// ======= Generate final path (Discard repeated component) ============== //
pub fn generate_path3(_g: Graph::<usize, String>, 
    scc_info_stk: &mut HashMap<NodeIndex, Vec<SccInfo>>,
    arr: Vec<NodeIndex>) -> Vec<usize> {

    #[derive(Debug)]
    struct Ele {
        counts: Vec<usize>,
        temp_path: Vec<Vec<usize>>,
        prefix: Vec<usize>,
    }

    let mut fin : Vec<usize>;
    // let mut letter = String::from("");
    let limit : usize= 3;
    // let mut stk :Vec<(Vec<usize>, Vec<usize>)> = vec!()                                                                                                                                          ;
    let mut stk :Vec<Ele> = vec!()                                                                                                                                 ;
    // let mut path_stk :Vec<Vec<usize>> = vec!();
    // let mut stk :Vec<usize> = vec!();
    // let mut record = true;
    let mut is_loop = false;
    let case = 1;
    let path: Vec<usize>;
    if case ==1 {
        path = vec![
            0, 1, 2, 7, 3, 7, 4, 7, 3, 7, 3, 7, 3,3, 3,3, 3, 3,3 , 3,3, 7, 4, 7, 3, 7, 3, 7, 4, 7, 4, 5, 6];
            // 0, 1, 2, 7, 3, 7, 4, 7, 3, 7, 3, 7, 4, 7, 3, 7, 3, 7, 4, 7, 4, 5, 6];
        //    [0, 1, 2, 7, 3, 7, 4, 7, 3, 7, 3, 7, 4, 7, 4, 7, 5, 6]
    } else {
        // big graph test path
        path = vec![0, 1, 2, 3, 
        5, 6, 7, 8, 
        5, 6, 7, 8, 
        5, 6, 7, 8, 
        5, 6, 7, 8, 
        5, 6, 7, 9,
        // 9, 9, 9, 9, 9 ,9, 
        10, 11, 10, 11, 10, 11, 10, 11, 10, 11, 
        12, 13];
        // [0, 1, 2, 3, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 9, 9, 9, 9, 10, 11, 10, 11, 10, 11, 10, 11, 12, 13]
        // let path :Vec<usize>= vec![0, 1, 2, 3, 4, 3, 4, 3,4, 3, 4, 5, 6];
        // [0, 2, 3, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 9, 10, 11, 10, 11, 10, 11, 10, 11, 12, 13]
    }
    // [0, 1, 2, 3, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 8, 5, 6, 7, 9, 10, 11, 10, 11, 10, 11, 10, 11, 12, 13]

    println!("============= Generate Path ================");
    println!("path INFO: {:?} {:?} ", path.len(), path.clone());

    fin = vec!();
    fin.push(path[0] as usize);

    for idx in 0..path.len()-1 {
        let s : usize = path[idx] as usize;   // bb_n in i32 (todo => usize)
        let t : usize = path[idx+1].try_into().unwrap();
        println!("---------{:?} -> {:?}--------", s, t);
        let mut recorded = false;

        let mut s_idx = 0;
        let mut t_idx = 0;

        while s_idx < scc_info_stk[&arr[s]].len() 
            && t_idx < scc_info_stk[&arr[t]].len() 
            && scc_info_stk[&arr[s]][s_idx]._id == scc_info_stk[&arr[t]][t_idx]._id {
                s_idx += 1;
                t_idx += 1;
        }

        while s_idx < scc_info_stk[&arr[s]].len() {

            // ============= old code ============= //
            // stk.pop();
            // record = true;
            // for in_stk in &stk {
            //     for el in in_stk {
            //         if *el > limit {
            //             record = false;
            //         }
            //     }
            // }
            // ============= old code ============= //
            // println!("[3] Exit edge (POP) {:?} {:?} {:?} {} {}", stk, scc_info_stk[&arr[s]], scc_info_stk[&arr[t]], s_idx, t_idx);
            // println!("pop before {:?} {:?} {:?} {:?}", stk, tmp_path, fin, path_stk);
            if let Some(mut prev) = stk.pop() {
                prev.temp_path.push(prev.prefix.clone());

                if let Some(last) = stk.last_mut() {
                    for p in prev.temp_path {
                        for pp in p {
                            last.prefix.push(pp);
                        }
                    }
                } else {
                    for p in prev.temp_path {
                        for pp in p {
                            fin.push(pp);
                        }
                    }
                }
            }
            println!("[3] Exit edge, pop after {:?} {:?} {:?}", stk, is_loop, fin);

            s_idx += 1;
            is_loop=false;

            // let prev_path = path_stk.last_mut().unwrap();
            // path_stk.pop();
            // if path_stk.last_mut() != None {
            //     for p in prev_path {
            //         path_stk.last_mut().unwrap().push(*p);
            //     }
            // } else {
            //     for p in prev_path {
            //         fin.push(*p);
            //     }    
            // }

            // tmp_path.push(t);
            // for p in &tmp_path {
            //     fin.push(*p);
            // }
            // println!("temp path = {:?} {:?} ", tmp_path, path_stk);
            // tmp_path = vec!();e
            // is_loop = false;
        } 
        // self loop
        // if s == t {
            // tmp_path.push(t);
            // println!("[2-1] self loop edge {:?} {:?} {:?} {:?} {} {}", record, stk, scc_info_stk[&arr[s]], scc_info_stk[&arr[t]], s_idx, t_idx);
            // println!("SELF LOOP {:?} {:?} {:?} {:?}", stk, tmp_path, fin, path_stk);
            // let a :usize = 1;
            // stk.last_mut().unwrap()[0] += a;
            // if stk.last_mut().unwrap()[0] > limit {
            //         // record = false;
            //     } else {
            //         // record = true;
            //         // for p in &tmp_path {
            //         //     fin.push(*p);
            //         // }
            //         // println!("self loop =- temp path = {:?} {:?}", tmp_path, fin);
            //         // tmp_path = vec!();
            //     }
        // } else {
            // 5 [0, 1, 2, 7, 3]
            s_idx = 0;
            t_idx = 0;
            while s_idx < scc_info_stk[&arr[s]].len() 
            && t_idx < scc_info_stk[&arr[t]].len() 
            && scc_info_stk[&arr[s]][s_idx]._id == scc_info_stk[&arr[t]][t_idx]._id {
                is_loop=true;
                
                if s== t { // self loop
                    if let Some(last) = stk.last_mut() {
                        // option 1
                        if recorded==false {
                            last.prefix.push(t);
                            recorded=true;
                        }
                        let a :usize = 1;
                        last.counts[0] += a;
                        if last.counts[0] <= limit {
                            // option 2
                            // if recorded==false {
                            //     last.prefix.push(t);
                            //     recorded=true;
                            // }
                            last.temp_path.push(last.prefix.clone());
                        }
                        last.prefix = vec!();
                    }
                    s_idx = scc_info_stk[&arr[s]].len();
                    t_idx = scc_info_stk[&arr[t]].len();
                    break;
                }

                if scc_info_stk[&arr[s]][s_idx]._n_type == 'L' 
                && scc_info_stk[&arr[t]][t_idx]._n_type == 'H' {
                    // back edge

                    if let Some(last) = stk.last_mut() {
                        // option 1
                        if recorded==false {
                            last.prefix.push(t);
                            recorded=true;
                        }
                        let a :usize = 1;
                        last.counts[scc_info_stk[&arr[s]][s_idx]._n_info] += a;
                        if last.counts[scc_info_stk[&arr[s]][s_idx]._n_info] <= limit {
                            // option 2
                            // if recorded==false {
                            //     last.prefix.push(t);
                            //     recorded=true;
                            // }
                            last.temp_path.push(last.prefix.clone());
                        }
                        last.prefix = vec!();
                    }

                    // if stk.last_mut().unwrap().counts[scc_info_stk[&arr[s]][s_idx]._n_info] < limit {
                    //     // record = false;
                    //     stk.last_mut().unwrap().temp_path.push(stk.last_mut().unwrap().prefix);
                    //     stk.last_mut().unwrap().prefix = vec!();
                    // } 
                    // -================================================== old code -==================================================
                    // println!("[1] back edge {:?} {:?} {:?} {:?} {:?} {:?} {:?} {} {}", scc_info_stk[&arr[s]],scc_info_stk[&arr[s]][s_idx], scc_info_stk[&arr[s]][s_idx]._n_info, record, stk, scc_info_stk[&arr[s]], scc_info_stk[&arr[t]], s_idx, t_idx);
                    // let a :usize = 1;
                    // stk.last_mut().unwrap()[scc_info_stk[&arr[s]][s_idx]._n_info] += a;
                    // if stk.last_mut().unwrap()[scc_info_stk[&arr[s]][s_idx]._n_info] > limit {
                    //     record = false;
                    // } else {
                    //     record = true;
                    // println!("[1] back edge {:?}, {:} -> {:?}", stk, s, t);
                    
                    println!("[2] back edge" );
                    for e in &stk {
                        println!("  * {:?}", e);
                    }// for p in &tmp_path {
                    //         // fin.push(*p);
                    //         path_stk.last_mut().unwrap().push(*p);
                    //     }
                    //     tmp_path = vec!();
                    // }
                    // println!("normal back edge= temp path = {:?} {:?}", tmp_path, fin);

                    // println!("{:?} {:?}", record, stk.last_mut().unwrap()[scc_info_stk[&arr[s]][s_idx]._n_info]);
                    //-================================================== old code -==================================================
                    // back_idx = s_idx;
                } 
            // else if scc_info_stk[&arr[s]][s_idx]._n_type == 'H' 
            // && scc_info_stk[&arr[t]][t_idx]._n_type == 'H' {
            //     println!("[2] self loop edge {:?} {:?} {:?} {:?} {} {}", record, stk, scc_info_stk[&arr[s]], scc_info_stk[&arr[t]], s_idx, t_idx);
            //     let a :usize = 1;
            //     stk.last_mut().unwrap()[0] += a;
            //     println!("{:?} ", record);
            //     // stk.last_mut().unwrap()[scc_info_stk[&arr[s]][s_idx]._n_info] += a;
            // }
                else {
                    // [SccInfo { _id: 0, _n_type: 'L', _n_info: 0 }, 
                    // SccInfo { _id: 1, _n_type: 'H', _n_info: 1 }, 
                    // SccInfo { _id: 4, _n_type: 'H', _n_info: 1 }] 
                    
                    // [SccInfo { _id: 0, _n_type: 'L', _n_info: 1 }, 
                    // SccInfo { _id: 1, _n_type: 'X', _n_info: 9999 }, 
                    // SccInfo { _id: 4, _n_type: 'L', _n_info: 0 }]
                    if recorded==false {
                        stk.last_mut().unwrap().prefix.push(t);
                        recorded=true;
                    }
                    // println!("[2] normal edge {:?} {:?} {:?} {:?} {} {}", stk, is_loop, scc_info_stk[&arr[s]], scc_info_stk[&arr[t]], s_idx, t_idx);
                    println!("[1] normal edge" );
                    for e in &stk {
                        println!("  * {:?}", e);
                    }
                    
                    // tmp_path.push(t);
                    // if scc_info_stk[&arr[t]][t_idx]._n_type == 'L' && stk.last_mut().unwrap()[scc_info_stk[&arr[t]][t_idx]._n_info] < limit {
                    //     record = true;
                    // } 
                    // else {
                    //     record = false;
                    // }
                }
                s_idx += 1;
                t_idx += 1;
        }
        // }

        // header node, entering edge
        while t_idx < scc_info_stk[&arr[t]].len() {
            // scc_info_stk[&arr[t]][t_idx]
            is_loop = true;
            // == new vesion using temp path
            //= ================ push stdck
            let mut tmp = vec!();
            for _i in 0..scc_info_stk[&arr[t]][t_idx]._n_info {
                tmp.push(0);
            }

            let ele1;
            // in case that never meet back edge
            if recorded {
                ele1 = Ele {
                    counts: tmp, 
                    temp_path: vec!(vec!()), 
                    prefix: vec!(),
                };
            } else {
                ele1 = Ele {
                    counts: tmp, 
                    temp_path: vec!(vec!(t)), 
                    prefix: vec!(),
                };
                recorded = true;
            }

            stk.push(ele1);
            //= ================ push stdck
            // tmp_path = vec!();
            // tmp_path.push(t);
            // path_stk.push(vec!());
            // println!("header node temp path = {:?} {:?}", tmp_path, fin);

            t_idx += 1;
            // println!("[4] Entering edge (Push) {:?} {:?} {:?} {} {}", stk, scc_info_stk[&arr[s]], scc_info_stk[&arr[t]], s_idx, t_idx);
            println!("[4] Entering edge (Push)" );
            for e in &stk {
                println!("  * {:?}", e);
            }
        }
            // }
            // println!("push into fin: {:?} {:?}", back_idx, fin);

        // if record{
        if is_loop == false {
            fin.push(t);
            // println!("[5] not loop is loop? {:?} {:?} {:?}", is_loop, stk, t);
            println!("[5] not loop" );
            for e in &stk {
                println!("  * {:?}", e);
            }
                    // back_idx < scc_info_stk[&arr[s]].len() && 
            // stk.last_mut().unwrap()[scc_info_stk[&arr[s]][back_idx]._n_info] < limit {
        }
        println!("-----------------");
    }
    
    println!("fin: {:?} {:?}", fin.len(), fin);
    return fin;
    //// evaluate_path(fin, &mut final_paths);
    //// final_paths.push(fin.clone());
}

// ========================= TODO: remove OLD CODE ========================= //
fn _node_in_which_scc(n_idx: NodeIndex, sccs: Vec<Vec<NodeIndex>>) -> usize {
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
use std::collections::HashMap;

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

fn _mir_to_my_graph() {

    // ====================== = mir basic block to Graph part = ======================
    // let mut my_g = Graph::<usize, String>::new();
    // let new_g = Graph::<usize, String>::new();
    // let mut copy_g = Graph::<usize, String>::new();

    // let mut cnt: usize = 0;
    // let mut arr = vec![];
    // for _tmp in body.basic_blocks.iter() {
    //     let node1 = my_g.add_node(cnt);
    //     let _node2 = copy_g.add_node(cnt);
    //     arr.push(node1);
    //     cnt = cnt + 1;
    // }

    // for (source, _) in body.basic_blocks.iter_enumerated() {
    //     // let def_id = body.source.def_id();
    //     // let def_name = format!("{}_{}", def_id.krate.index(), def_id.index.index(),);

    //     let terminator = body[source].terminator();
    //     let labels = terminator.kind.fmt_successor_labels();

    //     for (target, _label) in terminator.successors().zip(labels) {

    //         my_g.update_edge(arr[source.index()], arr[target.index()], String::from(""));
    //         copy_g.update_edge(arr[source.index()], arr[target.index()], String::from(""));

    //     }
    // }

    // // println!("{:?}", Dot::with_config(&my_g, &[Config::EdgeIndexLabel]));


    // // println!("<<<<new graph>>>> {:?}", Dot::with_config(&my_g.clone(), &[Config::EdgeIndexLabel]));
    // println!("## NEW GRAPH ##");
    // for edge in my_g.clone().raw_edges() {
    //     println!("{:?}", edge);
    // }
    // // my_g = clone
}
