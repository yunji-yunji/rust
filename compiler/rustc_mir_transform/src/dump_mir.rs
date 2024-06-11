//! This pass just dumps MIR at a specified point.

use std::fs::File;
// use std::fs::{self, OpenOptions};
use std::io;
// use std::io::Write;

use crate::MirPass;
use rustc_middle::mir::write_mir_pretty;
use rustc_middle::mir::Body;
use rustc_middle::ty::TyCtxt;
use rustc_session::config::{OutFileName, OutputType};

use rustc_middle::bug;
// use rustc_middle::mir::mono::MonoItem;
// use rustc_middle::ty::ParamEnv;
// use rustc_middle::ty::context::{PaflDump, PaflCrate,};
// use rustc_span::def_id::{LOCAL_CRATE};
use rustc_codegen_ssa::pafl::dump;
// use rustc_data_structures::sync;
// use rustc_data_structures::fx::FxHashMap;
// use rustc_monomorphize::collector::{self, MonoItemCollectionStrategy};
// use rustc_monomorphize::partitioning::{partition, assert_symbols_are_distinct};
// use rustc_monomorphize::errors::UnknownCguCollectionMode;

pub struct Marker(pub &'static str);

impl<'tcx> MirPass<'tcx> for Marker {
    fn name(&self) -> &'static str {
        self.0
    }

    fn run_pass(&self, _tcx: TyCtxt<'tcx>, _body: &mut Body<'tcx>) {}
}

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
                    // if src.starts_with(&prefix) {
                    if src.clone().into_local_path().expect("get local path").starts_with(&prefix) {
                        println!("Before dump$$$! {:?}", src.clone());
                        dump(tcx, &outdir);
                    }
                }
            }
        }
    };

    match tcx.output_filenames(()).path(OutputType::Mir) {
        OutFileName::Stdout => {
            let mut f = io::stdout();
            write_mir_pretty(tcx, None, &mut f)?;
        }
        OutFileName::Real(path) => {
            let mut f = io::BufWriter::new(File::create(&path)?);
            write_mir_pretty(tcx, None, &mut f)?;
            if tcx.sess.opts.json_artifact_notifications {
                tcx.dcx().emit_artifact_notification(&path, "mir");
            }
        }
    }
    Ok(())
}
