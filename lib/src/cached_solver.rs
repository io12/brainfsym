use std::collections::HashMap;
use std::rc::Rc;

pub struct CachedSolver<'ctx> {
    cache: HashMap<z3::ast::Bool<'ctx>, SolverResultModel<'ctx>>,
}

#[derive(Clone)]
pub enum Error {
    Unsat,
    Unknown,
}

pub type SolverResult<T> = Result<T, Error>;

pub type SolverResultModel<'ctx> = SolverResult<Rc<z3::Model<'ctx>>>;

impl<'ctx> CachedSolver<'ctx> {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn solve(
        &mut self,
        ctx: &'ctx z3::Context,
        expr: z3::ast::Bool<'ctx>,
    ) -> SolverResultModel {
        match self.cache.get(&expr) {
            Some(res) => res.clone(),
            None => {
                let solver = z3::Solver::new(ctx);
                solver.assert(&expr);
                let res = match solver.check() {
                    z3::SatResult::Sat => Ok(Rc::new(solver.get_model())),
                    z3::SatResult::Unsat => Err(Error::Unsat),
                    z3::SatResult::Unknown => Err(Error::Unknown),
                };
                self.cache.insert(expr, res.clone());
                res
            }
        }
    }
}
