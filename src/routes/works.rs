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
use web_sys::{Element, Node, SvgsvgElement};
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
    repository_nodes: Rc<RefCell<Vec<Element>>>,
    language_nodes: Rc<RefCell<Vec<Element>>>,
    technology_nodes: Rc<RefCell<Vec<Element>>>,
    repository_language_edges: Rc<RefCell<Vec<BTreeMap<usize, Element>>>>,
    repository_technology_edges: Rc<RefCell<Vec<BTreeMap<usize, Element>>>>,
    repository_repository_edges: Rc<RefCell<Vec<BTreeMap<usize, Element>>>>,

    repository_connected_languages: Vec<BTreeSet<usize>>,
    repository_connected_technologies: Vec<BTreeSet<usize>>,
    repository_connected_repositories: Vec<BTreeSet<usize>>,
    language_connected_repositories: Vec<BTreeSet<usize>>,
    technology_connected_repositories: Vec<BTreeSet<usize>>,
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
        Self {
            link,
            tasks,
            works_data: None,
            works_svg: None,
            node_ref: Default::default(),
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

fn reset_all_color(repository_nodes: impl Deref<Target=Vec<Element>>,
                   language_nodes: impl Deref<Target=Vec<Element>>,
                   technology_nodes: impl Deref<Target=Vec<Element>>,
                   repository_language_edges: impl Deref<Target=Vec<BTreeMap<usize, Element>>>,
                   repository_technology_edges: impl Deref<Target=Vec<BTreeMap<usize, Element>>>,
                   repository_repository_edges: impl Deref<Target=Vec<BTreeMap<usize, Element>>>) {
    repository_nodes.iter()
        .chain(language_nodes.iter())
        .chain(technology_nodes.iter())
        .chain(repository_language_edges.iter().map(|map| map.iter().map(|(_, e)| e)).flatten())
        .chain(repository_technology_edges.iter().map(|map| map.iter().map(|(_, e)| e)).flatten())
        .chain(repository_repository_edges.iter().map(|map| map.iter().map(|(_, e)| e)).flatten())
        .for_each(|element| {
            element.query_selector("polygon,path,ellipse").expect("failed querySelector").expect("failed to find polygon|path|ellipse")
                .set_attribute("stroke", "lightgray");
        });
}

impl Works {
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
        *self.repository_nodes.borrow_mut() = repository_nodes;
        *self.language_nodes.borrow_mut() = language_nodes;
        *self.technology_nodes.borrow_mut() = technology_nodes;
        *self.repository_language_edges.borrow_mut() = repository_language_edges;
        *self.repository_technology_edges.borrow_mut() = repository_technology_edges;
        *self.repository_repository_edges.borrow_mut() = repository_repository_edges;
    }
    fn set_action(&mut self, svg: &SvgsvgElement) {
        self.construct_elements(svg);
        let callback = {
            let repository_nodes = Rc::clone(&self.repository_nodes);
            let language_nodes = Rc::clone(&self.language_nodes);
            let technology_nodes = Rc::clone(&self.technology_nodes);
            let repository_language_edges = Rc::clone(&self.repository_language_edges);
            let repository_technology_edges = Rc::clone(&self.repository_technology_edges);
            let repository_repository_edges = Rc::clone(&self.repository_repository_edges);
            let repository_connected_languages = std::mem::take(&mut self.repository_connected_languages);
            let repository_connected_technologies = std::mem::take(&mut self.repository_connected_technologies);
            let repository_connected_repositories = std::mem::take(&mut self.repository_connected_repositories);
            let language_connected_repositories = std::mem::take(&mut self.language_connected_repositories);
            let technology_connected_repositories = std::mem::take(&mut self.technology_connected_repositories);
            Closure::wrap(Box::new(move |event: MouseEvent| {
                // web_sys::console::log_1(event.as_ref());
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
                reset_all_color(repository_nodes.borrow(),
                                language_nodes.borrow(),
                                technology_nodes.borrow(),
                                repository_language_edges.borrow(),
                                repository_technology_edges.borrow(),
                                repository_repository_edges.borrow());
                let repository_nodes = repository_nodes.borrow();
                let language_nodes = language_nodes.borrow();
                let technology_nodes = technology_nodes.borrow();
                let repository_language_edges = repository_language_edges.borrow();
                let repository_technology_edges = repository_technology_edges.borrow();
                let repository_repository_edges = repository_repository_edges.borrow();
                let id = element.id();
                let captures = re.captures(&id).unwrap();
                fn set_color(element: &Element, color: &str) {
                    element.query_selector("polygon,path,ellipse").expect("failed querySelector").expect("failed to find polygon|path|ellipse")
                        .set_attribute("stroke", color);
                }
                let i: usize = captures.name("index").unwrap().as_str().parse().unwrap();
                const PRIMARY_COLOR: &str = "forestgreen";
                const SECONDARY_COLOR: &str = "black";
                const EDGE_COLOR: &str = "black";
                match captures.name("type").unwrap().as_str() {
                    "repository" => {
                        set_color(&repository_nodes[i], PRIMARY_COLOR);
                        for &j in &repository_connected_languages[i] {
                            set_color(&language_nodes[j], SECONDARY_COLOR);
                            set_color(&repository_language_edges[i][&j], EDGE_COLOR);
                        }
                        for &j in &repository_connected_technologies[i] {
                            set_color(&technology_nodes[j], SECONDARY_COLOR);
                            set_color(&repository_technology_edges[i][&j], EDGE_COLOR);
                        }
                        for &j in &repository_connected_repositories[i] {
                            set_color(&repository_nodes[j], SECONDARY_COLOR);
                            set_color(repository_repository_edges[i].get(&j).or(repository_repository_edges[j].get(&i)).unwrap(), EDGE_COLOR);
                        }
                    }
                    "language" => {
                        set_color(&language_nodes[i], PRIMARY_COLOR);
                        for &j in &language_connected_repositories[i] {
                            set_color(&repository_nodes[j], SECONDARY_COLOR);
                            set_color(&repository_language_edges[j][&i], EDGE_COLOR);
                        }
                    }
                    "technology" => {
                        set_color(&technology_nodes[i], PRIMARY_COLOR);
                        for &j in &technology_connected_repositories[i] {
                            set_color(&repository_nodes[j], SECONDARY_COLOR);
                            set_color(&repository_technology_edges[j][&i], EDGE_COLOR);
                        }
                    }
                    _ => unreachable!(),
                }
            }) as Box<dyn Fn(_)>)
        };
        self.repository_nodes.borrow().iter().for_each(|element| element.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref()).expect("failed to add click event listener"));
        self.language_nodes.borrow().iter().for_each(|element| element.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref()).expect("failed to add click event listener"));
        self.technology_nodes.borrow().iter().for_each(|element| element.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref()).expect("failed to add click event listener"));
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
