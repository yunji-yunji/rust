//! This pass just dumps MIR at a specified point.

// use std::fs::File;
use std::io;

use crate::MirPass;
// use rustc_middle::mir::write_mir_pretty;
use rustc_middle::mir::Body;
use rustc_middle::ty::TyCtxt;
// use rustc_session::config::{OutFileName, OutputType};
use rustc_codegen_ssa::pafl::dump;
use rustc_middle::ty::context::{PaflDump, PaflCrate,};
use std::fs::{self, OpenOptions};
use std::io::Write;
// use std::path::Path;
use rustc_data_structures::fx::FxHashMap;
use rustc_middle::bug;
use rustc_middle::mir::mono::MonoItem;
use rustc_middle::ty::ParamEnv;
use rustc_span::def_id::{LOCAL_CRATE};

use rustc_data_structures::sync;
// use rustc_monomorphize::collector::UsageMap;
use rustc_monomorphize::collector::{self, MonoItemCollectionStrategy};
use rustc_monomorphize::partitioning::{partition, assert_symbols_are_distinct};
use rustc_monomorphize::errors::UnknownCguCollectionMode;
pub struct Marker(pub &'static str);

impl<'tcx> MirPass<'tcx> for Marker {
    fn name(&self) -> &'static str {
        self.0
    }

    fn run_pass(&self, _tcx: TyCtxt<'tcx>, _body: &mut Body<'tcx>) {
    }
}

#[allow(rustc::potential_query_instability)]
pub fn emit_mir(tcx: TyCtxt<'_>) -> io::Result<()> {
    println!("emit_mir is called");
    match std::env::var_os("PAFL_EMIT") {
        None => {},
        Some(val) => {
            let outdir = std::path::PathBuf::from(val.clone());
            let prefix = match std::env::var_os("PAFL_TARGET_PREFIX") {
                None => bug!("environment variable PAFL_TARGET_PREFIX not set"),
                Some(v) => std::path::PathBuf::from(v),
            };
            match tcx.sess.local_crate_source_file() {
                None => bug!("unable to locate local crate source file"),
                Some(src) => {
                    if src.starts_with(&prefix) {
                        println!("Before dump$$$! {:?}", src.clone());
                        dump(tcx, &outdir);
                    }
                }
            }
        }
<<<<<<< HEAD
        OutFileName::Real(path) => {
            let mut f = io::BufWriter::new(File::create(&path)?);
            write_mir_pretty(tcx, None, &mut f)?;
            if tcx.sess.opts.json_artifact_notifications {
                tcx.dcx().emit_artifact_notification(&path, "mir");
            }
        }
    }
=======
    };

    match std::env::var_os("PAFL_EMIT2") { 
        None => {},
        Some(_val) => {
            let outdir = std::path::PathBuf::from("/home/y23kim/rust/last_rust/aptos-core/third_party/move/move-bytecode-verifier/src/regression_tests/fuzz");
            let prefix = std::path::PathBuf::from("third_party/move");
            println!("My custom dump");

            // prepare directory layout
            fs::create_dir_all(outdir.clone()).expect("unable to create output directory");
            let path_meta = outdir.join("meta");
            fs::create_dir_all(&path_meta).expect("unable to create meta directory");
            let path_data = outdir.join("data");
            fs::create_dir_all(&path_data).expect("unable to create data directory");
            let path_build = outdir.join("build");
            fs::create_dir_all(&path_build).expect("unable to create build directory");
        
            // verbosity
            let verbose = std::env::var_os("PAFL_VERBOSE")
                .and_then(|v| v.into_string().ok())
                .map_or(false, |v| v.as_str() == "1");
        
            // extract the mir for each codegen unit
            let mut cache = FxHashMap::default();
            let mut summary = PaflCrate { functions: Vec::new() };
        
            match tcx.sess.local_crate_source_file() {
                None => bug!("unable to locate local crate source file"),
                Some(src) => {
                    if src.starts_with(&prefix) {
                        println!("in miri aacod33ege11n@#");
                        // let (_, units) = tcx.collect_and_partition_mono_items(());
                        let collection_mode = match tcx.sess.opts.unstable_opts.print_mono_items {
                            Some(ref s) => {
                                let mode = s.to_lowercase();
                                let mode = mode.trim();
                                if mode == "eager" {
                                    MonoItemCollectionStrategy::Eager
                                } else {
                                    if mode != "lazy" {
                                        tcx.dcx().emit_warn(UnknownCguCollectionMode { mode });
                                    }
                    
                                    MonoItemCollectionStrategy::Lazy
                                }
                            }
                            None => {
                                if tcx.sess.link_dead_code() {
                                    MonoItemCollectionStrategy::Eager
                                } else {
                                    MonoItemCollectionStrategy::Lazy
                                }
                            }
                        };
                        let (items, usage_map) = collector::collect_crate_mono_items(tcx, collection_mode);

                        let (codegen_units, _) = tcx.sess.time("partition_and_assert_distinct_symbols", || {
                            sync::join(
                                || {
                                    let mut codegen_units = partition(tcx, items.iter().copied(), &usage_map);
                                    codegen_units[0].make_primary();
                                    &*tcx.arena.alloc_from_iter(codegen_units)
                                },
                                || assert_symbols_are_distinct(tcx, items.iter()),
                            )
                        });
                        for unit in codegen_units {
                            for mono_item in unit.items().keys() {
                                let instance = match mono_item {
                                    MonoItem::Fn(i) => *i,
                                    MonoItem::Static(_) => continue,
                                    MonoItem::GlobalAsm(_) => bug!("unexpected assembly"),
                                };
                                if !instance.def_id().is_local() {
                                    continue;
                                }
                    
                                // process it and save the result to summary
                                let mut stack = vec![];
                                PaflDump::summarize_instance(
                                    tcx,
                                    ParamEnv::reveal_all(),
                                    instance,
                                    verbose,
                                    &path_meta,
                                    &path_data,
                                    &mut stack,
                                    &mut cache,
                                    &mut summary.functions,
                                );
                                if !stack.is_empty() {
                                    bug!("unbalanced call stack");
                                }
                            }
                        }

                        let content =
                            serde_json::to_string_pretty(&summary).expect("unexpected failure on JSON encoding");
                        let symbol = tcx.crate_name(LOCAL_CRATE);
                        let crate_name = symbol.as_str();
                        let output = path_build.join(crate_name).with_extension("json");
                        println!("out_crate={:?}", output.to_str());
                    
                        // let mut file = OpenOptions::new()
                        //     .write(true)
                        //     .create_new(true)
                        //     .open(output)
                        //     .expect("unable to create output file");
                        let mut file = OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .create(true)
                            .open(output)
                            .expect("unable to create output file2");
                        file.write_all(content.as_bytes()).expect("unexpected failure on outputting to file");
                    }
                }
            }
        }
    }

>>>>>>> d7ee2f7dfce (code clean up)
    Ok(())
}
