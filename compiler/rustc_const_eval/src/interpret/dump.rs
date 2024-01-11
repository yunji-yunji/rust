use crate::const_eval::CompileTimeEvalContext;
use super::InterpCx;
// use super::InterpResult;
use super::Machine;
// use super::terminator;
use either::Either;
use std::path::Path;
use std::path::PathBuf;

use rustc_data_structures::fx::FxHashMap;

use rustc_middle::mir::Body;
use rustc_middle::mir::TerminatorKind;
use rustc_middle::mir::Terminator;
use rustc_middle::ty;
use rustc_middle::ty::GenericArgsRef;
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::ParamEnv;
use rustc_middle::ty::GenericArgKind;
use rustc_middle::ty::Instance;
// use rustc_middle::query::TyCtxtAt;

// use std::intrinsics::mir::BasicBlock;
// use rustc_middle::mir::BasicBlock;
use rustc_middle::ty::context::{
    Trace, Step,
    /*Ident2,ValueTree, TyInstKey, PaflConst,*/ PaflType, PaflGeneric, FnInstKey,
};
use std::fs;
use std::fs::OpenOptions;
// use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Write;


use rustc_hir::def::DefKind;
use rustc_hir::def_id::{LOCAL_CRATE, DefId};
use rustc_hir::definitions::{DefPath, DisambiguatedDefPathData};

// use rustc_codegen_ssa::pafl::{FnInstKey, PaflGeneric, PaflType, , /*PaflConst, PaflType,*/};
use rustc_codegen_ssa::pafl::{PaflDump, PaflCrate};

use colored::Colorize;

pub fn dump_in_eval_query( // eval_queries.rs => DUMP_ON
    tcx: TyCtxt<'_>,
    body: &Body<'_>,
    outdir: &Path,
) {
    match std::env::var_os("FILE") {
        None => (),
        Some(_val) => {
            // === File setup === //
            fs::create_dir_all(outdir).expect("Fail to open directory.");
            let symbol = tcx.crate_name(LOCAL_CRATE);
            let file_name = symbol.as_str();
            // let stable_create_id: StableCrateId = tcx.stable_crate_id(LOCAL_CRATE);
            // let file_name = stable_create_id.as_u64().to_string();
            println!("FILE: outdir{:?} file_name {:?}", outdir, file_name);
            let output = outdir.join(file_name).with_extension("json");
            let mut _file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)
                .expect("Fail to create a file.");
        }
    }
    let mut content = String::new();

    let instance_def = body.source.instance;
    let def_id: DefId = instance_def.def_id();
    
    let crate_name2 = tcx.crate_name(def_id.krate);
    content.push_str(&format!("[{:?}]", crate_name2));
    let s1 = format!("[{:?}]", crate_name2);
    print!("{}", s1.red());

    let def_kind: DefKind = tcx.def_kind(def_id);
    content.push_str(&format!("[{:?}]", def_kind));
    let s2 = format!("[{:?}]", def_kind);
    print!("{}", s2.blue());

    let def_path: DefPath = tcx.def_path(def_id);
    let def_paths: Vec<DisambiguatedDefPathData> = def_path.data;
    for item in &def_paths {
        content.push_str(&format!("[{:?}][{:?}]", item.data, item.disambiguator));
        let s3 = format!("[{:?}][{:?}]", item.data, item.disambiguator);
        print!("{}", s3.green());
    }
    println!("");
    // println!("{:?}", content);
    let _tmp = content;
    
    // file.write_all(content.as_bytes()).expect("Fail to write file.");
}

pub fn bb_dump<'mir, 'tcx>(
    ecx: &CompileTimeEvalContext<'mir, 'tcx>
) {
    
    // let bbs = body.basic_blocks.as_mut();
    let loc= ecx.frame().loc;
    // let bb_id = ecx.frame().loc.left();
    if let Either::Left(l_loc) = loc {
        let block = l_loc.block;
        let statement_idx = l_loc.statement_index;
        println!("[s][{:?}][{:?}]\n", block, statement_idx);
        // let bb_id = 
        // info!("// executing {:?}", loc.block);
    }
    // self.frame_mut().loc.as_mut().left().unwrap().statement_index += 1;

    // let bbs = &body.basic_blocks;
    // println!("{:?}", bbs);
}


// pub fn dump_in_step( // step.rs => STEP
//     // ecx: &CompileTimeEvalContext<'mir, 'tcx>
//     tcx: TyCtxt<'_>,
//     body: &Body<'_>,
//     // outdir: &Path,
// ) {
//     print!("{}", "[step]".green());
//     let outdir = std::path::PathBuf::from("/home/y23kim/aptos-core/third_party/move/move-bytecode-verifier/src/regression_tests/dump_yj");
//     dump_in_eval_query(tcx, body, &outdir);
// }
impl<'mir, 'tcx: 'mir, M: Machine<'mir, 'tcx>> InterpCx<'mir, 'tcx, M> {
    #[inline(always)]
    pub fn bb_dump_in_step(&mut self) { // step.rs => STEPP

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
        // let loc= self.frame().loc;
        // // let bb_id = ecx.frame().loc.left();
        // if let Either::Left(l_loc) = loc {
        //     let block = l_loc.block;
        //     let statement_idx = l_loc.statement_index;
        //     println!("[{:?}][{:?}]\n", block, statement_idx);
        //     // let bb_id = 
        //     // info!("// executing {:?}", loc.block);
        // }
    }

    pub fn bb_trace(&mut self) -> Step {
        if let Some(last) = self.stack().last() {
            /*
            let body = self.body();
            let tcx = self.tcx; // self.tcx.tcx 
            let instance_def = body.source.instance;
            let def_id: DefId = instance_def.def_id();

            let crate_name2 = tcx.crate_name(def_id.krate);
            let s1 = format!(":[{:?}]", crate_name2);
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
             */

            // let dummy_fn_inst_key = FnInstKey {
            //     krate: None,
            //     index: 0,
            //     path: String::from(""),
            //     generics: vec![],
            // };
            // let trace : Trace = Trace { _entry: dummy_fn_inst_key, _steps: vec![] };
            // self.tcx._trace = trace;

            // let tt = last.tcx._trace;
            // let tcx= &mut self.tcx;
            // let trace1 = &mut tcx._trace;
            // trace1._entry.index += 1;


            let loc = last.loc;

            if let Either::Left(l_loc) = loc {
                let block = l_loc.block;
                let _statement_idx = l_loc.statement_index;
                print!(":[{:?}]", block);

                let step = Step::Block(block);
                step
            } else {
                bug!("yj: bb_trace: loc doesn't exist");
                    // Step::Err
            }
        } else {
            bug!("yj: bb_trace: last doesn't exist");
            // Step::Err
        }
    }
    
    pub fn dump_return(&mut self, outdir: &Path, prev_steps: &mut Vec<Step>) -> Step {
        if let Some(_last) = self.stack().last() {
            // let loc = last.loc;
            // if let Either::Left(l_loc) = loc {
            //     let _block = l_loc.block;
            //     // let statement_idx = l_loc.statement_index;
            //     // print!(":[{:?}][{:?}]", block, trace._steps);
            //     // let bb_id = 
            //     // info!("// executing {:?}", loc.block);
            // }

            let tcx = self.tcx; 
            let tcx = tcx.tcx;

            let body = self.body();
            // let instance_def = body.source.instance;
            // let def_id: DefId = instance_def.def_id();
            
            // self.inst_dump(my_instance.args, outdir, prev_steps)

            // =======================



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
                        // let prev_steps = vec![];
            let trace = Trace {
                _entry: fn_inst_key,
                _steps: prev_steps.to_vec(),
            };
            // *steps = vec![];
            // *prev_steps = vec![];

            let step = Step::Call(trace.clone());
            step


            // =======================





        } else {
            Step::Err
        }
    }

    pub fn inst_dump(&mut self, args: GenericArgsRef<'tcx>, outdir: &Path, prev_steps: &mut Vec<Step> ) 
    -> Step { // step.rs => 

        if let Some(last) = self.stack().last() {

            // crate information
            let body = self.body();
            // what is DIFFerence BETWEEN TyCtxt and TCXtxtAt
            let tcx = self.tcx; // self.tcx.tcx 
            let tcx2 = tcx.tcx;
            // get generic arg ref
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
                tcx: tcx2,
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

            // let inst = self.resolve_fn_key(def_id, generic_args);

            // let inst_args = my_instance.args;
            // let args_ref: GenericArgsRef<'tcx> = inst_args;
            let krate =
            if def_id.is_local() { None } else { Some(self.tcx.crate_name(def_id.krate).to_string()) };
            // let dumper :PaflDump = Default::default();
            // dumper.initialize(my_instance);
            let mut my_generics: Vec<PaflGeneric> = vec![];
            for arg in args {
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

            print!("[Inst][{:?}];", fn_inst_key.generics.len()); 

            print!("[{:?}]", prev_steps);
                        // let prev_steps = vec![];
            let trace = Trace {
                _entry: fn_inst_key,
                _steps: prev_steps.to_vec(),
            };
            // *steps = vec![];
            // *prev_steps = vec![];

            let step = Step::Call(trace.clone());

            // print!("[{:?}]", step);

                // fn_inst_key.krate, 
                // fn_inst_key.index, 
                // fn_inst_key.path);
            let crate_name2 = tcx.crate_name(def_id.krate);
            let s1 = format!(":[{:?}]", crate_name2);
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
            // println!("");

            // BASIC BLOCK and statement number
            let loc = last.loc;
            if let Either::Left(l_loc) = loc {
                let block = l_loc.block;
                // let statement_idx = l_loc.statement_index;
                print!(":[{:?}][{:?}]", block, trace._steps);
                // let bb_id = 
                // info!("// executing {:?}", loc.block);
            }
            step
        } else {
            Step::Err
        }
        // let loc= self.frame().loc;
        // // let bb_id = ecx.frame().loc.left();
        // if let Either::Left(l_loc) = loc {
        //     let block = l_loc.block;
        //     let statement_idx = l_loc.statement_index;
        //     println!("[{:?}][{:?}]\n", block, statement_idx);
        //     // let bb_id = 
        //     // info!("// executing {:?}", loc.block);
        // }
    }


    pub fn dump_in_term(&mut self, term: &Terminator<'tcx>, steps: &mut Vec<Step> ) { // step.rs => TERM
        // let term = terminator;

        match std::env::var_os("TERM") {
            None => (),
            Some(val) => {
                let outdir = std::path::PathBuf::from(val);
                let kind = &term.kind;
                let s1 = format!("[{:?}]", kind.name());
                print!("{}", s1.red());
                
                // let step : Step = 
                match kind {
                    // TerminatorKind::Goto { target } => todo!(),
                    // TerminatorKind::SwitchInt { discr, targets } => todo!(),
                    // TerminatorKind::UnwindResume => todo!(),
                    // TerminatorKind::UnwindTerminate(_) => todo!(),
                    // TerminatorKind::Return => {
                    //     let _step = self.dump_return(&outdir, steps);
                    //     // if let Step::Call(t) = step {
                    //     //     println!("/[{:?}][{:?}]", t._steps.len(), t._steps);
                    //     // }
                    // },
                    // TerminatorKind::Unreachable => todo!(),
                    // TerminatorKind::Drop { place, target, unwind, replace } => todo!(),
                    TerminatorKind::Call { func, args: _, destination: _, target: _, unwind: _, call_source: _, fn_span: _ } => 
                    {
                        let body = self.body();
                        let instance_def = body.source.instance;
                        let def_id: DefId = instance_def.def_id();
                        let crate_name2 = self.tcx.crate_name(def_id.krate);
                        let filename = format!("yj_{}.json", crate_name2);
                        let const_ty = match func.constant() {
                            None => {
                                println!("callee is not a constant:");
                                return;
                            },
                            Some(const_op) => const_op.const_.ty(),
                        };

                        let (_def_id, generic_args) = match const_ty.kind() {
                            ty::Closure(def_id, generic_args)
                            | ty::FnDef(def_id, generic_args) => {
                                (*def_id, *generic_args)
                            },
                            _ => bug!("callee is not a function or closure"),
                        };
                        // let fn_inst_key = pafl::resolve_fn_key(def_id, generic_args);
                        print!("//[{:?}]", steps);
                        let step = self.inst_dump(generic_args, &outdir, steps);
                        if let Step::Call(t) = step {
                            println!("/[{:?}][{:?}]", t._steps.len(), t._steps);
                        }
                        // *steps = vec![];

                        // let dummy_generics: Vec<PaflGeneric> = vec![];
                        let dummy_fn_inst_key = FnInstKey {
                            krate: None,
                            index: 0,
                            path: String::from(""),
                            generics: vec![],
                        };
                        let trace : Trace = Trace { _entry: dummy_fn_inst_key, _steps: steps.to_vec() };

                        let serialized_data = serde_json::to_string(&trace);
                        let file = File::create(filename);
                        let _ = file.expect("file created").write_all(serialized_data.expect("serialzed yj").as_bytes());

                        // func == callee: &Operand<'tcx>, span: Span
                        // let my_instance = Instance::
                    },
                    // TerminatorKind::Assert { cond, expected, msg, target, unwind } => 
                    // {
                    //     todo!()
                    // },
                    // TerminatorKind::Yield { value, resume, resume_arg, drop } => todo!(),
                    // TerminatorKind::CoroutineDrop => todo!(),
                    // TerminatorKind::FalseEdge { real_target, imaginary_target } => todo!(),
                    // TerminatorKind::FalseUnwind { real_target, unwind } => todo!(),
                    // TerminatorKind::InlineAsm { template, operands, options, line_spans, destination, unwind } => todo!(),
                    _ => { 
                        let step = self.bb_trace();
                        steps.push(step);
                        println!(":[{:?}]", steps);
                    },
                }
                // ;
                // match step {
                //     Step::Block(id) => print!("[{:?}]", id),
                //     Step::Call(ref t) => print!("[{:?}]", t._steps.len()),
                //     Step::Err => print!("[ERR]"),
                // }
            }
        }

        // let kind = match &term.kind {
        //     // basics
        //     TerminatorKind::Goto { target } => TermKind::Goto((*target).into()),
        //     TerminatorKind::SwitchInt { discr: _, targets } => {
        //         TermKind::Switch(targets.all_targets().iter().map(|b| (*b).into()).collect())
        //     }
        //     TerminatorKind::Unreachable => TermKind::Unreachable,
        //     TerminatorKind::Return => TermKind::Return,
        //     // call (which may unwind)
        //     TerminatorKind::Call {
        //         func,
        //         args: _,
        //         destination: _,
        //         target,
        //         unwind,
        //         call_source: _,
        //         fn_span: _,
        //     } => TermKind::Call {
        //         site: self.process_callsite(func, term.source_info.span),
        //         target: target.as_ref().map(|t| (*t).into()),
        //         unwind: unwind.into(),
        //     },
        //     TerminatorKind::Drop { place: _, target, unwind, replace: _ } => {
        //         TermKind::Drop { target: (*target).into(), unwind: unwind.into() }
        //     }
        //     TerminatorKind::Assert { cond: _, expected: _, msg: _, target, unwind } => {
        //         TermKind::Assert { target: (*target).into(), unwind: unwind.into() }
        //     }
        //     // unwinding
        //     TerminatorKind::UnwindResume => TermKind::UnwindResume,
        //     TerminatorKind::UnwindTerminate(..) => TermKind::UnwindFinish,
        //     // imaginary
        //     TerminatorKind::FalseEdge { real_target, imaginary_target: _ }
        //     | TerminatorKind::FalseUnwind { real_target, unwind: _ } => {
        //         TermKind::Goto((*real_target).into())
        //     }
        //     // coroutine
        //     TerminatorKind::Yield { .. } | TerminatorKind::CoroutineDrop => {
        //         bug!("unexpected coroutine")
        //     }
        //     // assembly
        //     TerminatorKind::InlineAsm { .. } => bug!("unexpected inline assembly"),
        // };
    }

}

// #[feature(custom_mir)]


