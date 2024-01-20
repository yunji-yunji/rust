use rustc_middle::{
    // ty, 
    ty::layout::LayoutOf,
    mir::{self, 
        // interpret::{InterpErrorInfo},
         ConstAlloc}
};
use crate::{const_eval::{
    report, get_span_and_frames,
    // CanAccessStatics, 
    CompileTimeEvalContext, CompileTimeInterpreter}, errors::ConstEvalError};
use crate::interpret::{
    // intern_const_alloc_recursive, CtfeValidationMode, 
    GlobalId, 
    // Immediate, InternKind, 
    InterpCx,
    // InterpError, 
    InterpResult, 
    MPlaceTy, 
    MemoryKind, 
    // OpTy, RefTracking, 
    StackPopCleanup,
};


pub fn hook_ecx2<'mir, 'tcx>(
    mut ecx: InterpCx<'mir, 'tcx, CompileTimeInterpreter<'mir, 'tcx>>,
    cid: GlobalId<'tcx>,
    _is_static: bool,
) -> ::rustc_middle::mir::interpret::EvalToAllocationRawResult<'tcx> {
    let res = ecx.load_mir(cid.instance.def, cid.promoted);
    match res.and_then(|body| scan_stack(&mut ecx, cid, body)) {
        // match res.and_then(|body| eval_body_using_ecx(&mut ecx, cid, body)) {
        Err(error) => {
            let (error, backtrace) = error.into_parts();
            backtrace.print_backtrace();

            let (kind, instance) = 
            // if is_static {
                ("static", String::new());
                // ("const", String::new())
            // };

            Err(report(
                *ecx.tcx,
                error,
                None,
                || get_span_and_frames(ecx.tcx, &ecx.machine),
                |span, frames| ConstEvalError {
                    span,
                    error_kind: kind,
                    instance,
                    frame_notes: frames,
                },
            ))
        }
        Ok(mplace) => {
            // Since evaluation had no errors, validate the resulting constant.
            // This is a separate `try` block to provide more targeted error reporting.
            // let validation =
            //     const_validate_mplace(&ecx, &mplace, is_static, cid.promoted.is_some());

            let alloc_id = mplace.ptr().provenance.unwrap().alloc_id();

            // // Validation failed, report an error.
            // if let Err(error) = validation {
            //     Err(const_report_error(&ecx, error, alloc_id))
            // } else {
            //     // Convert to raw constant
            Ok(ConstAlloc { alloc_id, ty: mplace.layout.ty })
        }
    }
}


fn scan_stack<'mir, 'tcx>(
    ecx: &mut CompileTimeEvalContext<'mir, 'tcx>,
    cid: GlobalId<'tcx>,
    body: &'mir mir::Body<'tcx>,
) -> InterpResult<'tcx, MPlaceTy<'tcx>> {
    // print!("check_stack: {:?}, {:?}", cid, ecx.param_env);
    let tcx = *ecx.tcx;

    let layout = ecx.layout_of(body.bound_return_ty().instantiate(tcx, cid.instance.args))?;
    let ret = ecx.allocate(layout, MemoryKind::Stack)?;
    // let ret = ret.unwrap();
    ecx.push_stack_frame(
        cid.instance,
        body,
        &ret.clone().into(),
        StackPopCleanup::Root { cleanup: false },
    )?;
    // let _ = a.unwrap();

    Ok(ret)

}


// pub fn hook_ecx<'mir, 'tcx>(
//     // mut ecx: InterpCx<'mir, 'tcx, CompileTimeInterpreter<'mir, 'tcx>>,
//     ecx: InterpCx<'mir, 'tcx, CompileTimeInterpreter<'mir, 'tcx>>,
//     cid: GlobalId<'tcx>,
//     ctec: &mut CompileTimeEvalContext<'mir, 'tcx>,

// // ) -> &'tcx mut CompileTimeEvalContext<'mir, 'tcx> {
// // ) -> ::rustc_middle::mir::interpret::EvalToAllocationRawResult<'tcx> {
// ) {
//     // load mir=> get body
//     // let body = __
//     // let instance: ty::InstanceDef<'tcx> = cid.instance.def;
//     // let promoted: Option<mir::Promoted> = cid.promoted;
//     // trace!("load mir(instance={:?}, promoted={:?})", instance, promoted);
//     // let body = if let Some(promoted) = promoted {
//     //     let def = instance.def_id();
//     //     ecx.tcx.promoted_mir(def)[promoted]
//     // } else {
//     //     M::load_mir(ecx, instance)?
//     // };
//     let res: Result<&mir::Body<'_>, InterpErrorInfo<'_>> = ecx.load_mir(cid.instance.def, cid.promoted);
//     // let _ = res.and_then(|body: &mir::Body<'_> | check_stack(&mut ecx, cid, body));
//     if let Ok(body) = res {
//         // let (_, out_ecx) = check_stack(&mut ecx, cid, body);
//         let _ = check_stack(ctec, cid, body);
//         // match check_stack(&mut ecx, cid, body) {
//         //     Err(_err) => {
//         //         // Err()
//         //         bug!("err");
//         //     },
//         //     Ok(mplace) => {
//         //         let alloc_id = mplace.ptr().provenance.unwrap().alloc_id();
//         //         Ok(ConstAlloc { alloc_id, ty: mplace.layout.ty })
//         //     }
//         // }
//     } else {
//         bug!("err");
//     }
// }

// // self == ecx
// fn check_stack<'mir, 'tcx>(
//     ecx: &mut CompileTimeEvalContext<'mir, 'tcx>,
//     cid: GlobalId<'tcx>,
//     body: &'mir mir::Body<'tcx>,
// ) -> InterpResult<'tcx, MPlaceTy<'tcx>> {
// // ) -> (InterpResult<'tcx, MPlaceTy<'tcx>>, &'tcx mut CompileTimeEvalContext<'mir, 'tcx>) {
// // ) -> &'tcx mut CompileTimeEvalContext<'mir, 'tcx> {
//     print!("check_stack: {:?}, {:?}", cid, ecx.param_env);
//     let tcx = *ecx.tcx;

//     let layout = ecx.layout_of(body.bound_return_ty().instantiate(tcx, cid.instance.args))?;
//     let ret = ecx.allocate(layout, MemoryKind::Stack)?;
//     // let ret = ret.unwrap();
//     ecx.push_stack_frame(
//         cid.instance,
//         body,
//         &ret.clone().into(),
//         StackPopCleanup::Root { cleanup: false },
//     )?;
//     // let _ = a.unwrap();

//     Ok(ret)
//     // (Ok(ret), ecx)
// }