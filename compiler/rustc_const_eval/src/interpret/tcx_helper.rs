use super::{InterpCx, Machine};
use rustc_middle::mir::Terminator;
use rustc_middle::mir::BasicBlock;
use rustc_middle::ty::layout::HasTyCtxt;
use rustc_middle::ty::print::with_no_trimmed_paths;
use rustc_middle::ty::{self, GenericArgKind, InstanceDef, ParamEnv, Instance};
use rustc_middle::ty::context::{PaflType, PaflGeneric, FnInstKey, Step, Trace, PaflDump, PaflCrate};

use std::fs;
use std::path::PathBuf;
use either::Either;
use colored::Colorize;

use rustc_hir::def::DefKind;
use rustc_hir::def_id::DefId;
use rustc_hir::definitions::{DefPath, DisambiguatedDefPathData};

use rustc_data_structures::fx::FxHashMap;

use std::fs::OpenOptions;
use std::io::Write;

impl<'tcx, M: Machine<'tcx>> InterpCx<'tcx, M> {
    pub fn crate_info(&mut self,) -> String {
        let mut v: Vec<String> = vec![];
        let res: String;
        with_no_trimmed_paths!({
            let body = self.body();
            let instance_def = body.source.instance;
            let def_id = instance_def.def_id();

            // 0. terminator kind
            // let term_kind = &terminator.kind;
            // let s = format!("{:?}", term_kind);
            // let name = with_no_trimmed_paths!(s);
            // v.push(name);

            // 1. krate name
            let krate_name = self.tcx.crate_name(def_id.krate).to_string();
            let tmp = with_no_trimmed_paths!(krate_name.to_string());
            v.push(tmp);

            // 3. def path
            let def_path = self.tcx.def_path(def_id);
            let def_paths = def_path.data;
            for item in &def_paths {
                // let tmp = format!("[{:?}][{:?}]", item.data, item.disambiguator);
                // let tmp2 = with_no_trimmed_paths!(tmp.to_string());
                let name = with_no_trimmed_paths!(item.data.to_string());
                v.push(name);
                let num = with_no_trimmed_paths!(item.disambiguator.to_string());
                v.push(num);
            }

            res = v.join(":");
        });

        res
    }

    pub fn get_fn_inst_key(&self, instance: Instance<'tcx>) -> FnInstKey {
        let tcx = self.tcx();

        let path_meta = PathBuf::new();
        let path_data = PathBuf::new();
        let path_prefix = PathBuf::new();
        let verbose = false;
        // verbosity
        // let mut cache = FxHashMap::default();
        let mut cache: FxHashMap<Instance<'tcx>, FnInstKey> = FxHashMap::default();
        let mut stack = vec![];
        let param_env = ParamEnv::reveal_all();
        // let mut cache = FxHashMap::default();
        let mut summary = PaflCrate { functions: Vec::new() };
    
        if let Some(cached) = cache.get(&instance) {
            return cached.clone();
        }

        // let id = instance.def_id();
        // let path = self.tcx.def_path(id);
        // let depth = stack.len();


        // construct the worker
        let dumper = PaflDump {
            tcx,
            param_env,
            verbose,
            path_meta: path_meta.to_path_buf(),
            path_data: path_data.to_path_buf(),
            path_prefix,
            stack: &mut stack,
            cache: &mut cache,
            summary: &mut summary.functions,
        };

        // method 1: not accurate
        // let id = instance.def_id();

        // method 2
        let inst_def: ty::InstanceDef<'_> = instance.def;
        let id : DefId = match inst_def {
            InstanceDef::Item(def)
            | InstanceDef::Intrinsic(def)
            | InstanceDef::VTableShim(def)
            | InstanceDef::ReifyShim(def, _)
            | InstanceDef::FnPtrShim(def, _)
            | InstanceDef::Virtual(def, _)
            | InstanceDef::ThreadLocalShim(def) 
            | InstanceDef::DropGlue(def, _)
            | InstanceDef::CloneShim(def, _)
            | InstanceDef::FnPtrAddrShim(def, _) => { def },
            InstanceDef::ClosureOnceShim { call_once, .. } => { call_once }, 
            InstanceDef::ConstructCoroutineInClosureShim { coroutine_closure_def_id, .. } => { coroutine_closure_def_id },
            InstanceDef::CoroutineKindShim { coroutine_def_id, .. } => { coroutine_def_id },
            InstanceDef::AsyncDropGlueCtorShim(def, _) => { def },
        };

        let inst = dumper.resolve_fn_key(id, instance.args);
        inst
    }

    pub fn create_fn_inst_key3(&mut self, func_inst: ty::Instance<'tcx>) -> FnInstKey {
        let func_instance: ty::InstanceDef<'_> = func_inst.def;

        let print = !func_inst.args.is_empty();
        if print {println!("4.2.1) create_fn_key args=[{:?}]", func_inst.args);}

        let def: DefId = match func_instance {
            // InstanceDef::Item(_) => {
            //     if self.verbose {
            //         println!(" ~> direct");
            //     }
            //     let inst = PaflDump::summarize_instance(
            //         self.tcx,
            //         self.param_env,
            //         resolved,
            //         self.verbose,
            //         &self.path_meta,
            //         &self.path_data,
            //         self.stack,
            //         self.cache,
            //         self.summary,
            //     );
            //     // CallSite { inst, kind: CallKind::Direct }
            // },
            InstanceDef::Item(def) | 
            InstanceDef::Intrinsic(def) |
            InstanceDef::VTableShim(def)
            | InstanceDef::ReifyShim(def, _)
            | InstanceDef::FnPtrShim(def, _)
            | InstanceDef::Virtual(def, _)
            | InstanceDef::ThreadLocalShim(def) 
            | InstanceDef::DropGlue(def, _)
            | InstanceDef::CloneShim(def, _)
            | InstanceDef::FnPtrAddrShim(def, _) => { def },
            InstanceDef::ClosureOnceShim { call_once, .. } => { call_once }, 
            InstanceDef::ConstructCoroutineInClosureShim { coroutine_closure_def_id, .. } => { coroutine_closure_def_id },
            InstanceDef::CoroutineKindShim { coroutine_def_id, .. } => { coroutine_def_id },
            InstanceDef::AsyncDropGlueCtorShim(def, _) => { def },
        };

        let tcx = self.tcx.tcx;
        // 1. krate
        // let krate = if def.is_local() { None } else { Some(tcx.crate_name(def.krate).to_string()) };
        let krate = Some(tcx.crate_name(def.krate).to_string());

        // 2.1. dumper ===============================================
        let param_env: ParamEnv<'_> = self.param_env;
        let verbose = false;

        let outdir= PathBuf::from("./tmp_to_get_inst_key/");
        fs::create_dir_all(outdir.clone()).expect("unable to create output directory");
        let path_meta = outdir.join("meta");
        fs::create_dir_all(&path_meta).expect("unable to create meta directory");
        let path_data = outdir.join("data");
        fs::create_dir_all(&path_data).expect("unable to create meta directory");

        let path_prefix: PathBuf = PathBuf::default();
        let mut stack = vec![];
        let mut cache = FxHashMap::default();
        
        let pafl_crate = PaflCrate { functions: Vec::new() };
        let mut summary = pafl_crate.functions;

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

        // =======
        // ================ ===============================================

        let generic_args = func_inst.args;
        // // 2.2. args
        // let const_ty = match func.constant() {
        //     None => {
        //         bug!("callee is not a constant:");
        //     },
        //     Some(const_op) => const_op.const_.ty(),
        // };
        // let (_def_id, generic_args) = match const_ty.kind() {
        //     ty::Closure(def_id, generic_args)
        //     | ty::FnDef(def_id, generic_args) => {
        //         (*def_id, *generic_args)
        //     },
        //     _ => bug!("callee is not a function or closure"),
        // };

        // 2.3. generics
        let mut my_generics: Vec<PaflGeneric> = vec![];
        for arg in generic_args {
            let sub = match arg.unpack() {
                GenericArgKind::Lifetime(_region) => PaflGeneric::Lifetime,
                GenericArgKind::Type(_item) => PaflGeneric::Type(PaflType::Never),
                // GenericArgKind::Type(item) => PaflGeneric::Type(dumper.process_type(item)),
                GenericArgKind::Const(item) => PaflGeneric::Const(dumper.process_const(item)),
                // _ => {},
            };
            my_generics.push(sub);
        }

        // 3. FnInstKey ===============================================
        let fn_inst_key = FnInstKey {
            krate,
            index: def.index.as_usize(),
            path: tcx.def_path(def).to_string_no_crate_verbose(),
            generics: my_generics,
        };
        // print!("[createFnKey({:?})];", fn_inst_key.generics.len()); 
        fn_inst_key
    }

    // TODO: remove
    pub fn _log_in_eval_query(
        &mut self, 
    ) {
        let tcx = self.tcx.tcx;
        let body = self.body();
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

    // TODO: remove
    #[inline(always)]
    pub fn _bb_dump_in_step(&mut self) { 

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

    // TODO: remove
    fn _print_crate_info(&mut self, /*def: DefId, */ _term: &Terminator<'tcx>) {
        if let Some(last) = self.stack().last() {

            let tcx = self.tcx;
            // 1. def_id
            let body = self.body();
            let instance_def = body.source.instance;
            let def_id: DefId = instance_def.def_id();

            // 2. crate name
            let crate_name2 = tcx.crate_name(def_id.krate);
            let s1 = format!(":[{:?}]", crate_name2);
            print!("{}", s1.red());
        
            // 3. def_kind
            let def_kind: DefKind = tcx.def_kind(def_id);
            let s2 = format!("[{:?}]", def_kind);
            print!("{}", s2.blue());
        
            let def_path: DefPath = tcx.def_path(def_id);
            let def_paths: Vec<DisambiguatedDefPathData> = def_path.data;
            for item in &def_paths {
                let s3 = format!("[{:?}][{:?}]", item.data, item.disambiguator);
                print!("{}", s3.green());
            }
            // println!("");

            // 4. terminator kind

            // 5. BASIC BLOCK and statement number
            let loc = last.loc;
            if let Either::Left(l_loc) = loc {
                let block = l_loc.block;
                // let statement_idx = l_loc.statement_index;
                print!(":[{:?}]", block);
                // info!("// executing {:?}", loc.block);
            }
        }

    }

    pub fn dump_trace(&mut self, file_path: &str) {
        let trace = self._trace_stack.last().unwrap();
        let size = self._trace_stack.len();
        println!("[dump] size of trace stack {}, file_path {}", size, file_path,);
        // assert_eq!(size, 1);
        dbg!(&self._trace_stack);
        // if trace._steps.len() > 0 {
        //     println!("after miri2 {:?}", trace._steps.last().unwrap());
        // } else {
        //     println!("empty trace");
        // };

        let content =
            serde_json::to_string_pretty(&*trace).expect("unexpected failure on JSON encoding");

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .expect("unable to create output file");
        file.write_all(content.as_bytes()).expect("unexpected failure on outputting to file");
    }

    // new) called by call
    pub fn push_trace_stack1(&mut self, fn_key: FnInstKey) {
        // println!("call {:?}", fn_key);
        self._trace_stack.push(Trace {_entry: fn_key, _steps: Vec::new()});
        // let info = self.inst_to_info(fn_key);
    }

    // new) called by return
    pub fn merge_trace_stack1(&mut self/* , info: String*/) {
        let trace = self._trace_stack.pop().unwrap();
        // println!("return {:?}", trace._entry);
        let l = self._trace_stack.len();
        if l == 0 {
            println!("WARNING: call stack exceeded!");
            self._trace_stack.push(trace);
        } else {
            self._trace_stack.last_mut().unwrap()._steps.push(Step::Call(trace));
        };
    }

    // new) called by BB(X)
    pub fn push_bb_stack1(&mut self, bb: BasicBlock) {
        self._trace_stack.last_mut().unwrap()._steps.push(Step::B(bb.as_usize()));
    }

}
