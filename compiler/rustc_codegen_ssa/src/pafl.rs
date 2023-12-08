use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path};

use rustc_data_structures::fx::FxHashMap;
use rustc_middle::mir::mono::MonoItem;
use rustc_middle::ty::{ParamEnv, TyCtxt,};
use rustc_middle::ty::context::{PaflDump, PaflCrate, };
use rustc_span::def_id::{LOCAL_CRATE};

/// A complete dump of both the control-flow graph and the call graph of the compilation context
// pub fn dump(tcx: TyCtxt<'_>, outdir: &Path) {
pub fn dump<'tcx>(tcx: TyCtxt<'tcx>, outdir: &Path) {
    println!("dump func is called (old)");
    // prepare directory layout
    fs::create_dir_all(outdir).expect("unable to create output directory");
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

    let (_, units) = tcx.collect_and_partition_mono_items(());
    for unit in units {
        println!("unit {:?}---------------", unit);
        for item in unit.items().keys() {
            println!("* {:?}", item);

            // filter
            let instance = match item {
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
        println!("===========================");

    }

    // dump output
    let content =
        serde_json::to_string_pretty(&summary).expect("unexpected failure on JSON encoding");
    let symbol = tcx.crate_name(LOCAL_CRATE);
    let crate_name = symbol.as_str();
    let output = path_build.join(crate_name).with_extension("json");
    println!("out={:?}", output.to_str());

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
