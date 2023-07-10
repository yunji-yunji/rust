//! This pass removes storage markers if they won't be emitted during codegen.

use crate::MirPass;
use rustc_middle::mir::*;
use rustc_middle::ty::TyCtxt;

// pub struct RemoveStorageMarkers;

// impl<'tcx> MirPass<'tcx> for RemoveStorageMarkers {
//     fn is_enabled(&self, sess: &rustc_session::Session) -> bool {
//         sess.mir_opt_level() > 0
//     }

//     fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
//         if tcx.sess.emit_lifetime_markers() {
//             return;
//         }

//         trace!("Running RemoveStorageMarkers on {:?}", body.source);
//         for data in body.basic_blocks.as_mut_preserves_cfg() {
//             data.statements.retain(|statement| match statement.kind {
//                 StatementKind::StorageLive(..)
//                 | StatementKind::StorageDead(..)
//                 | StatementKind::Nop => false,
//                 _ => true,
//             })
//         }
//     }
// }



// use rustc_middle::mir::MirPass;
// use rustc_middle::mir::{BasicBlock, Body, TerminatorKind, BasicBlockData, };
use rustc_ast::ast::{InlineAsmOptions, InlineAsmTemplatePiece};

pub struct DummyYJ();
impl<'tcx> MirPass<'tcx> for DummyYJ {
    fn is_enabled(&self, sess: &rustc_session::Session) -> bool {
        // sess.instrument_coverage()
        // sess.mir_opt_level() > 0
        sess.mir_opt_level() == 0
        // true
    }
    // #[instrument(skip(self, tcx, body))]
    fn run_pass(&self, tcx: TyCtxt<'tcx>, mir_body: &mut Body<'tcx>) {
        // ================= try mir pass
        println!("yunji: add bb.rs");
        // let bbs = mir.basic_blocks_mut();
        let bbs = mir_body.basic_blocks_mut();
        // let bbs = tcx.body().basic_blocks_mut();
        // let bbs = tcx.promoted.basic_blocks_mut();
        // let bbs = tcx.hir().body().basic_blocks_mut();

        let template_piece = InlineAsmTemplatePiece::String(String::from("yunji mir pass test"));
        let template = [template_piece];
        // let template = tcx.arena.alloc(&template);
        // let template = tcx.arena.alloc([template_piece]);
        let template = tcx.arena.alloc_from_iter(template);

        let asm_terminator_kind = TerminatorKind::InlineAsm {
            template,
            operands: vec![],
            options: InlineAsmOptions::empty(),
            line_spans: &[],
            destination: Some(bbs.next_index()),
            // cleanup: None,
            unwind: UnwindAction::Unreachable,
        };

        let len = bbs.len();
        let original_last_block = bbs.get_mut(BasicBlock::from_usize(len-1)).expect("No last block!");

        let mut new_terminator = original_last_block.terminator.as_ref().expect("no terminator").clone();
        // let mut new_terminator = original_last_block.terminator.as_mut().expect("no terminator");
        let original_last_block_terminator = original_last_block.terminator_mut();
        // let original_last_block_terminator = original_last_block.terminator();
        new_terminator.kind = asm_terminator_kind;

        let _new_bb = BasicBlockData {
            statements: vec![],
            terminator: Some(original_last_block_terminator.to_owned()),
            is_cleanup: false,
        };

        // bbs.push(new_bb);

        // ================= try mir pass

        let _mir_source = mir_body.source;

        // if mir_source.promoted.is_some() {
        //     trace!(
        //         "MINE skipped for {:?} (already promoted for Miri evaluation)",
        //         mir_source.def_id()
        //     );
        //     return;
        // }

        // let is_fn_like =
        //     tcx.hir().get_by_def_id(mir_source.def_id().expect_local()).fn_kind().is_some();

        // if !is_fn_like {
        //     trace!("MIRYUNJIPASS skipped for {:?} (not an fn-like)", mir_source.def_id());
        //     return;
        // }

        // match mir_body.basic_blocks[mir::START_BLOCK].terminator().kind {
        //     TerminatorKind::Unreachable => {
        //         trace!("MIRYUNJIPASS skipped for unreachable `START_BLOCK`");
        //         return;
        //     }
        //     _ => {}
        // }

        // let codegen_fn_attrs = tcx.codegen_fn_attrs(mir_source.def_id());
        // if codegen_fn_attrs.flags.contains(CodegenFnAttrFlags::NO_COVERAGE) {
        //     return;
        // }

        // trace!("MIRYUNJIPASS starting for {:?}", mir_source.def_id());
        // Instrumentor::new(&self.name(), tcx, mir_body).inject_counters();
        // trace!("MIRYUNJIPASS done for {:?}", mir_source.def_id());
    }
}

// pub trait MirPass<'tcx> {
//     fn name(&self) -> &str {
//         let name = std::any::type_name::<Self>();
//         if let Some((_, tail)) = name.rsplit_once(':') { tail } else { name }
//     }

//     /// Returns `true` if this pass is enabled with the current combination of compiler flags.
//     fn is_enabled(&self, _sess: &Session) -> bool {
//         true
//     }

//     fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>);

//     fn is_mir_dump_enabled(&self) -> bool {
//         true
//     }
// }
/*
use std::io;

pub fn add_single_bb(tcx: TyCtxt<'_>, mir_body: &'body Body<'tcx>) -> io::Result<()> {

    println!("RUN MirPass YuNJI");
    // let bbs = mir.basic_blocks_mut();
    let bbs = mir_body.basic_blocks_mut();
    // let bbs = tcx.body().basic_blocks_mut();
    // let bbs = tcx.promoted.basic_blocks_mut();
    // let bbs = tcx.hir().body().basic_blocks_mut();

    let template_piece = InlineAsmTemplatePiece::String(String::from("yunji mir pass test"));
    let template = [template_piece];
    // let template = tcx.arena.alloc(&template);
    // let template = tcx.arena.alloc([template_piece]);
    let template = tcx.arena.alloc_from_iter(template);

    let asm_terminator_kind = TerminatorKind::InlineAsm {
        template, 
        operands: vec![], 
        options: InlineAsmOptions::empty(),
        line_spans: &[],
        destination: Some(bbs.next_index()),
        cleanup: None,
    };

    let len = bbs.len();
    let original_last_block = bbs.get_mut(BasicBlock::from_usize(len-1)).expect("No last block!");

    let mut new_terminator = original_last_block.terminator.as_ref().expect("no terminator").clone();
    // let mut new_terminator = original_last_block.terminator.as_mut().expect("no terminator");
    let original_last_block_terminator = original_last_block.terminator_mut();
    // let original_last_block_terminator = original_last_block.terminator();
    new_terminator.kind = asm_terminator_kind;

    let new_bb = BasicBlockData {
        statements: vec![],
        terminator: Some(original_last_block_terminator.to_owned()),
        is_cleanup: false,
    };

    bbs.push(new_bb);
    Ok(())
}


*/

