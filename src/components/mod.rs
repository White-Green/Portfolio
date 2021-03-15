pub(crate) mod nav;
pub(crate) mod footer;

#[macro_export]
macro_rules! pure_component (
    ($struct:ident, $create:expr, $html:expr)=>{
        use yew::prelude::*;
        #[derive(Clone, Debug, Properties, PartialEq)]
        pub(crate) P(Children);
        impl Component for $struct{
                type Message = ();
                type Properties = P;
                fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self { $create(props.0) }
                fn update(&mut self, _: Self::Message) -> bool { false }
                fn change(&mut self, props: Self::Properties) -> bool {
                    if props != self.0 {
                        self.0 = props;
                        true
                    } else {
                        false
                    }
                }
                fn view(&self) -> Html { $html(&self.0) }
        }
    }
);
