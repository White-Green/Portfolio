use std::ops::Deref;

use anyhow::Error;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use yew::format::Json;
use yew::prelude::*;
use yew::services::fetch::*;
use yewtil::NeqAssign;

use crate::routes::request;

#[derive(Debug)]
pub(crate) struct License {
    props: LicenseProperties,
    link: ComponentLink<Self>,
    task: Option<FetchTask>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub(crate) struct LicenseProperties {
    pub(crate) show: bool
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum LicenseMessage {
    FetchLicenseData(Vec<LicenseData>),
    None,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub(crate) struct LicenseData {
    name: String,
    version: Option<String>,
    authors: Option<String>,
    repository: Option<String>,
    license: Option<String>,
    description: Option<String>,
}

static CELL: OnceCell<Vec<LicenseData>> = OnceCell::new();

impl Component for License {
    type Message = LicenseMessage;
    type Properties = LicenseProperties;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut component = Self { props, link, task: None };
        component.init();
        component
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            LicenseMessage::FetchLicenseData(data) => {
                log::info!("{:?}", data);
                CELL.set(data).is_ok()
            }
            LicenseMessage::None => false,
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props.neq_assign(props) {
            self.init();
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let default = Vec::new();
        let link_table = CELL.get().unwrap_or(&default)
            .iter()
            .map(|data| {
                html! {
                    <div class="card">
                        <div class="card-header">
                            { &data.name }
                        </div>
                        <ul class="list-group list-group-flush">
                            { data.version.as_ref().map(|value| html!{<li class="list-group-item">{"Version: "}{value}</li>}).unwrap_or(html!{}) }
                            { data.authors.as_ref().map(|value| html!{<li class="list-group-item">{"Authors: "}{value}</li>}).unwrap_or(html!{}) }
                            { data.repository.as_ref().map(|value| html!{<li class="list-group-item">{"Repository: "}<a href={value.deref()} target="_blank">{value}</a></li>}).unwrap_or(html!{}) }
                            { data.license.as_ref().map(|value| html!{<li class="list-group-item">{"License: "}{value}</li>}).unwrap_or(html!{}) }
                            { data.description.as_ref().map(|value| html!{<li class="list-group-item">{"Description: "}{value}</li>}).unwrap_or(html!{}) }
                        </ul>
                    </div>
                }
            });
        let style = if self.props.show { "" } else { "display: none;" };
        html! {
            <>
                <h1 class="m-2" style={style}>{"License: Cargo crates"}</h1>
                <div class="card-columns" style={style}>
                    {for link_table}
                </div>
            </>
        }
    }
}

impl License {
    fn init(&mut self) {
        self.task = if CELL.get().is_none() && self.props.show {
            let callback = self.link.callback(|response: Response<Json<Result<Vec<LicenseData>, Error>>>| {
                if response.status().is_success() {
                    match response.into_body() {
                        Json(Ok(s)) => {
                            LicenseMessage::FetchLicenseData(s)
                        }
                        Json(Err(e)) => {
                            log::error!("error in fetching link.data.json: {:?}", e);
                            LicenseMessage::None
                        }
                    }
                } else {
                    log::error!("error in fetching link.data.json code: {}", response.status());
                    LicenseMessage::None
                }
            });
            let task = request("/license.data.61b04f4a.json", callback);
            Some(task)
        } else {
            None
        };
    }
}
