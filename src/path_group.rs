use crate::ast;
use crate::state::ConcreteState;
use crate::state::State;
use crate::state::SymBytes;

use std::cmp;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Rc;

use priority_queue::PriorityQueue;

pub struct PathGroup<'ctx> {
    next: PriorityQueue<State<'ctx>, cmp::Reverse<usize>>,
    visited: HashSet<State<'ctx>>,
}

/// Type returned by `explore_until()` callback
pub enum ExploreFnResult<T> {
    /// Found the target state. This includes an inner type so the caller can
    /// examine the resulting symbolic state, concrete state, or any other data.
    Done(T),

    /// This state is invalid. Path exploration will discard it and not compute
    /// continuations
    Invalid,

    /// This state is valid. It is not the target state, but continuations
    /// should be computed.
    Valid,
}

impl<'ctx> PathGroup<'ctx> {
    pub fn make_entry(ctx: &'ctx z3::Context, prog: Rc<ast::Prog>, mem_size: usize) -> Self {
        Self {
            next: vec![(State::make_entry(ctx, prog, mem_size), cmp::Reverse(0))].into(),
            visited: HashSet::new(),
        }
    }

    fn add_continuations(&mut self, state: &State<'ctx>) {
        for state in state.step() {
            if !self.visited.contains(&state) {
                let priority = cmp::Reverse(state.input.0.len());
                self.next.push(state, priority);
            }
        }
    }

    pub fn explore_until<F, T>(&mut self, fcn: F) -> Option<T>
    where
        F: Fn(&State) -> ExploreFnResult<T>,
    {
        loop {
            debug!(
                "num next: {}, num_visited: {}",
                self.next.len(),
                self.visited.len()
            );
            let (state, _) = self.next.pop()?;
            trace!("state: {:#?}", state);
            if let Err(z3::SatResult::Unsat) = state.check_sat() {
                continue;
            }
            match fcn(&state) {
                ExploreFnResult::Done(v) => return Some(v),
                ExploreFnResult::Invalid => continue,
                ExploreFnResult::Valid => {
                    self.visited.insert(state.clone());
                    self.add_continuations(&state)
                }
            }
        }
    }

    pub fn explore_until_output(&mut self, output: &[u8]) -> Option<ConcreteState> {
        self.explore_until(|state| {
            let sym_len = state.output.0.len();
            let concr_len = output.len();

            debug!("state output len: {}/{}", sym_len, concr_len);

            let cmp = sym_len.cmp(&concr_len);
            match cmp {
                Ordering::Greater => ExploreFnResult::Invalid,
                Ordering::Less | Ordering::Equal => {
                    let output_eq = SymBytes::syms_eq(state.ctx, &state.output, output);
                    let state = state.concretize_with(&output_eq);
                    match state {
                        Ok(state) if cmp == Ordering::Equal => ExploreFnResult::Done(state),
                        Ok(_) | Err(z3::SatResult::Unknown) => ExploreFnResult::Valid,
                        Err(z3::SatResult::Unsat) => ExploreFnResult::Invalid,
                        Err(z3::SatResult::Sat) => unreachable!(),
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);

        let prog = ast::Prog::from_str(",>,[-<+>]<.").unwrap();
        let mut path_group = PathGroup::make_entry(&ctx, Rc::new(prog), 16);
        let res = path_group.explore_until_output(&[2]).unwrap();
        assert_eq!(res.input.iter().sum::<u8>(), 2);
    }

    #[test]
    fn test_rev() {
        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);

        let prog = ast::Prog::from_str("+[>,]+[<.-]").unwrap();
        let mut path_group = PathGroup::make_entry(&ctx, Rc::new(prog), 16);
        let res = path_group.explore_until_output(b"ABC").unwrap();
        assert_eq!(res.input, b"CBA\x00");
    }
}
