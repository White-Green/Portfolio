use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};

use clap::Arg;
use once_cell::sync::OnceCell;
use serde::Deserialize;

use crate::repository_list::{get_works, Language, Repository, RepositoryLanguage, RepositoryRelatedRepositories, RepositoryTechnologyStacks, TechnologyStack, Works};

mod repository_list;

static USER_NAME: OnceCell<String> = OnceCell::new();
static TOKEN: OnceCell<String> = OnceCell::new();

fn main() {
    let matches = clap::App::new("works_generator")
        .arg(Arg::with_name("username")
            .short("n")
            .long("username")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("token")
            .short("t")
            .long("token")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("data_output")
            .short("d")
            .long("data")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("graph_output")
            .short("g")
            .long("graph")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("additional_information")
            .short("i")
            .long("info")
            .takes_value(true)
            .required(false))
        .get_matches();
    let username = matches.value_of("username").unwrap();
    USER_NAME.set(username.to_owned()).unwrap();
    let token = matches.value_of("token");
    if let Some(token) = token {
        TOKEN.set(token.to_owned()).unwrap();
    }

    let info = get_additional_information(matches.value_of("additional_information"));
    let works = get_works(info);
    write_graph(&works, matches.value_of("graph_output"));
    write_works(&works, matches.value_of("data_output"));
}

fn write_or_print(data: &str, path: Option<&str>) {
    if let Some(path) = path {
        if let Ok(_) = std::fs::write(path, data) {
            return;
        }
    }
    println!("{}", data);
}

fn write_works(works: &Works, path: Option<&str>) {
    write_or_print(&serde_json::to_string(works).expect("failed to serialize"), path);
}

fn write_graph(works: &Works, path: Option<&str>) {
    write_or_print(&to_graph(works), path);
}

fn to_graph(works: &Works) -> String {
    let mut result = String::new();
    let Works { repositories: repos, languages: lang, technologies: tech } = works;
    result.push_str("graph{graph[rankdir=LR];node[style=\"filled\",color=\"lightgray\"];edge[color=\"lightgray\"];");
    {
        result.push_str("subgraph language{node[shape=box];");
        for (i, Language { name, .. }) in lang.iter().enumerate() {
            result.push_str(&format!("language{}[label=\"{}\",id=\"language{0}\"];", i, name));
        }
        result.push_str("{rank=min;");
        for i in 0..lang.len() {
            result.push_str(&format!("language{};", i));
        }
        result.push_str("}");
        result.push_str("}");
    }
    {
        result.push_str("subgraph repository{node[];");
        for (i, Repository { name, .. }) in repos.iter().enumerate() {
            result.push_str(&format!("repository{}[label=\"{}\",id=\"repository{0}\"];", i, name));
        }
        result.push_str("{rank=same;");
        for i in 0..repos.len() {
            result.push_str(&format!("repository{};", i));
        }
        result.push_str("}");
        result.push_str("}");
    }
    {
        result.push_str("subgraph technology{node[shape=octagon];");
        for (i, TechnologyStack { name, .. }) in tech.iter().enumerate() {
            result.push_str(&format!("technology{}[label=\"{}\",id=\"technology{0}\"];", i, name));
        }
        result.push_str("{rank=max;");
        for i in 0..tech.len() {
            result.push_str(&format!("technology{};", i));
        }
        result.push_str("}");
        result.push_str("}");
    }

    for (i, repo) in repos.iter().enumerate() {
        match &repo.language {
            RepositoryLanguage::IndexList(list) => {
                for (lang, _) in list {
                    result.push_str(&format!("repository{}--language{}[id=\"repository{0}_language{1}\"];", i, lang));
                }
            }
            _ => unreachable!(),
        }
        match &repo.technology_stacks {
            RepositoryTechnologyStacks::IndexList(list) => for tech in list {
                result.push_str(&format!("repository{}--technology{}[id=\"repository{0}_technology{1}\"];", i, tech));
            }
            _ => unreachable!()
        }
        match &repo.related_repositories {
            RepositoryRelatedRepositories::IndexList(list) => for repo in list {
                result.push_str(&format!("repository{}--repository{}[id=\"repository{0}_repository{1}\"];", i, repo));
            }
            _ => unreachable!()
        }
    }

    result.push_str("}");
    result
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
struct RepositoryAdditionalInformation {
    technology_stacks: HashMap<String, Vec<String>>,
    repository_relations: HashMap<String, Vec<String>>,
    technology_stack_info: Vec<TechnologyStack>,
    language_info: Vec<Language>,
}

fn get_additional_information(path: Option<&str>) -> RepositoryAdditionalInformation {
    let path = if let Some(path) = path { path } else {
        eprintln!("additional information file path is None");
        return Default::default();
    };
    let file = match std::fs::read(path) {
        Ok(vec) => vec,
        Err(e) => {
            eprintln!("failed to read from additional information file by {:?}", e);
            return Default::default();
        }
    };
    serde_json::from_slice(&file)
        .map(|info: RepositoryAdditionalInformation| {
            RepositoryAdditionalInformation { repository_relations: simplify(info.repository_relations), ..info }
        })
        .unwrap_or_else(|e| {
            eprintln!("failed to parse additional information stack file by {:?}", e);
            Default::default()
        })
}

fn simplify(map: HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>> {
    map.into_iter()
        .map(|(k, v)|
            v.into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .zip(Some(k).into_iter().cycle())
                .filter_map(|(a, b)| {
                    match std::cmp::Ord::cmp(&a, &b) {
                        Ordering::Less => Some((a, b)),
                        Ordering::Greater => Some((b, a)),
                        Ordering::Equal => None,
                    }
                }))
        .flatten()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .fold(HashMap::new(), |mut map, next| {
            let (key, val) = next;
            map.entry(key)
                .or_insert_with(|| HashSet::new())
                .insert(val);
            map
        })
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect()
}
