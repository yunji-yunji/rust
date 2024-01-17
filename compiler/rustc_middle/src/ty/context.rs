//! Type context book-keeping.

#![allow(rustc::usage_of_ty_tykind)]

pub mod tls;

use crate::arena::Arena;
use crate::dep_graph::{DepGraph, DepKindStruct};
use crate::infer::canonical::{CanonicalParamEnvCache, CanonicalVarInfo, CanonicalVarInfos};
use crate::lint::lint_level;
use crate::metadata::ModChild;
use crate::middle::codegen_fn_attrs::CodegenFnAttrs;
use crate::middle::resolve_bound_vars;
use crate::middle::stability;
use crate::mir::interpret::{self, Allocation, ConstAllocation};
use crate::mir::{Body, Local, Place, PlaceElem, ProjectionKind, Promoted};
use crate::query::plumbing::QuerySystem;
use crate::query::LocalCrate;
use crate::query::Providers;
use crate::query::{IntoQueryParam, TyCtxtAt};
use crate::thir::Thir;
use crate::traits;
use crate::traits::solve;
use crate::traits::solve::{
    ExternalConstraints, ExternalConstraintsData, PredefinedOpaques, PredefinedOpaquesData,
};
use crate::ty::{
    self, AdtDef, AdtDefData, AdtKind, Binder, Clause, Const, ConstData, GenericParamDefKind,
    ImplPolarity, List, ParamConst, ParamTy, PolyExistentialPredicate, PolyFnSig, Predicate,
    PredicateKind, PredicatePolarity, Region, RegionKind, ReprOptions, TraitObjectVisitor, Ty,
    TyKind, TyVid, TypeVisitable, Visibility,
};
use crate::ty::{GenericArg, GenericArgs, GenericArgsRef};
use rustc_ast::{self as ast, attr};
use rustc_data_structures::fingerprint::Fingerprint;
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_data_structures::intern::Interned;
use rustc_data_structures::profiling::SelfProfilerRef;
use rustc_data_structures::sharded::{IntoPointer, ShardedHashMap};
use rustc_data_structures::stable_hasher::{HashStable, StableHasher};
use rustc_data_structures::steal::Steal;
use rustc_data_structures::sync::{self, FreezeReadGuard, Lock, Lrc, WorkerLocal};
#[cfg(parallel_compiler)]
use rustc_data_structures::sync::{DynSend, DynSync};
use rustc_data_structures::unord::UnordSet;
use rustc_errors::{
    Applicability, Diag, DiagCtxt, DiagMessage, ErrorGuaranteed, LintDiagnostic, MultiSpan,
};
use rustc_hir as hir;
use rustc_hir::def::DefKind;
use rustc_hir::def_id::{CrateNum, DefId, LocalDefId, LOCAL_CRATE};
use rustc_hir::definitions::Definitions;
use rustc_hir::intravisit::Visitor;
use rustc_hir::lang_items::LangItem;
use rustc_hir::{HirId, Node, TraitCandidate};
use rustc_index::IndexVec;
use rustc_macros::HashStable;
use rustc_query_system::dep_graph::DepNodeIndex;
use rustc_query_system::ich::StableHashingContext;
use rustc_serialize::opaque::{FileEncodeResult, FileEncoder};
use rustc_session::config::CrateType;
use rustc_session::cstore::{CrateStoreDyn, Untracked};
use rustc_session::lint::Lint;
use rustc_session::{Limit, MetadataKind, Session};
use rustc_span::def_id::{DefPathHash, StableCrateId, CRATE_DEF_ID};
use rustc_span::symbol::{kw, sym, Ident, Symbol};
use rustc_span::{Span, DUMMY_SP};
use rustc_target::abi::{FieldIdx, Layout, LayoutS, TargetDataLayout, VariantIdx};
use rustc_target::spec::abi;
use rustc_type_ir::TyKind::*;
use rustc_type_ir::WithCachedTypeInfo;
use rustc_type_ir::{CollectAndApply, Interner, TypeFlags};

use std::borrow::Borrow;
use std::cell::RefCell;
// use std::rc::Rc;
use std::cmp::Ordering;
use std::{fmt, fs};
use std::hash::{Hash, Hasher};
use std::iter;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Bound, Deref};
use std::path::PathBuf;
// use rustc_codegen_ssa::pafl::{};

#[allow(rustc::usage_of_ty_tykind)]
impl<'tcx> Interner for TyCtxt<'tcx> {
    type DefId = DefId;
    type AdtDef = ty::AdtDef<'tcx>;
    type GenericArgs = ty::GenericArgsRef<'tcx>;
    type GenericArg = ty::GenericArg<'tcx>;
    type Term = ty::Term<'tcx>;

    type Binder<T: TypeVisitable<TyCtxt<'tcx>>> = Binder<'tcx, T>;
    type BoundVars = &'tcx List<ty::BoundVariableKind>;
    type BoundVar = ty::BoundVariableKind;
    type CanonicalVars = CanonicalVarInfos<'tcx>;

    type Ty = Ty<'tcx>;
    type Tys = &'tcx List<Ty<'tcx>>;
    type AliasTy = ty::AliasTy<'tcx>;
    type ParamTy = ParamTy;
    type BoundTy = ty::BoundTy;
    type PlaceholderTy = ty::PlaceholderType;

    type ErrorGuaranteed = ErrorGuaranteed;
    type BoundExistentialPredicates = &'tcx List<PolyExistentialPredicate<'tcx>>;
    type PolyFnSig = PolyFnSig<'tcx>;
    type AllocId = crate::mir::interpret::AllocId;

    type Const = ty::Const<'tcx>;
    type AliasConst = ty::UnevaluatedConst<'tcx>;
    type PlaceholderConst = ty::PlaceholderConst;
    type ParamConst = ty::ParamConst;
    type BoundConst = ty::BoundVar;
    type ValueConst = ty::ValTree<'tcx>;
    type ExprConst = ty::Expr<'tcx>;

    type Region = Region<'tcx>;
    type EarlyParamRegion = ty::EarlyParamRegion;
    type BoundRegion = ty::BoundRegion;
    type LateParamRegion = ty::LateParamRegion;
    type InferRegion = ty::RegionVid;
    type PlaceholderRegion = ty::PlaceholderRegion;

    type Predicate = Predicate<'tcx>;
    type TraitPredicate = ty::TraitPredicate<'tcx>;
    type RegionOutlivesPredicate = ty::RegionOutlivesPredicate<'tcx>;
    type TypeOutlivesPredicate = ty::TypeOutlivesPredicate<'tcx>;
    type ProjectionPredicate = ty::ProjectionPredicate<'tcx>;
    type NormalizesTo = ty::NormalizesTo<'tcx>;
    type SubtypePredicate = ty::SubtypePredicate<'tcx>;
    type CoercePredicate = ty::CoercePredicate<'tcx>;
    type ClosureKind = ty::ClosureKind;

    fn mk_canonical_var_infos(self, infos: &[ty::CanonicalVarInfo<Self>]) -> Self::CanonicalVars {
        self.mk_canonical_var_infos(infos)
    }
}

type InternedSet<'tcx, T> = ShardedHashMap<InternedInSet<'tcx, T>, ()>;

pub struct CtxtInterners<'tcx> {
    /// The arena that types, regions, etc. are allocated from.
    arena: &'tcx WorkerLocal<Arena<'tcx>>,

    // Specifically use a speedy hash algorithm for these hash sets, since
    // they're accessed quite often.
    type_: InternedSet<'tcx, WithCachedTypeInfo<TyKind<'tcx>>>,
    const_lists: InternedSet<'tcx, List<ty::Const<'tcx>>>,
    args: InternedSet<'tcx, GenericArgs<'tcx>>,
    type_lists: InternedSet<'tcx, List<Ty<'tcx>>>,
    canonical_var_infos: InternedSet<'tcx, List<CanonicalVarInfo<'tcx>>>,
    region: InternedSet<'tcx, RegionKind<'tcx>>,
    poly_existential_predicates: InternedSet<'tcx, List<PolyExistentialPredicate<'tcx>>>,
    predicate: InternedSet<'tcx, WithCachedTypeInfo<ty::Binder<'tcx, PredicateKind<'tcx>>>>,
    clauses: InternedSet<'tcx, List<Clause<'tcx>>>,
    projs: InternedSet<'tcx, List<ProjectionKind>>,
    place_elems: InternedSet<'tcx, List<PlaceElem<'tcx>>>,
    const_: InternedSet<'tcx, WithCachedTypeInfo<ConstData<'tcx>>>,
    const_allocation: InternedSet<'tcx, Allocation>,
    bound_variable_kinds: InternedSet<'tcx, List<ty::BoundVariableKind>>,
    layout: InternedSet<'tcx, LayoutS<FieldIdx, VariantIdx>>,
    adt_def: InternedSet<'tcx, AdtDefData>,
    external_constraints: InternedSet<'tcx, ExternalConstraintsData<'tcx>>,
    predefined_opaques_in_body: InternedSet<'tcx, PredefinedOpaquesData<'tcx>>,
    fields: InternedSet<'tcx, List<FieldIdx>>,
    local_def_ids: InternedSet<'tcx, List<LocalDefId>>,
    offset_of: InternedSet<'tcx, List<(VariantIdx, FieldIdx)>>,
}

impl<'tcx> CtxtInterners<'tcx> {
    fn new(arena: &'tcx WorkerLocal<Arena<'tcx>>) -> CtxtInterners<'tcx> {
        CtxtInterners {
            arena,
            type_: Default::default(),
            const_lists: Default::default(),
            args: Default::default(),
            type_lists: Default::default(),
            region: Default::default(),
            poly_existential_predicates: Default::default(),
            canonical_var_infos: Default::default(),
            predicate: Default::default(),
            clauses: Default::default(),
            projs: Default::default(),
            place_elems: Default::default(),
            const_: Default::default(),
            const_allocation: Default::default(),
            bound_variable_kinds: Default::default(),
            layout: Default::default(),
            adt_def: Default::default(),
            external_constraints: Default::default(),
            predefined_opaques_in_body: Default::default(),
            fields: Default::default(),
            local_def_ids: Default::default(),
            offset_of: Default::default(),
        }
    }

    /// Interns a type. (Use `mk_*` functions instead, where possible.)
    #[allow(rustc::usage_of_ty_tykind)]
    #[inline(never)]
    fn intern_ty(&self, kind: TyKind<'tcx>, sess: &Session, untracked: &Untracked) -> Ty<'tcx> {
        Ty(Interned::new_unchecked(
            self.type_
                .intern(kind, |kind| {
                    let flags = super::flags::FlagComputation::for_kind(&kind);
                    let stable_hash = self.stable_hash(&flags, sess, untracked, &kind);

                    InternedInSet(self.arena.alloc(WithCachedTypeInfo {
                        internee: kind,
                        stable_hash,
                        flags: flags.flags,
                        outer_exclusive_binder: flags.outer_exclusive_binder,
                    }))
                })
                .0,
        ))
    }

    /// Interns a const. (Use `mk_*` functions instead, where possible.)
    #[allow(rustc::usage_of_ty_tykind)]
    #[inline(never)]
    fn intern_const(
        &self,
        data: ty::ConstData<'tcx>,
        sess: &Session,
        untracked: &Untracked,
    ) -> Const<'tcx> {
        Const(Interned::new_unchecked(
            self.const_
                .intern(data, |data: ConstData<'_>| {
                    let flags = super::flags::FlagComputation::for_const(&data.kind, data.ty);
                    let stable_hash = self.stable_hash(&flags, sess, untracked, &data);

                    InternedInSet(self.arena.alloc(WithCachedTypeInfo {
                        internee: data,
                        stable_hash,
                        flags: flags.flags,
                        outer_exclusive_binder: flags.outer_exclusive_binder,
                    }))
                })
                .0,
        ))
    }

    fn stable_hash<'a, T: HashStable<StableHashingContext<'a>>>(
        &self,
        flags: &ty::flags::FlagComputation,
        sess: &'a Session,
        untracked: &'a Untracked,
        val: &T,
    ) -> Fingerprint {
        // It's impossible to hash inference variables (and will ICE), so we don't need to try to cache them.
        // Without incremental, we rarely stable-hash types, so let's not do it proactively.
        if flags.flags.intersects(TypeFlags::HAS_INFER) || sess.opts.incremental.is_none() {
            Fingerprint::ZERO
        } else {
            let mut hasher = StableHasher::new();
            let mut hcx = StableHashingContext::new(sess, untracked);
            val.hash_stable(&mut hcx, &mut hasher);
            hasher.finish()
        }
    }

    /// Interns a predicate. (Use `mk_predicate` instead, where possible.)
    #[inline(never)]
    fn intern_predicate(
        &self,
        kind: Binder<'tcx, PredicateKind<'tcx>>,
        sess: &Session,
        untracked: &Untracked,
    ) -> Predicate<'tcx> {
        Predicate(Interned::new_unchecked(
            self.predicate
                .intern(kind, |kind| {
                    let flags = super::flags::FlagComputation::for_predicate(kind);

                    let stable_hash = self.stable_hash(&flags, sess, untracked, &kind);

                    InternedInSet(self.arena.alloc(WithCachedTypeInfo {
                        internee: kind,
                        stable_hash,
                        flags: flags.flags,
                        outer_exclusive_binder: flags.outer_exclusive_binder,
                    }))
                })
                .0,
        ))
    }
}

// For these preinterned values, an alternative would be to have
// variable-length vectors that grow as needed. But that turned out to be
// slightly more complex and no faster.

const NUM_PREINTERNED_TY_VARS: u32 = 100;
const NUM_PREINTERNED_FRESH_TYS: u32 = 20;
const NUM_PREINTERNED_FRESH_INT_TYS: u32 = 3;
const NUM_PREINTERNED_FRESH_FLOAT_TYS: u32 = 3;

// This number may seem high, but it is reached in all but the smallest crates.
const NUM_PREINTERNED_RE_VARS: u32 = 500;
const NUM_PREINTERNED_RE_LATE_BOUNDS_I: u32 = 2;
const NUM_PREINTERNED_RE_LATE_BOUNDS_V: u32 = 20;

pub struct CommonTypes<'tcx> {
    pub unit: Ty<'tcx>,
    pub bool: Ty<'tcx>,
    pub char: Ty<'tcx>,
    pub isize: Ty<'tcx>,
    pub i8: Ty<'tcx>,
    pub i16: Ty<'tcx>,
    pub i32: Ty<'tcx>,
    pub i64: Ty<'tcx>,
    pub i128: Ty<'tcx>,
    pub usize: Ty<'tcx>,
    pub u8: Ty<'tcx>,
    pub u16: Ty<'tcx>,
    pub u32: Ty<'tcx>,
    pub u64: Ty<'tcx>,
    pub u128: Ty<'tcx>,
    pub f16: Ty<'tcx>,
    pub f32: Ty<'tcx>,
    pub f64: Ty<'tcx>,
    pub f128: Ty<'tcx>,
    pub str_: Ty<'tcx>,
    pub never: Ty<'tcx>,
    pub self_param: Ty<'tcx>,

    /// Dummy type used for the `Self` of a `TraitRef` created for converting
    /// a trait object, and which gets removed in `ExistentialTraitRef`.
    /// This type must not appear anywhere in other converted types.
    /// `Infer(ty::FreshTy(0))` does the job.
    pub trait_object_dummy_self: Ty<'tcx>,

    /// Pre-interned `Infer(ty::TyVar(n))` for small values of `n`.
    pub ty_vars: Vec<Ty<'tcx>>,

    /// Pre-interned `Infer(ty::FreshTy(n))` for small values of `n`.
    pub fresh_tys: Vec<Ty<'tcx>>,

    /// Pre-interned `Infer(ty::FreshIntTy(n))` for small values of `n`.
    pub fresh_int_tys: Vec<Ty<'tcx>>,

    /// Pre-interned `Infer(ty::FreshFloatTy(n))` for small values of `n`.
    pub fresh_float_tys: Vec<Ty<'tcx>>,
}

pub struct CommonLifetimes<'tcx> {
    /// `ReStatic`
    pub re_static: Region<'tcx>,

    /// Erased region, used outside of type inference.
    pub re_erased: Region<'tcx>,

    /// Pre-interned `ReVar(ty::RegionVar(n))` for small values of `n`.
    pub re_vars: Vec<Region<'tcx>>,

    /// Pre-interned values of the form:
    /// `ReBound(DebruijnIndex(i), BoundRegion { var: v, kind: BrAnon })`
    /// for small values of `i` and `v`.
    pub re_late_bounds: Vec<Vec<Region<'tcx>>>,
}

pub struct CommonConsts<'tcx> {
    pub unit: Const<'tcx>,
    pub true_: Const<'tcx>,
    pub false_: Const<'tcx>,
}

impl<'tcx> CommonTypes<'tcx> {
    fn new(
        interners: &CtxtInterners<'tcx>,
        sess: &Session,
        untracked: &Untracked,
    ) -> CommonTypes<'tcx> {
        let mk = |ty| interners.intern_ty(ty, sess, untracked);

        let ty_vars =
            (0..NUM_PREINTERNED_TY_VARS).map(|n| mk(Infer(ty::TyVar(TyVid::from(n))))).collect();
        let fresh_tys: Vec<_> =
            (0..NUM_PREINTERNED_FRESH_TYS).map(|n| mk(Infer(ty::FreshTy(n)))).collect();
        let fresh_int_tys: Vec<_> =
            (0..NUM_PREINTERNED_FRESH_INT_TYS).map(|n| mk(Infer(ty::FreshIntTy(n)))).collect();
        let fresh_float_tys: Vec<_> =
            (0..NUM_PREINTERNED_FRESH_FLOAT_TYS).map(|n| mk(Infer(ty::FreshFloatTy(n)))).collect();

        CommonTypes {
            unit: mk(Tuple(List::empty())),
            bool: mk(Bool),
            char: mk(Char),
            never: mk(Never),
            isize: mk(Int(ty::IntTy::Isize)),
            i8: mk(Int(ty::IntTy::I8)),
            i16: mk(Int(ty::IntTy::I16)),
            i32: mk(Int(ty::IntTy::I32)),
            i64: mk(Int(ty::IntTy::I64)),
            i128: mk(Int(ty::IntTy::I128)),
            usize: mk(Uint(ty::UintTy::Usize)),
            u8: mk(Uint(ty::UintTy::U8)),
            u16: mk(Uint(ty::UintTy::U16)),
            u32: mk(Uint(ty::UintTy::U32)),
            u64: mk(Uint(ty::UintTy::U64)),
            u128: mk(Uint(ty::UintTy::U128)),
            f16: mk(Float(ty::FloatTy::F16)),
            f32: mk(Float(ty::FloatTy::F32)),
            f64: mk(Float(ty::FloatTy::F64)),
            f128: mk(Float(ty::FloatTy::F128)),
            str_: mk(Str),
            self_param: mk(ty::Param(ty::ParamTy { index: 0, name: kw::SelfUpper })),

            trait_object_dummy_self: fresh_tys[0],

            ty_vars,
            fresh_tys,
            fresh_int_tys,
            fresh_float_tys,
        }
    }
}

impl<'tcx> CommonLifetimes<'tcx> {
    fn new(interners: &CtxtInterners<'tcx>) -> CommonLifetimes<'tcx> {
        let mk = |r| {
            Region(Interned::new_unchecked(
                interners.region.intern(r, |r| InternedInSet(interners.arena.alloc(r))).0,
            ))
        };

        let re_vars =
            (0..NUM_PREINTERNED_RE_VARS).map(|n| mk(ty::ReVar(ty::RegionVid::from(n)))).collect();

        let re_late_bounds = (0..NUM_PREINTERNED_RE_LATE_BOUNDS_I)
            .map(|i| {
                (0..NUM_PREINTERNED_RE_LATE_BOUNDS_V)
                    .map(|v| {
                        mk(ty::ReBound(
                            ty::DebruijnIndex::from(i),
                            ty::BoundRegion { var: ty::BoundVar::from(v), kind: ty::BrAnon },
                        ))
                    })
                    .collect()
            })
            .collect();

        CommonLifetimes {
            re_static: mk(ty::ReStatic),
            re_erased: mk(ty::ReErased),
            re_vars,
            re_late_bounds,
        }
    }
}

impl<'tcx> CommonConsts<'tcx> {
    fn new(
        interners: &CtxtInterners<'tcx>,
        types: &CommonTypes<'tcx>,
        sess: &Session,
        untracked: &Untracked,
    ) -> CommonConsts<'tcx> {
        let mk_const = |c| {
            interners.intern_const(
                c, sess, // This is only used to create a stable hashing context.
                untracked,
            )
        };

        CommonConsts {
            unit: mk_const(ty::ConstData {
                kind: ty::ConstKind::Value(ty::ValTree::zst()),
                ty: types.unit,
            }),
            true_: mk_const(ty::ConstData {
                kind: ty::ConstKind::Value(ty::ValTree::Leaf(ty::ScalarInt::TRUE)),
                ty: types.bool,
            }),
            false_: mk_const(ty::ConstData {
                kind: ty::ConstKind::Value(ty::ValTree::Leaf(ty::ScalarInt::FALSE)),
                ty: types.bool,
            }),
        }
    }
}

/// This struct contains information regarding a free parameter region,
/// either a `ReEarlyParam` or `ReLateParam`.
#[derive(Debug)]
pub struct FreeRegionInfo {
    /// `LocalDefId` of the free region.
    pub def_id: LocalDefId,
    /// the bound region corresponding to free region.
    pub bound_region: ty::BoundRegionKind,
    /// checks if bound region is in Impl Item
    pub is_impl_item: bool,
}

/// This struct should only be created by `create_def`.
#[derive(Copy, Clone)]
pub struct TyCtxtFeed<'tcx, KEY: Copy> {
    pub tcx: TyCtxt<'tcx>,
    // Do not allow direct access, as downstream code must not mutate this field.
    key: KEY,
}

/// Never return a `Feed` from a query. Only queries that create a `DefId` are
/// allowed to feed queries for that `DefId`.
impl<KEY: Copy, CTX> !HashStable<CTX> for TyCtxtFeed<'_, KEY> {}

/// The same as `TyCtxtFeed`, but does not contain a `TyCtxt`.
/// Use this to pass around when you have a `TyCtxt` elsewhere.
/// Just an optimization to save space and not store hundreds of
/// `TyCtxtFeed` in the resolver.
#[derive(Copy, Clone)]
pub struct Feed<'tcx, KEY: Copy> {
    _tcx: PhantomData<TyCtxt<'tcx>>,
    // Do not allow direct access, as downstream code must not mutate this field.
    key: KEY,
}

/// Never return a `Feed` from a query. Only queries that create a `DefId` are
/// allowed to feed queries for that `DefId`.
impl<KEY: Copy, CTX> !HashStable<CTX> for Feed<'_, KEY> {}

impl<T: fmt::Debug + Copy> fmt::Debug for Feed<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.key.fmt(f)
    }
}

/// Some workarounds to use cases that cannot use `create_def`.
/// Do not add new ways to create `TyCtxtFeed` without consulting
/// with T-compiler and making an analysis about why your addition
/// does not cause incremental compilation issues.
impl<'tcx> TyCtxt<'tcx> {
    /// Can only be fed before queries are run, and is thus exempt from any
    /// incremental issues. Do not use except for the initial query feeding.
    pub fn feed_unit_query(self) -> TyCtxtFeed<'tcx, ()> {
        self.dep_graph.assert_ignored();
        TyCtxtFeed { tcx: self, key: () }
    }

    /// Can only be fed before queries are run, and is thus exempt from any
    /// incremental issues. Do not use except for the initial query feeding.
    pub fn feed_local_crate(self) -> TyCtxtFeed<'tcx, CrateNum> {
        self.dep_graph.assert_ignored();
        TyCtxtFeed { tcx: self, key: LOCAL_CRATE }
    }

    /// Only used in the resolver to register the `CRATE_DEF_ID` `DefId` and feed
    /// some queries for it. It will panic if used twice.
    pub fn create_local_crate_def_id(self, span: Span) -> TyCtxtFeed<'tcx, LocalDefId> {
        let key = self.untracked().source_span.push(span);
        assert_eq!(key, CRATE_DEF_ID);
        TyCtxtFeed { tcx: self, key }
    }

    /// In order to break cycles involving `AnonConst`, we need to set the expected type by side
    /// effect. However, we do not want this as a general capability, so this interface restricts
    /// to the only allowed case.
    pub fn feed_anon_const_type(self, key: LocalDefId, value: ty::EarlyBinder<Ty<'tcx>>) {
        debug_assert_eq!(self.def_kind(key), DefKind::AnonConst);
        TyCtxtFeed { tcx: self, key }.type_of(value)
    }
}

impl<'tcx, KEY: Copy> TyCtxtFeed<'tcx, KEY> {
    #[inline(always)]
    pub fn key(&self) -> KEY {
        self.key
    }

    #[inline(always)]
    pub fn downgrade(self) -> Feed<'tcx, KEY> {
        Feed { _tcx: PhantomData, key: self.key }
    }
}

impl<'tcx, KEY: Copy> Feed<'tcx, KEY> {
    #[inline(always)]
    pub fn key(&self) -> KEY {
        self.key
    }

    #[inline(always)]
    pub fn upgrade(self, tcx: TyCtxt<'tcx>) -> TyCtxtFeed<'tcx, KEY> {
        TyCtxtFeed { tcx, key: self.key }
    }
}

impl<'tcx> TyCtxtFeed<'tcx, LocalDefId> {
    #[inline(always)]
    pub fn def_id(&self) -> LocalDefId {
        self.key
    }

    // Caller must ensure that `self.key` ID is indeed an owner.
    pub fn feed_owner_id(&self) -> TyCtxtFeed<'tcx, hir::OwnerId> {
        TyCtxtFeed { tcx: self.tcx, key: hir::OwnerId { def_id: self.key } }
    }

    // Fills in all the important parts needed by HIR queries
    pub fn feed_hir(&self) {
        self.local_def_id_to_hir_id(HirId::make_owner(self.def_id()));

        let node = hir::OwnerNode::Synthetic;
        let bodies = Default::default();
        let attrs = hir::AttributeMap::EMPTY;

        let (opt_hash_including_bodies, _) = self.tcx.hash_owner_nodes(node, &bodies, &attrs.map);
        let node = node.into();
        self.opt_hir_owner_nodes(Some(self.tcx.arena.alloc(hir::OwnerNodes {
            opt_hash_including_bodies,
            nodes: IndexVec::from_elem_n(
                hir::ParentedNode { parent: hir::ItemLocalId::INVALID, node },
                1,
            ),
            bodies,
        })));
        self.feed_owner_id().hir_attrs(attrs);
    }
}

// ===============
use serde::Serialize;
use crate::mir::BasicBlock;
use rustc_middle::mir::graphviz::write_mir_fn_graphviz;

use super::layout::HasTyCtxt;
// use std::cell::Cell;
use rustc_middle::mir::{BasicBlockData, Operand, TerminatorKind, UnwindAction};
use rustc_middle::ty::{
    ConstKind, EarlyBinder, ExistentialPredicate, FloatTy, GenericArgKind,
    Instance, InstanceDef, IntTy, ParamEnv, UintTy, ValTree,
    // ParamEnv, Instance, ValTree
};
use rustc_target::spec::abi::Abi;
use rustc_type_ir::Mutability;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

#[derive(Serialize)]
pub enum Native {
    TLSWith,
}

impl Native {
    const BUILTINS: [Native; 1] = [Self::TLSWith];

    /// unlock the pattern triple
    pub fn pattern(&self) -> (&str, &str, &str) {
        match self {
            Self::TLSWith => ("std", "::thread::local::", "with"),
        }
    }

    /// probe whether a def_id is a native built-in
    pub fn probe(tcx: TyCtxt<'_>, id: DefId) -> Option<Self> {
        if id.is_local() {
            return None;
        }

        let krate = tcx.crate_name(id.krate).to_string();
        let path = tcx.def_path(id).to_string_no_crate_verbose();

        for item in Self::BUILTINS {
            let (k, prefix, suffix) = item.pattern();
            if &krate != k {
                continue;
            }
            match path.as_str().strip_prefix(prefix).and_then(|s| s.strip_suffix(suffix)) {
                None => continue,
                Some(_) => return Some(item),
            }
        }
        None
    }
}




/// Identifier mimicking `DefId`
#[derive(Serialize, Clone, Debug)]
pub struct Ident2 {
    pub krate: usize,
    pub index: usize,
}

impl From<DefId> for Ident2 {
    fn from(id: DefId) -> Self {
        Self { krate: id.krate.as_usize(), index: id.index.as_usize() }
    }
}


/// Constant value or aggregates
#[derive(Serialize, Clone, Debug)]
pub enum ValueTree {
    Scalar { bit: usize, val: u128 },
    Struct(Vec<ValueTree>),
}

/// Serializable information about a Rust const
#[derive(Serialize, Clone, Debug)]
pub enum PaflConst {
    Param { index: u32, name: String },
    Value(ValueTree),
}

/// Serializable information about a Rust type
#[derive(Serialize, Clone, Debug)]
pub enum PaflType {
    Never,
    Bool,
    Char,
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Str,
    Param { index: u32, name: String },
    Adt(TyInstKey),
    Alias(TyInstKey),
    Opaque(Ident2),
    FnPtr(Vec<PaflType>, Box<PaflType>),
    FnDef(FnInstKey),
    Closure(FnInstKey),
    Dynamic(Vec<Ident2>),
    ImmRef(Box<PaflType>),
    MutRef(Box<PaflType>),
    ImmPtr(Box<PaflType>),
    MutPtr(Box<PaflType>),
    Slice(Box<PaflType>),
    Array(Box<PaflType>, PaflConst),
    Tuple(Vec<PaflType>),
}

/// Serializable information about a Rust generic argument
#[derive(Serialize, Clone, Debug)]
pub enum PaflGeneric {
    Lifetime,
    Type(PaflType),
    Const(PaflConst),
}

/// Identifier for type instance
#[derive(Serialize, Clone, Debug)]
pub struct TyInstKey {
    pub krate: Option<String>,
    pub index: usize,
    pub path: String,
    pub generics: Vec<PaflGeneric>,
}

/// Identifier for function instance
#[derive(Serialize, Clone, Debug)]
pub struct FnInstKey {
    pub krate: Option<String>,
    pub index: usize,
    pub path: String,
    pub generics: Vec<PaflGeneric>,
}


// ==================


/// Kind of a call instruction
#[derive(Serialize)]
pub enum CallKind {
    Direct,
    Bridge,
    Virtual(usize),
    Builtin(Native),
    Intrinsic,
}

/// Callee of a call instruction
#[derive(Serialize)]
pub struct CallSite {
    inst: FnInstKey,
    kind: CallKind,
}

/// Identifier mimicking `BasicBlock`
#[derive(Serialize)]
pub struct BlkId {
    index: usize,
}

impl From<BasicBlock> for BlkId {
    fn from(id: BasicBlock) -> Self {
        Self { index: id.as_usize() }
    }
}

/// How unwind might work
#[derive(Serialize)]
pub enum UnwindRoute {
    Resume,
    Terminate,
    Unreachable,
    Cleanup(BlkId),
}

impl From<&UnwindAction> for UnwindRoute {
    fn from(action: &UnwindAction) -> Self {
        match action {
            UnwindAction::Continue => Self::Resume,
            UnwindAction::Unreachable => Self::Unreachable,
            UnwindAction::Terminate(..) => Self::Terminate,
            UnwindAction::Cleanup(blk) => Self::Cleanup((*blk).into()),
        }
    }
}

/// Kinds of terminator instructions of a basic block
#[derive(Serialize)]
pub enum TermKind {
    Unreachable,
    Goto(BlkId),
    Switch(Vec<BlkId>),
    Return,
    UnwindResume,
    UnwindFinish,
    Assert { target: BlkId, unwind: UnwindRoute },
    Drop { target: BlkId, unwind: UnwindRoute },
    Call { site: CallSite, target: Option<BlkId>, unwind: UnwindRoute },
}

/// Serializable information about a basic block
#[derive(Serialize)]
pub struct PaflBlock {
    id: BlkId,
    term: TermKind,
}

/// Serializable information about a user-defined function
#[derive(Serialize)]
pub struct PaflCFG {
    blocks: Vec<PaflBlock>,
}

/// Serializable information about a user-defined function
#[derive(Serialize)]
pub enum FnBody {
    Defined(PaflCFG),
    Bridged(PaflCFG),
    Skipped,
    Intrinsic,
}

/// Serializable information about a user-defined function
#[derive(Serialize)]
pub struct PaflFunction {
    inst: FnInstKey,
    body: FnBody,
}

/// Serializable information about the entire crate
#[derive(Serialize)]
pub struct PaflCrate {
    pub functions: Vec<PaflFunction>,
}

/// Helper for dumping path-AFL related information
pub struct PaflDump<'sum, 'tcx> {
    /// context provider
    pub tcx: TyCtxt<'tcx>,
    /// parameter environment
    pub param_env: ParamEnv<'tcx>,
    /// verbosity
    pub verbose: bool,
    /// path to meta directory
    pub path_meta: PathBuf,
    /// path to data directory
    pub path_data: PathBuf,
    /// path to the data file
    pub path_prefix: PathBuf,
    /// call stack
    pub stack: &'sum mut Vec<Instance<'tcx>>,
    /// information cache
    pub cache: &'sum mut FxHashMap<Instance<'tcx>, FnInstKey>,
    /// summary repository
    pub summary: &'sum mut Vec<PaflFunction>,
}

impl<'sum, 'tcx> PaflDump<'sum, 'tcx> {
    /// initialize the context for information dumping
    pub fn initialize(&self, instance: Instance<'tcx>) {
        // normalize and check consistency
        let normalized_ty = instance.ty(self.tcx, self.param_env);
        match normalized_ty.kind() {
            ty::FnDef(ty_def_id, ty_def_args) | ty::Closure(ty_def_id, ty_def_args) => {
                if *ty_def_id != instance.def_id() {
                    bug!("normalized type def_id mismatch");
                }
                if ty_def_args.len() != instance.args.len() {
                    bug!("normalized type generics length mismatch");
                }
                for (t1, t2) in ty_def_args.iter().zip(instance.args.iter()) {
                    if t1 != t2 {
                        bug!("normalized type generics content mismatch");
                    }
                }
            }
            _ => bug!("normalized type is neither function nor closure"),
        }
    }
}

impl<'sum, 'tcx> PaflDump<'sum, 'tcx> {
    /// Resolve an instantiation to a ty key
    pub fn resolve_ty_key(&self, id: DefId, args: GenericArgsRef<'tcx>) -> TyInstKey {
        let krate =
            if id.is_local() { None } else { Some(self.tcx.crate_name(id.krate).to_string()) };
        TyInstKey {
            krate,
            index: id.index.as_usize(),
            path: self.tcx.def_path(id).to_string_no_crate_verbose(),
            generics: self.process_generics(args),
        }
    }

    /// Resolve an instantiation to a fn key
    pub fn resolve_fn_key(&self, id: DefId, args: GenericArgsRef<'tcx>) -> FnInstKey {
        let krate =
            if id.is_local() { None } else { Some(self.tcx.crate_name(id.krate).to_string()) };
        FnInstKey {
            krate,
            index: id.index.as_usize(),
            path: self.tcx.def_path(id).to_string_no_crate_verbose(),
            generics: self.process_generics(args),
        }
    }
}

impl<'sum, 'tcx> PaflDump<'sum, 'tcx> {
    /// Process a value tree
    pub fn process_vtree(&self, tree: ValTree<'tcx>) -> ValueTree {
        match tree {
            ValTree::Leaf(scalar) => ValueTree::Scalar {
                bit: scalar.size().bits_usize(),
                val: scalar.to_bits(scalar.size()).expect("scalar value"),
            },
            ValTree::Branch(items) => {
                let mut subs = vec![];
                for item in items {
                    subs.push(self.process_vtree(*item));
                }
                ValueTree::Struct(subs)
            }
        }
    }

    /// Process a constant
    pub fn process_const(&self, item: Const<'tcx>) -> PaflConst {
        match item.kind() {
            ConstKind::Param(param) => {
                PaflConst::Param { index: param.index, name: param.name.to_string() }
            }
            ConstKind::Value(value) => PaflConst::Value(self.process_vtree(value)),
            _ => bug!("unrecognized constant: {:?}", item),
        }
    }

    /// Process the type
    pub fn process_type(&self, item: Ty<'tcx>) -> PaflType {
        match item.kind() {
            ty::Never => PaflType::Never,
            ty::Bool => PaflType::Bool,
            ty::Char => PaflType::Char,
            ty::Int(IntTy::Isize) => PaflType::Isize,
            ty::Int(IntTy::I8) => PaflType::I8,
            ty::Int(IntTy::I16) => PaflType::I16,
            ty::Int(IntTy::I32) => PaflType::I32,
            ty::Int(IntTy::I64) => PaflType::I64,
            ty::Int(IntTy::I128) => PaflType::I128,
            ty::Uint(UintTy::Usize) => PaflType::Usize,
            ty::Uint(UintTy::U8) => PaflType::U8,
            ty::Uint(UintTy::U16) => PaflType::U16,
            ty::Uint(UintTy::U32) => PaflType::U32,
            ty::Uint(UintTy::U64) => PaflType::U64,
            ty::Uint(UintTy::U128) => PaflType::U128,
            ty::Float(FloatTy::F32) => PaflType::F32,
            ty::Float(FloatTy::F64) => PaflType::F64,
            ty::Str => PaflType::Str,
            ty::Param(p) => PaflType::Param { index: p.index, name: p.name.to_string() },
            ty::Adt(def, args) => PaflType::Adt(self.resolve_ty_key(def.did(), args)),
            ty::Alias(_, alias) => PaflType::Alias(self.resolve_ty_key(alias.def_id, alias.args)),
            ty::Foreign(def_id) => PaflType::Opaque((*def_id).into()),
            ty::FnPtr(binder) => {
                if !matches!(binder.abi(), Abi::Rust | Abi::RustCall) {
                    bug!("fn ptr not following the RustCall ABI: {}", binder.abi());
                }
                if binder.c_variadic() {
                    bug!("variadic not supported yet");
                }

                let mut inputs = vec![];
                for item in binder.inputs().iter() {
                    let ty = *item.skip_binder();
                    inputs.push(self.process_type(ty));
                }
                let output = self.process_type(binder.output().skip_binder());
                PaflType::FnPtr(inputs, output.into())
            }
            ty::FnDef(def_id, args) => PaflType::FnDef(self.resolve_fn_key(*def_id, *args)),
            ty::Closure(def_id, args) => PaflType::Closure(self.resolve_fn_key(*def_id, *args)),
            ty::Ref(_region, sub, mutability) => {
                let converted = self.process_type(*sub);
                match mutability {
                    Mutability::Not => PaflType::ImmRef(converted.into()),
                    Mutability::Mut => PaflType::MutRef(converted.into()),
                }
            }
            ty::RawPtr(ty_and_mut) => {
                let converted = self.process_type(ty_and_mut.ty);
                match ty_and_mut.mutbl {
                    Mutability::Not => PaflType::ImmPtr(converted.into()),
                    Mutability::Mut => PaflType::MutPtr(converted.into()),
                }
            }
            ty::Slice(sub) => PaflType::Slice(self.process_type(*sub).into()),
            ty::Array(sub, len) => {
                PaflType::Array(self.process_type(*sub).into(), self.process_const(*len))
            }
            ty::Tuple(elems) => {
                PaflType::Tuple(elems.iter().map(|e| self.process_type(e)).collect())
            }
            ty::Dynamic(binders, _region, _) => {
                let mut traits = vec![];
                for binder in *binders {
                    let predicate = binder.skip_binder();
                    let def_id = match predicate {
                        ExistentialPredicate::Trait(r) => r.def_id,
                        ExistentialPredicate::Projection(r) => r.def_id,
                        ExistentialPredicate::AutoTrait(r) => r,
                    };
                    traits.push(def_id.into());
                }
                PaflType::Dynamic(traits)
            }
            _ => bug!("unrecognized type: {:?}", item),
        }
    }

    /// Process the generic arguments
    pub fn process_generics(&self, args: GenericArgsRef<'tcx>) -> Vec<PaflGeneric> {
        let mut generics = vec![];
        for arg in args {
            let sub = match arg.unpack() {
                GenericArgKind::Lifetime(_region) => PaflGeneric::Lifetime,
                GenericArgKind::Type(item) => PaflGeneric::Type(self.process_type(item)),
                GenericArgKind::Const(item) => PaflGeneric::Const(self.process_const(item)),
            };
            generics.push(sub);
        }
        generics
    }

    /// Resolve the call target
    pub fn process_callsite(&mut self, callee: &Operand<'tcx>, span: Span) -> CallSite {
        // extract def_id and generic arguments for callee
        let cty = match callee.constant() {
            None => bug!("callee is not a constant: {:?}", span),
            Some(c) => c.const_.ty(),
        };
        let (def_id, generic_args) = match cty.kind() {
            ty::Closure(def_id, generic_args) | ty::FnDef(def_id, generic_args) => {
                (*def_id, *generic_args)
            }
            _ => bug!("callee is not a function or closure: {:?}", span),
        };

        // test if we should skip this function
        if let Some(native) = Native::probe(self.tcx, def_id) {
            let inst = self.resolve_fn_key(def_id, generic_args);
            return CallSite { inst, kind: CallKind::Builtin(native) };
        }

        if self.verbose {
            print!(
                "{} - resolving: {}{}",
                "  ".repeat(self.stack.len()),
                self.tcx.crate_name(def_id.krate).to_string(),
                self.tcx.def_path(def_id).to_string_no_crate_verbose(),
            );
        }

        // resolve trait targets, if possible
        let resolved = Instance::expect_resolve(self.tcx, self.param_env, def_id, generic_args);
        let call_site = match resolved.def {
            InstanceDef::Item(_) => {
                if self.verbose {
                    println!(" ~> direct");
                }
                let inst = PaflDump::summarize_instance(
                    self.tcx,
                    self.param_env,
                    resolved,
                    self.verbose,
                    &self.path_meta,
                    &self.path_data,
                    self.stack,
                    self.cache,
                    self.summary,
                );
                CallSite { inst, kind: CallKind::Direct }
            }
            InstanceDef::ClosureOnceShim { .. } => {
                if self.verbose {
                    println!(" ~> closure");
                }

                // extract the actual callee
                assert_eq!(resolved.args.len(), 2);
                let unwrapped = match resolved.args.get(0).unwrap().expect_ty().kind() {
                    ty::Closure(closure_id, closure_args) => {
                        Instance::new(*closure_id, *closure_args)
                    }
                    _ => bug!("expect closure"),
                };

                // handle the actual callee
                let inst = PaflDump::summarize_instance(
                    self.tcx,
                    self.param_env,
                    unwrapped,
                    self.verbose,
                    &self.path_meta,
                    &self.path_data,
                    self.stack,
                    self.cache,
                    self.summary,
                );
                CallSite { inst, kind: CallKind::Direct }
            }
            InstanceDef::FnPtrShim(shim_id, _) => {
                if self.verbose {
                    println!(" ~> indirect");
                }

                // extract the actual callee
                let body = self.tcx.instance_mir(resolved.def).clone();
                let instantiated = resolved.instantiate_mir_and_normalize_erasing_regions(
                    self.tcx,
                    self.param_env,
                    EarlyBinder::bind(body),
                );

                let shim_crate = self.tcx.crate_name(shim_id.krate).to_string();
                let shim_path = self.tcx.def_path(shim_id).to_string_no_crate_verbose();

                let fn_ty = match shim_crate.as_str() {
                    "core" => match shim_path.as_str() {
                        "::ops::function::FnOnce::call_once" => {
                            let args: Vec<_> = instantiated.args_iter().collect();
                            assert_eq!(args.len(), 2);

                            let arg0 = *args.get(0).unwrap();
                            instantiated.local_decls.get(arg0).unwrap().ty
                        }
                        "::ops::function::Fn::call" => {
                            let args: Vec<_> = instantiated.args_iter().collect();
                            assert_eq!(args.len(), 2);

                            let arg0 = *args.get(0).unwrap();
                            match instantiated.local_decls.get(arg0).unwrap().ty.kind() {
                                ty::Ref(_, t, Mutability::Not) => *t,
                                _ => bug!("invalid argument type for call"),
                            }
                        }
                        "::ops::function::FnMut::call_mut" => {
                            let args: Vec<_> = instantiated.args_iter().collect();
                            assert_eq!(args.len(), 2);

                            let arg0 = *args.get(0).unwrap();
                            match instantiated.local_decls.get(arg0).unwrap().ty.kind() {
                                ty::Ref(_, t, Mutability::Mut) => *t,
                                _ => bug!("invalid argument type for call_mut"),
                            }
                        }
                        _ => bug!("unrecognized fn ptr shim: {}{}", shim_crate, shim_path),
                    },
                    _ => bug!("unrecognized fn ptr shim: {}{}", shim_crate, shim_path),
                };

                let unwrapped = match fn_ty.kind() {
                    ty::Closure(fn_def_id, fn_generic_args)
                    | ty::FnDef(fn_def_id, fn_generic_args) => {
                        Instance::new(*fn_def_id, *fn_generic_args)
                    }
                    _ => bug!(
                        "{}{} into neither a function nor closure: {:?}",
                        shim_crate,
                        shim_path,
                        span
                    ),
                };

                // handle the actual callee
                let inst = PaflDump::summarize_instance(
                    self.tcx,
                    self.param_env,
                    unwrapped,
                    self.verbose,
                    &self.path_meta,
                    &self.path_data,
                    self.stack,
                    self.cache,
                    self.summary,
                );
                CallSite { inst, kind: CallKind::Direct }
            }
            InstanceDef::DropGlue(_, _) | InstanceDef::CloneShim(_, _) => {
                if self.verbose {
                    println!(" ~> bridge");
                }
                let inst = PaflDump::summarize_instance(
                    self.tcx,
                    self.param_env,
                    resolved,
                    self.verbose,
                    &self.path_meta,
                    &self.path_data,
                    self.stack,
                    self.cache,
                    self.summary,
                );
                CallSite { inst, kind: CallKind::Bridge }
            }
            InstanceDef::Virtual(virtual_id, offset) => {
                if self.verbose {
                    println!(" ~> virtual#{}", offset);
                }
                let inst = self.resolve_fn_key(virtual_id, resolved.args);
                CallSite { inst, kind: CallKind::Virtual(offset) }
            }
            InstanceDef::Intrinsic(intrinsic_id) => {
                if self.verbose {
                    println!(" ~> intrinsic");
                }
                let inst: FnInstKey = self.resolve_fn_key(intrinsic_id, resolved.args);
                CallSite { inst, kind: CallKind::Intrinsic }
            }
            InstanceDef::VTableShim(..)
            | InstanceDef::ReifyShim(..)
            | InstanceDef::FnPtrAddrShim(..)
            | InstanceDef::ThreadLocalShim(..) => {
                bug!("unusual calls are not supported yet: {}", resolved);
            }
        };

        // done with the resolution
        call_site
    }

    /// Process the mir for one basic block
    pub fn process_block(&mut self, id: BasicBlock, data: &BasicBlockData<'tcx>) -> PaflBlock {
        let term = data.terminator();

        // match by the terminator
        let kind = match &term.kind {
            // basics
            TerminatorKind::Goto { target } => TermKind::Goto((*target).into()),
            TerminatorKind::SwitchInt { discr: _, targets } => {
                TermKind::Switch(targets.all_targets().iter().map(|b| (*b).into()).collect())
            }
            TerminatorKind::Unreachable => TermKind::Unreachable,
            TerminatorKind::Return => TermKind::Return,
            // call (which may unwind)
            TerminatorKind::Call {
                func,
                args: _,
                destination: _,
                target,
                unwind,
                call_source: _,
                fn_span: _,
            } => TermKind::Call {
                site: self.process_callsite(func, term.source_info.span),
                target: target.as_ref().map(|t| (*t).into()),
                unwind: unwind.into(),
            },
            TerminatorKind::Drop { place: _, target, unwind, replace: _ } => {
                TermKind::Drop { target: (*target).into(), unwind: unwind.into() }
            }
            TerminatorKind::Assert { cond: _, expected: _, msg: _, target, unwind } => {
                TermKind::Assert { target: (*target).into(), unwind: unwind.into() }
            }
            // unwinding
            TerminatorKind::UnwindResume => TermKind::UnwindResume,
            TerminatorKind::UnwindTerminate(..) => TermKind::UnwindFinish,
            // imaginary
            TerminatorKind::FalseEdge { real_target, imaginary_target: _ }
            | TerminatorKind::FalseUnwind { real_target, unwind: _ } => {
                TermKind::Goto((*real_target).into())
            }
            // coroutine
            TerminatorKind::Yield { .. } | TerminatorKind::CoroutineDrop => {
                bug!("unexpected coroutine")
            }
            // assembly
            TerminatorKind::InlineAsm { .. } => bug!("unexpected inline assembly"),
        };

        // done
        PaflBlock { id: id.into(), term: kind }
    }

    /// Process the mir body for one function
    pub fn process_cfg(&mut self, id: DefId, body: &Body<'tcx>) -> PaflCFG {
        let path = self.tcx.def_path(id).to_string_no_crate_verbose();

        // dump the control flow graph if requested
        match std::env::var_os("PAFL_CFG") {
            None => (),
            Some(v) => {
                println!("PAFL CFG v={:?}, path = {:?}", v, path.as_str());
                if v.to_str().map_or(false, |s| s == path.as_str()) {
                    let dot_path = self.path_prefix.with_extension("dot");
                    println!("PAFL CFG dot_path={:?}", dot_path);
                    let mut dot_file = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&dot_path)
                        .expect("unable to create dot file");
                    write_mir_fn_graphviz(self.tcx, body, false, &mut dot_file)
                        .expect("failed to create dot file");
                }
            }
        }

        // iterate over each basic blocks
        let mut blocks = vec![];
        for blk_id in body.basic_blocks.reverse_postorder() {
            let blk_data = body.basic_blocks.get(*blk_id).unwrap();
            blocks.push(self.process_block(*blk_id, blk_data));
        }

        // done
        PaflCFG { blocks }
    }

    /// Process a codegen instance
    pub fn summarize_instance(
        tcx: TyCtxt<'tcx>,
        param_env: ParamEnv<'tcx>,
        instance: Instance<'tcx>,
        verbose: bool,
        path_meta: &Path,
        path_data: &Path,
        stack: &'sum mut Vec<Instance<'tcx>>,
        cache: &'sum mut FxHashMap<Instance<'tcx>, FnInstKey>,
        summary: &'sum mut Vec<PaflFunction>,
    ) -> FnInstKey {
        // check if we have seen the instance
        if let Some(cached) = cache.get(&instance) {
            return cached.clone();
        }

        let id = instance.def_id();
        let path = tcx.def_path(id);
        let depth = stack.len();

        // create a place holder
        let index = loop {
            let mut count: usize = 0;
            for entry in fs::read_dir(path_meta).expect("list meta directory") {
                let _ = entry.expect("iterate meta directory entry");
                count += 1;
            }
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path_meta.join(count.to_string()))
            {
                Ok(mut file) => {
                    let content = format!("{}", path.to_string_no_crate_verbose(),);
                    file.write_all(content.as_bytes()).expect("save meta content");
                    break count;
                }
                Err(_) => continue,
            }
        };
        let path_prefix = path_data.join(index.to_string());

        // construct the worker
        let mut dumper = PaflDump {
            tcx,
            param_env,
            verbose,
            path_meta: path_meta.to_path_buf(),
            path_data: path_data.to_path_buf(),
            path_prefix,
            stack,
            cache,
            summary,
        };

        // derive the inst key and mark it in the cache
        let inst = dumper.resolve_fn_key(id, instance.args);

        // mark beginning of processing
        if verbose {
            println!(
                "{}[->] {}{}",
                "  ".repeat(depth),
                inst.krate.as_ref().map_or("@", |s| s.as_str()),
                inst.path
            );
        }

        // normalize, check consistency, and initialize
        dumper.initialize(instance);
        dumper.stack.push(instance);
        dumper.cache.insert(instance, inst.clone());

        // branch processing by instance type
        let body = match &instance.def {
            InstanceDef::Item(id) => {
                if dumper.tcx.is_mir_available(*id) {
                    let body = dumper.tcx.instance_mir(instance.def).clone();
                    let instantiated = instance.instantiate_mir_and_normalize_erasing_regions(
                        dumper.tcx,
                        dumper.param_env,
                        EarlyBinder::bind(body),
                    );
                    let cfg = dumper.process_cfg(*id, &instantiated);
                    FnBody::Defined(cfg)
                } else {
                    FnBody::Skipped
                }
            }
            InstanceDef::ClosureOnceShim { call_once: id, track_caller: _ }
            | InstanceDef::DropGlue(id, _)
            | InstanceDef::CloneShim(id, _)
            | InstanceDef::FnPtrShim(id, _) => {
                let body = dumper.tcx.instance_mir(instance.def).clone();
                let instantiated = instance.instantiate_mir_and_normalize_erasing_regions(
                    dumper.tcx,
                    dumper.param_env,
                    EarlyBinder::bind(body),
                );
                let cfg = dumper.process_cfg(*id, &instantiated);
                FnBody::Bridged(cfg)
            }
            InstanceDef::Intrinsic(..) => FnBody::Intrinsic,
            // not supported
            InstanceDef::Virtual(..)
            | InstanceDef::VTableShim(..)
            | InstanceDef::ReifyShim(..)
            | InstanceDef::FnPtrAddrShim(..)
            | InstanceDef::ThreadLocalShim(..) => {
                bug!("unexpected instance type: {}", instance);
            }
        };

        if dumper.stack.pop().map_or(true, |v| v != instance) {
            bug!("unbalanced stack");
        }
        dumper.summary.push(PaflFunction { inst: inst.clone(), body });

        // mark end of processing
        if verbose {
            println!(
                "{}[<-] {}{}",
                "  ".repeat(depth),
                inst.krate.as_ref().map_or("@", |s| s.as_str()),
                inst.path
            );
        }

        // return the instantiation key
        inst
    }
}

// ==============

#[allow(dead_code)]

#[derive(/*Serialize,*/ Clone, Debug)]
// pub enum Step<'a> {
// pub enum Step {
//     B(BasicBlock),
//     // Call(&'a Trace<'a>),
//     // Call(Box<&'a Trace<'a>>),
//     Call(Box<Trace>),
//     // Call(Box<Ref)
//     // Err,
// }

pub enum Step<'a> {
    B(BasicBlock),
    // Call(&'a Trace<'a>),
    // Call(Box<&'a Trace<'a>>),
    Call(Box<*mut Trace<'a>>),
}



#[derive(Serialize, Clone, Debug)]
pub struct Trace<'a> {
    pub _entry: FnInstKey,
    pub _steps: Vec<Step<'a>>,
}

impl<'a> Serialize for Step<'a> {
    // impl<'a> Serialize for *mut Trace<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Step::Call(inner) => {
                // Serialize the "Call" variant by recursively serializing the inner Trace
                let raw_ptr = *inner.clone();
                if !raw_ptr.is_null() {
                    // Dereference the raw pointer and clone the value
                    unsafe { 
                        let a = (*raw_ptr).clone(); 
                        a.serialize(serializer) 
                    }
                } else {
                    // Handle the case where the raw pointer is null (optional)
                    // You might want to return a default value or panic depending on your use case
                    panic!("yyyjj Attempted to dereference a null pointer.")
                }
                // inner.serialize(serializer)
            }, 
            Step::B(bb) => {
                bb.serialize(serializer)
            }
        }
        // Example: serializer.serialize_some_function(*self)
    }
}


// pub struct Trace {
//     pub _entry: FnInstKey,
//     pub _steps: Vec<Step>,
// }

// impl<'a> Clone for Trace<'a> {
//     fn clone(&self) -> Self {
//         Trace {
//             _entry: self._entry,
//             _steps: self._steps.clone(),
//         }
//     }
// }


/// The central data structure of the compiler. It stores references
/// to the various **arenas** and also houses the results of the
/// various **compiler queries** that have been performed. See the
/// [rustc dev guide] for more details.
///
/// [rustc dev guide]: https://rustc-dev-guide.rust-lang.org/ty.html
///
/// An implementation detail: `TyCtxt` is a wrapper type for [GlobalCtxt],
/// which is the struct that actually holds all the data. `TyCtxt` derefs to
/// `GlobalCtxt`, and in practice `TyCtxt` is passed around everywhere, and all
/// operations are done via `TyCtxt`. A `TyCtxt` is obtained for a `GlobalCtxt`
/// by calling `enter` with a closure `f`. That function creates both the
/// `TyCtxt`, and an `ImplicitCtxt` around it that is put into TLS. Within `f`:
/// - The `ImplicitCtxt` is available implicitly via TLS.
/// - The `TyCtxt` is available explicitly via the `tcx` parameter, and also
///   implicitly within the `ImplicitCtxt`. Explicit access is preferred when
///   possible.
#[derive(Copy, Clone)]
#[rustc_diagnostic_item = "TyCtxt"]
#[rustc_pass_by_value]
pub struct TyCtxt<'tcx> {
    gcx: &'tcx GlobalCtxt<'tcx>,
}

// Explicitly implement `DynSync` and `DynSend` for `TyCtxt` to short circuit trait resolution.
#[cfg(parallel_compiler)]
unsafe impl DynSend for TyCtxt<'_> {}
#[cfg(parallel_compiler)]
unsafe impl DynSync for TyCtxt<'_> {}
fn _assert_tcx_fields() {
    sync::assert_dyn_sync::<&'_ GlobalCtxt<'_>>();
    // sync::assert_dyn_send::<&'_ GlobalCtxt<'_>>();
    sync::assert_dyn_send::<GlobalCtxt<'_>>();
}

impl<'tcx> Deref for TyCtxt<'tcx> {
    type Target = &'tcx GlobalCtxt<'tcx>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.gcx
    }
}

/// See [TyCtxt] for details about this type.
pub struct GlobalCtxt<'tcx> {
    pub arena: &'tcx WorkerLocal<Arena<'tcx>>,
    pub hir_arena: &'tcx WorkerLocal<hir::Arena<'tcx>>,

    interners: CtxtInterners<'tcx>,

    pub sess: &'tcx Session,
    crate_types: Vec<CrateType>,
    /// The `stable_crate_id` is constructed out of the crate name and all the
    /// `-C metadata` arguments passed to the compiler. Its value forms a unique
    /// global identifier for the crate. It is used to allow multiple crates
    /// with the same name to coexist. See the
    /// `rustc_symbol_mangling` crate for more information.
    stable_crate_id: StableCrateId,

    pub dep_graph: DepGraph,

    pub prof: SelfProfilerRef,

    /// Common types, pre-interned for your convenience.
    pub types: CommonTypes<'tcx>,

    /// Common lifetimes, pre-interned for your convenience.
    pub lifetimes: CommonLifetimes<'tcx>,

    /// Common consts, pre-interned for your convenience.
    pub consts: CommonConsts<'tcx>,

    /// Hooks to be able to register functions in other crates that can then still
    /// be called from rustc_middle.
    pub(crate) hooks: crate::hooks::Providers,

    untracked: Untracked,

    pub query_system: QuerySystem<'tcx>,
    pub(crate) query_kinds: &'tcx [DepKindStruct<'tcx>],

    // Internal caches for metadata decoding. No need to track deps on this.
    pub ty_rcache: Lock<FxHashMap<ty::CReaderCacheKey, Ty<'tcx>>>,
    pub pred_rcache: Lock<FxHashMap<ty::CReaderCacheKey, Predicate<'tcx>>>,

    /// Caches the results of trait selection. This cache is used
    /// for things that do not have to do with the parameters in scope.
    pub selection_cache: traits::SelectionCache<'tcx>,

    /// Caches the results of trait evaluation. This cache is used
    /// for things that do not have to do with the parameters in scope.
    /// Merge this with `selection_cache`?
    pub evaluation_cache: traits::EvaluationCache<'tcx>,

    /// Caches the results of goal evaluation in the new solver.
    pub new_solver_evaluation_cache: solve::EvaluationCache<'tcx>,
    pub new_solver_coherence_evaluation_cache: solve::EvaluationCache<'tcx>,

    pub canonical_param_env_cache: CanonicalParamEnvCache<'tcx>,

    /// Data layout specification for the current target.
    pub data_layout: TargetDataLayout,

    /// Stores memory for globals (statics/consts).
    pub(crate) alloc_map: Lock<interpret::AllocMap<'tcx>>,

    // pub _trace: RefCell<&'tcx Trace>,
    // pub _trace: RefCell<Trace<'static>>,
    pub _trace: RefCell<Trace<'tcx>>,
    // pub _curr_t: RefCell<Rc<Trace>>,
    // pub _t_idx_stk: RefCell<Vec<usize>>,
    // pub _curr_t: RefCell<Option<Vec<Step>>>,
    // pub _curr_t: RefCell<Option<&'tcx RefCell<Trace>>>,
    pub _curr_t: RefCell<Option<Box<RefCell<Trace<'tcx>>>>>,
    // pub _trace: &'tcx mut Trace,
    // pub _trace: Trace,
    // pub _ptr: RefCell<&
}
use rustc_middle::mir::Terminator;

impl<'tcx> GlobalCtxt<'tcx> {
    /// Installs `self` in a `TyCtxt` and `ImplicitCtxt` for the duration of
    /// `f`.
    pub fn enter<'a: 'tcx, F, R>(&'a self, f: F) -> R
    where
        F: FnOnce(TyCtxt<'tcx>) -> R,
    {
        let icx = tls::ImplicitCtxt::new(self);
        tls::enter_context(&icx, || f(icx.tcx))
    }

    pub fn finish(&self) -> FileEncodeResult {
        self.dep_graph.finish_encoding()
    }
}

impl<'tcx> TyCtxt<'tcx> {
    /// Expects a body and returns its codegen attributes.
    ///
    /// Unlike `codegen_fn_attrs`, this returns `CodegenFnAttrs::EMPTY` for
    /// constants.
    pub fn body_codegen_attrs(self, def_id: DefId) -> &'tcx CodegenFnAttrs {
        let def_kind = self.def_kind(def_id);
        if def_kind.has_codegen_attrs() {
            self.codegen_fn_attrs(def_id)
        } else if matches!(
            def_kind,
            DefKind::AnonConst | DefKind::AssocConst | DefKind::Const | DefKind::InlineConst
        ) {
            CodegenFnAttrs::EMPTY
        } else {
            bug!(
                "body_codegen_fn_attrs called on unexpected definition: {:?} {:?}",
                def_id,
                def_kind
            )
        }
    }

    pub fn alloc_steal_thir(self, thir: Thir<'tcx>) -> &'tcx Steal<Thir<'tcx>> {
        self.arena.alloc(Steal::new(thir))
    }

    pub fn alloc_steal_mir(self, mir: Body<'tcx>) -> &'tcx Steal<Body<'tcx>> {
        self.arena.alloc(Steal::new(mir))
    }

    pub fn alloc_steal_promoted(
        self,
        promoted: IndexVec<Promoted, Body<'tcx>>,
    ) -> &'tcx Steal<IndexVec<Promoted, Body<'tcx>>> {
        self.arena.alloc(Steal::new(promoted))
    }

    pub fn mk_adt_def(
        self,
        did: DefId,
        kind: AdtKind,
        variants: IndexVec<VariantIdx, ty::VariantDef>,
        repr: ReprOptions,
        is_anonymous: bool,
    ) -> ty::AdtDef<'tcx> {
        self.mk_adt_def_from_data(ty::AdtDefData::new(
            self,
            did,
            kind,
            variants,
            repr,
            is_anonymous,
        ))
    }

    /// Allocates a read-only byte or string literal for `mir::interpret`.
    pub fn allocate_bytes(self, bytes: &[u8]) -> interpret::AllocId {
        // Create an allocation that just contains these bytes.
        let alloc = interpret::Allocation::from_bytes_byte_aligned_immutable(bytes);
        let alloc = self.mk_const_alloc(alloc);
        self.reserve_and_set_memory_alloc(alloc)
    }

    /// Returns a range of the start/end indices specified with the
    /// `rustc_layout_scalar_valid_range` attribute.
    // FIXME(eddyb) this is an awkward spot for this method, maybe move it?
    pub fn layout_scalar_valid_range(self, def_id: DefId) -> (Bound<u128>, Bound<u128>) {
        let get = |name| {
            let Some(attr) = self.get_attr(def_id, name) else {
                return Bound::Unbounded;
            };
            debug!("layout_scalar_valid_range: attr={:?}", attr);
            if let Some(
                &[
                    ast::NestedMetaItem::Lit(ast::MetaItemLit {
                        kind: ast::LitKind::Int(a, _),
                        ..
                    }),
                ],
            ) = attr.meta_item_list().as_deref()
            {
                Bound::Included(a.get())
            } else {
                self.dcx().span_delayed_bug(
                    attr.span,
                    "invalid rustc_layout_scalar_valid_range attribute",
                );
                Bound::Unbounded
            }
        };
        (
            get(sym::rustc_layout_scalar_valid_range_start),
            get(sym::rustc_layout_scalar_valid_range_end),
        )
    }

    pub fn lift<T: Lift<'tcx>>(self, value: T) -> Option<T::Lifted> {
        value.lift_to_tcx(self)
    }

    pub fn create_fn_inst_key2(self, def: DefId, term: &Terminator<'tcx>) -> FnInstKey {

        let tcx = self.tcx();
        let param_env = self.param_env(def);
        // 1. krate
        // let krate = if def.is_local() { None } else { Some(tcx.crate_name(def.krate).to_string()) };
        let krate = Some(tcx.crate_name(def.krate).to_string());

        // 2.1. dumper ===============================================
        // let param_env: ParamEnv<'_> = self.param_env;
        let verbose = false;

        let outdir= PathBuf::from("./yjtmp/");
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

        // ================ ===============================================

        let kind = &term.kind;
        match kind {
            TerminatorKind::Call { func, args: _, destination: _, target: _, unwind: _, call_source: _, fn_span: _ } => 
            {
                // 2.2. args
                let const_ty = match func.constant() {
                    None => {
                        bug!("callee is not a constant:");
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
            },
            _ => {
                bug!("Terminator kind is not Call");
            }
        }
    }


    pub fn create_call_step(self,def: DefId, term: &Terminator<'tcx>) {
        // get mut mother trace
        // let mut fin_trace = self.tcx._trace.borrow_mut();
        let mut fin_trace = self._trace.borrow_mut();

        // cretae trace
        let entry_fn_key = self.create_fn_inst_key2(def, term);
        // let dummy_fn_inst_key = FnInstKey {
        //     krate: None,
        //     index: 100,
        //     path: String::from("modified"),
        //     generics: vec![],
        // };
        let empty_steps: Vec<Step<'_>> = vec![];
        let new_trace : Trace<'_> = Trace { _entry: entry_fn_key, _steps: empty_steps };
        let trace_ptr: *mut Trace<'_> = Box::into_raw(Box::new(new_trace));
        // let new_trace : Trace<'_> = Trace { _entry: dummy_fn_inst_key, _steps: empty_steps };

        // create step
        let s = Step::Call(Box::new(trace_ptr));
        // let s = Step::Call(Box::new(&new_trace));

        // push it to tcx.
        fin_trace._steps.push(s);
        println!("fin1=[{:?}]", fin_trace.clone());

    }

    /// Creates a type context. To use the context call `fn enter` which
    /// provides a `TyCtxt`.
    ///
    /// By only providing the `TyCtxt` inside of the closure we enforce that the type
    /// context and any interned alue (types, args, etc.) can only be used while `ty::tls`
    /// has a valid reference to the context, to allow formatting values that need it.
    pub fn create_global_ctxt(
        s: &'tcx Session,
        crate_types: Vec<CrateType>,
        stable_crate_id: StableCrateId,
        arena: &'tcx WorkerLocal<Arena<'tcx>>,
        hir_arena: &'tcx WorkerLocal<hir::Arena<'tcx>>,
        untracked: Untracked,
        dep_graph: DepGraph,
        query_kinds: &'tcx [DepKindStruct<'tcx>],
        query_system: QuerySystem<'tcx>,
        hooks: crate::hooks::Providers,
    ) -> GlobalCtxt<'tcx> {
        let data_layout = s.target.parse_data_layout().unwrap_or_else(|err| {
            s.dcx().emit_fatal(err);
        });
        let interners = CtxtInterners::new(arena);
        let common_types = CommonTypes::new(&interners, s, &untracked);
        let common_lifetimes = CommonLifetimes::new(&interners);
        let common_consts = CommonConsts::new(&interners, &common_types, s, &untracked);
        
        // let dummy_generics: Vec<PaflGeneric> = vec![];
        let steps: Vec<Step<'_>> = vec![];
        let dummy_fn_inst_key = FnInstKey {
            krate: None,
            index: 0,
            path: String::from(""),
            generics: vec![],
        };
        let fin_trace : Trace<'_> = Trace { _entry: dummy_fn_inst_key, _steps: steps.to_vec() };
        // let trace_idx_vec : Vec<usize> = vec![0];
        // let curr = Some(&)
        GlobalCtxt {
            sess: s,
            crate_types,
            stable_crate_id,
            arena,
            hir_arena,
            interners,
            dep_graph,
            hooks,
            prof: s.prof.clone(),
            types: common_types,
            lifetimes: common_lifetimes,
            consts: common_consts,
            untracked,
            query_system,
            query_kinds,
            ty_rcache: Default::default(),
            pred_rcache: Default::default(),
            selection_cache: Default::default(),
            evaluation_cache: Default::default(),
            new_solver_evaluation_cache: Default::default(),
            new_solver_coherence_evaluation_cache: Default::default(),
            canonical_param_env_cache: Default::default(),
            data_layout,
            alloc_map: Lock::new(interpret::AllocMap::new()),
            _trace: RefCell::new(fin_trace.clone()),
            // _curr_t: RefCell::new(fin_trace.clone().into()),
            // _curr_t: RefCell::new(None),
            _curr_t: RefCell::new(Some(Box::new(RefCell::new(fin_trace)))),
            // _trace: trace,
        }
    }

    pub fn consider_optimizing<T: Fn() -> String>(self, msg: T) -> bool {
        self.sess.consider_optimizing(|| self.crate_name(LOCAL_CRATE), msg)
    }

    /// Obtain all lang items of this crate and all dependencies (recursively)
    pub fn lang_items(self) -> &'tcx rustc_hir::lang_items::LanguageItems {
        self.get_lang_items(())
    }

    /// Obtain the given diagnostic item's `DefId`. Use `is_diagnostic_item` if you just want to
    /// compare against another `DefId`, since `is_diagnostic_item` is cheaper.
    pub fn get_diagnostic_item(self, name: Symbol) -> Option<DefId> {
        self.all_diagnostic_items(()).name_to_id.get(&name).copied()
    }

    /// Obtain the diagnostic item's name
    pub fn get_diagnostic_name(self, id: DefId) -> Option<Symbol> {
        self.diagnostic_items(id.krate).id_to_name.get(&id).copied()
    }

    /// Check whether the diagnostic item with the given `name` has the given `DefId`.
    pub fn is_diagnostic_item(self, name: Symbol, did: DefId) -> bool {
        self.diagnostic_items(did.krate).name_to_id.get(&name) == Some(&did)
    }

    pub fn is_coroutine(self, def_id: DefId) -> bool {
        self.coroutine_kind(def_id).is_some()
    }

    /// Returns the movability of the coroutine of `def_id`, or panics
    /// if given a `def_id` that is not a coroutine.
    pub fn coroutine_movability(self, def_id: DefId) -> hir::Movability {
        self.coroutine_kind(def_id).expect("expected a coroutine").movability()
    }

    /// Returns `true` if the node pointed to by `def_id` is a coroutine for an async construct.
    pub fn coroutine_is_async(self, def_id: DefId) -> bool {
        matches!(
            self.coroutine_kind(def_id),
            Some(hir::CoroutineKind::Desugared(hir::CoroutineDesugaring::Async, _))
        )
    }

    /// Returns `true` if the node pointed to by `def_id` is a general coroutine that implements `Coroutine`.
    /// This means it is neither an `async` or `gen` construct.
    pub fn is_general_coroutine(self, def_id: DefId) -> bool {
        matches!(self.coroutine_kind(def_id), Some(hir::CoroutineKind::Coroutine(_)))
    }

    /// Returns `true` if the node pointed to by `def_id` is a coroutine for a `gen` construct.
    pub fn coroutine_is_gen(self, def_id: DefId) -> bool {
        matches!(
            self.coroutine_kind(def_id),
            Some(hir::CoroutineKind::Desugared(hir::CoroutineDesugaring::Gen, _))
        )
    }

    /// Returns `true` if the node pointed to by `def_id` is a coroutine for a `async gen` construct.
    pub fn coroutine_is_async_gen(self, def_id: DefId) -> bool {
        matches!(
            self.coroutine_kind(def_id),
            Some(hir::CoroutineKind::Desugared(hir::CoroutineDesugaring::AsyncGen, _))
        )
    }

    pub fn stability(self) -> &'tcx stability::Index {
        self.stability_index(())
    }

    pub fn features(self) -> &'tcx rustc_feature::Features {
        self.features_query(())
    }

    pub fn def_key(self, id: impl IntoQueryParam<DefId>) -> rustc_hir::definitions::DefKey {
        let id = id.into_query_param();
        // Accessing the DefKey is ok, since it is part of DefPathHash.
        if let Some(id) = id.as_local() {
            self.definitions_untracked().def_key(id)
        } else {
            self.cstore_untracked().def_key(id)
        }
    }

    /// Converts a `DefId` into its fully expanded `DefPath` (every
    /// `DefId` is really just an interned `DefPath`).
    ///
    /// Note that if `id` is not local to this crate, the result will
    ///  be a non-local `DefPath`.
    pub fn def_path(self, id: DefId) -> rustc_hir::definitions::DefPath {
        // Accessing the DefPath is ok, since it is part of DefPathHash.
        if let Some(id) = id.as_local() {
            self.definitions_untracked().def_path(id)
        } else {
            self.cstore_untracked().def_path(id)
        }
    }

    #[inline]
    pub fn def_path_hash(self, def_id: DefId) -> rustc_hir::definitions::DefPathHash {
        // Accessing the DefPathHash is ok, it is incr. comp. stable.
        if let Some(def_id) = def_id.as_local() {
            self.definitions_untracked().def_path_hash(def_id)
        } else {
            self.cstore_untracked().def_path_hash(def_id)
        }
    }

    #[inline]
    pub fn crate_types(self) -> &'tcx [CrateType] {
        &self.crate_types
    }

    pub fn metadata_kind(self) -> MetadataKind {
        self.crate_types()
            .iter()
            .map(|ty| match *ty {
                CrateType::Executable | CrateType::Staticlib | CrateType::Cdylib => {
                    MetadataKind::None
                }
                CrateType::Rlib => MetadataKind::Uncompressed,
                CrateType::Dylib | CrateType::ProcMacro => MetadataKind::Compressed,
            })
            .max()
            .unwrap_or(MetadataKind::None)
    }

    pub fn needs_metadata(self) -> bool {
        self.metadata_kind() != MetadataKind::None
    }

    pub fn needs_crate_hash(self) -> bool {
        // Why is the crate hash needed for these configurations?
        // - debug_assertions: for the "fingerprint the result" check in
        //   `rustc_query_system::query::plumbing::execute_job`.
        // - incremental: for query lookups.
        // - needs_metadata: for putting into crate metadata.
        // - instrument_coverage: for putting into coverage data (see
        //   `hash_mir_source`).
        cfg!(debug_assertions)
            || self.sess.opts.incremental.is_some()
            || self.needs_metadata()
            || self.sess.instrument_coverage()
    }

    #[inline]
    pub fn stable_crate_id(self, crate_num: CrateNum) -> StableCrateId {
        if crate_num == LOCAL_CRATE {
            self.stable_crate_id
        } else {
            self.cstore_untracked().stable_crate_id(crate_num)
        }
    }

    /// Maps a StableCrateId to the corresponding CrateNum. This method assumes
    /// that the crate in question has already been loaded by the CrateStore.
    #[inline]
    pub fn stable_crate_id_to_crate_num(self, stable_crate_id: StableCrateId) -> CrateNum {
        if stable_crate_id == self.stable_crate_id(LOCAL_CRATE) {
            LOCAL_CRATE
        } else {
            self.cstore_untracked().stable_crate_id_to_crate_num(stable_crate_id)
        }
    }

    /// Converts a `DefPathHash` to its corresponding `DefId` in the current compilation
    /// session, if it still exists. This is used during incremental compilation to
    /// turn a deserialized `DefPathHash` into its current `DefId`.
    pub fn def_path_hash_to_def_id(self, hash: DefPathHash, err: &mut dyn FnMut() -> !) -> DefId {
        debug!("def_path_hash_to_def_id({:?})", hash);

        let stable_crate_id = hash.stable_crate_id();

        // If this is a DefPathHash from the local crate, we can look up the
        // DefId in the tcx's `Definitions`.
        if stable_crate_id == self.stable_crate_id(LOCAL_CRATE) {
            self.untracked.definitions.read().local_def_path_hash_to_def_id(hash, err).to_def_id()
        } else {
            // If this is a DefPathHash from an upstream crate, let the CrateStore map
            // it to a DefId.
            let cstore = &*self.cstore_untracked();
            let cnum = cstore.stable_crate_id_to_crate_num(stable_crate_id);
            cstore.def_path_hash_to_def_id(cnum, hash)
        }
    }

    pub fn def_path_debug_str(self, def_id: DefId) -> String {
        // We are explicitly not going through queries here in order to get
        // crate name and stable crate id since this code is called from debug!()
        // statements within the query system and we'd run into endless
        // recursion otherwise.
        let (crate_name, stable_crate_id) = if def_id.is_local() {
            (self.crate_name(LOCAL_CRATE), self.stable_crate_id(LOCAL_CRATE))
        } else {
            let cstore = &*self.cstore_untracked();
            (cstore.crate_name(def_id.krate), cstore.stable_crate_id(def_id.krate))
        };

        format!(
            "{}[{:04x}]{}",
            crate_name,
            // Don't print the whole stable crate id. That's just
            // annoying in debug output.
            stable_crate_id.as_u64() >> (8 * 6),
            self.def_path(def_id).to_string_no_crate_verbose()
        )
    }

    pub fn dcx(self) -> &'tcx DiagCtxt {
        self.sess.dcx()
    }
}

impl<'tcx> TyCtxtAt<'tcx> {
    /// Create a new definition within the incr. comp. engine.
    pub fn create_def(
        self,
        parent: LocalDefId,
        name: Symbol,
        def_kind: DefKind,
    ) -> TyCtxtFeed<'tcx, LocalDefId> {
        let feed = self.tcx.create_def(parent, name, def_kind);

        feed.def_span(self.span);
        feed
    }
}

impl<'tcx> TyCtxt<'tcx> {
    /// `tcx`-dependent operations performed for every created definition.
    pub fn create_def(
        self,
        parent: LocalDefId,
        name: Symbol,
        def_kind: DefKind,
    ) -> TyCtxtFeed<'tcx, LocalDefId> {
        let data = def_kind.def_path_data(name);
        // The following call has the side effect of modifying the tables inside `definitions`.
        // These very tables are relied on by the incr. comp. engine to decode DepNodes and to
        // decode the on-disk cache.
        //
        // Any LocalDefId which is used within queries, either as key or result, either:
        // - has been created before the construction of the TyCtxt;
        // - has been created by this call to `create_def`.
        // As a consequence, this LocalDefId is always re-created before it is needed by the incr.
        // comp. engine itself.
        //
        // This call also writes to the value of `source_span` and `expn_that_defined` queries.
        // This is fine because:
        // - those queries are `eval_always` so we won't miss their result changing;
        // - this write will have happened before these queries are called.
        let def_id = self.untracked.definitions.write().create_def(parent, data);

        // This function modifies `self.definitions` using a side-effect.
        // We need to ensure that these side effects are re-run by the incr. comp. engine.
        // Depending on the forever-red node will tell the graph that the calling query
        // needs to be re-evaluated.
        self.dep_graph.read_index(DepNodeIndex::FOREVER_RED_NODE);

        let feed = TyCtxtFeed { tcx: self, key: def_id };
        feed.def_kind(def_kind);
        // Unique types created for closures participate in type privacy checking.
        // They have visibilities inherited from the module they are defined in.
        // Visibilities for opaque types are meaningless, but still provided
        // so that all items have visibilities.
        if matches!(def_kind, DefKind::Closure | DefKind::OpaqueTy) {
            let parent_mod = self.parent_module_from_def_id(def_id).to_def_id();
            feed.visibility(ty::Visibility::Restricted(parent_mod));
        }

        feed
    }

    pub fn iter_local_def_id(self) -> impl Iterator<Item = LocalDefId> + 'tcx {
        // Create a dependency to the red node to be sure we re-execute this when the amount of
        // definitions change.
        self.dep_graph.read_index(DepNodeIndex::FOREVER_RED_NODE);

        let definitions = &self.untracked.definitions;
        std::iter::from_coroutine(|| {
            let mut i = 0;

            // Recompute the number of definitions each time, because our caller may be creating
            // new ones.
            while i < { definitions.read().num_definitions() } {
                let local_def_index = rustc_span::def_id::DefIndex::from_usize(i);
                yield LocalDefId { local_def_index };
                i += 1;
            }

            // Freeze definitions once we finish iterating on them, to prevent adding new ones.
            definitions.freeze();
        })
    }

    pub fn def_path_table(self) -> &'tcx rustc_hir::definitions::DefPathTable {
        // Create a dependency to the crate to be sure we re-execute this when the amount of
        // definitions change.
        self.dep_graph.read_index(DepNodeIndex::FOREVER_RED_NODE);

        // Freeze definitions once we start iterating on them, to prevent adding new ones
        // while iterating. If some query needs to add definitions, it should be `ensure`d above.
        self.untracked.definitions.freeze().def_path_table()
    }

    pub fn def_path_hash_to_def_index_map(
        self,
    ) -> &'tcx rustc_hir::def_path_hash_map::DefPathHashMap {
        // Create a dependency to the crate to be sure we re-execute this when the amount of
        // definitions change.
        self.ensure().hir_crate(());
        // Freeze definitions once we start iterating on them, to prevent adding new ones
        // while iterating. If some query needs to add definitions, it should be `ensure`d above.
        self.untracked.definitions.freeze().def_path_hash_to_def_index_map()
    }

    /// Note that this is *untracked* and should only be used within the query
    /// system if the result is otherwise tracked through queries
    #[inline]
    pub fn cstore_untracked(self) -> FreezeReadGuard<'tcx, CrateStoreDyn> {
        FreezeReadGuard::map(self.untracked.cstore.read(), |c| &**c)
    }

    /// Give out access to the untracked data without any sanity checks.
    pub fn untracked(self) -> &'tcx Untracked {
        &self.untracked
    }
    /// Note that this is *untracked* and should only be used within the query
    /// system if the result is otherwise tracked through queries
    #[inline]
    pub fn definitions_untracked(self) -> FreezeReadGuard<'tcx, Definitions> {
        self.untracked.definitions.read()
    }

    /// Note that this is *untracked* and should only be used within the query
    /// system if the result is otherwise tracked through queries
    #[inline]
    pub fn source_span_untracked(self, def_id: LocalDefId) -> Span {
        self.untracked.source_span.get(def_id).unwrap_or(DUMMY_SP)
    }

    #[inline(always)]
    pub fn with_stable_hashing_context<R>(
        self,
        f: impl FnOnce(StableHashingContext<'_>) -> R,
    ) -> R {
        f(StableHashingContext::new(self.sess, &self.untracked))
    }

    pub fn serialize_query_result_cache(self, encoder: FileEncoder) -> FileEncodeResult {
        self.query_system.on_disk_cache.as_ref().map_or(Ok(0), |c| c.serialize(self, encoder))
    }

    #[inline]
    pub fn local_crate_exports_generics(self) -> bool {
        debug_assert!(self.sess.opts.share_generics());

        self.crate_types().iter().any(|crate_type| {
            match crate_type {
                CrateType::Executable
                | CrateType::Staticlib
                | CrateType::ProcMacro
                | CrateType::Cdylib => false,

                // FIXME rust-lang/rust#64319, rust-lang/rust#64872:
                // We want to block export of generics from dylibs,
                // but we must fix rust-lang/rust#65890 before we can
                // do that robustly.
                CrateType::Dylib => true,

                CrateType::Rlib => true,
            }
        })
    }

    /// Returns the `DefId` and the `BoundRegionKind` corresponding to the given region.
    pub fn is_suitable_region(self, mut region: Region<'tcx>) -> Option<FreeRegionInfo> {
        let (suitable_region_binding_scope, bound_region) = loop {
            let def_id = match region.kind() {
                ty::ReLateParam(fr) => fr.bound_region.get_id()?.as_local()?,
                ty::ReEarlyParam(ebr) => ebr.def_id.as_local()?,
                _ => return None, // not a free region
            };
            let scope = self.local_parent(def_id);
            if self.def_kind(scope) == DefKind::OpaqueTy {
                // Lifetime params of opaque types are synthetic and thus irrelevant to
                // diagnostics. Map them back to their origin!
                region = self.map_opaque_lifetime_to_parent_lifetime(def_id);
                continue;
            }
            break (scope, ty::BrNamed(def_id.into(), self.item_name(def_id.into())));
        };

        let is_impl_item = match self.hir_node_by_def_id(suitable_region_binding_scope) {
            Node::Item(..) | Node::TraitItem(..) => false,
            Node::ImplItem(..) => self.is_bound_region_in_impl_item(suitable_region_binding_scope),
            _ => false,
        };

        Some(FreeRegionInfo { def_id: suitable_region_binding_scope, bound_region, is_impl_item })
    }

    /// Given a `DefId` for an `fn`, return all the `dyn` and `impl` traits in its return type.
    pub fn return_type_impl_or_dyn_traits(
        self,
        scope_def_id: LocalDefId,
    ) -> Vec<&'tcx hir::Ty<'tcx>> {
        let hir_id = self.local_def_id_to_hir_id(scope_def_id);
        let Some(hir::FnDecl { output: hir::FnRetTy::Return(hir_output), .. }) =
            self.hir().fn_decl_by_hir_id(hir_id)
        else {
            return vec![];
        };

        let mut v = TraitObjectVisitor(vec![], self.hir());
        v.visit_ty(hir_output);
        v.0
    }

    /// Given a `DefId` for an `fn`, return all the `dyn` and `impl` traits in
    /// its return type, and the associated alias span when type alias is used,
    /// along with a span for lifetime suggestion (if there are existing generics).
    pub fn return_type_impl_or_dyn_traits_with_type_alias(
        self,
        scope_def_id: LocalDefId,
    ) -> Option<(Vec<&'tcx hir::Ty<'tcx>>, Span, Option<Span>)> {
        let hir_id = self.local_def_id_to_hir_id(scope_def_id);
        let mut v = TraitObjectVisitor(vec![], self.hir());
        // when the return type is a type alias
        if let Some(hir::FnDecl { output: hir::FnRetTy::Return(hir_output), .. }) = self.hir().fn_decl_by_hir_id(hir_id)
            && let hir::TyKind::Path(hir::QPath::Resolved(
                None,
                hir::Path { res: hir::def::Res::Def(DefKind::TyAlias, def_id), .. }, )) = hir_output.kind
            && let Some(local_id) = def_id.as_local()
            && let Some(alias_ty) = self.hir_node_by_def_id(local_id).alias_ty() // it is type alias
            && let Some(alias_generics) = self.hir_node_by_def_id(local_id).generics()
        {
            v.visit_ty(alias_ty);
            if !v.0.is_empty() {
                return Some((
                    v.0,
                    alias_generics.span,
                    alias_generics.span_for_lifetime_suggestion(),
                ));
            }
        }
        return None;
    }

    /// Checks if the bound region is in Impl Item.
    pub fn is_bound_region_in_impl_item(self, suitable_region_binding_scope: LocalDefId) -> bool {
        let container_id = self.parent(suitable_region_binding_scope.to_def_id());
        if self.impl_trait_ref(container_id).is_some() {
            // For now, we do not try to target impls of traits. This is
            // because this message is going to suggest that the user
            // change the fn signature, but they may not be free to do so,
            // since the signature must match the trait.
            //
            // FIXME(#42706) -- in some cases, we could do better here.
            return true;
        }
        false
    }

    /// Determines whether identifiers in the assembly have strict naming rules.
    /// Currently, only NVPTX* targets need it.
    pub fn has_strict_asm_symbol_naming(self) -> bool {
        self.sess.target.arch.contains("nvptx")
    }

    /// Returns `&'static core::panic::Location<'static>`.
    pub fn caller_location_ty(self) -> Ty<'tcx> {
        Ty::new_imm_ref(
            self,
            self.lifetimes.re_static,
            self.type_of(self.require_lang_item(LangItem::PanicLocation, None))
                .instantiate(self, self.mk_args(&[self.lifetimes.re_static.into()])),
        )
    }

    /// Returns a displayable description and article for the given `def_id` (e.g. `("a", "struct")`).
    pub fn article_and_description(self, def_id: DefId) -> (&'static str, &'static str) {
        let kind = self.def_kind(def_id);
        (self.def_kind_descr_article(kind, def_id), self.def_kind_descr(kind, def_id))
    }

    pub fn type_length_limit(self) -> Limit {
        self.limits(()).type_length_limit
    }

    pub fn recursion_limit(self) -> Limit {
        self.limits(()).recursion_limit
    }

    pub fn move_size_limit(self) -> Limit {
        self.limits(()).move_size_limit
    }

    pub fn all_traits(self) -> impl Iterator<Item = DefId> + 'tcx {
        iter::once(LOCAL_CRATE)
            .chain(self.crates(()).iter().copied())
            .flat_map(move |cnum| self.traits(cnum).iter().copied())
    }

    #[inline]
    pub fn local_visibility(self, def_id: LocalDefId) -> Visibility {
        self.visibility(def_id).expect_local()
    }

    /// Returns the origin of the opaque type `def_id`.
    #[instrument(skip(self), level = "trace", ret)]
    pub fn opaque_type_origin(self, def_id: LocalDefId) -> hir::OpaqueTyOrigin {
        self.hir().expect_item(def_id).expect_opaque_ty().origin
    }
}

/// A trait implemented for all `X<'a>` types that can be safely and
/// efficiently converted to `X<'tcx>` as long as they are part of the
/// provided `TyCtxt<'tcx>`.
/// This can be done, for example, for `Ty<'tcx>` or `GenericArgsRef<'tcx>`
/// by looking them up in their respective interners.
///
/// However, this is still not the best implementation as it does
/// need to compare the components, even for interned values.
/// It would be more efficient if `TypedArena` provided a way to
/// determine whether the address is in the allocated range.
///
/// `None` is returned if the value or one of the components is not part
/// of the provided context.
/// For `Ty`, `None` can be returned if either the type interner doesn't
/// contain the `TyKind` key or if the address of the interned
/// pointer differs. The latter case is possible if a primitive type,
/// e.g., `()` or `u8`, was interned in a different context.
pub trait Lift<'tcx>: fmt::Debug {
    type Lifted: fmt::Debug + 'tcx;
    fn lift_to_tcx(self, tcx: TyCtxt<'tcx>) -> Option<Self::Lifted>;
}

macro_rules! nop_lift {
    ($set:ident; $ty:ty => $lifted:ty) => {
        impl<'a, 'tcx> Lift<'tcx> for $ty {
            type Lifted = $lifted;
            fn lift_to_tcx(self, tcx: TyCtxt<'tcx>) -> Option<Self::Lifted> {
                // Assert that the set has the right type.
                // Given an argument that has an interned type, the return type has the type of
                // the corresponding interner set. This won't actually return anything, we're
                // just doing this to compute said type!
                fn _intern_set_ty_from_interned_ty<'tcx, Inner>(
                    _x: Interned<'tcx, Inner>,
                ) -> InternedSet<'tcx, Inner> {
                    unreachable!()
                }
                fn _type_eq<T>(_x: &T, _y: &T) {}
                fn _test<'tcx>(x: $lifted, tcx: TyCtxt<'tcx>) {
                    // If `x` is a newtype around an `Interned<T>`, then `interner` is an
                    // interner of appropriate type. (Ideally we'd also check that `x` is a
                    // newtype with just that one field. Not sure how to do that.)
                    let interner = _intern_set_ty_from_interned_ty(x.0);
                    // Now check that this is the same type as `interners.$set`.
                    _type_eq(&interner, &tcx.interners.$set);
                }

                tcx.interners
                    .$set
                    .contains_pointer_to(&InternedInSet(&*self.0.0))
                    // SAFETY: `self` is interned and therefore valid
                    // for the entire lifetime of the `TyCtxt`.
                    .then(|| unsafe { mem::transmute(self) })
            }
        }
    };
}

macro_rules! nop_list_lift {
    ($set:ident; $ty:ty => $lifted:ty) => {
        impl<'a, 'tcx> Lift<'tcx> for &'a List<$ty> {
            type Lifted = &'tcx List<$lifted>;
            fn lift_to_tcx(self, tcx: TyCtxt<'tcx>) -> Option<Self::Lifted> {
                // Assert that the set has the right type.
                if false {
                    let _x: &InternedSet<'tcx, List<$lifted>> = &tcx.interners.$set;
                }

                if self.is_empty() {
                    return Some(List::empty());
                }
                tcx.interners
                    .$set
                    .contains_pointer_to(&InternedInSet(self))
                    .then(|| unsafe { mem::transmute(self) })
            }
        }
    };
}

nop_lift! {type_; Ty<'a> => Ty<'tcx>}
nop_lift! {region; Region<'a> => Region<'tcx>}
nop_lift! {const_; Const<'a> => Const<'tcx>}
nop_lift! {const_allocation; ConstAllocation<'a> => ConstAllocation<'tcx>}
nop_lift! {predicate; Predicate<'a> => Predicate<'tcx>}
nop_lift! {predicate; Clause<'a> => Clause<'tcx>}
nop_lift! {layout; Layout<'a> => Layout<'tcx>}

nop_list_lift! {type_lists; Ty<'a> => Ty<'tcx>}
nop_list_lift! {poly_existential_predicates; PolyExistentialPredicate<'a> => PolyExistentialPredicate<'tcx>}
nop_list_lift! {bound_variable_kinds; ty::BoundVariableKind => ty::BoundVariableKind}

// This is the impl for `&'a GenericArgs<'a>`.
nop_list_lift! {args; GenericArg<'a> => GenericArg<'tcx>}

macro_rules! nop_slice_lift {
    ($ty:ty => $lifted:ty) => {
        impl<'a, 'tcx> Lift<'tcx> for &'a [$ty] {
            type Lifted = &'tcx [$lifted];
            fn lift_to_tcx(self, tcx: TyCtxt<'tcx>) -> Option<Self::Lifted> {
                if self.is_empty() {
                    return Some(&[]);
                }
                tcx.interners
                    .arena
                    .dropless
                    .contains_slice(self)
                    .then(|| unsafe { mem::transmute(self) })
            }
        }
    };
}

nop_slice_lift! {ty::ValTree<'a> => ty::ValTree<'tcx>}

TrivialLiftImpls! {
    ImplPolarity, PredicatePolarity, Promoted
}

macro_rules! sty_debug_print {
    ($fmt: expr, $ctxt: expr, $($variant: ident),*) => {{
        // Curious inner module to allow variant names to be used as
        // variable names.
        #[allow(non_snake_case)]
        mod inner {
            use crate::ty::{self, TyCtxt};
            use crate::ty::context::InternedInSet;

            #[derive(Copy, Clone)]
            struct DebugStat {
                total: usize,
                lt_infer: usize,
                ty_infer: usize,
                ct_infer: usize,
                all_infer: usize,
            }

            pub fn go(fmt: &mut std::fmt::Formatter<'_>, tcx: TyCtxt<'_>) -> std::fmt::Result {
                let mut total = DebugStat {
                    total: 0,
                    lt_infer: 0,
                    ty_infer: 0,
                    ct_infer: 0,
                    all_infer: 0,
                };
                $(let mut $variant = total;)*

                for shard in tcx.interners.type_.lock_shards() {
                    let types = shard.keys();
                    for &InternedInSet(t) in types {
                        let variant = match t.internee {
                            ty::Bool | ty::Char | ty::Int(..) | ty::Uint(..) |
                                ty::Float(..) | ty::Str | ty::Never => continue,
                            ty::Error(_) => /* unimportant */ continue,
                            $(ty::$variant(..) => &mut $variant,)*
                        };
                        let lt = t.flags.intersects(ty::TypeFlags::HAS_RE_INFER);
                        let ty = t.flags.intersects(ty::TypeFlags::HAS_TY_INFER);
                        let ct = t.flags.intersects(ty::TypeFlags::HAS_CT_INFER);

                        variant.total += 1;
                        total.total += 1;
                        if lt { total.lt_infer += 1; variant.lt_infer += 1 }
                        if ty { total.ty_infer += 1; variant.ty_infer += 1 }
                        if ct { total.ct_infer += 1; variant.ct_infer += 1 }
                        if lt && ty && ct { total.all_infer += 1; variant.all_infer += 1 }
                    }
                }
                writeln!(fmt, "Ty interner             total           ty lt ct all")?;
                $(writeln!(fmt, "    {:18}: {uses:6} {usespc:4.1}%, \
                            {ty:4.1}% {lt:5.1}% {ct:4.1}% {all:4.1}%",
                    stringify!($variant),
                    uses = $variant.total,
                    usespc = $variant.total as f64 * 100.0 / total.total as f64,
                    ty = $variant.ty_infer as f64 * 100.0  / total.total as f64,
                    lt = $variant.lt_infer as f64 * 100.0  / total.total as f64,
                    ct = $variant.ct_infer as f64 * 100.0  / total.total as f64,
                    all = $variant.all_infer as f64 * 100.0  / total.total as f64)?;
                )*
                writeln!(fmt, "                  total {uses:6}        \
                          {ty:4.1}% {lt:5.1}% {ct:4.1}% {all:4.1}%",
                    uses = total.total,
                    ty = total.ty_infer as f64 * 100.0  / total.total as f64,
                    lt = total.lt_infer as f64 * 100.0  / total.total as f64,
                    ct = total.ct_infer as f64 * 100.0  / total.total as f64,
                    all = total.all_infer as f64 * 100.0  / total.total as f64)
            }
        }

        inner::go($fmt, $ctxt)
    }}
}

impl<'tcx> TyCtxt<'tcx> {
    pub fn debug_stats(self) -> impl std::fmt::Debug + 'tcx {
        struct DebugStats<'tcx>(TyCtxt<'tcx>);

        impl<'tcx> std::fmt::Debug for DebugStats<'tcx> {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                sty_debug_print!(
                    fmt,
                    self.0,
                    Adt,
                    Array,
                    Slice,
                    RawPtr,
                    Ref,
                    FnDef,
                    FnPtr,
                    Placeholder,
                    Coroutine,
                    CoroutineWitness,
                    Dynamic,
                    Closure,
                    CoroutineClosure,
                    Tuple,
                    Bound,
                    Param,
                    Infer,
                    Alias,
                    Foreign
                )?;

                writeln!(fmt, "GenericArgs interner: #{}", self.0.interners.args.len())?;
                writeln!(fmt, "Region interner: #{}", self.0.interners.region.len())?;
                writeln!(
                    fmt,
                    "Const Allocation interner: #{}",
                    self.0.interners.const_allocation.len()
                )?;
                writeln!(fmt, "Layout interner: #{}", self.0.interners.layout.len())?;

                Ok(())
            }
        }

        DebugStats(self)
    }
}

// This type holds a `T` in the interner. The `T` is stored in the arena and
// this type just holds a pointer to it, but it still effectively owns it. It
// impls `Borrow` so that it can be looked up using the original
// (non-arena-memory-owning) types.
struct InternedInSet<'tcx, T: ?Sized>(&'tcx T);

impl<'tcx, T: 'tcx + ?Sized> Clone for InternedInSet<'tcx, T> {
    fn clone(&self) -> Self {
        InternedInSet(self.0)
    }
}

impl<'tcx, T: 'tcx + ?Sized> Copy for InternedInSet<'tcx, T> {}

impl<'tcx, T: 'tcx + ?Sized> IntoPointer for InternedInSet<'tcx, T> {
    fn into_pointer(&self) -> *const () {
        self.0 as *const _ as *const ()
    }
}

#[allow(rustc::usage_of_ty_tykind)]
impl<'tcx, T> Borrow<T> for InternedInSet<'tcx, WithCachedTypeInfo<T>> {
    fn borrow(&self) -> &T {
        &self.0.internee
    }
}

impl<'tcx, T: PartialEq> PartialEq for InternedInSet<'tcx, WithCachedTypeInfo<T>> {
    fn eq(&self, other: &InternedInSet<'tcx, WithCachedTypeInfo<T>>) -> bool {
        // The `Borrow` trait requires that `x.borrow() == y.borrow()` equals
        // `x == y`.
        self.0.internee == other.0.internee
    }
}

impl<'tcx, T: Eq> Eq for InternedInSet<'tcx, WithCachedTypeInfo<T>> {}

impl<'tcx, T: Hash> Hash for InternedInSet<'tcx, WithCachedTypeInfo<T>> {
    fn hash<H: Hasher>(&self, s: &mut H) {
        // The `Borrow` trait requires that `x.borrow().hash(s) == x.hash(s)`.
        self.0.internee.hash(s)
    }
}

impl<'tcx, T> Borrow<[T]> for InternedInSet<'tcx, List<T>> {
    fn borrow(&self) -> &[T] {
        &self.0[..]
    }
}

impl<'tcx, T: PartialEq> PartialEq for InternedInSet<'tcx, List<T>> {
    fn eq(&self, other: &InternedInSet<'tcx, List<T>>) -> bool {
        // The `Borrow` trait requires that `x.borrow() == y.borrow()` equals
        // `x == y`.
        self.0[..] == other.0[..]
    }
}

impl<'tcx, T: Eq> Eq for InternedInSet<'tcx, List<T>> {}

impl<'tcx, T: Hash> Hash for InternedInSet<'tcx, List<T>> {
    fn hash<H: Hasher>(&self, s: &mut H) {
        // The `Borrow` trait requires that `x.borrow().hash(s) == x.hash(s)`.
        self.0[..].hash(s)
    }
}

macro_rules! direct_interners {
    ($($name:ident: $vis:vis $method:ident($ty:ty): $ret_ctor:ident -> $ret_ty:ty,)+) => {
        $(impl<'tcx> Borrow<$ty> for InternedInSet<'tcx, $ty> {
            fn borrow<'a>(&'a self) -> &'a $ty {
                &self.0
            }
        }

        impl<'tcx> PartialEq for InternedInSet<'tcx, $ty> {
            fn eq(&self, other: &Self) -> bool {
                // The `Borrow` trait requires that `x.borrow() == y.borrow()`
                // equals `x == y`.
                self.0 == other.0
            }
        }

        impl<'tcx> Eq for InternedInSet<'tcx, $ty> {}

        impl<'tcx> Hash for InternedInSet<'tcx, $ty> {
            fn hash<H: Hasher>(&self, s: &mut H) {
                // The `Borrow` trait requires that `x.borrow().hash(s) ==
                // x.hash(s)`.
                self.0.hash(s)
            }
        }

        impl<'tcx> TyCtxt<'tcx> {
            $vis fn $method(self, v: $ty) -> $ret_ty {
                $ret_ctor(Interned::new_unchecked(self.interners.$name.intern(v, |v| {
                    InternedInSet(self.interners.arena.alloc(v))
                }).0))
            }
        })+
    }
}

// Functions with a `mk_` prefix are intended for use outside this file and
// crate. Functions with an `intern_` prefix are intended for use within this
// crate only, and have a corresponding `mk_` function.
direct_interners! {
    region: pub(crate) intern_region(RegionKind<'tcx>): Region -> Region<'tcx>,
    const_allocation: pub mk_const_alloc(Allocation): ConstAllocation -> ConstAllocation<'tcx>,
    layout: pub mk_layout(LayoutS<FieldIdx, VariantIdx>): Layout -> Layout<'tcx>,
    adt_def: pub mk_adt_def_from_data(AdtDefData): AdtDef -> AdtDef<'tcx>,
    external_constraints: pub mk_external_constraints(ExternalConstraintsData<'tcx>):
        ExternalConstraints -> ExternalConstraints<'tcx>,
    predefined_opaques_in_body: pub mk_predefined_opaques_in_body(PredefinedOpaquesData<'tcx>):
        PredefinedOpaques -> PredefinedOpaques<'tcx>,
}

macro_rules! slice_interners {
    ($($field:ident: $vis:vis $method:ident($ty:ty)),+ $(,)?) => (
        impl<'tcx> TyCtxt<'tcx> {
            $($vis fn $method(self, v: &[$ty]) -> &'tcx List<$ty> {
                if v.is_empty() {
                    List::empty()
                } else {
                    self.interners.$field.intern_ref(v, || {
                        InternedInSet(List::from_arena(&*self.arena, v))
                    }).0
                }
            })+
        }
    );
}

// These functions intern slices. They all have a corresponding
// `mk_foo_from_iter` function that interns an iterator. The slice version
// should be used when possible, because it's faster.
slice_interners!(
    const_lists: pub mk_const_list(Const<'tcx>),
    args: pub mk_args(GenericArg<'tcx>),
    type_lists: pub mk_type_list(Ty<'tcx>),
    canonical_var_infos: pub mk_canonical_var_infos(CanonicalVarInfo<'tcx>),
    poly_existential_predicates: intern_poly_existential_predicates(PolyExistentialPredicate<'tcx>),
    clauses: intern_clauses(Clause<'tcx>),
    projs: pub mk_projs(ProjectionKind),
    place_elems: pub mk_place_elems(PlaceElem<'tcx>),
    bound_variable_kinds: pub mk_bound_variable_kinds(ty::BoundVariableKind),
    fields: pub mk_fields(FieldIdx),
    local_def_ids: intern_local_def_ids(LocalDefId),
    offset_of: pub mk_offset_of((VariantIdx, FieldIdx)),
);

impl<'tcx> TyCtxt<'tcx> {
    /// Given a `fn` type, returns an equivalent `unsafe fn` type;
    /// that is, a `fn` type that is equivalent in every way for being
    /// unsafe.
    pub fn safe_to_unsafe_fn_ty(self, sig: PolyFnSig<'tcx>) -> Ty<'tcx> {
        assert_eq!(sig.unsafety(), hir::Unsafety::Normal);
        Ty::new_fn_ptr(
            self,
            sig.map_bound(|sig| ty::FnSig { unsafety: hir::Unsafety::Unsafe, ..sig }),
        )
    }

    /// Given the def_id of a Trait `trait_def_id` and the name of an associated item `assoc_name`
    /// returns true if the `trait_def_id` defines an associated item of name `assoc_name`.
    pub fn trait_may_define_assoc_item(self, trait_def_id: DefId, assoc_name: Ident) -> bool {
        self.super_traits_of(trait_def_id).any(|trait_did| {
            self.associated_items(trait_did)
                .filter_by_name_unhygienic(assoc_name.name)
                .any(|item| self.hygienic_eq(assoc_name, item.ident(self), trait_did))
        })
    }

    /// Given a `ty`, return whether it's an `impl Future<...>`.
    pub fn ty_is_opaque_future(self, ty: Ty<'_>) -> bool {
        let ty::Alias(ty::Opaque, ty::AliasTy { def_id, .. }) = ty.kind() else { return false };
        let future_trait = self.require_lang_item(LangItem::Future, None);

        self.explicit_item_super_predicates(def_id).skip_binder().iter().any(|&(predicate, _)| {
            let ty::ClauseKind::Trait(trait_predicate) = predicate.kind().skip_binder() else {
                return false;
            };
            trait_predicate.trait_ref.def_id == future_trait
                && trait_predicate.polarity == PredicatePolarity::Positive
        })
    }

    /// Computes the def-ids of the transitive supertraits of `trait_def_id`. This (intentionally)
    /// does not compute the full elaborated super-predicates but just the set of def-ids. It is used
    /// to identify which traits may define a given associated type to help avoid cycle errors.
    /// Returns a `DefId` iterator.
    fn super_traits_of(self, trait_def_id: DefId) -> impl Iterator<Item = DefId> + 'tcx {
        let mut set = FxHashSet::default();
        let mut stack = vec![trait_def_id];

        set.insert(trait_def_id);

        iter::from_fn(move || -> Option<DefId> {
            let trait_did = stack.pop()?;
            let generic_predicates = self.super_predicates_of(trait_did);

            for (predicate, _) in generic_predicates.predicates {
                if let ty::ClauseKind::Trait(data) = predicate.kind().skip_binder() {
                    if set.insert(data.def_id()) {
                        stack.push(data.def_id());
                    }
                }
            }

            Some(trait_did)
        })
    }

    /// Given a closure signature, returns an equivalent fn signature. Detuples
    /// and so forth -- so e.g., if we have a sig with `Fn<(u32, i32)>` then
    /// you would get a `fn(u32, i32)`.
    /// `unsafety` determines the unsafety of the fn signature. If you pass
    /// `hir::Unsafety::Unsafe` in the previous example, then you would get
    /// an `unsafe fn (u32, i32)`.
    /// It cannot convert a closure that requires unsafe.
    pub fn signature_unclosure(
        self,
        sig: PolyFnSig<'tcx>,
        unsafety: hir::Unsafety,
    ) -> PolyFnSig<'tcx> {
        sig.map_bound(|s| {
            let params = match s.inputs()[0].kind() {
                ty::Tuple(params) => *params,
                _ => bug!(),
            };
            self.mk_fn_sig(params, s.output(), s.c_variadic, unsafety, abi::Abi::Rust)
        })
    }

    #[inline]
    pub fn mk_predicate(self, binder: Binder<'tcx, PredicateKind<'tcx>>) -> Predicate<'tcx> {
        self.interners.intern_predicate(
            binder,
            self.sess,
            // This is only used to create a stable hashing context.
            &self.untracked,
        )
    }

    #[inline]
    pub fn reuse_or_mk_predicate(
        self,
        pred: Predicate<'tcx>,
        binder: Binder<'tcx, PredicateKind<'tcx>>,
    ) -> Predicate<'tcx> {
        if pred.kind() != binder { self.mk_predicate(binder) } else { pred }
    }

    #[inline(always)]
    pub(crate) fn check_and_mk_args(
        self,
        _def_id: DefId,
        args: impl IntoIterator<Item: Into<GenericArg<'tcx>>>,
    ) -> GenericArgsRef<'tcx> {
        let args = args.into_iter().map(Into::into);
        #[cfg(debug_assertions)]
        {
            let generics = self.generics_of(_def_id);

            let n = if let DefKind::AssocTy = self.def_kind(_def_id)
                && let DefKind::Impl { of_trait: false } = self.def_kind(self.parent(_def_id))
            {
                // If this is an inherent projection.
                generics.params.len() + 1
            } else {
                generics.count()
            };
            assert_eq!(
                (n, Some(n)),
                args.size_hint(),
                "wrong number of generic parameters for {_def_id:?}: {:?}",
                args.collect::<Vec<_>>(),
            );
        }
        self.mk_args_from_iter(args)
    }

    #[inline]
    pub fn mk_ct_from_kind(self, kind: ty::ConstKind<'tcx>, ty: Ty<'tcx>) -> Const<'tcx> {
        self.interners.intern_const(
            ty::ConstData { kind, ty },
            self.sess,
            // This is only used to create a stable hashing context.
            &self.untracked,
        )
    }

    // Avoid this in favour of more specific `Ty::new_*` methods, where possible.
    #[allow(rustc::usage_of_ty_tykind)]
    #[inline]
    pub fn mk_ty_from_kind(self, st: TyKind<'tcx>) -> Ty<'tcx> {
        self.interners.intern_ty(
            st,
            self.sess,
            // This is only used to create a stable hashing context.
            &self.untracked,
        )
    }

    pub fn mk_param_from_def(self, param: &ty::GenericParamDef) -> GenericArg<'tcx> {
        match param.kind {
            GenericParamDefKind::Lifetime => {
                ty::Region::new_early_param(self, param.to_early_bound_region_data()).into()
            }
            GenericParamDefKind::Type { .. } => Ty::new_param(self, param.index, param.name).into(),
            GenericParamDefKind::Const { .. } => ty::Const::new_param(
                self,
                ParamConst { index: param.index, name: param.name },
                self.type_of(param.def_id)
                    .no_bound_vars()
                    .expect("const parameter types cannot be generic"),
            )
            .into(),
        }
    }

    pub fn mk_place_field(self, place: Place<'tcx>, f: FieldIdx, ty: Ty<'tcx>) -> Place<'tcx> {
        self.mk_place_elem(place, PlaceElem::Field(f, ty))
    }

    pub fn mk_place_deref(self, place: Place<'tcx>) -> Place<'tcx> {
        self.mk_place_elem(place, PlaceElem::Deref)
    }

    pub fn mk_place_downcast(
        self,
        place: Place<'tcx>,
        adt_def: AdtDef<'tcx>,
        variant_index: VariantIdx,
    ) -> Place<'tcx> {
        self.mk_place_elem(
            place,
            PlaceElem::Downcast(Some(adt_def.variant(variant_index).name), variant_index),
        )
    }

    pub fn mk_place_downcast_unnamed(
        self,
        place: Place<'tcx>,
        variant_index: VariantIdx,
    ) -> Place<'tcx> {
        self.mk_place_elem(place, PlaceElem::Downcast(None, variant_index))
    }

    pub fn mk_place_index(self, place: Place<'tcx>, index: Local) -> Place<'tcx> {
        self.mk_place_elem(place, PlaceElem::Index(index))
    }

    /// This method copies `Place`'s projection, add an element and reintern it. Should not be used
    /// to build a full `Place` it's just a convenient way to grab a projection and modify it in
    /// flight.
    pub fn mk_place_elem(self, place: Place<'tcx>, elem: PlaceElem<'tcx>) -> Place<'tcx> {
        let mut projection = place.projection.to_vec();
        projection.push(elem);

        Place { local: place.local, projection: self.mk_place_elems(&projection) }
    }

    pub fn mk_poly_existential_predicates(
        self,
        eps: &[PolyExistentialPredicate<'tcx>],
    ) -> &'tcx List<PolyExistentialPredicate<'tcx>> {
        assert!(!eps.is_empty());
        assert!(
            eps.array_windows()
                .all(|[a, b]| a.skip_binder().stable_cmp(self, &b.skip_binder())
                    != Ordering::Greater)
        );
        self.intern_poly_existential_predicates(eps)
    }

    pub fn mk_clauses(self, clauses: &[Clause<'tcx>]) -> &'tcx List<Clause<'tcx>> {
        // FIXME consider asking the input slice to be sorted to avoid
        // re-interning permutations, in which case that would be asserted
        // here.
        self.intern_clauses(clauses)
    }

    pub fn mk_local_def_ids(self, clauses: &[LocalDefId]) -> &'tcx List<LocalDefId> {
        // FIXME consider asking the input slice to be sorted to avoid
        // re-interning permutations, in which case that would be asserted
        // here.
        self.intern_local_def_ids(clauses)
    }

    pub fn mk_local_def_ids_from_iter<I, T>(self, iter: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<LocalDefId, &'tcx List<LocalDefId>>,
    {
        T::collect_and_apply(iter, |xs| self.mk_local_def_ids(xs))
    }

    pub fn mk_const_list_from_iter<I, T>(self, iter: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<ty::Const<'tcx>, &'tcx List<ty::Const<'tcx>>>,
    {
        T::collect_and_apply(iter, |xs| self.mk_const_list(xs))
    }

    // Unlike various other `mk_*_from_iter` functions, this one uses `I:
    // IntoIterator` instead of `I: Iterator`, and it doesn't have a slice
    // variant, because of the need to combine `inputs` and `output`. This
    // explains the lack of `_from_iter` suffix.
    pub fn mk_fn_sig<I, T>(
        self,
        inputs: I,
        output: I::Item,
        c_variadic: bool,
        unsafety: hir::Unsafety,
        abi: abi::Abi,
    ) -> T::Output
    where
        I: IntoIterator<Item = T>,
        T: CollectAndApply<Ty<'tcx>, ty::FnSig<'tcx>>,
    {
        T::collect_and_apply(inputs.into_iter().chain(iter::once(output)), |xs| ty::FnSig {
            inputs_and_output: self.mk_type_list(xs),
            c_variadic,
            unsafety,
            abi,
        })
    }

    pub fn mk_poly_existential_predicates_from_iter<I, T>(self, iter: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<
                PolyExistentialPredicate<'tcx>,
                &'tcx List<PolyExistentialPredicate<'tcx>>,
            >,
    {
        T::collect_and_apply(iter, |xs| self.mk_poly_existential_predicates(xs))
    }

    pub fn mk_clauses_from_iter<I, T>(self, iter: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<Clause<'tcx>, &'tcx List<Clause<'tcx>>>,
    {
        T::collect_and_apply(iter, |xs| self.mk_clauses(xs))
    }

    pub fn mk_type_list_from_iter<I, T>(self, iter: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<Ty<'tcx>, &'tcx List<Ty<'tcx>>>,
    {
        T::collect_and_apply(iter, |xs| self.mk_type_list(xs))
    }

    pub fn mk_args_from_iter<I, T>(self, iter: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<GenericArg<'tcx>, &'tcx List<GenericArg<'tcx>>>,
    {
        T::collect_and_apply(iter, |xs| self.mk_args(xs))
    }

    pub fn mk_canonical_var_infos_from_iter<I, T>(self, iter: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<CanonicalVarInfo<'tcx>, &'tcx List<CanonicalVarInfo<'tcx>>>,
    {
        T::collect_and_apply(iter, |xs| self.mk_canonical_var_infos(xs))
    }

    pub fn mk_place_elems_from_iter<I, T>(self, iter: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<PlaceElem<'tcx>, &'tcx List<PlaceElem<'tcx>>>,
    {
        T::collect_and_apply(iter, |xs| self.mk_place_elems(xs))
    }

    pub fn mk_fields_from_iter<I, T>(self, iter: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<FieldIdx, &'tcx List<FieldIdx>>,
    {
        T::collect_and_apply(iter, |xs| self.mk_fields(xs))
    }

    pub fn mk_offset_of_from_iter<I, T>(self, iter: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<(VariantIdx, FieldIdx), &'tcx List<(VariantIdx, FieldIdx)>>,
    {
        T::collect_and_apply(iter, |xs| self.mk_offset_of(xs))
    }

    pub fn mk_args_trait(
        self,
        self_ty: Ty<'tcx>,
        rest: impl IntoIterator<Item = GenericArg<'tcx>>,
    ) -> GenericArgsRef<'tcx> {
        self.mk_args_from_iter(iter::once(self_ty.into()).chain(rest))
    }

    pub fn mk_bound_variable_kinds_from_iter<I, T>(self, iter: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<ty::BoundVariableKind, &'tcx List<ty::BoundVariableKind>>,
    {
        T::collect_and_apply(iter, |xs| self.mk_bound_variable_kinds(xs))
    }

    /// Emit a lint at `span` from a lint struct (some type that implements `LintDiagnostic`,
    /// typically generated by `#[derive(LintDiagnostic)]`).
    #[track_caller]
    pub fn emit_node_span_lint(
        self,
        lint: &'static Lint,
        hir_id: HirId,
        span: impl Into<MultiSpan>,
        decorator: impl for<'a> LintDiagnostic<'a, ()>,
    ) {
        let msg = decorator.msg();
        let (level, src) = self.lint_level_at_node(lint, hir_id);
        lint_level(self.sess, lint, level, src, Some(span.into()), msg, |diag| {
            decorator.decorate_lint(diag);
        })
    }

    /// Emit a lint at the appropriate level for a hir node, with an associated span.
    ///
    /// [`lint_level`]: rustc_middle::lint::lint_level#decorate-signature
    #[rustc_lint_diagnostics]
    #[track_caller]
    pub fn node_span_lint(
        self,
        lint: &'static Lint,
        hir_id: HirId,
        span: impl Into<MultiSpan>,
        msg: impl Into<DiagMessage>,
        decorate: impl for<'a, 'b> FnOnce(&'b mut Diag<'a, ()>),
    ) {
        let (level, src) = self.lint_level_at_node(lint, hir_id);
        lint_level(self.sess, lint, level, src, Some(span.into()), msg, decorate);
    }

    /// Find the crate root and the appropriate span where `use` and outer attributes can be
    /// inserted at.
    pub fn crate_level_attribute_injection_span(self, hir_id: HirId) -> Option<Span> {
        for (_hir_id, node) in self.hir().parent_iter(hir_id) {
            if let hir::Node::Crate(m) = node {
                return Some(m.spans.inject_use_span.shrink_to_lo());
            }
        }
        None
    }

    pub fn disabled_nightly_features<E: rustc_errors::EmissionGuarantee>(
        self,
        diag: &mut Diag<'_, E>,
        hir_id: Option<HirId>,
        features: impl IntoIterator<Item = (String, Symbol)>,
    ) {
        if !self.sess.is_nightly_build() {
            return;
        }

        let span = hir_id.and_then(|id| self.crate_level_attribute_injection_span(id));
        for (desc, feature) in features {
            // FIXME: make this string translatable
            let msg =
                format!("add `#![feature({feature})]` to the crate attributes to enable{desc}");
            if let Some(span) = span {
                diag.span_suggestion_verbose(
                    span,
                    msg,
                    format!("#![feature({feature})]\n"),
                    Applicability::MachineApplicable,
                );
            } else {
                diag.help(msg);
            }
        }
    }

    /// Emit a lint from a lint struct (some type that implements `LintDiagnostic`, typically
    /// generated by `#[derive(LintDiagnostic)]`).
    #[track_caller]
    pub fn emit_node_lint(
        self,
        lint: &'static Lint,
        id: HirId,
        decorator: impl for<'a> LintDiagnostic<'a, ()>,
    ) {
        self.node_lint(lint, id, decorator.msg(), |diag| {
            decorator.decorate_lint(diag);
        })
    }

    /// Emit a lint at the appropriate level for a hir node.
    ///
    /// [`lint_level`]: rustc_middle::lint::lint_level#decorate-signature
    #[rustc_lint_diagnostics]
    #[track_caller]
    pub fn node_lint(
        self,
        lint: &'static Lint,
        id: HirId,
        msg: impl Into<DiagMessage>,
        decorate: impl for<'a, 'b> FnOnce(&'b mut Diag<'a, ()>),
    ) {
        let (level, src) = self.lint_level_at_node(lint, id);
        lint_level(self.sess, lint, level, src, None, msg, decorate);
    }

    pub fn in_scope_traits(self, id: HirId) -> Option<&'tcx [TraitCandidate]> {
        let map = self.in_scope_traits_map(id.owner)?;
        let candidates = map.get(&id.local_id)?;
        Some(candidates)
    }

    pub fn named_bound_var(self, id: HirId) -> Option<resolve_bound_vars::ResolvedArg> {
        debug!(?id, "named_region");
        self.named_variable_map(id.owner).and_then(|map| map.get(&id.local_id).cloned())
    }

    pub fn is_late_bound(self, id: HirId) -> bool {
        self.is_late_bound_map(id.owner).is_some_and(|set| set.contains(&id.local_id))
    }

    pub fn late_bound_vars(self, id: HirId) -> &'tcx List<ty::BoundVariableKind> {
        self.mk_bound_variable_kinds(
            &self
                .late_bound_vars_map(id.owner)
                .and_then(|map| map.get(&id.local_id).cloned())
                .unwrap_or_else(|| {
                    bug!("No bound vars found for {}", self.hir().node_to_string(id))
                }),
        )
    }

    /// Given the def-id of an early-bound lifetime on an opaque corresponding to
    /// a duplicated captured lifetime, map it back to the early- or late-bound
    /// lifetime of the function from which it originally as captured. If it is
    /// a late-bound lifetime, this will represent the liberated (`ReLateParam`) lifetime
    /// of the signature.
    // FIXME(RPITIT): if we ever synthesize new lifetimes for RPITITs and not just
    // re-use the generics of the opaque, this function will need to be tweaked slightly.
    pub fn map_opaque_lifetime_to_parent_lifetime(
        self,
        mut opaque_lifetime_param_def_id: LocalDefId,
    ) -> ty::Region<'tcx> {
        debug_assert!(
            matches!(self.def_kind(opaque_lifetime_param_def_id), DefKind::LifetimeParam),
            "{opaque_lifetime_param_def_id:?} is a {}",
            self.def_descr(opaque_lifetime_param_def_id.to_def_id())
        );

        loop {
            let parent = self.local_parent(opaque_lifetime_param_def_id);
            let hir::OpaqueTy { lifetime_mapping, .. } =
                self.hir_node_by_def_id(parent).expect_item().expect_opaque_ty();

            let Some((lifetime, _)) = lifetime_mapping
                .iter()
                .find(|(_, duplicated_param)| *duplicated_param == opaque_lifetime_param_def_id)
            else {
                bug!("duplicated lifetime param should be present");
            };

            match self.named_bound_var(lifetime.hir_id) {
                Some(resolve_bound_vars::ResolvedArg::EarlyBound(ebv)) => {
                    let new_parent = self.parent(ebv);

                    // If we map to another opaque, then it should be a parent
                    // of the opaque we mapped from. Continue mapping.
                    if matches!(self.def_kind(new_parent), DefKind::OpaqueTy) {
                        debug_assert_eq!(self.parent(parent.to_def_id()), new_parent);
                        opaque_lifetime_param_def_id = ebv.expect_local();
                        continue;
                    }

                    let generics = self.generics_of(new_parent);
                    return ty::Region::new_early_param(
                        self,
                        ty::EarlyParamRegion {
                            def_id: ebv,
                            index: generics
                                .param_def_id_to_index(self, ebv)
                                .expect("early-bound var should be present in fn generics"),
                            name: self.hir().name(self.local_def_id_to_hir_id(ebv.expect_local())),
                        },
                    );
                }
                Some(resolve_bound_vars::ResolvedArg::LateBound(_, _, lbv)) => {
                    let new_parent = self.parent(lbv);
                    return ty::Region::new_late_param(
                        self,
                        new_parent,
                        ty::BoundRegionKind::BrNamed(
                            lbv,
                            self.hir().name(self.local_def_id_to_hir_id(lbv.expect_local())),
                        ),
                    );
                }
                Some(resolve_bound_vars::ResolvedArg::Error(guar)) => {
                    return ty::Region::new_error(self, guar);
                }
                _ => {
                    return ty::Region::new_error_with_message(
                        self,
                        lifetime.ident.span,
                        "cannot resolve lifetime",
                    );
                }
            }
        }
    }

    /// Whether the `def_id` counts as const fn in the current crate, considering all active
    /// feature gates
    pub fn is_const_fn(self, def_id: DefId) -> bool {
        if self.is_const_fn_raw(def_id) {
            match self.lookup_const_stability(def_id) {
                Some(stability) if stability.is_const_unstable() => {
                    // has a `rustc_const_unstable` attribute, check whether the user enabled the
                    // corresponding feature gate.
                    self.features()
                        .declared_lib_features
                        .iter()
                        .any(|&(sym, _)| sym == stability.feature)
                }
                // functions without const stability are either stable user written
                // const fn or the user is using feature gates and we thus don't
                // care what they do
                _ => true,
            }
        } else {
            false
        }
    }

    /// Whether the trait impl is marked const. This does not consider stability or feature gates.
    pub fn is_const_trait_impl_raw(self, def_id: DefId) -> bool {
        let Some(local_def_id) = def_id.as_local() else { return false };
        let node = self.hir_node_by_def_id(local_def_id);

        matches!(
            node,
            hir::Node::Item(hir::Item {
                kind: hir::ItemKind::Impl(hir::Impl { generics, .. }),
                ..
            }) if generics.params.iter().any(|p| matches!(p.kind, hir::GenericParamKind::Const { is_host_effect: true, .. }))
        )
    }

    pub fn intrinsic(self, def_id: impl IntoQueryParam<DefId> + Copy) -> Option<ty::IntrinsicDef> {
        match self.def_kind(def_id) {
            DefKind::Fn | DefKind::AssocFn => {}
            _ => return None,
        }
        self.intrinsic_raw(def_id)
    }

    pub fn next_trait_solver_globally(self) -> bool {
        self.sess.opts.unstable_opts.next_solver.map_or(false, |c| c.globally)
    }

    pub fn next_trait_solver_in_coherence(self) -> bool {
        self.sess.opts.unstable_opts.next_solver.map_or(false, |c| c.coherence)
    }

    pub fn is_impl_trait_in_trait(self, def_id: DefId) -> bool {
        self.opt_rpitit_info(def_id).is_some()
    }

    /// Named module children from all kinds of items, including imports.
    /// In addition to regular items this list also includes struct and variant constructors, and
    /// items inside `extern {}` blocks because all of them introduce names into parent module.
    ///
    /// Module here is understood in name resolution sense - it can be a `mod` item,
    /// or a crate root, or an enum, or a trait.
    ///
    /// This is not a query, making it a query causes perf regressions
    /// (probably due to hashing spans in `ModChild`ren).
    pub fn module_children_local(self, def_id: LocalDefId) -> &'tcx [ModChild] {
        self.resolutions(()).module_children.get(&def_id).map_or(&[], |v| &v[..])
    }

    pub fn resolver_for_lowering(self) -> &'tcx Steal<(ty::ResolverAstLowering, Lrc<ast::Crate>)> {
        self.resolver_for_lowering_raw(()).0
    }

    /// Given an `impl_id`, return the trait it implements.
    /// Return `None` if this is an inherent impl.
    pub fn impl_trait_ref(
        self,
        def_id: impl IntoQueryParam<DefId>,
    ) -> Option<ty::EarlyBinder<ty::TraitRef<'tcx>>> {
        Some(self.impl_trait_header(def_id)?.trait_ref)
    }

    pub fn impl_polarity(self, def_id: impl IntoQueryParam<DefId>) -> ty::ImplPolarity {
        self.impl_trait_header(def_id).map_or(ty::ImplPolarity::Positive, |h| h.polarity)
    }
}

/// Parameter attributes that can only be determined by examining the body of a function instead
/// of just its signature.
///
/// These can be useful for optimization purposes when a function is directly called. We compute
/// them and store them into the crate metadata so that downstream crates can make use of them.
///
/// Right now, we only have `read_only`, but `no_capture` and `no_alias` might be useful in the
/// future.
#[derive(Clone, Copy, PartialEq, Debug, Default, TyDecodable, TyEncodable, HashStable)]
pub struct DeducedParamAttrs {
    /// The parameter is marked immutable in the function and contains no `UnsafeCell` (i.e. its
    /// type is freeze).
    pub read_only: bool,
}

pub fn provide(providers: &mut Providers) {
    providers.maybe_unused_trait_imports =
        |tcx, ()| &tcx.resolutions(()).maybe_unused_trait_imports;
    providers.names_imported_by_glob_use = |tcx, id| {
        tcx.arena.alloc(UnordSet::from(
            tcx.resolutions(()).glob_map.get(&id).cloned().unwrap_or_default(),
        ))
    };

    providers.extern_mod_stmt_cnum =
        |tcx, id| tcx.resolutions(()).extern_crate_map.get(&id).cloned();
    providers.is_panic_runtime =
        |tcx, LocalCrate| attr::contains_name(tcx.hir().krate_attrs(), sym::panic_runtime);
    providers.is_compiler_builtins =
        |tcx, LocalCrate| attr::contains_name(tcx.hir().krate_attrs(), sym::compiler_builtins);
    providers.has_panic_handler = |tcx, LocalCrate| {
        // We want to check if the panic handler was defined in this crate
        tcx.lang_items().panic_impl().is_some_and(|did| did.is_local())
    };
    providers.source_span = |tcx, def_id| tcx.untracked.source_span.get(def_id).unwrap_or(DUMMY_SP);
}
