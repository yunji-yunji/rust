use super::{InterpCx, Machine};
use rustc_middle::mir::BasicBlock;
use rustc_middle::ty::context::FnInstKey;
use rustc_middle::ty::context::{Step, Trace};
use rustc_middle::ty::print::with_no_trimmed_paths;

// use crate::interpret::dump;
use std::fs::OpenOptions;
use std::io::Write;

impl<'mir, 'tcx: 'mir, M: Machine<'mir, 'tcx>> InterpCx<'mir, 'tcx, M> {
    pub fn crate_info(
        &mut self,
        // _terminator: &mir::Terminator<'tcx>,
    ) -> String {

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

    pub fn inst_to_info(&mut self, key: FnInstKey) -> String {
        let res: String;
        let krate_name = match key.krate {
            Some(val) => val,
            None => {return String::from("no crate");}
        };

        let path = key.path;
        res = krate_name + &path;

        res
    }

    // push for tcx._bb_seq
    pub fn push_bb(&mut self, s: String) {
        let mut tmp_vec: std::cell::RefMut<'_, Vec<String>> = self.tcx._bb_seq.borrow_mut();
        tmp_vec.push(s);
    }

    pub fn call_stk_push(&mut self, s: String) {
        let mut vec_str: std::cell::RefMut<'_, Vec<String>> = self.tcx._call_stack.borrow_mut();
        vec_str.push(s);
    }
    pub fn call_stk_pop(&mut self,) {
        let mut vec_str: std::cell::RefMut<'_, Vec<String>> = self.tcx._call_stack.borrow_mut();
        vec_str.pop();
    }
    pub fn set_skip_true(&mut self,) {
        let mut skip: std::cell::RefMut<'_, bool> = self.tcx._ret_can_skip.borrow_mut();
        *skip = true;
    }
    pub fn set_skip_false(&mut self,) {
        let mut skip: std::cell::RefMut<'_, bool> = self.tcx._ret_can_skip.borrow_mut();
        *skip = false;
    }

    // called by Call
    pub fn update_fn_key(&mut self, fn_key: FnInstKey) {
        let mut tmp_trace = self.tcx._tmp_trace.borrow_mut();
        tmp_trace._entry = fn_key;
    }

    // called by BB
    pub fn push_step_bb(&mut self, bb: BasicBlock) {
        let mut steps= self.tcx._tmp_steps.borrow_mut();
        steps.push(Step::B(bb));
    }

    // new) called by call
    pub fn push_trace_stack1(&mut self, fn_key: FnInstKey) {
        // println!("call {:?}", fn_key);
        let can_skip = fn_key.can_skip();
        if self._skip_counter == 0 {
            self._trace_stack.push(Trace {_entry: fn_key, _steps: Vec::new()});
            // let info = self.inst_to_info(fn_key);
            // self.push_to_ecx("[Call<term>".to_string());
            // self.push_to_ecx(info);
        };
        if can_skip {
            println!("???");
            self._skip_counter += 1;
        };
    }

    // new) called by return
    pub fn merge_trace_stack1(&mut self/* , info: String*/) {
        // can't be empty, unless return unmatched with call
        if self._trace_stack.last().unwrap()._entry.can_skip() {
            self._skip_counter -= 1;
        };
        if self._skip_counter == 0 {
            // self.push_to_ecx(info);
            // self.push_to_ecx(String::from("Ret]"));
            let trace = self._trace_stack.pop().unwrap();
            // println!("return {:?}", trace._entry);
            if !trace._entry.can_skip() {
                let l = self._trace_stack.len();
                if l == 0 {
                    // println!("WARNING: call stack exceeded!");
                    self._trace_stack.push(trace);
                } else {
                    self._trace_stack.last_mut().unwrap()._steps.push(Step::Call(trace));
                };
            };
        };
    }

    // new) called by BB(X)
    pub fn push_bb_stack1(&mut self, bb: BasicBlock) {
        if self._skip_counter == 0 {
            self._trace_stack.last_mut().unwrap()._steps.push(Step::B(bb));
        };
    }

    // test: env var DUMP_FIN_TRACE
    pub fn dump_fin_trace(&mut self, file_name: &str) {
        let t = self.tcx._trace.borrow();
        let content =
            serde_json::to_string_pretty(&*t).expect("unexpected failure on JSON encoding");

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_name)
            .expect("unable to create output file");
        file.write_all(content.as_bytes()).expect("unexpected failure on outputting to file");
    }

    // test: env var DUMP_TMP_TRACE
    pub fn dump_tmp_trace(&mut self, dump_path: &str) {
        let t = self.tcx._tmp_trace.borrow();
        let content =
            serde_json::to_string_pretty(&*t).expect("unexpected failure on JSON encoding");

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(dump_path)
            .expect("unable to create output file");
        file.write_all(content.as_bytes()).expect("unexpected failure on outputting to file");
    }

    // pair calls and returns
    pub fn push_to_ecx(&mut self, s: String) {
        let mut tmp_vec: std::cell::RefMut<'_, Vec<String>> = self.call_return_vec.borrow_mut();
        tmp_vec.push(s);
    }
    pub fn keep_call_push(&mut self, fn_inst: FnInstKey) {
        let mut keep_call_vec: std::cell::RefMut<'_, Vec<FnInstKey>> = self.keep_call.borrow_mut();
        keep_call_vec.push(fn_inst);
    }
    pub fn keep_call_pop(&mut self,) {
        let mut keep_call_vec: std::cell::RefMut<'_, Vec<FnInstKey>> = self.keep_call.borrow_mut();
        keep_call_vec.pop();
    }
    pub fn keep_call_cond(&mut self, caller_inst: FnInstKey) -> bool {
        match self.keep_call.borrow().last() {
            None => {
                // println!("[F1] nothing to keep caller={:?}/{:?}", caller_inst.clone().krate, caller_inst.clone().path);
                false
            },
            Some(key) => {
                if key.path == caller_inst.path {
                    true
                } else {
                    // println!("[F2] not same key to keep={:?} caller={:?}", key, caller_inst);
                    false
                }
            }
        }
    }

}
