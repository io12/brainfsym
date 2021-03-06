#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        use Insn::*;

        assert_eq!(
            Prog::from_str(",>,[-<+>]<.").unwrap(),
            Prog(vec![
                In,
                Right,
                In,
                JmpIfZero(9),
                Dec,
                Left,
                Inc,
                Right,
                JmpIfNonZero(4),
                Left,
                Out
            ])
        );

        assert_eq!(
            Prog::from_str("+[>,]+[<.-]").unwrap(),
            Prog(vec![
                Inc,
                JmpIfZero(5),
                Right,
                In,
                JmpIfNonZero(2),
                Inc,
                JmpIfZero(11),
                Left,
                Out,
                Dec,
                JmpIfNonZero(7),
            ])
        );

        assert_eq!(
            Prog::from_str("[]").unwrap(),
            Prog(vec![JmpIfZero(2), JmpIfNonZero(1)])
        );

        assert_eq!(
            Prog::from_str("[][]").unwrap(),
            Prog(vec![
                JmpIfZero(2),
                JmpIfNonZero(1),
                JmpIfZero(4),
                JmpIfNonZero(3)
            ])
        );

        assert_eq!(
            Prog::from_str("[[]]").unwrap(),
            Prog(vec![
                JmpIfZero(4),
                JmpIfZero(3),
                JmpIfNonZero(2),
                JmpIfNonZero(1),
            ])
        );
    }
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct Prog(pub Vec<Insn>);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Insn {
    /// '>' - Move the data pointer to the right
    Right,

    /// '<' - Move the data pointer to the left
    Left,

    /// '+' - Increment the memory cell under the pointer
    Inc,

    /// '-' - Decrement the memory cell under the pointer
    Dec,

    /// '.' - Output the character signified by the cell at the pointer
    Out,

    /// ',' - Input the character signified by the cell at the pointer
    In,

    /// '[' - Jump to the inner index if the cell under the pointer is 0
    JmpIfZero(usize),

    /// ']' - Jump to the inner index if the cell under the pointer is nonzero
    JmpIfNonZero(usize),
}

pub type ParseError<'a> = nom::Err<nom::types::CompleteStr<'a>>;
pub type ParseResult<'a> = Result<Prog, ParseError<'a>>;

impl Prog {
    pub fn from_str(s: &str) -> ParseResult {
        brainfuck::parser::parse(nom::types::CompleteStr(s))
            .map(|prog| Self::from_brainf_block(0, &prog))
    }

    fn from_brainf_block(insn_ptr: usize, block: &brainfuck::ast::Block) -> Self {
        Self(block.into_iter().fold(vec![], |acc, node| {
            let insns = Self::from_brainf_node(insn_ptr + acc.len(), node).0;
            [acc, insns].concat()
        }))
    }

    fn from_brainf_node(insn_ptr: usize, node: &brainfuck::ast::Node) -> Self {
        use brainfuck::ast::Node::*;
        Self(match node {
            LShift => vec![Insn::Left],
            RShift => vec![Insn::Right],
            Inc => vec![Insn::Inc],
            Dec => vec![Insn::Dec],
            PutCh => vec![Insn::Out],
            GetCh => vec![Insn::In],
            Loop(block) => {
                let inner = Self::from_brainf_block(insn_ptr + 1, block);
                [
                    vec![Insn::JmpIfZero(insn_ptr + 1 + inner.0.len() + 1)],
                    inner.0,
                    vec![Insn::JmpIfNonZero(insn_ptr + 1)],
                ]
                .concat()
            }
        })
    }
}
