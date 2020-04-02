use std::rc::Rc;

#[test]
/// Credit:
/// https://aodrulez.blogspot.com/2011/09/detailed-analysis-of-my-brainfuck.html
fn aodrulez_crackme() {
    let cfg = z3::Config::new();
    let ctx = z3::Context::new(&cfg);
    let mut solver = brainfsym::CachedSolver::new();

    let prog = brainfsym::ast::Prog::from_str(concat!(
        "Aodrulez's Brainfuck Crackme V1",
        "# -------------------------------------------------",
        "# (Its very Easy)",
        ">++++++++++[>++++++++>++++++++++>+++++++++++>++++++",
        "+++++>++++++++++>+++++++++++>+++>++++++>+++><<<<<<<",
        "<<<-]>+++>+>++++>----->--->-->++>-->++><<<<<<<<<<>.",
        ">.>.>.>.>.>.>.>.>,>,>,>,>,>,<[>-<-]#>[>+++>++++++>+",
        "+++>+++>+++++++>+++++++++++>+++++++++++>++++++++++>",
        "+++++++++++>++++++++++>++++++++++++>++++++++++++>++",
        "+++++++++>++++++++++>++++++++++++>+++++++++++>+++++",
        "++++++>+++++++++++>++++++++++++>+++++>+++><<<<<<<<<",
        "<<<<<<<<<<<<<-]>++>-->+>++>--->+>>+++>++++>--->----",
        ">--->-->--->---->----->+>>----->---->++><<<<<<<<<<<",
        "<<<<<<<<<<<>.>.>.>.>.>.>.>.>.>.>.>.>.>.>.>.>.>.>.>.>.",
    ))
    .unwrap();
    let mut path_group = brainfsym::PathGroup::make_entry(&ctx, Rc::new(prog), 64);
    let res = path_group
        .explore_until_output(&ctx, &mut solver, b"Serial :  :) Congratulations.")
        .unwrap();

    // Constraints for keygen
    assert_eq!(res.input.len(), 6);
    assert_eq!(res.input[4] + 10, res.input[5]);
}
