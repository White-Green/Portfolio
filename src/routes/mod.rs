use yew::format::Nothing;
use yew::prelude::*;
use yew::services::fetch::*;
use yew::services::FetchService;
use yew_router::prelude::*;

pub(crate) mod home;
pub(crate) mod profile;
pub(crate) mod qualifications;
pub(crate) mod links;
pub(crate) mod works;
pub(crate) mod license;

/// App routes
#[derive(Switch, Debug, Clone, PartialEq)]
pub enum AppRoute {
    #[to = "/#profile"]
    Profile,
    #[to = "/#qualification"]
    Qualifications,
    #[to = "/#link"]
    Links,
    #[to = "/#works"]
    Works,
    #[to = "/#license"]
    License,
    #[to = "/"]
    Home,
}

impl ToString for AppRoute {
    fn to_string(&self) -> String {
        match self {
            AppRoute::Profile => "Profile",
            AppRoute::Qualifications => "Qualifications",
            AppRoute::Links => "Links",
            AppRoute::Works => "Works",
            AppRoute::License => "License",
            AppRoute::Home => "Home",
        }.to_string()
    }
}

fn request<R: 'static + From<std::result::Result<Vec<u8>, anyhow::Error>>>(addr: &str, callback: Callback<Response<R>>) -> FetchTask {
    let request = Request::get(addr).body(Nothing).unwrap();
    FetchService::fetch_binary(request, callback).unwrap()
}
