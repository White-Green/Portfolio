use std::collections::{BTreeMap, BTreeSet, HashMap};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::{RepositoryAdditionalInformation, TOKEN, USER_NAME};

const APP_NAME: &str = "works_generator";
const FETCH_SIZE: usize = 100;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Repository {
    pub html_url: String,
    pub name: String,
    pub homepage: Option<String>,
    pub language: RepositoryLanguage,
    pub community_profile: Option<CommunityProfile>,
    #[serde(default)]
    pub technology_stacks: RepositoryTechnologyStacks,
    #[serde(default)]
    pub related_repositories: RepositoryRelatedRepositories,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RepositoryLanguage {
    None,
    Single(String),
    StringList(HashMap<String, usize>),
    IndexList(Vec<(usize, usize)>),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RepositoryTechnologyStacks {
    None,
    StringList(Vec<String>),
    IndexList(Vec<usize>),
}

impl Default for RepositoryTechnologyStacks {
    fn default() -> Self {
        RepositoryTechnologyStacks::None
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RepositoryRelatedRepositories {
    None,
    StringList(Vec<String>),
    IndexList(Vec<usize>),
}

impl Default for RepositoryRelatedRepositories {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommunityProfile {
    health_percentage: usize,
    description: Option<String>,
    documentation: Option<String>,
    files: CommunityProfileFiles,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommunityProfileFiles {
    license: Option<HashMap<String, Option<String>>>,
    readme: Option<HashMap<String, String>>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub link: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechnologyStack {
    pub name: String,
    pub link: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Works {
    pub repositories: Vec<Repository>,
    pub languages: Vec<Language>,
    pub technologies: Vec<TechnologyStack>,
}

pub(crate) fn get_works(info: RepositoryAdditionalInformation) -> Works {
    let client = Client::new();
    let mut repositories = Vec::new();
    for i in 1.. {
        let mut result: Vec<Repository> = fetch_json(&client, &format!("/users/{}/repos?type=owner&per_page={}&page={}", USER_NAME.get().unwrap(), FETCH_SIZE, i));
        let result_len = result.len();
        repositories.append(&mut result);
        if result_len < FETCH_SIZE { break; }
    }
    let RepositoryAdditionalInformation { mut technology_stacks, mut repository_relations, technology_stack_info, language_info } = info;
    let mut repositories = repositories.into_iter()
        .filter_map(|mut repository| {
            let community_profile = get_community_profile(&client, &USER_NAME.get().unwrap(), &repository.name);
            if let None = community_profile.description { return None; }
            repository.community_profile = Some(community_profile);
            let languages = get_languages(&client, &USER_NAME.get().unwrap(), &repository.name);
            repository.language = RepositoryLanguage::StringList(languages);
            repository.technology_stacks = RepositoryTechnologyStacks::StringList(technology_stacks.remove(&repository.name).unwrap_or_default());
            repository.related_repositories = RepositoryRelatedRepositories::StringList(repository_relations.remove(&repository.name).unwrap_or_default());
            Some(repository)
        })
        .collect();
    let (repo, lang, tech) = create_index_map(&repositories);
    for r in &mut repositories {
        let list = if let RepositoryLanguage::StringList(list) = std::mem::replace(&mut r.language, RepositoryLanguage::None) {
            list.into_iter().map(|(k, v)| (*lang.get(&k).unwrap(), v)).collect()
        } else { unreachable!() };
        r.language = RepositoryLanguage::IndexList(list);
        let list = if let RepositoryTechnologyStacks::StringList(list) = std::mem::replace(&mut r.technology_stacks, RepositoryTechnologyStacks::None) {
            list.into_iter().map(|name| *tech.get(&name).unwrap()).collect()
        } else { unreachable!() };
        r.technology_stacks = RepositoryTechnologyStacks::IndexList(list);
        let list = if let RepositoryRelatedRepositories::StringList(list) = std::mem::replace(&mut r.related_repositories, RepositoryRelatedRepositories::None) {
            list.into_iter().map(|name| *repo.get(&name).unwrap()).collect()
        } else { unreachable!() };
        r.related_repositories = RepositoryRelatedRepositories::IndexList(list);
    }
    let mut technology_stack_info = technology_stack_info.into_iter().map(|tech| (tech.name.clone(), tech)).collect::<HashMap<_, _>>();
    let mut language_info = language_info.into_iter().map(|lang| (lang.name.clone(), lang)).collect::<HashMap<_, _>>();
    Works {
        repositories,
        languages: lang.into_iter().map(|(name, _)| language_info.remove(&name).unwrap_or(Language { name, ..Default::default() })).collect(),
        technologies: tech.into_iter().map(|(name, _)| technology_stack_info.remove(&name).unwrap_or(TechnologyStack { name, ..Default::default() })).collect(),
    }
}

fn create_index_map(repos: &Vec<Repository>) -> (BTreeMap<String, usize>, BTreeMap<String, usize>, BTreeMap<String, usize>) {
    let mut repository_name = BTreeMap::new();
    let mut language = BTreeSet::new();
    let mut technology_stack = BTreeSet::new();
    for repo in repos {
        repository_name.insert(repo.name.clone(), repository_name.len());
        match &repo.language {
            RepositoryLanguage::StringList(list) => for lang in list.keys() {
                if !language.contains(lang) { language.insert(lang.clone()); }
            }
            _ => unreachable!(),
        }
        match &repo.technology_stacks {
            RepositoryTechnologyStacks::StringList(list) => for tech in list {
                if !technology_stack.contains(tech) { technology_stack.insert(tech.clone()); }
            }
            _ => unreachable!()
        }
    }
    (repository_name, language.into_iter().enumerate().map(|(a, b)| (b, a)).collect(), technology_stack.into_iter().enumerate().map(|(a, b)| (b, a)).collect())
}

fn fetch_json<'a, T: DeserializeOwned>(client: &Client, url: &str) -> T {
    let response = client.get(format!("https://api.github.com{}", url))
        .header("User-Agent", APP_NAME)
        .header("accept", "application/vnd.github.v3+json")
        .basic_auth(USER_NAME.get().unwrap(), TOKEN.get())
        .send().expect("");
    let body = response.bytes().expect("");
    let mut body: &[u8] = &body;
    match serde_json::from_reader(&mut body) {
        Ok(result) => result,
        Err(e) => panic!("Error '{}' by '{}'", e, String::from_utf8_lossy(&body)),
    }
}


fn get_languages(client: &Client, username: &str, repository_name: &str) -> HashMap<String, usize> {
    fetch_json(client, &format!("/repos/{}/{}/languages", username, repository_name))
}

fn get_community_profile(client: &Client, username: &str, repository_name: &str) -> CommunityProfile {
    fetch_json(client, &format!("/repos/{}/{}/community/profile", username, repository_name))
}
