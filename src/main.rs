use std::rc::Rc;

fn main() {
    env_logger::init();

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
    let mut path_group = brainfsym::PathGroup::make_entry(&ctx, Rc::new(prog), 16);
    let res = path_group
        .explore_until_output(&ctx, &mut solver, b"Serial :  :) Congratulations.")
        .unwrap();
    assert_eq!(res.input, b"CBA\x00");
}
