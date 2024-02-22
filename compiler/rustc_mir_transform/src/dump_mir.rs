//! This pass just dumps MIR at a specified point.

// use std::fs::File;
use std::io;

use crate::MirPass;
// use rustc_middle::mir::write_mir_pretty;
use rustc_middle::mir::Body;
use rustc_middle::ty::TyCtxt;
// use rustc_session::config::{OutFileName, OutputType};
use rustc_codegen_ssa::pafl::dump;

pub struct Marker(pub &'static str);

impl<'tcx> MirPass<'tcx> for Marker {
    fn name(&self) -> &'static str {
        self.0
    }

    fn run_pass(&self, tcx: TyCtxt<'tcx>, _body: &mut Body<'tcx>) {
    match std::env::var_os("YJPAFL") {
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
                        println!("Before dump2");
                        dump(tcx, &outdir);
                    }
                }
            }
        }
    };

    }
}

pub fn emit_mir(tcx: TyCtxt<'_>) -> io::Result<()> {
    // match tcx.output_filenames(()).path(OutputType::Mir) {
    //     OutFileName::Stdout => {
    //         let mut f = io::stdout();
    //         write_mir_pretty(tcx, None, &mut f)?;
    //     }
    //     OutFileName::Real(path) => {
    //         let mut f = io::BufWriter::new(File::create(&path)?);
    //         write_mir_pretty(tcx, None, &mut f)?;
    //     }
    // }
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
    };
    Ok(())
}
