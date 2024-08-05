

use super::{InterpCx, Machine};

use std::io::Write;
use std::fs::OpenOptions;
use byteorder::{LittleEndian, WriteBytesExt};

use rustc_hir::def_id::DefId;
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_middle::mir::BasicBlock;
use rustc_middle::ty::{self, InstanceKind, ParamEnv, Instance};
use rustc_middle::ty::layout::HasTyCtxt;
use rustc_middle::ty::fuzz_static_dump::{PaflCrate, PaflDump, FnInstKey, Trace, Step};

impl<'tcx, M: Machine<'tcx>> InterpCx<'tcx, M> {
    // called by BB(x)
    pub fn push_bb(&mut self, bb: BasicBlock) {
        if !self.trace_enabled {
            return
        }
        let trace = self.trace_stack.last_mut().unwrap();
        trace.steps.push(Step::B(bb.as_usize()));
    }

    // called by Call
    pub fn push_trace(&mut self, instance: &Instance<'tcx>) {
        if !self.trace_enabled {
            return
        }
        let fnkey: FnInstKey = self.instance_to_key(instance);
        self.trace_stack.push(Trace {
            entry: fnkey,
            steps: Vec::new(),
        });
    }

    // called by Return
    pub fn merge_trace(&mut self) {
        if !self.trace_enabled {
            return
        }
        let trace_top = self.trace_stack.pop().unwrap();
        let stk_len = self.trace_stack.len();
        if stk_len == 0 {
            println!("WARNING: call stack exceeded!");
            self.trace_stack.push(trace_top);
        } else {
            let trace = self.trace_stack.last_mut().unwrap();
            trace.steps.push(Step::Call(trace_top));
        }
    }

    fn instance_to_key(&self, instance: &Instance<'tcx>) -> FnInstKey {
        // prep for PaflDump object
        let tcx = self.tcx();
        let verbose = false;
        let mut cache: FxHashMap<Instance<'tcx>, FnInstKey> = FxHashMap::default();
        let mut stack = vec![];
        // ty::ParamEnv::reveal_all()
        let param_env = ParamEnv::reveal_all();
        let mut summary = PaflCrate { functions: Vec::new() };
    
        if let Some(cached) = cache.get(instance) {
            return cached.clone();
        }

        let dumper = PaflDump {
            tcx,
            param_env,
            verbose,
            stack: &mut stack,
            cache: &mut cache,
            summary: &mut summary.functions,
        };

        let inst_def: ty::InstanceKind<'_> = instance.def;
        let id : DefId = match inst_def {
            InstanceKind::Item(def)
            | InstanceKind::Intrinsic(def)
            | InstanceKind::VTableShim(def)
            | InstanceKind::ReifyShim(def, _)
            | InstanceKind::FnPtrShim(def, _)
            | InstanceKind::Virtual(def, _)
            | InstanceKind::ThreadLocalShim(def) 
            | InstanceKind::DropGlue(def, _)
            | InstanceKind::CloneShim(def, _)
            | InstanceKind::FnPtrAddrShim(def, _) => { def },
            InstanceKind::ClosureOnceShim { call_once, .. } => { call_once }, 
            InstanceKind::ConstructCoroutineInClosureShim { coroutine_closure_def_id, .. } => { coroutine_closure_def_id },
            InstanceKind::CoroutineKindShim { coroutine_def_id, .. } => { coroutine_def_id },
            InstanceKind::AsyncDropGlueCtorShim(def, _) => { def },
        };

        let inst = dumper.resolve_fn_key(id, instance.args);
        inst
    }

    #[allow(rustc::potential_query_instability)]
    pub fn runtime_dump(&mut self, file_path: &str) {
        let trace = self.trace_stack.last().unwrap();
        let size = self.trace_stack.len();
        println!("[RUSTC] size of trace stack {}, file_path {}", size, file_path);
        // assert_eq!(size, 1);
        for i in 0..size {
            println!("on stack: {:?}", self.trace_stack[i].entry);
        }
        // if trace._steps.len() > 0 {
        //     println!("after miri2 {:?}", trace._steps.last().unwrap());
        // } else {
        //     println!("empty trace");
        // };

        let mut st = Vec::new();
        let mut id_set = FxHashSet::default();
        st.push(trace);
        while let Some(cur) = st.pop() {
            // when can this fail?
            let entry = serde_cbor::ser::to_vec_packed(&cur.entry).unwrap();
            id_set.insert(entry);
            for step in cur.steps.iter() {
                if let Step::Call(nxt) = step {
                    st.push(nxt);
                }
            }
        }
        let mut id_vec: Vec<Vec<u8>> = id_set.drain().collect();
        id_vec.sort();

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .expect("unable to create output file");

        file.write_u32::<LittleEndian>(id_vec.len() as u32).unwrap();
        for buf in id_vec.iter() {
            file.write_u16::<LittleEndian>(buf.len() as u16).unwrap();
            file.write_all(buf).unwrap();
        }

        let id_map: FxHashMap<_, _> = id_vec.into_iter().enumerate().map(|(a, b)| (b, a)).collect();

        let mut st = Vec::new();
        file.write_u8(1).unwrap();
        st.push((trace, 0));
        'outer: while let Some((cur, mut idx)) = st.pop() {
            while idx < cur.steps.len() {
                let step = &cur.steps[idx];
                idx += 1;
                match step {
                    Step::B(bb) => {
                        file.write_u8(2).unwrap();
                        file.write_u24::<LittleEndian>(*bb as u32).unwrap();
                    },
                    Step::Call(nxt) => {
                        st.push((cur, idx));
                        file.write_u8(1).unwrap();
                        st.push((nxt, 0));
                        continue 'outer;
                    }
                }
            }
            if idx == cur.steps.len() {
                let entry = serde_cbor::ser::to_vec_packed(&cur.entry).unwrap();
                let id = id_map[&entry];
                file.write_u8(3).unwrap();
                file.write_u24::<LittleEndian>(id as u32).unwrap();
            }
        }      
    }
}