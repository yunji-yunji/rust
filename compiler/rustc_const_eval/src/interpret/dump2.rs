use std::ffi::OsString;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Write};
use either::Either;

use super::InterpCx;
use super::Machine;

use rustc_hir::def_id::{DefId};

// use rustc_data_structures::fx::FxHashMap;
// use rustc_codegen_ssa::pafl::{PaflDump, PaflCrate};

use rustc_middle::mir::{Terminator, TerminatorKind};
// use rustc_middle::ty::{self, TyCtxt, GenericArgKind};
// use rustc_middle::ty::context::{
//     Trace, Step, PaflType, PaflGeneric, FnInstKey,
// };


// /* 
// pub fn create_empty_trace<'tcx>(tcx: TyCtxt<'tcx>, def: DefId, term: &Terminator<'tcx>) -> Trace {

//     // 1. krate
//     let krate = if def.is_local() { None } else { Some(tcx.crate_name(def.krate).to_string()) };


//     // 2.1. dumper
//     let param_env: ParamEnv<'_> = self.param_env;
//     let verbose = false;

//     let outdir="./yjtmp/";
//     fs::create_dir_all(outdir).expect("unable to create output directory");
//     let path_meta = outdir.join("meta");
//     fs::create_dir_all(&path_meta).expect("unable to create meta directory");
//     let path_data = outdir.join("data");
//     fs::create_dir_all(&path_data).expect("unable to create meta directory");

//     let path_prefix: PathBuf = PathBuf::default();
//     let mut stack = vec![];
//     let mut cache = FxHashMap::default();
    
//     let pafl_crate = PaflCrate { functions: Vec::new() };
//     let mut summary = pafl_crate.functions;

//     let dumper: PaflDump<'_, '_> = PaflDump {
//         tcx: tcx,
//         param_env: param_env,
//         verbose: verbose,
//         path_meta: path_meta.to_path_buf(),
//         path_data: path_data.to_path_buf(),
//         path_prefix: path_prefix,
//         stack: &mut stack,
//         cache: &mut cache,
//         summary: &mut summary,
//     };

//     let kind = &term.kind;
//     match kind {
//         TerminatorKind::Call { func, args: _, destination: _, target: _, unwind: _, call_source: _, fn_span: _ } => 
//         {


//             // 2.2. args
//             let const_ty = match func.constant() {
//                 None => {
//                     bug!("callee is not a constant:");
//                 },
//                 Some(const_op) => const_op.const_.ty(),
//             };
//             let (_def_id, generic_args) = match const_ty.kind() {
//                 ty::Closure(def_id, generic_args)
//                 | ty::FnDef(def_id, generic_args) => {
//                     (*def_id, *generic_args)
//                 },
//                 _ => bug!("callee is not a function or closure"),
//             };

//             // 2.3. generics
//             let mut my_generics: Vec<PaflGeneric> = vec![];
//             for arg in generic_args {
//                 let sub = match arg.unpack() {
//                     GenericArgKind::Lifetime(_region) => PaflGeneric::Lifetime,
//                     GenericArgKind::Type(_item) => PaflGeneric::Type(PaflType::Never),
//                     // GenericArgKind::Type(item) => PaflGeneric::Type(dumper.process_type(item)),
//                     GenericArgKind::Const(item) => PaflGeneric::Const(dumper.process_const(item)),
//                     // _ => {},
//                 };
//                 my_generics.push(sub);
//             }

//         },
//         _ => {}
//     }

//     // 3. FnInstKey
//     let fn_inst_key = FnInstKey {
//         krate,
//         index: def.index.as_usize(),
//         path: tcx.def_path(def).to_string_no_crate_verbose(),
//         generics: my_generics,
//     };
//     // let dummy_fn_inst_key = FnInstKey {
//     //     krate: None,
//     //     index: 0,
//     //     path: String::from(""),
//     //     generics: vec![],
//     // };

//     // 4. Trace
//     let steps : Vec<Step> = vec![];
//     Trace { _entry: fn_inst_key, _steps: steps }

// }
// */

impl<'mir, 'tcx: 'mir, M: Machine<'mir, 'tcx>> InterpCx<'mir, 'tcx, M> {
    pub fn dump2(&mut self, term: &Terminator<'tcx>, dump_str: OsString) {
        let outdir = std::path::PathBuf::from(dump_str);
        fs::create_dir_all(outdir).expect("Fail to open directory.");
        let tcx = self.tcx;
        let body = self.body();
        let instance_def = body.source.instance;
        let def_id: DefId = instance_def.def_id();

        // let krate_name =
        // if def_id.is_local() { None } else { Some(self.tcx.crate_name(def_id.krate).to_string()) };
        let krate_name = tcx.crate_name(def_id.krate).to_string();
        // let file_name = krate_name;
        let file_name = format!("yj{:?}.json", krate_name);
        println!("yj: file name {:?}", file_name);
        // let _output = outdir.join(file_name).with_extension("json");

        let kind = &term.kind;
        match kind {
            // function call
            // 1) create fnInstkey
            // 2) create Trace
            // 3) add Call(Trace) to steps
            TerminatorKind::Call { func: _, args: _, destination: _, target: _, unwind: _, call_source: _, fn_span: _ } => 
            {
                let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(file_name);

                if let Some(last) = self.stack().last() {
                    let _loc = last.loc;
                    println!("CALL");
                    // if let Either::Left(l_loc) = loc {
                    //     let block = l_loc.block;
                        let block_with_space = format!("Call({:?}) ", def_id);
                        let _ = file.expect("yj: open file, term Call").write_all(block_with_space.as_bytes());

                        // print!(":[{:?}]", block);
                        // let step = Step::Block(block);
                        // exec_t._steps.push(step);
                } else {
                    bug!("yj: bb_trace: last doesn't exist");
                }
            },
            // add basic block to current steps
            _ => {
                let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(file_name);

                if let Some(last) = self.stack().last() {
                    let loc = last.loc;
        
                    if let Either::Left(l_loc) = loc {
                        let block = l_loc.block;
                        let block_with_space = format!("{:?} ", block);
                        let _ = file.expect("yj: open file").write_all(block_with_space.as_bytes());

                        // print!(":[{:?}]", block);
                        // let step = Step::Block(block);
                        // exec_t._steps.push(step);
                    } else {
                        bug!("yj: bb_trace: loc doesn't exist");
                    }
                } else {
                    bug!("yj: bb_trace: last doesn't exist");
                }
            },
        }
    }


}
//     pub fn dump1(&mut self, term: &Terminator<'tcx>, exec_t: &mut Trace, dump_str: OsString) {
//         let outdir = std::path::PathBuf::from(dump_str);
//         fs::create_dir_all(outdir).expect("Fail to open directory.");
        
//         let file_name = stable_create_id.as_u64().to_string();
//         let output = outdir.join(file_name).with_extension("json");


//         let kind = &term.kind;
//         match kind {
//             // function call
//             // 1) create fnInstkey
//             // 2) create Trace
//             // 3) add Call(Trace) to steps
//             TerminatorKind::Call { func, args: _, destination: _, target: _, unwind: _, call_source: _, fn_span: _ } => 
//             {

//             },
//             // add basic block to current steps
//             _ => {
//                 if let Some(last) = self.stack().last() {
//                     let loc = last.loc;
        
//                     if let Either::Left(l_loc) = loc {
//                         let block = l_loc.block;
//                         // print!(":[{:?}]", block);
//                         let step = Step::Block(block);
//                         exec_t._steps.push(step);
//                     } else {
//                         bug!("yj: bb_trace: loc doesn't exist");
//                     }
//                 } else {
//                     bug!("yj: bb_trace: last doesn't exist");
//                 }
//             },
//         }
//     }
// }