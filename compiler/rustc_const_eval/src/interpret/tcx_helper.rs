use super::{InterpCx, Machine};
use rustc_middle::mir::BasicBlock;
use rustc_middle::ty::context::FnInstKey;
use rustc_middle::ty::context::{Step, Trace};

// use crate::interpret::dump;
use std::fs::OpenOptions;
use std::io::Write;

impl<'mir, 'tcx: 'mir, M: Machine<'mir, 'tcx>> InterpCx<'mir, 'tcx, M> {

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
    
    // called by Return
    pub fn push_step_call(&mut self,) {
        let mut tmp_trace = self.tcx._tmp_trace.borrow_mut();
        let mut tmp_steps= self.tcx._tmp_steps.borrow_mut();
        tmp_trace._steps = tmp_steps.to_vec();
        *tmp_steps = vec![];

        let mut final_trace= self.tcx._trace.borrow_mut();
        final_trace._steps.push(Step::Call(tmp_trace.clone()));
    }

    // new) called by call
    pub fn push_trace_stack1(&mut self, fn_key: FnInstKey) {
        // println!("call {:?}", fn_key);
        let skip = *self.tcx._skip_counter.borrow();
        let can_skip = fn_key.can_skip();
        if skip == 0 {
            self.tcx._trace_stack.borrow_mut().push(Trace {_entry: fn_key, _steps: Vec::new()});
        };
        if can_skip {
            self.tcx._skip_counter.replace(skip + 1);
        };
    }

    // new) called by return
    pub fn merge_trace_stack1(&mut self, ) {
        // can't be empty, unless return unmatched with call
        let mut skip = *self.tcx._skip_counter.borrow();
        if self.tcx._trace_stack.borrow().last().unwrap()._entry.can_skip() {
            skip -= 1;
            self.tcx._skip_counter.replace(skip);
        };
        if skip == 0 {
            let trace = self.tcx._trace_stack.borrow_mut().pop().unwrap();
            // println!("return {:?}", trace._entry);
            let l = self.tcx._trace_stack.borrow().len();
            if l == 0 {
                println!("WARNING: call stack exceeded!");
                self.tcx._trace_stack.borrow_mut().push(trace);
            } else {
                self.tcx._trace_stack.borrow_mut().last_mut().unwrap()._steps.push(Step::Call(trace));
            };
        };
    }

    // new) called by BB(X)
    pub fn push_bb_stack1(&mut self, bb: BasicBlock) {
        if *self.tcx._skip_counter.borrow() == 0 {
            self.tcx._trace_stack.borrow_mut().last_mut().unwrap()._steps.push(Step::B(bb));
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
}