use anyhow::Error;
use serde::Deserialize;
use yew::format::Json;
use yew::prelude::*;
use yew::services::fetch::*;

use crate::routes::request;

#[derive(Clone, Debug)]
enum ProfileDataList {
    Raw(Vec<ProfileData>),
    Encrypted(Vec<ProfileData>),
}

impl ProfileDataList {
    fn data(&self) -> &Vec<ProfileData> {
        match self {
            ProfileDataList::Raw(d) | ProfileDataList::Encrypted(d) => d
        }
    }
}

#[derive(Debug)]
pub(crate) struct Profile {
    props: ProfileProperties,
    link: ComponentLink<Self>,
    profile_data: ProfileDataList,
    profile_data_enc: Vec<u8>,
    task: (FetchTask, FetchTask),
}

#[derive(Clone, Debug)]
pub(crate) enum ProfileMessage {
    FetchProfileData(Vec<ProfileData>),
    FetchProfileDataEnc(Vec<u8>),
    None,
}

#[derive(Clone, Debug, Default, PartialEq, Properties)]
pub(crate) struct ProfileProperties {
    #[prop_or_default]
    pub(crate) encrypt_key: Option<aes::Key>
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct RubyString {
    value: String,
    ruby: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
enum WrappedString {
    None,
    Normal(String),
    WithRuby(Vec<RubyString>),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct ProfileValueData {
    key: String,
    value: WrappedString,
    status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct ProfileData {
    category_name: String,
    values: Vec<ProfileValueData>,
}

impl Component for Profile {
    type Message = ProfileMessage;
    type Properties = ProfileProperties;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|response: Response<Json<Result<Vec<ProfileData>, Error>>>| {
            if response.status().is_success() {
                match response.into_body() {
                    Json(Ok(s)) => {
                        ProfileMessage::FetchProfileData(s)
                    }
                    Json(Err(e)) => {
                        log::error!("error in fetching profile.data.json: {:?}", e);
                        ProfileMessage::None
                    }
                }
            } else {
                log::error!("error in fetching profile.data.json code: {}", response.status());
                ProfileMessage::None
            }
        });
        let task_json = request("/profile.data.a36e7575.json", callback);
        let callback = link.callback(|response: Response<Result<Vec<u8>, Error>>| {
            if response.status().is_success() {
                match response.into_body() {
                    Ok(data) => ProfileMessage::FetchProfileDataEnc(data),
                    Err(e) => {
                        log::error!("error in fetching profile.data.enc.bin: {:?}", e);
                        ProfileMessage::None
                    }
                }
            } else {
                log::error!("error in fetching profile.data.enc.bin code: {}", response.status());
                ProfileMessage::None
            }
        });
        let task_enc = request("/profile.data.enc.4cdf9278.bin", callback);
        Self { props, link, profile_data: ProfileDataList::Raw(Vec::new()), profile_data_enc: Vec::new(), task: (task_json, task_enc) }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            ProfileMessage::FetchProfileData(s) => {
                match &mut self.profile_data {
                    ProfileDataList::Raw(d) => {
                        *d = s;
                        true
                    }
                    ProfileDataList::Encrypted(_) => false
                }
            }
            ProfileMessage::FetchProfileDataEnc(s) => {
                self.profile_data_enc = s;
                self.try_decrypt()
            }
            ProfileMessage::None => false
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props != props {
            self.props = props;
            self.try_decrypt();
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let data = self.profile_data.data().iter()
            .map(|data| {
                let values = data.values.iter()
                    .map(|data| {
                        let value: Html = match &data.value {
                            WrappedString::None => html! {},
                            WrappedString::Normal(s) => html! {{s}},
                            WrappedString::WithRuby(s) => html! {
                                { for s.iter()
                                    .map(|RubyString { value, ruby }| html! {
                                        <ruby>{value}<rt>{ruby}</rt></ruby>
                                    }) }
                            }
                        };
                        if let Some(status) = &data.status {
                            html! {
                                <div class="row mt-3">
                                    <div class="h4 col-12 col-md-3">
                                        { &data.key }
                                    </div>
                                    <div class="h4 col-12 col-sm-8 col-md-6 ml-3 ml-md-0">
                                        { value }
                                    </div>
                                    <div class="h4 col-12 col-sm-3 ml-3 ml-sm-0">
                                        { status }
                                    </div>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="row mt-3">
                                    <div class="h4 col-12 col-md-3">
                                        { &data.key }
                                    </div>
                                    <div class="h4 col-12 col-md-9">
                                        { value }
                                    </div>
                                </div>
                            }
                        }
                    });
                html! {
                    <>
                        <h3 class="mt-5">
                            { &data.category_name }
                        </h3>
                        { for values }
                    </>
                }
            });

        html! {
            <>
                { for data }
            </>
        }
    }
}

impl Profile {
    fn try_decrypt(&mut self) -> bool {
        if let Some(key) = &self.props.encrypt_key {
            let mut decrypted = aes::decrypt(&self.profile_data_enc, key);
            while decrypted.last() == Some(&0) { decrypted.pop(); }
            match Json::<Result<Vec<ProfileData>, Error>>::from(Ok(decrypted)) {
                Json(Ok(data)) => {
                    match &mut self.profile_data {
                        ProfileDataList::Raw(_) => {
                            self.profile_data = ProfileDataList::Encrypted(data);
                            true
                        }
                        ProfileDataList::Encrypted(d) => {
                            if d == &data {
                                false
                            } else {
                                *d = data;
                                true
                            }
                        }
                    }
                }
                Json(Err(_)) => false
            }
        } else {
            false
        }
    }
}
