//! Codegen the MIR to the LLVM IR.
//!
//! Hopefully useful general knowledge about codegen:
//!
//! * There's no way to find out the [`Ty`] type of a [`Value`]. Doing so
//!   would be "trying to get the eggs out of an omelette" (credit:
//!   pcwalton). You can, instead, find out its [`llvm::Type`] by calling [`val_ty`],
//!   but one [`llvm::Type`] corresponds to many [`Ty`]s; for instance, `tup(int, int,
//!   int)` and `rec(x=int, y=int, z=int)` will have the same [`llvm::Type`].
//!
//! [`Ty`]: rustc_middle::ty::Ty
//! [`val_ty`]: crate::common::val_ty

use super::ModuleLlvm;

use crate::attributes;
use crate::builder::Builder;
use crate::context::CodegenCx;
use crate::llvm;
use crate::value::Value;

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


use rustc_codegen_ssa::base::maybe_create_entry_wrapper;
use rustc_codegen_ssa::mono_item::MonoItemExt;
use rustc_codegen_ssa::traits::*;
use rustc_codegen_ssa::{ModuleCodegen, ModuleKind};
use rustc_data_structures::small_c_str::SmallCStr;
use rustc_middle::dep_graph;
use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrs;
use rustc_middle::mir::mono::{Linkage, Visibility};
use rustc_middle::ty::TyCtxt;
use rustc_session::config::DebugInfo;
use rustc_span::symbol::Symbol;
use rustc_target::spec::SanitizerSet;

use std::time::Instant;

pub struct ValueIter<'ll> {
    cur: Option<&'ll Value>,
    step: unsafe extern "C" fn(&'ll Value) -> Option<&'ll Value>,
}

impl<'ll> Iterator for ValueIter<'ll> {
    type Item = &'ll Value;

    fn next(&mut self) -> Option<&'ll Value> {
        let old = self.cur;
        if let Some(old) = old {
            self.cur = unsafe { (self.step)(old) };
        }
        old
    }
}

pub fn iter_globals(llmod: &llvm::Module) -> ValueIter<'_> {
    unsafe { ValueIter { cur: llvm::LLVMGetFirstGlobal(llmod), step: llvm::LLVMGetNextGlobal } }
}

pub fn compile_codegen_unit(tcx: TyCtxt<'_>, cgu_name: Symbol) -> (ModuleCodegen<ModuleLlvm>, u64) {
    let start_time = Instant::now();

    let dep_node = tcx.codegen_unit(cgu_name).codegen_dep_node(tcx);
    let (module, _) = tcx.dep_graph.with_task(
        dep_node,
        tcx,
        cgu_name,
        module_codegen,
        Some(dep_graph::hash_result),
    );
    let time_to_codegen = start_time.elapsed();

    // We assume that the cost to run LLVM on a CGU is proportional to
    // the time we needed for codegenning it.
    let cost = time_to_codegen.as_nanos() as u64;

    fn module_codegen(tcx: TyCtxt<'_>, cgu_name: Symbol) -> ModuleCodegen<ModuleLlvm> {
        let cgu = tcx.codegen_unit(cgu_name);
        let _prof_timer =
            tcx.prof.generic_activity_with_arg_recorder("codegen_module", |recorder| {
                recorder.record_arg(cgu_name.to_string());
                recorder.record_arg(cgu.size_estimate().to_string());
            });
        // Instantiate monomorphizations without filling out definitions yet...
        let llvm_module = ModuleLlvm::new(tcx, cgu_name.as_str());
        {
            let cx = CodegenCx::new(tcx, cgu, &llvm_module);
            let mono_items = cx.codegen_unit.items_in_deterministic_order(cx.tcx);
            for &(mono_item, data) in &mono_items {
                mono_item.predefine::<Builder<'_, '_, '_>>(&cx, data.linkage, data.visibility);
            }
            match std::env::var_os("MIR_DUMP5") { 
                None => {},
                Some(val) => {
        
                    let outdir = std::path::PathBuf::from(val.clone());
                    let prefix = match std::env::var_os("PAFL_TARGET_PREFIX") {
                        None => bug!("environment variable PAFL_TARGET_PREFIX not set"),
                        Some(v) => std::path::PathBuf::from(v),
                    };
                    println!("dump custom func 5 is called");
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
                                let (_, units) = tcx.collect_and_partition_mono_items(());
                                for unit in units {
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
                                println!("out124aa2={:?}", output.to_str());
                            
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
            println!("is module_codegen executed?! please");
            match std::env::var_os("MIR_DUMP2") { 
                None => {},
                Some(val) => {
        
                    let outdir = std::path::PathBuf::from(val.clone());
                    let prefix = match std::env::var_os("PAFL_TARGET_PREFIX") {
                        None => bug!("environment variable PAFL_TARGET_PREFIX not set"),
                        Some(v) => std::path::PathBuf::from(v),
                    };
                    println!("we got env var2");
                    match cx.tcx.sess.local_crate_source_file() {
                        None => bug!("unable to locate local crate source file"),
                        Some(src) => {
                            if src.starts_with(&prefix) {
                                println!("in 31231miri codege11n@#");
                                dump(cx.tcx, &outdir);
                            }
                        }
                    }
                }
            }


            match std::env::var_os("MIR_DUMP3") { 
                None => {},
                Some(val) => {
        
                    let outdir = std::path::PathBuf::from(val.clone());
                    let prefix = match std::env::var_os("PAFL_TARGET_PREFIX") {
                        None => bug!("environment variable PAFL_TARGET_PREFIX not set"),
                        Some(v) => std::path::PathBuf::from(v),
                    };
                    println!("we got env var3");
                    match tcx.sess.local_crate_source_file() {
                        None => bug!("unable to locate local crate source file"),
                        Some(src) => {
                            if src.starts_with(&prefix) {
                                println!("in miri 112codege11n@#");
                                dump(tcx, &outdir);
                            }
                        }
                    }
                }
            }
            // ... and now that we have everything pre-defined, fill out those definitions.
            for &(mono_item, _) in &mono_items {
                mono_item.define::<Builder<'_, '_, '_>>(&cx);
            }

            // when i use "mono_items" instead of "collect_all_mono_items"
            // many parts are missing..
            // small size json is genrated..
            // for &(mono_item, _) in &mono_items {
            match std::env::var_os("MIR_DUMP4") { 
                None => {},
                Some(val) => {
        
                    let outdir = std::path::PathBuf::from(val.clone());
                    let prefix = match std::env::var_os("PAFL_TARGET_PREFIX") {
                        None => bug!("environment variable PAFL_TARGET_PREFIX not set"),
                        Some(v) => std::path::PathBuf::from(v),
                    };
                    println!("dump custom func is called");
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
                                println!("in miri cod33ege11n@#");
                                for &(mono_item, _) in &mono_items {
                                    let instance = match mono_item {
                                        MonoItem::Fn(i) => i,
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

                                let content =
                                    serde_json::to_string_pretty(&summary).expect("unexpected failure on JSON encoding");
                                let symbol = tcx.crate_name(LOCAL_CRATE);
                                let crate_name = symbol.as_str();
                                let output = path_build.join(crate_name).with_extension("json");
                                println!("out1242={:?}", output.to_str());
                            
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
            // If this codegen unit contains the main function, also create the
            // wrapper here
            if let Some(entry) = maybe_create_entry_wrapper::<Builder<'_, '_, '_>>(&cx) {
                let attrs = attributes::sanitize_attrs(&cx, SanitizerSet::empty());
                attributes::apply_to_llfn(entry, llvm::AttributePlace::Function, &attrs);
            }

            // Finalize code coverage by injecting the coverage map. Note, the coverage map will
            // also be added to the `llvm.compiler.used` variable, created next.
            if cx.sess().instrument_coverage() {
                cx.coverageinfo_finalize();
            }

            // Create the llvm.used and llvm.compiler.used variables.
            if !cx.used_statics.borrow().is_empty() {
                cx.create_used_variable_impl(c"llvm.used", &*cx.used_statics.borrow());
            }
            if !cx.compiler_used_statics.borrow().is_empty() {
                cx.create_used_variable_impl(
                    c"llvm.compiler.used",
                    &*cx.compiler_used_statics.borrow(),
                );
            }

            // Run replace-all-uses-with for statics that need it. This must
            // happen after the llvm.used variables are created.
            for &(old_g, new_g) in cx.statics_to_rauw().borrow().iter() {
                unsafe {
                    llvm::LLVMReplaceAllUsesWith(old_g, new_g);
                    llvm::LLVMDeleteGlobal(old_g);
                }
            }

            // Finalize debuginfo
            if cx.sess().opts.debuginfo != DebugInfo::None {
                cx.debuginfo_finalize();
            }
        }

        ModuleCodegen {
            name: cgu_name.to_string(),
            module_llvm: llvm_module,
            kind: ModuleKind::Regular,
        }
    }

    (module, cost)
}

pub fn set_link_section(llval: &Value, attrs: &CodegenFnAttrs) {
    let Some(sect) = attrs.link_section else { return };
    unsafe {
        let buf = SmallCStr::new(sect.as_str());
        llvm::LLVMSetSection(llval, buf.as_ptr());
    }
}

pub fn linkage_to_llvm(linkage: Linkage) -> llvm::Linkage {
    match linkage {
        Linkage::External => llvm::Linkage::ExternalLinkage,
        Linkage::AvailableExternally => llvm::Linkage::AvailableExternallyLinkage,
        Linkage::LinkOnceAny => llvm::Linkage::LinkOnceAnyLinkage,
        Linkage::LinkOnceODR => llvm::Linkage::LinkOnceODRLinkage,
        Linkage::WeakAny => llvm::Linkage::WeakAnyLinkage,
        Linkage::WeakODR => llvm::Linkage::WeakODRLinkage,
        Linkage::Appending => llvm::Linkage::AppendingLinkage,
        Linkage::Internal => llvm::Linkage::InternalLinkage,
        Linkage::Private => llvm::Linkage::PrivateLinkage,
        Linkage::ExternalWeak => llvm::Linkage::ExternalWeakLinkage,
        Linkage::Common => llvm::Linkage::CommonLinkage,
    }
}

pub fn visibility_to_llvm(linkage: Visibility) -> llvm::Visibility {
    match linkage {
        Visibility::Default => llvm::Visibility::Default,
        Visibility::Hidden => llvm::Visibility::Hidden,
        Visibility::Protected => llvm::Visibility::Protected,
    }
}
