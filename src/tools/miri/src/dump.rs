use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use rustc_middle::ty::{TyCtxt, Generics};

use rustc_session::StableCrateId;
use rustc_session::config::EntryFnType;

use rustc_span::Symbol;
use rustc_span::sym::crate_name;
use rustc_span::def_id::{LocalDefId, DefIndex, CrateNum};

use rustc_hir::definitions::{DefPath, DisambiguatedDefPathData};
use rustc_hir::def_id::{LOCAL_CRATE, DefId};
use rustc_hir::def::DefKind;

#[derive(Debug)]
struct ItemShort<'a> {
    _crate_name: Symbol,
    _def_path: Vec<DisambiguatedDefPathData>,
    _generics: &'a Generics,
    _def_kind: DefKind,
}

pub fn dump_in_eval_entry(
    tcx: TyCtxt<'_>,
    entry_id: DefId,
    entry_type: EntryFnType,
    outdir: &Path,
) {
    // === File setup === //
    fs::create_dir_all(outdir).expect("Fail to open directory.");
    let stable_create_id: StableCrateId = tcx.stable_crate_id(LOCAL_CRATE);
    // let symbol = tcx.crate_name(LOCAL_CRATE);
    // let file_name = symbol.as_str();
    let file_name = stable_create_id.as_u64().to_string();
    let output = outdir.join(file_name).with_extension("json");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .expect("Fail to create a file.");

    // === Dump All Crate Items === //
    let mut content = String::new();
    let mut dump_items = vec!();

    let module_items = tcx.hir_crate_items(());
    for item in module_items.items() {
        let def_id: DefId = item.owner_id.def_id.to_def_id();
        let def_idx = def_id.index;
        let crate_num = def_id.krate;

        let crate_name2 = tcx.crate_name(LOCAL_CRATE);
        let stable_crate_id = tcx.stable_crate_id(LOCAL_CRATE);
        content.push_str(&format!("[{:?}]", crate_name2));

        let def_kind: DefKind = tcx.def_kind(def_id);
        content.push_str(&format!("[{:?}]", kin));

        let generics = tcx.generics_of(def_id);
        // content.push_str(&format!("[{:?}]", generics));

        let def_path: DefPath = tcx.def_path(def_id);
        let def_paths: Vec<DisambiguatedDefPathData> = def_path.data;
        for item in &def_paths {
            content.push_str(&format!("[{:?}][{:?}]", item.data, item.disambiguator));
        }

        content.push_str(&format!("\n"));

        let short_item = ItemShort {
            _crate_name: crate_name,
            _def_path: def_paths.clone(),
            _generics: generics,
            _def_kind: def_kind,
        };
        dump_items.push(short_item);
    }

    file.write_all(content.as_bytes()).expect("Fail to write file.");
}
