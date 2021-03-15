use std::convert::TryInto;

use sha3::Digest;
use yew::prelude::*;
use yew::utils::window;
use yew_router::prelude::*;

use crate::routes::AppRoute;

/// Nav component
pub(crate) struct Nav {
    props: NavProps,
    link: ComponentLink<Self>,
}

#[derive(Debug, PartialEq, Clone, Properties)]
pub(crate) struct NavProps {
    pub(crate) current_route: AppRoute,
    pub(crate) key_callback: Callback<aes::Key>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NavMessage {
    TryUnlock
}

impl Component for Nav {
    type Message = NavMessage;
    type Properties = NavProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Nav { props, link }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props != self.props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            NavMessage::TryUnlock => {
                let name = window().prompt_with_message("本サイト所有者の氏名をひらがなで入力してください").expect("error in prompt");
                if let Some(name) = name {
                    let name = regex::Regex::new("[^ぁ-ゖ]").unwrap().replace_all(&name, "");
                    log::info!("{}", name);
                    let key = sha3::Sha3_256::digest(name.as_bytes()).to_vec().try_into().unwrap();
                    let key = aes::Key::AES256(key);
                    self.props.key_callback.emit(key);
                }
                false
            }
        }
    }

    fn view(&self) -> Html {
        let navbar_links = [AppRoute::Home, AppRoute::Profile, AppRoute::Works, AppRoute::Qualifications, AppRoute::Links]
            .iter()
            .map(|route| if route == &self.props.current_route {
                html! {
                    <li class="nav-item active">
                        <RouterAnchor<AppRoute> route=route.clone() classes="nav-link" >{ route.to_string() } <span class="sr-only">{"(current)"}</span></RouterAnchor<AppRoute>>
                    </li>
                }
            } else {
                html! {
                    <li class="nav-item">
                        <RouterAnchor<AppRoute> route=route.clone() classes="nav-link" >{ route.to_string() }</RouterAnchor<AppRoute>>
                    </li>
                }
            });
        html! {
            <nav class="navbar navbar-expand-md navbar-light bg-light">
                <RouterAnchor<AppRoute> route=AppRoute::Home classes="navbar-brand" >
                    <img src="./icon.bac3c665.svg" width="30" height="30" class="rotate"/>
                    { "Portfolio" }
                </RouterAnchor<AppRoute>>
                <button class="navbar-toggler" type="button" data-toggle="collapse" data-target="#navbarNav" aria-controls="navbarNav" aria-expanded="false" aria-label="Toggle navigation">
                    <span class="navbar-toggler-icon"></span>
                </button>
                <div class="collapse navbar-collapse" id="navbarNav">
                  <ul class="navbar-nav">
                    {for navbar_links}
                    <li class="nav-item">
                      <a class="btn btn-primary nav-link" onclick=self.link.callback(|_|NavMessage::TryUnlock)>{"unlock"}</a>
                    </li>
                  </ul>
                </div>
            </nav>
        }
    }
}
