use crate::const_eval::CompileTimeEvalContext;
use super::InterpCx;
// use super::InterpResult;
use super::Machine;

use either::Either;
// use rustc_middle::query::TyCtxtAt;
use std::path::Path;
use rustc_middle::mir::Body;
use rustc_middle::ty::TyCtxt;

use std::fs;
use std::fs::OpenOptions;

use rustc_hir::def::DefKind;
use rustc_hir::def_id::{LOCAL_CRATE, DefId};
use rustc_hir::definitions::{DefPath, DisambiguatedDefPathData};

// use ansi_term::Colour;
use colored::Colorize;

pub fn dump_in_eval_query( // eval_queries.rs => DUMP_ON
    tcx: TyCtxt<'_>,
    body: &Body<'_>,
    outdir: &Path,
) {
    match std::env::var_os("FILE") {
        None => (),
        Some(_val) => {
            // === File setup === //
            fs::create_dir_all(outdir).expect("Fail to open directory.");
            let symbol = tcx.crate_name(LOCAL_CRATE);
            let file_name = symbol.as_str();
            // let stable_create_id: StableCrateId = tcx.stable_crate_id(LOCAL_CRATE);
            // let file_name = stable_create_id.as_u64().to_string();
            println!("FILE: outdir{:?} file_name {:?}", outdir, file_name);
            let output = outdir.join(file_name).with_extension("json");
            let mut _file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)
                .expect("Fail to create a file.");
        }
    }
    let mut content = String::new();

    let instance_def = body.source.instance;
    let def_id: DefId = instance_def.def_id();
    
    let crate_name2 = tcx.crate_name(def_id.krate);
    content.push_str(&format!("[{:?}]", crate_name2));
    let s1 = format!("[{:?}]", crate_name2);
    print!("{}", s1.red());

    let def_kind: DefKind = tcx.def_kind(def_id);
    content.push_str(&format!("[{:?}]", def_kind));
    let s2 = format!("[{:?}]", def_kind);
    print!("{}", s2.blue());

    let def_path: DefPath = tcx.def_path(def_id);
    let def_paths: Vec<DisambiguatedDefPathData> = def_path.data;
    for item in &def_paths {
        content.push_str(&format!("[{:?}][{:?}]", item.data, item.disambiguator));
        let s3 = format!("[{:?}][{:?}]", item.data, item.disambiguator);
        print!("{}", s3.green());
    }
    println!("");
    // println!("{:?}", content);
    let _tmp = content;
    
    // file.write_all(content.as_bytes()).expect("Fail to write file.");
}

pub fn bb_dump<'mir, 'tcx>(
    ecx: &CompileTimeEvalContext<'mir, 'tcx>
) {
    
    // let bbs = body.basic_blocks.as_mut();
    let loc= ecx.frame().loc;
    // let bb_id = ecx.frame().loc.left();
    if let Either::Left(l_loc) = loc {
        let block = l_loc.block;
        let statement_idx = l_loc.statement_index;
        println!("[s][{:?}][{:?}]\n", block, statement_idx);
        // let bb_id = 
        // info!("// executing {:?}", loc.block);
    }
    // self.frame_mut().loc.as_mut().left().unwrap().statement_index += 1;

    // let bbs = &body.basic_blocks;
    // println!("{:?}", bbs);
}


pub fn dump_in_step( // step.rs => STEP
    // ecx: &CompileTimeEvalContext<'mir, 'tcx>
    tcx: TyCtxt<'_>,
    body: &Body<'_>,
    // outdir: &Path,
) {
    print!("{}", "[step]".green());
    let outdir = std::path::PathBuf::from("/home/y23kim/aptos-core/third_party/move/move-bytecode-verifier/src/regression_tests/dump_yj");
    dump_in_eval_query(tcx, body, &outdir);
}

impl<'mir, 'tcx: 'mir, M: Machine<'mir, 'tcx>> InterpCx<'mir, 'tcx, M> {
    #[inline(always)]
    pub fn bb_dump_in_step(&mut self) { // step.rs => STEPP
        // Implementation of the function to dump basic blocks
        // Access fields and methods of InterpCx using `self`
        // ...
        if let Some(last) = self.stack().last() {
            // crate information
            let body = self.body();
            // DIFF BETWEEN TyCtxt and TyXtxtAt
            let tcx = self.tcx; // self.tcx.tcx 
            let instance_def = body.source.instance;
            let def_id: DefId = instance_def.def_id();
            let crate_name2 = tcx.crate_name(def_id.krate);
            let s1 = format!("[{:?}]", crate_name2);
            print!("{}", s1.red());
        
            let def_kind: DefKind = tcx.def_kind(def_id);
            let s2 = format!("[{:?}]", def_kind);
            print!("{}", s2.blue());
        
            let def_path: DefPath = tcx.def_path(def_id);
            let def_paths: Vec<DisambiguatedDefPathData> = def_path.data;
            for item in &def_paths {
                let s3 = format!("[{:?}][{:?}]", item.data, item.disambiguator);
                print!("{}", s3.green());
            }
            println!("");

            // BASIC BLOCK and statement number
            let loc = last.loc;

            if let Either::Left(l_loc) = loc {
                let block = l_loc.block;
                let statement_idx = l_loc.statement_index;
                println!("[{:?}][{:?}]", block, statement_idx);
                // let bb_id = 
                // info!("// executing {:?}", loc.block);
            }

        }
        // let loc= self.frame().loc;
        // // let bb_id = ecx.frame().loc.left();
        // if let Either::Left(l_loc) = loc {
        //     let block = l_loc.block;
        //     let statement_idx = l_loc.statement_index;
        //     println!("[{:?}][{:?}]\n", block, statement_idx);
        //     // let bb_id = 
        //     // info!("// executing {:?}", loc.block);
        // }
    }
}

// pub fn bb_dump_in_step<'mir, 'tcx>(
//     ecx: &CompileTimeEvalContext<'mir, 'tcx>
// ) {
    
//     // let bbs = body.basic_blocks.as_mut();

//     // self.frame_mut().loc.as_mut().left().unwrap().statement_index += 1;

//     // let bbs = &body.basic_blocks;
//     // println!("{:?}", bbs);

// }