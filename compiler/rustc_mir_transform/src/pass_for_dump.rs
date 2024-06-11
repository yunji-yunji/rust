//! This pass just dumps MIR at a specified point.

// use std::fs::File;
use crate::MirPass;
use rustc_middle::mir::Body;
use rustc_middle::ty::TyCtxt;
// use rustc_session::config::{OutFileName, OutputType};
use rustc_codegen_ssa::pafl::dump;
use rustc_middle::ty::print::with_no_trimmed_paths;

pub struct Dump;

impl<'tcx> MirPass<'tcx> for Dump {

    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {

        match std::env::var_os("MYPASS_DUMP") {
            None => (),
            Some(_val) => {

                let outdir = std::path::PathBuf::from("/home/y23kim/rust/last_rust/aptos-core/third_party/move/move-bytecode-verifier/src/regression_tests/fuzz");
                let prefix = std::path::PathBuf::from("third_party/move");
                with_no_trimmed_paths!({
                    match tcx.sess.local_crate_source_file() {
                        None => bug!("unable to locate local crate source file"),
                        Some(src) => {
                            if src.into_local_path().expect("get local path").starts_with(&prefix) {
                                // if src.starts_with(&prefix) {
                                println!("my pass dump: {:?}", src.clone());
                                dump(tcx, &outdir);
                            }
                        }
                    }
                });
            }
        }

        match std::env::var_os("MYPASS_CFG") {
            None => (),
            Some(val) => {
                let name = match val.into_string() {
                    Ok(s) =>{ s },
                    Err(_e) => { panic!("wrong env var") },
                };
                with_no_trimmed_paths!({
                    let instance_def = body.source.instance;
                    let def_id = instance_def.def_id();
                    let krate = tcx.crate_name(def_id.krate).to_string();
                    let path = tcx.def_path(def_id).to_string_no_crate_verbose();
                    if krate.contains(&name) | path.contains(&name) {
                        println!("-{:?}{:?}[{:?}] -------------------------", 
                        krate, path, body.basic_blocks.clone().len());
                        
                        for (source, _) in body.basic_blocks.iter_enumerated() {
                            let bb_data = &body.basic_blocks[source];
                            println!("* [{:?}][{:?}][{:?}][{:?}]", 
                            source, bb_data.statements.len(), bb_data.terminator.clone().unwrap().kind
                            , bb_data.statements);
                        }
                        println!("--------------------------");
                    }
                });
            }
        }

    }
}
