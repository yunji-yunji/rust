//! This module contains the `InterpCx` methods for executing a single step of the interpreter.
//!
//! The main entry point is the `step` method.

use either::Either;
use rustc_index::IndexVec;

use rustc_middle::mir;
use rustc_middle::mir::interpret::{InterpResult, Scalar};
use rustc_middle::ty::layout::LayoutOf;

use super::{ImmTy, InterpCx, Machine, Projectable};
use crate::util;

// yunji
use rustc_middle::*;
use std::default::Default;
use std::io::Write;
use std::fs;
use std::fs::File;
use std::path::Path;
use rustc_data_structures::fx::FxHashMap;
use std::string::String;
use rustc_middle::mir::{SccInfo, PathInfo, NodeType};
use std::default::Default as HashDefault;

impl<'mir, 'tcx: 'mir, M: Machine<'mir, 'tcx>> InterpCx<'mir, 'tcx, M> {
    /// Returns `true` as long as there are more things to do.
    ///
    /// This is used by [priroda](https://github.com/oli-obk/priroda)
    ///
    /// This is marked `#inline(always)` to work around adversarial codegen when `opt-level = 3`

    #[inline(always)]
    pub fn step(&mut self,
                path : &mut Vec<usize>, s: &mut usize, stk: &mut Vec<PathInfo>,
                is_loop: &mut bool, limit: usize,
                scc_info: &mut IndexVec<usize, Vec<SccInfo>>)
        -> InterpResult<'tcx, bool> {
        if self.stack().is_empty() {
            return Ok(false);
        }

        let Either::Left(loc) = self.frame().loc else {
            // We are unwinding and this fn has no cleanup code.
            // Just go on unwinding.
            trace!("unwinding: skipping frame");
            self.pop_stack_frame(/* unwinding */ true)?;
            return Ok(true);
        };
        let basic_block = &self.body().basic_blocks[loc.block];

        // yunji
        // let def_id = self.body().source.def_id();
        // println!("[step] def string = {:?}", self.tcx.def_path_str(def_id));

        if let Some(stmt) = basic_block.statements.get(loc.statement_index) {
            let old_frames = self.frame_idx();
            self.statement(stmt)?;
            // Make sure we are not updating `statement_index` of the wrong frame.
            assert_eq!(old_frames, self.frame_idx());
            // Advance the program counter.
            self.frame_mut().loc.as_mut().left().unwrap().statement_index += 1;
            return Ok(true);
        }

        M::before_terminator(self)?;

        // path.push(loc.block);

        let terminator = basic_block.terminator();

        self.terminator(terminator, path, s, stk, is_loop,
                        limit, scc_info)?;
        Ok(true)
    }

    /// Runs the interpretation logic for the given `mir::Statement` at the current frame and
    /// statement counter.
    ///
    /// This does NOT move the statement counter forward, the caller has to do that!
    pub fn statement(&mut self, stmt: &mir::Statement<'tcx>) -> InterpResult<'tcx> {
        info!("{:?}", stmt);

        use rustc_middle::mir::StatementKind::*;

        match &stmt.kind {
            Assign(box (place, rvalue)) => self.eval_rvalue_into_place(rvalue, *place)?,

            SetDiscriminant { place, variant_index } => {
                let dest = self.eval_place(**place)?;
                self.write_discriminant(*variant_index, &dest)?;
            }

            Deinit(place) => {
                let dest = self.eval_place(**place)?;
                self.write_uninit(&dest)?;
            }

            // Mark locals as alive
            StorageLive(local) => {
                self.storage_live(*local)?;
            }

            // Mark locals as dead
            StorageDead(local) => {
                self.storage_dead(*local)?;
            }

            // No dynamic semantics attached to `FakeRead`; MIR
            // interpreter is solely intended for borrowck'ed code.
            FakeRead(..) => {}

            // Stacked Borrows.
            Retag(kind, place) => {
                let dest = self.eval_place(**place)?;
                M::retag_place_contents(self, *kind, &dest)?;
            }

            Intrinsic(box intrinsic) => self.emulate_nondiverging_intrinsic(intrinsic)?,

            // Evaluate the place expression, without reading from it.
            PlaceMention(box place) => {
                let _ = self.eval_place(*place)?;
            }

            // This exists purely to guide borrowck lifetime inference, and does not have
            // an operational effect.
            AscribeUserType(..) => {}

            // Currently, Miri discards Coverage statements. Coverage statements are only injected
            // via an optional compile time MIR pass and have no side effects. Since Coverage
            // statements don't exist at the source level, it is safe for Miri to ignore them, even
            // for undefined behavior (UB) checks.
            //
            // A coverage counter inside a const expression (for example, a counter injected in a
            // const function) is discarded when the const is evaluated at compile time. Whether
            // this should change, and/or how to implement a const eval counter, is a subject of the
            // following issue:
            //
            // FIXME(#73156): Handle source code coverage in const eval
            Coverage(..) => {}

            ConstEvalCounter => {
                M::increment_const_eval_counter(self)?;
            }

            // Defined to do nothing. These are added by optimization passes, to avoid changing the
            // size of MIR constantly.
            Nop => {}
        }

        Ok(())
    }

    /// Evaluate an assignment statement.
    ///
    /// There is no separate `eval_rvalue` function. Instead, the code for handling each rvalue
    /// type writes its results directly into the memory specified by the place.
    pub fn eval_rvalue_into_place(
        &mut self,
        rvalue: &mir::Rvalue<'tcx>,
        place: mir::Place<'tcx>,
    ) -> InterpResult<'tcx> {
        let dest = self.eval_place(place)?;
        // FIXME: ensure some kind of non-aliasing between LHS and RHS?
        // Also see https://github.com/rust-lang/rust/issues/68364.

        use rustc_middle::mir::Rvalue::*;
        match *rvalue {
            ThreadLocalRef(did) => {
                let ptr = M::thread_local_static_base_pointer(self, did)?;
                self.write_pointer(ptr, &dest)?;
            }

            Use(ref operand) => {
                // Avoid recomputing the layout
                let op = self.eval_operand(operand, Some(dest.layout))?;
                self.copy_op(&op, &dest, /*allow_transmute*/ false)?;
            }

            CopyForDeref(place) => {
                let op = self.eval_place_to_op(place, Some(dest.layout))?;
                self.copy_op(&op, &dest, /* allow_transmute*/ false)?;
            }

            BinaryOp(bin_op, box (ref left, ref right)) => {
                let layout = util::binop_left_homogeneous(bin_op).then_some(dest.layout);
                let left = self.read_immediate(&self.eval_operand(left, layout)?)?;
                let layout = util::binop_right_homogeneous(bin_op).then_some(left.layout);
                let right = self.read_immediate(&self.eval_operand(right, layout)?)?;
                self.binop_ignore_overflow(bin_op, &left, &right, &dest)?;
            }

            CheckedBinaryOp(bin_op, box (ref left, ref right)) => {
                // Due to the extra boolean in the result, we can never reuse the `dest.layout`.
                let left = self.read_immediate(&self.eval_operand(left, None)?)?;
                let layout = util::binop_right_homogeneous(bin_op).then_some(left.layout);
                let right = self.read_immediate(&self.eval_operand(right, layout)?)?;
                self.binop_with_overflow(bin_op, &left, &right, &dest)?;
            }

            UnaryOp(un_op, ref operand) => {
                // The operand always has the same type as the result.
                let val = self.read_immediate(&self.eval_operand(operand, Some(dest.layout))?)?;
                let val = self.wrapping_unary_op(un_op, &val)?;
                assert_eq!(val.layout, dest.layout, "layout mismatch for result of {un_op:?}");
                self.write_immediate(*val, &dest)?;
            }

            Aggregate(box ref kind, ref operands) => {
                self.write_aggregate(kind, operands, &dest)?;
            }

            Repeat(ref operand, _) => {
                let src = self.eval_operand(operand, None)?;
                assert!(src.layout.is_sized());
                let dest = self.force_allocation(&dest)?;
                let length = dest.len(self)?;

                if length == 0 {
                    // Nothing to copy... but let's still make sure that `dest` as a place is valid.
                    self.get_place_alloc_mut(&dest)?;
                } else {
                    // Write the src to the first element.
                    let first = self.project_index(&dest, 0)?;
                    self.copy_op(&src, &first, /*allow_transmute*/ false)?;

                    // This is performance-sensitive code for big static/const arrays! So we
                    // avoid writing each operand individually and instead just make many copies
                    // of the first element.
                    let elem_size = first.layout.size;
                    let first_ptr = first.ptr();
                    let rest_ptr = first_ptr.offset(elem_size, self)?;
                    // No alignment requirement since `copy_op` above already checked it.
                    self.mem_copy_repeatedly(
                        first_ptr,
                        rest_ptr,
                        elem_size,
                        length - 1,
                        /*nonoverlapping:*/ true,
                    )?;
                }
            }

            Len(place) => {
                let src = self.eval_place(place)?;
                let len = src.len(self)?;
                self.write_scalar(Scalar::from_target_usize(len, self), &dest)?;
            }

            Ref(_, borrow_kind, place) => {
                let src = self.eval_place(place)?;
                let place = self.force_allocation(&src)?;
                let val = ImmTy::from_immediate(place.to_ref(self), dest.layout);
                // A fresh reference was created, make sure it gets retagged.
                let val = M::retag_ptr_value(
                    self,
                    if borrow_kind.allows_two_phase_borrow() {
                        mir::RetagKind::TwoPhase
                    } else {
                        mir::RetagKind::Default
                    },
                    &val,
                )?;
                self.write_immediate(*val, &dest)?;
            }

            AddressOf(_, place) => {
                // Figure out whether this is an addr_of of an already raw place.
                let place_base_raw = if place.is_indirect_first_projection() {
                    let ty = self.frame().body.local_decls[place.local].ty;
                    ty.is_unsafe_ptr()
                } else {
                    // Not a deref, and thus not raw.
                    false
                };

                let src = self.eval_place(place)?;
                let place = self.force_allocation(&src)?;
                let mut val = ImmTy::from_immediate(place.to_ref(self), dest.layout);
                if !place_base_raw {
                    // If this was not already raw, it needs retagging.
                    val = M::retag_ptr_value(self, mir::RetagKind::Raw, &val)?;
                }
                self.write_immediate(*val, &dest)?;
            }

            NullaryOp(ref null_op, ty) => {
                let ty = self.subst_from_current_frame_and_normalize_erasing_regions(ty)?;
                let layout = self.layout_of(ty)?;
                if let mir::NullOp::SizeOf | mir::NullOp::AlignOf = null_op
                    && layout.is_unsized()
                {
                    span_bug!(
                        self.frame().current_span(),
                        "{null_op:?} MIR operator called for unsized type {ty}",
                    );
                }
                let val = match null_op {
                    mir::NullOp::SizeOf => layout.size.bytes(),
                    mir::NullOp::AlignOf => layout.align.abi.bytes(),
                    mir::NullOp::OffsetOf(fields) => {
                        layout.offset_of_subfield(self, fields.iter()).bytes()
                    }
                };
                self.write_scalar(Scalar::from_target_usize(val, self), &dest)?;
            }

            ShallowInitBox(ref operand, _) => {
                let src = self.eval_operand(operand, None)?;
                let v = self.read_immediate(&src)?;
                self.write_immediate(*v, &dest)?;
            }

            Cast(cast_kind, ref operand, cast_ty) => {
                let src = self.eval_operand(operand, None)?;
                let cast_ty =
                    self.subst_from_current_frame_and_normalize_erasing_regions(cast_ty)?;
                self.cast(&src, cast_kind, cast_ty, &dest)?;
            }

            Discriminant(place) => {
                let op = self.eval_place_to_op(place, None)?;
                let variant = self.read_discriminant(&op)?;
                let discr = self.discriminant_for_variant(op.layout.ty, variant)?;
                self.write_immediate(*discr, &dest)?;
            }
        }

        trace!("{:?}", self.dump_place(&dest));

        Ok(())
    }
    // use rustc_middle::mir::{BasicBlock};
    /// Evaluate the given terminator. Will also adjust the stack frame and statement position accordingly.
    fn terminator(&mut self,
                  terminator: &mir::Terminator<'tcx>,
                  _path : &mut Vec<usize>,
                  s: &mut usize,
                  _stk:&mut Vec<PathInfo>,
                  _is_loop: &mut bool,
                  _limit:usize,
                  _scc_info: &mut IndexVec<usize, Vec<SccInfo>>,) -> InterpResult<'tcx> {
        info!("{:?}", terminator.kind);

        // yj: only if terminator.kind == Call
        // init starting index, ...
        println!("generate path starting index {:?}", s);
        self.eval_terminator(terminator)?;
        if !self.stack().is_empty() {
            if let Either::Left(loc) = self.frame().loc {
                // yj: my code
                let def_id = self.body().source.def_id();
                let def_name = self.tcx.def_path_str(def_id);
                println!("[terminator] BASIC INFO\n   * def string={:?}\n   * krate={:?}\n   * index={:?}\n   * terminator kind={:?}\n   * loc.block={:?}",
                         def_name,
                         def_id.krate.index(),
                         def_id.index.index(),
                         terminator.kind.name(),
                         loc.block.clone(),
                );

                let type_id = self.body().clone().local_decls;
                let body_span = self.body().clone().span;
                println!("[terminator] SPAN info\n   * type_id length={:?}\n   * type_id[0].span={:?}\n   * body.span={:?}",
                         type_id.len(),
                         type_id.raw[0].clone().source_info.span,
                         body_span,
                );

                if terminator.kind.name() != "Call" {
                    //          self.body().arg_count, def_name, type_id.raw[0].local_info.clone(), terminator.kind.name());
                } else {
                    println!("[step id is Call] yes generate path, def is constant  {:?}\n--------------------------",
                             self.body().arg_count);

                    // println!("print type!{:?}", Any::type_name(self.body().source.type_id()));
                    // let file_name = format!("/home/y23kim/rust/scc_info/{:?}.json", def_name);
                    let file_name = format!("/home/y23kim/rust/scc_info/{:?}_{:?}.json", def_id.krate, def_id.index);
                    let _file_name22:String = file_name.chars().take(50).collect();
                    if Path::new(&file_name).exists() {
                        let _file = File::open(file_name).expect("[step] Failed to open file");
                        // let scc_info_stk: FxHashMap<usize, Vec<SccInfo>> = serde_json::from_reader(file).expect("Failed to deserialize");
                        let scc_info_stk: FxHashMap<usize, Vec<SccInfo>> = Default::default();
                        // let scc_info_stk: IndexVec<usize, Vec<SccInfo>> = serde_json::from_reader(file).expect("Failed to deserialize");
                        println!("[STEP] Read scc_info file ={:?}", scc_info_stk);
                        // TODO FIX!!!!!!!! (remove)
                        // *s = 0;

                        if !scc_info_stk.is_empty() {
                            println!("size scc_info_stk {:?}", scc_info_stk.len());
                            // FxHashMap to IndexVec
                            let mut scc_info2: IndexVec<usize, Vec<SccInfo>>
                                = IndexVec::with_capacity(scc_info_stk.len());

                            for key in 0..scc_info_stk.len() {
                                if let Some(value) = scc_info_stk.get(&key) {
                                    scc_info2.push(value.clone());
                                } else {
                                    panic!("Missing key in FxHashMap: {}", key);
                                }
                            }

                            let t : usize = loc.block.index();
                            // println!("[DEBUG] s= {:?} t={:?} stk={:?} is_loop={:?}", *s, t, stk, is_loop);
                            // ---------------------- PATH

                            // FIX: HOW TO CHECK IF THE DEFINITEION'S TYPE
                            // def_id.is_top_level_module()
                            // self.tcx.data_layout;
                            // let ty = TyBuilder::def_ty(ctx.sema.db, def_id.into(), None);
                                                // yunji
                            // CONDITION: only when the file exist
                            // CONDITION: and only when the definition has MIR, not a construco
                            // only when it has cfg(content)
                            // only when it is not a definition of Struct or single value..
                            // generate_path(scc_info2, *s, t, stk, is_loop, limit, path);
                            // generate_path(self.body().scc_info.clone(), *s, t, stk, is_loop, limit, path);
                            // println!("[DEBUG] s= {:?} t= {:?} scc_info = {:?}", *s,  t, self.body().scc_info.clone()[t]);
                            *s = t;
                        } else {
                            println!("[step][terminator] MAP is EMPTY, check type{:?}", type_id);
                        }
                    } else {
                        println!("[step] file not exist {:?}, def = {:?}", file_name, def_name);
                    }

                }

                info!("[step] executing {:?}", loc.block);
            }
        }
        Ok(())
    }

}

fn _generate_path(scc_info_stk: IndexVec<usize, Vec<SccInfo>>,
                 s:usize, t:usize,
                 stk:&mut Vec<PathInfo>, is_loop: &mut bool, limit: usize,
                path: &mut Vec<usize>,
) {
    let mut recorded = false;
    println!("Run generate_path");
    // ============= Exiting edge ============= //
    let mut s_idx = 0;
    let mut t_idx = 0;

    while s_idx < scc_info_stk[s].len()
        && t_idx < scc_info_stk[t].len()
        && scc_info_stk[s][s_idx].id == scc_info_stk[t][t_idx].id {
        s_idx += 1;
        t_idx += 1;
    }

    while s_idx < scc_info_stk[s].len() {
        if let Some(mut prev) = stk.pop() {
            prev.temp_path.push(prev.prefix.clone());

            let sccid : i32 = scc_info_stk[s][s_idx].id as i32 * -1;
            if let Some(last) = stk.last_mut() {
                last.prefix.push(sccid.try_into().unwrap());
                for p in prev.temp_path {
                    for pp in p {
                        last.prefix.push(pp);
                    }
                }
                last.prefix.push(sccid.try_into().unwrap());
            } else {
                // fin.push(sccid.try_into().unwrap()); // for debuging

                for p in prev.temp_path {
                    for pp in p {
                        // fin.push(pp);
                        path.push(pp as usize);

                        // println!("push to final path (loop) {:?}", pp);
                        let bb_number = format!("{:?} ", pp);
                        let mut file = fs::OpenOptions::new().append(true).create(true)
                            .open("/home/y23kim/rust/test_progs/corpus/sub_dir/new_path").expect("Fail to write yunji");
                        // .open("/home/y23kim/rust/test_progs/path_dir").expect("Fail to write yunji");
                        file.write_all(bb_number.as_bytes()).expect("yunji: Fail to write.");

                    }
                }
                // fin.push(sccid.try_into().unwrap()); // for debugging
            }
        }
        // yunji comment
        // println!("[1] Exit edge");
        // for e in stk.iter() {
        //     println!("  * {:?}", e);
        // }
        s_idx += 1;
        *is_loop = false;
    }

    // ============= Normal & Back edge ============= //

    s_idx = 0;
    t_idx = 0;
    while s_idx < scc_info_stk[s].len()
        && t_idx < scc_info_stk[t].len()
        && scc_info_stk[s][s_idx].id == scc_info_stk[t][t_idx].id {
        *is_loop=true;

        // h1, l2, x3
        if s==t || (scc_info_stk[s][s_idx].node_type == NodeType::Latch && scc_info_stk[t][t_idx].node_type == NodeType::Header) {
            if let Some(last) = stk.last_mut() {
                if recorded==false {
                    last.prefix.push(t as i32);
                    recorded=true;
                }
                let mut content :Vec<i32> = vec!();
                let mut prefix_to_key :Vec<i32> = vec!();
                let mut k:i32;
                let mut i=0;
                while i<last.prefix.len() {
                    k = last.prefix[i];
                    while k < 0 { // until find inner scc
                        i += 1;
                        if last.prefix[i] < 0 { break;} // ignoore inner scc content
                        content.push(last.prefix[i].try_into().unwrap());
                    }
                    content.push(last.prefix[i].try_into().unwrap());
                    prefix_to_key.push(k.try_into().unwrap()); // k= scc_id
                    i += 1;
                }
                // println!("content {:?}", content);
                let mut flag = true;
                if let Some(val) = last.counts.get_mut(&prefix_to_key) {
                    *val += 1;
                    if *val >= limit { flag = false;}
                } else {
                    last.counts.insert(prefix_to_key, 1);
                }
                if flag {
                    last.temp_path.push(content);
                }
                last.prefix = vec!();
            }

            // println!("[2] back edge" );
            // for e in stk.iter() {
            //     println!("  * {:?}", e);
            // }

            if s==t {
                t_idx = scc_info_stk[t].len();
                // println!("[2-1] self loop back edge" );
                break;
            }

        }
        else {
            // println!("[3] normal edge" );
            // for e in stk.iter() {
            //     println!("  * {:?}", e);
            // }
            if recorded==false {
                stk.last_mut().unwrap().prefix.push(t as i32);
                recorded=true;
            }


        }
        s_idx += 1;
        t_idx += 1;
    }

    // ============= Entering edge (Header node) ============= //
    while t_idx < scc_info_stk[t].len() {
        *is_loop = true;

        let tmp;
        if recorded {
            tmp = vec!(vec!());
        } else {        // in case it never met back edge, push header node
            tmp = vec!(vec!(t as i32));
        }
        let el = PathInfo {
            counts: HashDefault::default(),
            temp_path: tmp,
            prefix: vec!(),
        };

        stk.push(el);
        t_idx += 1;

        // println!("[4] Entering edge (Push)" );
        // for e in stk.iter() {
        //     println!("  * {:?}", e);
        // }
    }

    if *is_loop == false {
        // println!("push to final path (no loop) {:?}", t);
        z1.push(t);
        let bb_number = format!("{:?} ", t);
        let mut file = fs::OpenOptions::new().append(true).create(true)
            .open("/home/y23kim/rust/test_progs/corpus/sub_dir/new_path").expect("Fail to write yunji");
        // .open("/home/y23kim/rust/test_progs/path_dir").expect("Fail to write yunji");

        file.write_all(bb_number.as_bytes()).expect("yunji: Fail to write.");

        // fin.push(t.try_into().unwrap());
        // println!("[5] Not loop" );
        // for e in stk.iter() {
        //     println!("  * {:?}", e);
        // }
    }
    // println!("at last stk={:?}", stk);
}