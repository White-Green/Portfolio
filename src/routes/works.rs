use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::RwLock;

use anyhow::Error;
use once_cell::sync::{Lazy, OnceCell};
use serde::Deserialize;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::{Element, HtmlElement, Node, SvgsvgElement};
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
    selected_node: SelectedNode,

    repository_nodes: Vec<Element>,
    language_nodes: Vec<Element>,
    technology_nodes: Vec<Element>,
    repository_language_edges: Vec<BTreeMap<usize, Element>>,
    repository_technology_edges: Vec<BTreeMap<usize, Element>>,
    repository_repository_edges: Vec<BTreeMap<usize, Element>>,

    repository_connected_languages: Vec<BTreeSet<usize>>,
    repository_connected_technologies: Vec<BTreeSet<usize>>,
    repository_connected_repositories: Vec<BTreeSet<usize>>,
    language_connected_repositories: Vec<BTreeSet<usize>>,
    technology_connected_repositories: Vec<BTreeSet<usize>>,
}

pub(crate) enum WorkMessage {
    FetchWorksData(WorksData),
    FetchWorksSvg(String),
    UpdateSelectedNode(SelectedNode),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SelectedNode {
    Repository(usize),
    Language(usize),
    Technology(usize),
    None,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Repository {
    pub html_url: String,
    pub name: String,
    pub homepage: Option<String>,
    pub language: Vec<(usize, usize)>,
    pub community_profile: CommunityProfile,
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
        Self {
            link,
            tasks,
            works_data: None,
            works_svg: None,
            node_ref: Default::default(),
            selected_node: SelectedNode::None,
            repository_nodes: Default::default(),
            language_nodes: Default::default(),
            technology_nodes: Default::default(),
            repository_language_edges: Default::default(),
            repository_technology_edges: Default::default(),
            repository_repository_edges: Default::default(),
            repository_connected_languages: Default::default(),
            repository_connected_technologies: Default::default(),
            repository_connected_repositories: Default::default(),
            language_connected_repositories: Default::default(),
            technology_connected_repositories: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            WorkMessage::FetchWorksData(works) => {
                self.construct_connection(&works);
                self.works_data = Some(works);
                self.render_svg();
                false
            }
            WorkMessage::FetchWorksSvg(svg) => {
                self.works_svg = Some(svg);
                self.render_svg();
                false
            }
            WorkMessage::UpdateSelectedNode(n) => {
                if self.selected_node == n { return false; }
                self.selected_node = n;
                self.coloring();
                true
            }
            WorkMessage::None => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let modal_title = match self.selected_node {
            SelectedNode::Repository(i) => self.works_data.as_ref().and_then(|w| w.repositories.get(i)).map(|r| r.name.as_str()).unwrap_or_default(),
            SelectedNode::Language(i) => self.works_data.as_ref().and_then(|w| w.languages.get(i)).map(|r| r.name.as_str()).unwrap_or_default(),
            SelectedNode::Technology(i) => self.works_data.as_ref().and_then(|w| w.technologies.get(i)).map(|r| r.name.as_str()).unwrap_or_default(),
            SelectedNode::None => "",
        };
        let url_regex = regex::Regex::new("^https?://").unwrap();
        let modal_body = match self.selected_node {
            SelectedNode::Repository(i) => if let Some(WorksData { repositories, languages, technologies }) = self.works_data.as_ref() {
                let repo = &repositories[i];
                html! {
                    <>
                        { repo.community_profile.description.as_ref().map(|d| html! {
                            <div class="h6 row">{ d }</div>
                        }).unwrap_or_default() }
                        <div class="row">
                            <a class="h6 col-12 col-sm-6" href=repo.html_url.as_str() target="_blank">{ "repository" }</a>
                            { repo.homepage.as_ref().and_then(|url|if url_regex.is_match(url) { Some(url) } else { None }).map(|p| html! {
                                <a class="h6 col" href=p.as_str() target="_blank">{ "homepage" }</a>
                            }).unwrap_or_default() }
                        </div>
                        { if !self.repository_connected_languages[i].is_empty() {
                            html!{
                                <div class="row">
                                    <div class="col-12 h5">{ "related languages:" }</div>
                                    { for self.repository_connected_languages[i].iter().map(|&i|html!{
                                        <button class="btn btn-secondary" onclick=self.link.callback(move|_|WorkMessage::UpdateSelectedNode(SelectedNode::Language(i)))>
                                            <span>{ languages[i].name.as_str() }</span>
                                        </button>
                                    }) }
                                </div>
                            }
                        } else { html!{} } }
                        { if !self.repository_connected_technologies[i].is_empty() {
                            html!{
                                <div class="row">
                                    <div class="col-12 h5">{ "related technologies:" }</div>
                                    { for self.repository_connected_technologies[i].iter().map(|&i|html!{
                                        <button class="btn btn-secondary" onclick=self.link.callback(move|_|WorkMessage::UpdateSelectedNode(SelectedNode::Technology(i)))>
                                            <span>{ technologies[i].name.as_str() }</span>
                                        </button>
                                    }) }
                                </div>
                            }
                        } else { html!{} } }
                        { if !self.repository_connected_repositories[i].is_empty() {
                            html!{
                                <div class="row">
                                    <div class="col-12 h5">{ "related repositories:" }</div>
                                    { for self.repository_connected_repositories[i].iter().map(|&i|html!{
                                        <button class="btn btn-secondary" onclick=self.link.callback(move|_|WorkMessage::UpdateSelectedNode(SelectedNode::Repository(i)))>
                                            <span>{ repositories[i].name.as_str() }</span>
                                        </button>
                                    }) }
                                </div>
                            }
                        } else { html!{} } }
                    </>
                }
            } else { html! {} },
            SelectedNode::Language(i) => if let Some(WorksData { repositories, languages, technologies }) = self.works_data.as_ref() {
                let lang = &languages[i];
                html! {
                    <>
                        { lang.link.as_ref().and_then(|url|if url_regex.is_match(url) { Some(url) } else { None }).map(|p| html! {
                            <a class="h6 row" href=p.as_str() target="_blank">{ "homepage" }</a>
                        }).unwrap_or_default() }
                        { if !self.language_connected_repositories[i].is_empty() {
                            html!{
                                <div class="row">
                                    <div class="col-12 h5">{ "related repositories:" }</div>
                                    { for self.language_connected_repositories[i].iter().map(|&i|html!{
                                        <button class="btn btn-secondary" onclick=self.link.callback(move|_|WorkMessage::UpdateSelectedNode(SelectedNode::Repository(i)))>
                                            <span>{ repositories[i].name.as_str() }</span>
                                        </button>
                                    }) }
                                </div>
                            }
                        } else { html!{} } }
                    </>
                }
            } else { html! {} },
            SelectedNode::Technology(i) => if let Some(WorksData { repositories, languages, technologies }) = self.works_data.as_ref() {
                let tech = &technologies[i];
                html! {
                    <>
                        { tech.description.as_ref().map(|d| html! {
                            <div class="h6 row">{ d }</div>
                        }).unwrap_or_default() }
                        { tech.link.as_ref().and_then(|url|if url_regex.is_match(url) { Some(url) } else { None }).map(|p| html! {
                            <a class="h6 row" href=p.as_str() target="_blank">{ "homepage" }</a>
                        }).unwrap_or_default() }
                        { if !self.technology_connected_repositories[i].is_empty() {
                            html!{
                                <div class="row">
                                    <div class="col-12 h5">{ "related repositories:" }</div>
                                    { for self.technology_connected_repositories[i].iter().map(|&i|html!{
                                        <button class="btn btn-secondary" onclick=self.link.callback(move|_|WorkMessage::UpdateSelectedNode(SelectedNode::Repository(i)))>
                                            <span>{ repositories[i].name.as_str() }</span>
                                        </button>
                                    }) }
                                </div>
                            }
                        } else { html!{} } }
                    </>
                }
            } else { html! {} },
            SelectedNode::None => html! {},
        };
        html! {
            <>
                <h1 class="m-2">{"Works"}</h1>
                <button type="button" class ="btn btn-secondary" data-toggle="modal" data-target="#exampleModal" disabled={self.selected_node == SelectedNode::None}>{ "show detail" }</button>
                <div class="modal" id="exampleModal" tabindex="-1" role="dialog" aria-labelledby="exampleModalLabel" aria-hidden="true">
                    <div class="modal-dialog modal-dialog-centered" role="document">
                        <div class="modal-content">
                            <div class="modal-header">
                                <h5 class="modal-title" id="exampleModalLabel">{ modal_title }</h5>
                                <button type="button" class="close" data-dismiss="modal" aria-label="Close">
                                    <span aria-hidden="true">{ "×" }</span>
                                </button>
                            </div>
                            <div class="modal-body">
                                <div class="container-fluid">
                                    { modal_body }
                                </div>
                            </div>
                            <div class="modal-footer">
                                <button type="button" class="btn btn-secondary" data-dismiss="modal">{ "close" }</button>
                            </div>
                        </div>
                    </div>
                </div>
                <div ref=self.node_ref.clone() class="m-2"/>
            </>
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        self.render_svg();
    }
}

impl Works {
    fn reset_all_color(&self) {
        self.repository_nodes.iter()
            .chain(self.language_nodes.iter())
            .chain(self.technology_nodes.iter())
            .chain(self.repository_language_edges.iter().map(|map| map.iter().map(|(_, e)| e)).flatten())
            .chain(self.repository_technology_edges.iter().map(|map| map.iter().map(|(_, e)| e)).flatten())
            .chain(self.repository_repository_edges.iter().map(|map| map.iter().map(|(_, e)| e)).flatten())
            .for_each(|element| {
                element.query_selector("polygon,path,ellipse").expect("failed querySelector").expect("failed to find polygon|path|ellipse")
                    .set_attribute("stroke", "lightgray");
            });
    }
    fn coloring(&self) {
        self.reset_all_color();
        fn set_color(element: &Element, color: &str) {
            element.query_selector("polygon,path,ellipse").expect("failed querySelector").expect("failed to find polygon|path|ellipse")
                .set_attribute("stroke", color);
        }
        const PRIMARY_COLOR: &str = "forestgreen";
        const SECONDARY_COLOR: &str = "black";
        const EDGE_COLOR: &str = "black";
        match &self.selected_node {
            &SelectedNode::Repository(i) => {
                set_color(&self.repository_nodes[i], PRIMARY_COLOR);
                for &j in &self.repository_connected_languages[i] {
                    set_color(&self.language_nodes[j], SECONDARY_COLOR);
                    set_color(&self.repository_language_edges[i][&j], EDGE_COLOR);
                }
                for &j in &self.repository_connected_technologies[i] {
                    set_color(&self.technology_nodes[j], SECONDARY_COLOR);
                    set_color(&self.repository_technology_edges[i][&j], EDGE_COLOR);
                }
                for &j in &self.repository_connected_repositories[i] {
                    set_color(&self.repository_nodes[j], SECONDARY_COLOR);
                    set_color(self.repository_repository_edges[i].get(&j).or(self.repository_repository_edges[j].get(&i)).unwrap(), EDGE_COLOR);
                }
            }
            &SelectedNode::Language(i) => {
                set_color(&self.language_nodes[i], PRIMARY_COLOR);
                for &j in &self.language_connected_repositories[i] {
                    set_color(&self.repository_nodes[j], SECONDARY_COLOR);
                    set_color(&self.repository_language_edges[j][&i], EDGE_COLOR);
                }
            }
            &SelectedNode::Technology(i) => {
                set_color(&self.technology_nodes[i], PRIMARY_COLOR);
                for &j in &self.technology_connected_repositories[i] {
                    set_color(&self.repository_nodes[j], SECONDARY_COLOR);
                    set_color(&self.repository_technology_edges[j][&i], EDGE_COLOR);
                }
            }
            SelectedNode::None => {}
        }
    }
    fn construct_connection(&mut self, works: &WorksData) {
        let mut repository_connected_languages = vec![BTreeSet::new(); works.repositories.len()];
        let mut repository_connected_technologies = vec![BTreeSet::new(); works.repositories.len()];
        let mut repository_connected_repositories = vec![BTreeSet::new(); works.repositories.len()];
        let mut language_connected_repositories = vec![BTreeSet::new(); works.languages.len()];
        let mut technology_connected_repositories = vec![BTreeSet::new(); works.technologies.len()];
        for (i, repository) in works.repositories.iter().enumerate() {
            for &(j, _) in &repository.language {
                repository_connected_languages[i].insert(j);
                language_connected_repositories[j].insert(i);
            }
            for &j in &repository.technology_stacks {
                repository_connected_technologies[i].insert(j);
                technology_connected_repositories[j].insert(i);
            }
            for &j in &repository.related_repositories {
                repository_connected_repositories[i].insert(j);
                repository_connected_repositories[j].insert(i);
            }
        }
        self.repository_connected_languages = repository_connected_languages;
        self.repository_connected_technologies = repository_connected_technologies;
        self.repository_connected_repositories = repository_connected_repositories;
        self.language_connected_repositories = language_connected_repositories;
        self.technology_connected_repositories = technology_connected_repositories;
    }
    fn construct_elements(&mut self, svg: &SvgsvgElement) {
        let WorksData { repositories, languages, technologies } = self.works_data.as_ref().unwrap();
        let mut repository_nodes = Vec::new();
        let mut language_nodes = Vec::new();
        let mut technology_nodes = Vec::new();
        let mut repository_language_edges = vec![BTreeMap::new(); repositories.len()];
        let mut repository_technology_edges = vec![BTreeMap::new(); repositories.len()];
        let mut repository_repository_edges = vec![BTreeMap::new(); repositories.len()];
        for i in 0..repositories.len() {
            let element = svg.get_element_by_id(&format!("repository{}", i)).expect("cannot find repository node");
            repository_nodes.push(element);
        }
        for i in 0..languages.len() {
            let element = svg.get_element_by_id(&format!("language{}", i)).expect("cannot find language node");
            language_nodes.push(element);
        }
        for i in 0..technologies.len() {
            let element = svg.get_element_by_id(&format!("technology{}", i)).expect("cannot find technology node");
            technology_nodes.push(element);
        }
        for (i, repository) in repositories.iter().enumerate() {
            if !repository.language.is_empty() {
                let map = repository.language.iter().map(|(j, _)| {
                    let element = svg.get_element_by_id(&format!("repository{}_language{}", i, j)).expect("cannot find repository_language edge");
                    (*j, element)
                }).collect();
                repository_language_edges[i] = map;
            }
            if !repository.technology_stacks.is_empty() {
                let map = repository.technology_stacks.iter().map(|j| {
                    let element = svg.get_element_by_id(&format!("repository{}_technology{}", i, j)).expect("cannot find repository_technology edge");
                    (*j, element)
                }).collect();
                repository_technology_edges[i] = map;
            }
            if !repository.related_repositories.is_empty() {
                let map = repository.related_repositories.iter().map(|j| {
                    let element = svg.get_element_by_id(&format!("repository{}_repository{}", i, j)).expect("cannot find repository_technology edge");
                    (*j, element)
                }).collect();
                repository_repository_edges[i] = map;
            }
        }
        self.repository_nodes = repository_nodes;
        self.language_nodes = language_nodes;
        self.technology_nodes = technology_nodes;
        self.repository_language_edges = repository_language_edges;
        self.repository_technology_edges = repository_technology_edges;
        self.repository_repository_edges = repository_repository_edges;
    }
    fn set_action(&mut self, svg: &SvgsvgElement) {
        self.construct_elements(svg);
        let callback = {
            let callback = self.link.callback(|n| WorkMessage::UpdateSelectedNode(n));
            Closure::wrap(Box::new(move |event: MouseEvent| {
                let event_target = if let Some(event_target) = event.target() { event_target } else { return; };
                let mut element = if let Ok(element) = event_target.dyn_into::<Element>() { element } else { return; };
                let re = regex::Regex::new("^(?P<type>repository|language|technology)(?P<index>\\d+)$").unwrap();
                while !re.is_match(&element.id()) {
                    if let Some(p) = element.parent_element() {
                        element = p;
                    } else {
                        return;
                    }
                }
                let id = element.id();
                let captures = re.captures(&id).unwrap();
                let i: usize = captures.name("index").unwrap().as_str().parse().unwrap();
                match captures.name("type").unwrap().as_str() {
                    "repository" => callback.emit(SelectedNode::Repository(i)),
                    "language" => callback.emit(SelectedNode::Language(i)),
                    "technology" => callback.emit(SelectedNode::Technology(i)),
                    _ => unreachable!(),
                }
            }) as Box<dyn Fn(_)>)
        };
        self.repository_nodes.iter().for_each(|element| element.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref()).expect("failed to add click event listener"));
        self.language_nodes.iter().for_each(|element| element.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref()).expect("failed to add click event listener"));
        self.technology_nodes.iter().for_each(|element| element.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref()).expect("failed to add click event listener"));
        callback.forget();
    }
    fn render_svg(&mut self) {
        let works_svg = if let Some(works_svg) = &self.works_svg { works_svg } else { return; };
        let works_data = if let Some(works_data) = &self.works_data { works_data } else { return; };
        let node = if let Some(node) = self.node_ref.get() { node } else { return; };
        if let Some(_) = node.first_child() { return; }
        let parser = web_sys::DomParser::new().expect("failed to construct DomParser");
        let doc = parser.parse_from_string(works_svg, web_sys::SupportedType::ImageSvgXml).expect("failed to parse svg");
        let doc = doc.root_element().expect("failed to get document_element");
        doc.set_attribute("width", "100%");
        self.set_action(&doc);

        node.append_child(doc.as_ref());
    }
}
