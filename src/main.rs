use std::rc::Rc;

fn main() {
    env_logger::init();
    let cfg = z3::Config::new();
    let ctx = z3::Context::new(&cfg);
    let prog = brainfsym::ast::Prog::from_str("+[>,]+[<.-]").unwrap();
    let mut path_group = brainfsym::PathGroup::make_entry(&ctx, Rc::new(prog), 16);
    let res = path_group.explore_until_output(&[1, 2]).unwrap();
    assert_eq!(res.input, [2, 1]);
}
