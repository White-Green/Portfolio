use std::collections::HashMap;

use anyhow::Error;
use serde::Deserialize;
use web_sys::Node;
use yew::format::Json;
use yew::prelude::*;
use yew::services::fetch::*;

use crate::routes::request;

pub(crate) struct Works {
    link: ComponentLink<Self>,
    tasks: [FetchTask; 2],
    works_data: Option<WorksData>,
    works_svg: Option<String>,
    node_ref: NodeRef,
}

pub(crate) enum WorkMessage {
    FetchWorksData(WorksData),
    FetchWorksSvg(String),
    None,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Repository {
    pub html_url: String,
    pub name: String,
    pub homepage: Option<String>,
    pub language: Vec<(usize, usize)>,
    pub community_profile: Option<CommunityProfile>,
    pub technology_stacks: Vec<usize>,
    pub related_repositories: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct CommunityProfile {
    health_percentage: usize,
    description: Option<String>,
    documentation: Option<String>,
    files: CommunityProfileFiles,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct CommunityProfileFiles {
    license: Option<HashMap<String, Option<String>>>,
    readme: Option<HashMap<String, String>>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct Language {
    pub name: String,
    pub link: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct TechnologyStack {
    pub name: String,
    pub link: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct WorksData {
    pub repositories: Vec<Repository>,
    pub languages: Vec<Language>,
    pub technologies: Vec<TechnologyStack>,
}


impl Component for Works {
    type Message = WorkMessage;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|response: Response<Json<Result<WorksData, Error>>>| {
            if response.status().is_success() {
                match response.into_body() {
                    Json(Ok(s)) => return WorkMessage::FetchWorksData(s),
                    Json(Err(e)) => log::error!("error in fetching works.data.json: {:?}", e),
                }
            } else {
                log::error!("error in fetching works.data.json code: {}", response.status());
            }
            WorkMessage::None
        });
        let task = request("/works.data.9a90fd0b.json", callback);
        let callback = link.callback(|response: Response<Result<Vec<u8>, Error>>| {
            if response.status().is_success() {
                match response.into_body().map(String::from_utf8) {
                    Ok(Ok(s)) => return WorkMessage::FetchWorksSvg(s),
                    Ok(Err(e)) => log::error!("error in fetching works.graph.svg: {:?}", e),
                    Err(e) => log::error!("error in fetching works.graph.svg: {:?}", e),
                }
            } else {
                log::error!("error in fetching works.graph.svg code: {}", response.status());
            }
            WorkMessage::None
        });
        let tasks = [task, request("/works.graph.83f45361.svg", callback)];
        Self { link, tasks, works_data: None, works_svg: None, node_ref: Default::default() }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            WorkMessage::FetchWorksData(works) => {
                self.works_data = Some(works);
                self.render_svg();
                false
            }
            WorkMessage::FetchWorksSvg(svg) => {
                self.works_svg = Some(svg);
                self.render_svg();
                false
            }
            WorkMessage::None => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <h1 class="m-2">{"Works"}</h1>
                <div ref=self.node_ref.clone() class="m-2"/>
            </>
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        self.render_svg();
    }
}

impl Works {
    fn render_svg(&mut self) {
        let works_svg = if let Some(works_svg) = &self.works_svg { works_svg } else { return; };
        let works_data = if let Some(works_data) = &self.works_data { works_data } else { return; };
        let node = if let Some(node) = self.node_ref.get() { node } else { return; };
        if let Some(_) = node.first_child() { return; }
        let parser = web_sys::DomParser::new().expect("failed to construct DomParser");
        let doc = parser.parse_from_string(works_svg, web_sys::SupportedType::ImageSvgXml).expect("failed to parse svg");
        let doc = doc.document_element().expect("failed to get document_element");
        doc.set_attribute("width", "100%");
        node.append_child(doc.as_ref());
    }
}
