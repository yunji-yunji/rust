use crate::MirPass;
use rustc_middle::ty::TyCtxt;
use rustc_middle::mir::Body;

pub struct LoopUnroll();
impl<'tcx> MirPass<'tcx> for LoopUnroll {
    fn is_enabled(&self, _sess: &rustc_session::Session) -> bool {
        // sess.instrument_coverage()
        // sess.mir_opt_level() > 0
        // sess.mir_opt_level() == 2
        true
    }

    // #[instrument(skip(self, tcx, body))]
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {

        let def_id = body.source.def_id();
        println!("[MIRPASS] def_id={:?} def_name={:?}", def_id, &tcx.def_path_str(def_id));
        // If move library crates pass through this MirPass, transformation part should be placed here.
        // But, they do not reach here.
        // So, I delete all codes.

    }
}

