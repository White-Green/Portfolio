use std::ops::Deref;

use anyhow::Error;
use serde::Deserialize;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::*;
use yew::services::FetchService;

pub(crate) struct Links {
    link: ComponentLink<Self>,
    task: FetchTask,
    link_data: Vec<LinkData>,
}

pub(crate) enum LinkMessage {
    FetchLinkData(Vec<LinkData>),
    None,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
enum LinkValueData {
    DisplayOnly(String),
    WithLink { display: String, link: String },
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
enum LinkKeyData {
    NameOnly(String),
    WithImage { name: String, image: String },
}

#[derive(Debug, PartialEq, Deserialize)]
pub(crate) struct LinkData {
    key: LinkKeyData,
    value: LinkValueData,
}

impl Component for Links {
    type Message = LinkMessage;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let request = Request::get("/link.data.67860567.json").body(Nothing).unwrap();
        let callback = link.callback(|response: Response<Json<Result<Vec<LinkData>, Error>>>| {
            if response.status().is_success() {
                match response.into_body() {
                    Json(Ok(s)) => {
                        LinkMessage::FetchLinkData(s)
                    }
                    Json(Err(e)) => {
                        log::error!("error in fetching link.data.json: {:?}", e);
                        LinkMessage::None
                    }
                }
            } else {
                log::error!("error in fetching link.data.json code: {}", response.status());
                LinkMessage::None
            }
        });
        let task = FetchService::fetch(request, callback).unwrap();
        Self { link, task, link_data: Vec::new() }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            LinkMessage::FetchLinkData(data) => {
                log::info!("{:?}", data);
                if self.link_data != data {
                    self.link_data = data;
                    true
                } else {
                    false
                }
            }
            LinkMessage::None => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let link_table = self.link_data
            .iter()
            .map(|data| {
                let key = match &data.key {
                    LinkKeyData::NameOnly(name) => html! {
                        <div class="col-12 col-sm-6 col-lg-3 h3">{name}</div>
                    },
                    LinkKeyData::WithImage { name, image } => html! {
                        <div class="col-12 col-sm-6 col-lg-3 h3"><img class="mr-2" style="max-width: 2rem; max-height: 2rem;" src={image.deref()}/>{name}</div>
                    }
                };
                let value = match &data.value {
                    LinkValueData::DisplayOnly(id) => html! {
                        <div class="col h3">{id}</div>
                    },
                    LinkValueData::WithLink { display, link } => html! {
                        <div class="col h3"><a target="_blank" href={link.deref()}>{display}</a></div>
                    }
                };
                html! {
                    <div class="row mt-3">
                        {key}
                        {value}
                    </div>
                }
            });
        html! {
            <>
                <h1 class="m-2">{"Links"}</h1>
                {for link_table}
            </>
        }
    }
}
