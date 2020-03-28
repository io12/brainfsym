use crate::ast;
use crate::state::ConcreteState;
use crate::state::State;

use std::cmp;
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
            debug!("state: {:#?}", state);
            self.visited.insert(state.clone());
            match fcn(&state) {
                ExploreFnResult::Done(v) => return Some(v),
                ExploreFnResult::Invalid => continue,
                ExploreFnResult::Valid => self.add_continuations(&state),
            }
        }
    }

    pub fn explore_until_output(&mut self, output: &[u8]) -> Option<ConcreteState> {
        self.explore_until(|state| match state.concretize() {
            Ok(state) => {
                debug!("concrete output: {:?}", state.output);
                if state.output == output {
                    ExploreFnResult::Done(state)
                } else {
                    ExploreFnResult::Valid
                }
            }
            Err(z3::SatResult::Sat) => unreachable!(),
            Err(z3::SatResult::Unknown) => ExploreFnResult::Valid,
            Err(z3::SatResult::Unsat) => ExploreFnResult::Invalid,
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
}
