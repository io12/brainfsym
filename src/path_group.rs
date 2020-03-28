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

impl<'ctx> PathGroup<'ctx> {
    pub fn make_entry(ctx: &'ctx z3::Context, prog: Rc<ast::Prog>, mem_size: usize) -> Self {
        Self {
            next: vec![(State::make_entry(ctx, prog, mem_size), cmp::Reverse(0))].into(),
            visited: HashSet::new(),
        }
    }

    pub fn explore_until<F, T>(&mut self, fcn: F) -> Option<T>
    where
        F: Fn(&State) -> Option<T>,
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
            let v = fcn(&state);
            if v.is_some() {
                return v;
            }
            for state in state.step() {
                if !self.visited.contains(&state) {
                    let priority = cmp::Reverse(state.input.0.len());
                    self.next.push(state, priority);
                }
            }
        }
    }

    pub fn explore_until_output(&mut self, output: &[u8]) -> Option<ConcreteState> {
        self.explore_until(|state| {
            let state = state.concretize().ok()?;
            debug!("concrete output: {:?}", state.output);
            if state.output == output {
                Some(state)
            } else {
                None
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
}
