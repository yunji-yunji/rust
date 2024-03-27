/* 
use rustc_middle::ty::Generics;
use rustc_span::Symbol;
use rustc_hir::definitions::DisambiguatedDefPathData;
use rustc_hir::def::DefKind;

#[derive(Debug)]
struct ItemShort<'a> {
    _crate_name: Symbol,
    _def_path: Vec<DisambiguatedDefPathData>,
    _generics: &'a Generics,
    _def_kind: DefKind,
}

// test : DUMP_IN_EVAL
pub fn _dump_in_eval_entry(
    tcx: TyCtxt<'_>,
    _entry_id: DefId,
    _entry_type: EntryFnType,
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
    // println!("Open file name = {:?}", file_name);

    // === Dump All Crate Items === //
    let mut content = String::new();
    let mut dump_items = vec!();

    let module_items = tcx.hir_crate_items(());
    for item in module_items.items() {
        let def_id: DefId = item.owner_id.def_id.to_def_id();
        let _def_idx = def_id.index;
        let _crate_num = def_id.krate;

        let crate_name2 = tcx.crate_name(def_id.krate);
        content.push_str(&format!("[{:?}]", crate_name2));
        // let stable_create_id2: StableCrateId = tcx.stable_crate_id(def_id.krate);

        let def_kind: DefKind = tcx.def_kind(def_id);
        content.push_str(&format!("[{:?}]", def_kind));

        let generics = tcx.generics_of(def_id);
        // content.push_str(&format!("[{:?}]", generics));

        let def_path: DefPath = tcx.def_path(def_id);
        let def_paths: Vec<DisambiguatedDefPathData> = def_path.data;
        for item in &def_paths {
            content.push_str(&format!("[{:?}][{:?}]", item.data, item.disambiguator));
        }

        content.push_str(&format!("\n"));

        let short_item = ItemShort {
            _crate_name: crate_name2,
            _def_path: def_paths.clone(),
            _generics: generics,
            _def_kind: def_kind,
        };
        dump_items.push(short_item);
    }

    file.write_all(content.as_bytes()).expect("Fail to write file.");
}

*/