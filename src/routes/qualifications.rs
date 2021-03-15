use std::ops::Deref;

use anyhow::Error;
use serde::Deserialize;
use yew::format::Json;
use yew::prelude::*;
use yew::services::fetch::*;

use crate::routes::request;

pub(crate) struct Qualifications {
    link: ComponentLink<Self>,
    task: FetchTask,
    data: Vec<QualificationData>,
}

pub(crate) enum QualificationMessage {
    FetchQualificationData(Vec<QualificationData>),
    None,
}

#[derive(Debug, PartialEq, Deserialize)]
struct QualificationValueData {
    name: String,
    time: String,
    link: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct QualificationGroupData {
    name: String
}

#[derive(Debug, PartialEq, Deserialize)]
pub(crate) struct QualificationData {
    category: QualificationGroupData,
    values: Vec<QualificationValueData>,
}

impl Component for Qualifications {
    type Message = QualificationMessage;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|response: Response<Json<Result<Vec<QualificationData>, Error>>>| {
            if response.status().is_success() {
                match response.into_body() {
                    Json(Ok(s)) => {
                        QualificationMessage::FetchQualificationData(s)
                    }
                    Json(Err(e)) => {
                        log::error!("error in fetching link.data.json: {:?}", e);
                        QualificationMessage::None
                    }
                }
            } else {
                log::error!("error in fetching link.data.json code: {}", response.status());
                QualificationMessage::None
            }
        });
        let task = request("/qualification.data.dc54e79d.json", callback);
        Self { link, task, data: Vec::new() }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            QualificationMessage::FetchQualificationData(data) => {
                log::info!("{:?}", data);
                if self.data != data {
                    self.data = data;
                    true
                } else {
                    false
                }
            }
            QualificationMessage::None => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let link_table = self.data
            .iter()
            .map(|data| {
                let data_list = data.values.iter().map(|data| {
                    let name = if let Some(link) = &data.link {
                        html! {
                            <div class="col h4"><a href={link.deref()} target="_blank">{data.name.deref()}</a></div>
                        }
                    } else {
                        html! {
                            <div class="col h4">{data.name.deref()}</div>
                        }
                    };
                    html! {
                        <div class="row col-12 mt-3 ml-2">
                            <div class="col-12 col-lg-4 col-xl-3 h4">{data.time.deref()}</div>
                            {name}
                        </div>
                    }
                });
                html! {
                    <div class="row mt-5">
                        <h3 class="col-12">
                            {data.category.name.deref()}
                        </h3>
                        {for data_list}
                    </div>
                }
            });
        html! {
            <>
                <h1 class="m-2">{"Qualifications"}</h1>
                {for link_table}
            </>
        }
    }
}
