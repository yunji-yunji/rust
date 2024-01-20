//! An interpreter for MIR used in CTFE and by miri

mod cast;
mod discriminant;
mod eval_context;
mod intern;
mod intrinsics;
mod machine;
mod memory;
mod operand;
mod operator;
mod place;
mod projection;
mod step;
mod terminator;
mod traits;
mod util;
mod validity;
mod visitor;
pub mod dump;
pub mod dump2;
pub mod track_stack;

pub use rustc_middle::mir::interpret::*; // have all the `interpret` symbols in one place: here

pub use self::eval_context::{Frame, FrameInfo, InterpCx, StackPopCleanup};
pub use self::intern::{
    intern_const_alloc_for_constprop, intern_const_alloc_recursive, InternKind,
};
pub use self::machine::{compile_time_machine, AllocMap, Machine, MayLeak, StackPopJump};
pub use self::memory::{AllocKind, AllocRef, AllocRefMut, FnVal, Memory, MemoryKind};
pub use self::operand::{ImmTy, Immediate, OpTy, Readable};
pub use self::place::{MPlaceTy, MemPlaceMeta, PlaceTy, Writeable};
pub use self::projection::{OffsetMode, Projectable};
pub use self::terminator::FnArg;
pub use self::validity::{CtfeValidationMode, RefTracking};
pub use self::visitor::ValueVisitor;

use self::{
    operand::Operand,
    place::{MemPlace, Place},
};

pub(crate) use self::intrinsics::eval_nullary_intrinsic;
use eval_context::{from_known_layout, mir_assign_valid_types};
