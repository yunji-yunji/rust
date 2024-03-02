use super::{InterpCx, Machine};
use rustc_middle::mir::BasicBlock;
use rustc_middle::ty::context::FnInstKey;
use rustc_middle::ty::context::Step;

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
        let mut prev_fn = self.tcx._prev.borrow_mut();
        let mut steps_before = self.tcx._s1.borrow_mut();
        let mut tmp_trace = self.tcx._tmp_trace.borrow_mut();
        tmp_trace._entry = prev_fn.clone();
        tmp_trace._steps = steps_before.clone();

        let mut tmp_vec = self.tcx._v1.borrow_mut();
        tmp_vec.push(tmp_trace.clone());

        *prev_fn = fn_key;
        *steps_before = vec![];
    }

    // new) called by return
    pub fn merge_trace_stack1(&mut self, ) {
        let mut prev_fn = self.tcx._prev.borrow_mut();
        let mut s1 = self.tcx._s1.borrow_mut();
        let mut tmp_trace = self.tcx._tmp_trace.borrow_mut();
        tmp_trace._entry = prev_fn.clone();
        tmp_trace._steps = s1.clone();

        let mut v1 = self.tcx._v1.borrow_mut();
        let last_trace = v1.pop();
        match last_trace {
            Some(mut trace) => {
                trace._steps.push(Step::Call(tmp_trace.clone()));
                *prev_fn = trace._entry;
                *s1 = trace._steps;
            },
            None => {
                match std::env::var_os("TET2") {
                    None => (),
                    Some(_val) => {
                        println!("Q1{:?}", s1);
                    }
                }
                match std::env::var_os("TET3") {
                    None => (),
                    Some(_val) => {
                        println!("Q2{:?}", tmp_trace);
                    }
                }
            },
        }
    }

    // new) called by BB(X)
    pub fn push_bb_stack1(&mut self, bb: BasicBlock) {
        let mut _s1= self.tcx._s1.borrow_mut();
        _s1.push(Step::B(bb));
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