struct WebUiData {
    solver: brainfsym::CachedSolver<'static>,
    prog: brainfsym::ast::Prog,
    path_group: brainfsym::PathGroup<'static>,
}

struct Model {
    data: Option<WebUiData>,
}

impl yew::Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: yew::html::ComponentLink<Self>) -> Self {
        Self { data: None }
    }

    fn update(&mut self, _msg: Self::Message) -> yew::html::ShouldRender {
        false
    }

    fn view(&self) -> yew::html::Html {
        yew::html!(<p>{"hello world"}</p>)
    }
}

fn run_app() {
    yew::start_app::<Model>();
}

fn main() {
    run_app()
}
