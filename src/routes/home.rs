use yew::prelude::*;

/// Home page
pub struct Home;

impl Component for Home {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Home {}
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <>
                <div class="h3 m-2">
                    { "White-Green's Portfolio." }
                </div>
                <div class="h4 ml-5">
                    { "Created with " }
                    <a href="https://yew.rs/docs/ja/" target="_blank"><img src="./logo.86ce68ea.svg" style="height: 2rem;"/>{ "Yew.rs" }</a>
                    { "." }
                </div>
            </>
        }
    }
}
