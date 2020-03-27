use crate::ast;

use std::iter;
use std::rc::Rc;

use derive_setters::Setters;
use z3::ast::Ast as Z3Ast;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct SymBytes<'ctx>(pub Vec<z3::ast::BV<'ctx>>);

/// Symbolic program state
#[derive(Clone, Setters, PartialEq, Debug)]
pub struct State<'ctx> {
    /// z3 context
    pub ctx: &'ctx z3::Context,

    /// Brainf*** program
    pub prog: Rc<ast::Prog>,

    /// Symbolic memory bytes
    pub mem: SymBytes<'ctx>,

    /// Instruction pointer
    pub insn_ptr: usize,

    /// Data pointer
    pub data_ptr: usize,

    /// Symbolic input bytes
    pub input: SymBytes<'ctx>,

    /// Symbolic output bytes
    pub output: SymBytes<'ctx>,

    /// Constraints required for this state to be valid. This is a collection of
    /// all the conditions that cause the program to branch to this state.
    pub path: z3::ast::Bool<'ctx>,
}

/// Concrete program state
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConcreteState {
    /// Memory bytes
    pub mem: Vec<u8>,

    /// Instruction pointer
    pub insn_ptr: usize,

    /// Data pointer
    pub data_ptr: usize,

    /// Input bytes
    pub input: Vec<u8>,

    /// Output bytes
    pub output: Vec<u8>,
}

fn init_mem(ctx: &z3::Context, mem_size: usize) -> SymBytes {
    let zero = z3::ast::BV::from_u64(ctx, 0, 8);
    SymBytes(iter::repeat(zero).take(mem_size).collect())
}

impl<'ctx> State<'ctx> {
    pub fn make_entry(ctx: &'ctx z3::Context, prog: Rc<ast::Prog>, mem_size: usize) -> Self {
        State {
            ctx,
            prog,
            mem: init_mem(ctx, mem_size),
            insn_ptr: 0,
            data_ptr: 0,
            input: SymBytes::default(),
            output: SymBytes::default(),
            path: z3::ast::Bool::from_bool(ctx, true),
        }
    }

    pub fn step(&self) -> Vec<Self> {
        match self.prog.0.get(self.insn_ptr) {
            Some(ast::Insn::Right) => vec![self.op_right()],
            Some(ast::Insn::Left) => vec![self.op_left()],
            Some(ast::Insn::Inc) => vec![self.op_inc()],
            Some(ast::Insn::Dec) => vec![self.op_dec()],
            Some(ast::Insn::Out) => vec![self.op_out()],
            Some(ast::Insn::In) => vec![self.op_in()],
            Some(ast::Insn::JmpIfZero(insn_ptr)) => self.op_jmp_if_zero(*insn_ptr),
            Some(ast::Insn::JmpIfNonZero(insn_ptr)) => self.op_jmp_if_non_zero(*insn_ptr),
            None => vec![],
        }
    }

    pub fn exited(&self) -> bool {
        self.insn_ptr == self.prog.0.len()
    }

    pub fn concretize(&self) -> Result<ConcreteState, z3::SatResult> {
        let solver = z3::Solver::new(self.ctx);
        solver.assert(&self.path);
        let err = solver.check();
        if err == z3::SatResult::Sat {
            Ok(ConcreteState::from_model(&solver.get_model(), self)
                .expect("failed concretizing state"))
        } else {
            Err(err)
        }
    }

    fn get_cell(&self) -> z3::ast::BV<'ctx> {
        self.mem.0[self.data_ptr].clone()
    }

    fn set_cell(&self, val: z3::ast::BV<'ctx>) -> Self {
        let mem = {
            let mut mem = self.mem.clone();
            mem.0[self.data_ptr] = val;
            mem
        };
        self.clone().mem(mem)
    }

    fn inc_insn_ptr(&self) -> Self {
        self.clone()
            .insn_ptr((self.insn_ptr + 1) % self.prog.0.len())
    }

    fn op_right(&self) -> Self {
        self.clone()
            .data_ptr((self.data_ptr + 1) % self.mem.0.len())
            .inc_insn_ptr()
    }

    fn op_left(&self) -> Self {
        self.clone()
            .data_ptr((self.data_ptr - 1) % self.mem.0.len())
            .inc_insn_ptr()
    }

    fn op_inc_dec_helper(&self, is_inc: bool) -> Self {
        let one = z3::ast::BV::from_u64(self.ctx, 1, 8);
        let fcn = if is_inc {
            z3::ast::BV::bvadd
        } else {
            z3::ast::BV::bvsub
        };
        let old_val = self.get_cell();
        let new_val = fcn(&old_val, &one);
        self.set_cell(new_val).inc_insn_ptr()
    }

    fn op_inc(&self) -> Self {
        self.op_inc_dec_helper(true)
    }

    fn op_dec(&self) -> Self {
        self.op_inc_dec_helper(false)
    }

    fn op_out(&self) -> Self {
        self.clone()
            .output(SymBytes(
                self.output
                    .0
                    .clone()
                    .into_iter()
                    .chain(iter::once(self.get_cell()))
                    .collect(),
            ))
            .inc_insn_ptr()
    }

    fn op_in(&self) -> Self {
        let name = format!("input[{}]", self.mem.0.len());
        let val = z3::ast::BV::new_const(self.ctx, name, 8);
        self.clone()
            .input(SymBytes(
                self.input
                    .0
                    .clone()
                    .into_iter()
                    .chain(iter::once(val.clone()))
                    .collect(),
            ))
            .set_cell(val)
            .inc_insn_ptr()
    }

    fn op_jmp_helper(&self, insn_ptr: usize, if_zero: bool) -> Vec<Self> {
        let cell_eq_zero = self.get_cell()._eq(&z3::ast::BV::from_u64(self.ctx, 0, 8));
        let cell_not_eq_zero = z3::ast::Bool::not(&cell_eq_zero);

        let zero_path = self.path.and(&[&cell_eq_zero]);
        let non_zero_path = self.path.and(&[&cell_not_eq_zero]);

        let (taken_path, not_taken_path) = if if_zero {
            (zero_path, non_zero_path)
        } else {
            (non_zero_path, zero_path)
        };

        let taken = self.clone().insn_ptr(insn_ptr).path(taken_path);
        let not_taken = self.inc_insn_ptr().path(not_taken_path);

        vec![taken, not_taken]
    }

    fn op_jmp_if_zero(&self, insn_ptr: usize) -> Vec<Self> {
        self.op_jmp_helper(insn_ptr, true)
    }

    fn op_jmp_if_non_zero(&self, insn_ptr: usize) -> Vec<Self> {
        self.op_jmp_helper(insn_ptr, false)
    }
}

impl<'ctx> SymBytes<'ctx> {
    fn concretize(&self, model: &z3::Model) -> Option<Vec<u8>> {
        self.0
            .iter()
            .map(|b| model.eval(b).and_then(|b| b.as_u64()).map(|b| b as u8))
            .collect::<Option<Vec<u8>>>()
    }
}

impl ConcreteState {
    fn from_model(model: &z3::Model, state: &State) -> Option<Self> {
        Some(Self {
            mem: state.mem.concretize(model)?,
            insn_ptr: state.insn_ptr,
            data_ptr: state.data_ptr,
            input: state.input.concretize(model)?,
            output: state.output.concretize(model)?,
        })
    }
}
