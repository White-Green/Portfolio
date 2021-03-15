use yew::prelude::*;
use yew_router::{prelude::*, route::Route};

use crate::components::{footer::footer, nav::Nav};
use crate::routes::{AppRoute, home::Home, license::License, links::Links, profile::Profile, qualifications::Qualifications, works::Works};

/// Root component
pub(crate) struct App {
    current_route: AppRoute,
    link: ComponentLink<Self>,
    key: Option<aes::Key>,
}

pub(crate) enum AppMessage {
    ChangeCurrentRoute(AppRoute),
    ChangeKey(aes::Key),
}

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App { current_route: AppRoute::Home, link, key: None }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            AppMessage::ChangeCurrentRoute(route) => {
                if route != self.current_route {
                    self.current_route = route;
                    true
                } else {
                    false
                }
            }
            AppMessage::ChangeKey(key) => {
                self.key = Some(key);
                true
            }
        }
    }

    fn view(&self) -> Html {
        let callback = self.link.callback(|route| AppMessage::ChangeCurrentRoute(route));
        let key_callback = self.link.callback(|key| AppMessage::ChangeKey(key));
        let key = self.key.clone();
        html! {
            <>
                <header>
                    <Nav current_route=self.current_route.clone() key_callback=key_callback/>
                </header>
                <main>
                    <div class="container">
                        <Router<AppRoute, ()>
                            render = Router::render(move|switch: AppRoute | {
                                log::info!("render");
                                callback.emit(switch.clone());
                                html!{
                                    <>
                                        {
                                            match switch {
                                                AppRoute::Home => html!{ <Home /> },
                                                AppRoute::Profile => html!{ <Profile encrypt_key=key.clone() /> },
                                                AppRoute::Qualifications => html!{ <Qualifications /> },
                                                AppRoute::Links => html!{ <Links /> },
                                                AppRoute::Works => html!{ <Works /> },
                                                AppRoute::License => html!{},
                                            }
                                        }
                                        <License show={switch == AppRoute::License}/>
                                    </>
                                }
                            } )
                            redirect = Router::redirect(|route: Route<()>| {
                                log::info!("redirect");
                                AppRoute::Home
                            })
                        />
                    </div>
                </main>
                <footer class="footer mt-auto py-3">
                    { footer() }
                </footer>
            </>
        }
    }
}
