use crate::ast;
use crate::cached_solver::CachedSolver;
use crate::cached_solver::Error;
use crate::state::ConcreteState;
use crate::state::State;
use crate::state::SymBytes;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Rc;

pub struct PathGroup<'ctx> {
    next: Vec<State<'ctx>>,
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
            next: vec![State::make_entry(ctx, prog, mem_size)],
            visited: HashSet::new(),
        }
    }

    fn add_continuations(&mut self, ctx: &'ctx z3::Context, state: &State<'ctx>) {
        for state in state.step(ctx) {
            if !self.visited.contains(&state) {
                self.next.push(state);
            }
        }
    }

    pub fn explore_until<F, T>(
        &mut self,
        ctx: &'ctx z3::Context,
        solver: &mut CachedSolver<'ctx>,
        mut fcn: F,
    ) -> Option<T>
    where
        F: FnMut(&State<'ctx>, &mut CachedSolver<'ctx>) -> ExploreFnResult<T>,
    {
        loop {
            debug!(
                "num next: {}, num_visited: {}",
                self.next.len(),
                self.visited.len()
            );
            let state = self.next.pop()?;
            trace!("state: {:#?}", state);
            if let Err(Error::Unsat) = state.concretize(ctx, solver) {
                continue;
            }
            match fcn(&state, solver) {
                ExploreFnResult::Done(v) => return Some(v),
                ExploreFnResult::Invalid => continue,
                ExploreFnResult::Valid => {
                    self.visited.insert(state.clone());
                    self.add_continuations(ctx, &state)
                }
            }
        }
    }

    pub fn explore_until_output(
        &mut self,
        ctx: &'ctx z3::Context,
        solver: &mut CachedSolver<'ctx>,
        output: &[u8],
    ) -> Option<ConcreteState> {
        self.explore_until(ctx, solver, |state, solver| {
            let sym_len = state.output.0.len();
            let concr_len = output.len();

            debug!("state output len: {}/{}", sym_len, concr_len);

            let cmp = sym_len.cmp(&concr_len);
            match cmp {
                Ordering::Greater => ExploreFnResult::Invalid,
                Ordering::Less | Ordering::Equal => {
                    let output_eq = SymBytes::syms_eq(ctx, &state.output, output);
                    let state = state.concretize_with(ctx, solver, &output_eq);
                    match state {
                        Ok(state) if cmp == Ordering::Equal => ExploreFnResult::Done(state),
                        Ok(_) | Err(Error::Unknown) => ExploreFnResult::Valid,
                        Err(Error::Unsat) => ExploreFnResult::Invalid,
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
        let mut solver = CachedSolver::new();

        let prog = ast::Prog::from_str(",>,[-<+>]<.").unwrap();
        let mut path_group = PathGroup::make_entry(&ctx, Rc::new(prog), 16);
        let res = path_group
            .explore_until_output(&ctx, &mut solver, &[2])
            .unwrap();
        assert_eq!(res.input.iter().sum::<u8>(), 2);
    }

    #[test]
    fn test_rev() {
        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);
        let mut solver = CachedSolver::new();

        let prog = ast::Prog::from_str("+[>,]+[<.-]").unwrap();
        let mut path_group = PathGroup::make_entry(&ctx, Rc::new(prog), 16);
        let res = path_group
            .explore_until_output(&ctx, &mut solver, b"ABC")
            .unwrap();
        assert_eq!(res.input, b"CBA\x00");
    }
}
