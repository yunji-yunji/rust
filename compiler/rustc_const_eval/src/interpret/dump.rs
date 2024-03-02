use super::InterpCx;
use super::Machine;
use either::Either;
use std::path::Path;
use std::path::PathBuf;

use rustc_data_structures::fx::FxHashMap;

use rustc_middle::mir::Body;
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::ParamEnv;
use rustc_middle::ty::GenericArgKind;
use rustc_middle::ty::Instance;
use rustc_middle::ty::context::{Step, PaflType, PaflGeneric, FnInstKey,};

use std::fs;

use rustc_hir::def::DefKind;
use rustc_hir::def_id::{DefId};
use rustc_hir::definitions::{DefPath, DisambiguatedDefPathData};

use rustc_middle::ty::context::{PaflDump, PaflCrate};

use colored::Colorize;

// Test: env var LOG_EVAL
pub fn log_in_eval_query( // eval_queries.rs => DUMP_ON
    tcx: TyCtxt<'_>,
    body: &Body<'_>,
) {

    let instance_def = body.source.instance;
    let def_id: DefId = instance_def.def_id();
    
    let crate_name2 = tcx.crate_name(def_id.krate);
    // content.push_str(&format!("[{:?}]", crate_name2));
    let s1 = format!("[{:?}]", crate_name2);
    print!("{}", s1.red());

    let def_kind: DefKind = tcx.def_kind(def_id);
    // content.push_str(&format!("[{:?}]", def_kind));
    let s2 = format!("[{:?}]", def_kind);
    print!("{}", s2.blue());

    let def_path: DefPath = tcx.def_path(def_id);
    let def_paths: Vec<DisambiguatedDefPathData> = def_path.data;
    for item in &def_paths {
        // content.push_str(&format!("[{:?}][{:?}]", item.data, item.disambiguator));
        let s3 = format!("[{:?}][{:?}]", item.data, item.disambiguator);
        print!("{}", s3.green());
    }
    println!("");
    // println!("{:?}", content);
}

impl<'mir, 'tcx: 'mir, M: Machine<'mir, 'tcx>> InterpCx<'mir, 'tcx, M> {
    #[inline(always)]
    pub fn bb_dump_in_step(&mut self) { // step.rs

        // Implementation of the function to dump basic blocks
        // Access fields and methods of InterpCx using `self`
        // ...
        if let Some(last) = self.stack().last() {
            // crate information
            let body = self.body();
            // what is DIFFerence BETWEEN TyCtxt and TCXtxtAt
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
    }

    pub fn dump_return(&mut self, outdir: &Path, prev_steps: &mut Vec<Step>)  {
        if let Some(_last) = self.stack().last() {

            let tcx = self.tcx; 
            let tcx = tcx.tcx;

            let body = self.body();

            let param_env: ParamEnv<'_> = self.param_env;
            // let inst = Instance::expect_resolve(self.tcx, self.param_env, def_id, generic_args);

            let path_prefix: PathBuf = PathBuf::default();

            fs::create_dir_all(outdir).expect("unable to create output directory");
            let path_meta = outdir.join("meta");
            fs::create_dir_all(&path_meta).expect("unable to create meta directory");
            let path_data = outdir.join("data");
            fs::create_dir_all(&path_data).expect("unable to create meta directory");
            let verbose = false;
            let mut stack = vec![];
            let mut cache = FxHashMap::default();
            let summary1 = PaflCrate { functions: Vec::new() };
            let mut summary = summary1.functions;
            //summary: &'sum mut Vec<PaflFunction>,

            let dumper: PaflDump<'_, '_> = PaflDump {
                tcx: tcx,
                param_env: param_env,
                verbose: verbose,
                path_meta: path_meta.to_path_buf(),
                path_data: path_data.to_path_buf(),
                path_prefix: path_prefix,
                stack: &mut stack,
                cache: &mut cache,
                summary: &mut summary,
            };

            // ty.kind 
            // FieldDef, Param, StaticItem, ConstItem
            let instance_def = body.source.instance;
            let def_id: DefId = instance_def.def_id();
            let my_instance = Instance::mono(tcx, def_id);

            // let inst_args = my_instance.args;
            // let args_ref: GenericArgsRef<'tcx> = inst_args;
            let krate =
            if def_id.is_local() { None } else { Some(self.tcx.crate_name(def_id.krate).to_string()) };
            // let dumper :PaflDump = Default::default();
            // dumper.initialize(my_instance);
            let mut my_generics: Vec<PaflGeneric> = vec![];
            for arg in my_instance.args {
                let sub = match arg.unpack() {
                    GenericArgKind::Lifetime(_region) => PaflGeneric::Lifetime,
                    GenericArgKind::Type(_item) => PaflGeneric::Type(PaflType::Never),
                    // GenericArgKind::Type(item) => PaflGeneric::Type(dumper.process_type(item)),

                    GenericArgKind::Const(item) => PaflGeneric::Const(dumper.process_const(item)),
                    // _ => {},
                };
                my_generics.push(sub);
            }
            
            let fn_inst_key = FnInstKey {
                krate,
                index: def_id.index.as_usize(),
                path: self.tcx.def_path(def_id).to_string_no_crate_verbose(),
                generics: my_generics,
            };

            print!("[Ret][{:?}];", fn_inst_key.generics.len()); 

            print!("[{:?}]", prev_steps);

        } else {
            bug!("bug in dump return!");
        }
    }

}
