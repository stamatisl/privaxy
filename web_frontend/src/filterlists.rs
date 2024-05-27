
use crate::save_button::BASE_BUTTON_CSS;
use reqwasm::http::Request;

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use wasm_bindgen_futures::spawn_local;
use yew::{html, Component, Context, Html};
use yew::prelude::*;
use web_sys::HtmlInputElement;
use yew::InputEvent;
use yew::html::Scope;


#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilterEntry {
    id: u64,
    name: String,
    #[serde(default = "default_description")]
    description: String,
    license_id: u64,
    syntax_ids: Vec<u64>,
    language_ids: Vec<u64>,
    tag_ids: Vec<u64>,
    #[serde(default)]
    primary_view_url: Option<String>,
    maintainer_ids: Vec<u64>,
}
fn default_description() -> String {
    "No description".to_string()
}

type FilterLists = Vec<FilterEntry>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterViewURL {
    segment_number: u32,
    primariness: u32,
    url: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterListDetail {
    id: u64,
    name: String,
    description: String,
    license_id: u64,
    syntax_ids: Vec<u64>,
    language_ids: Vec<u64>,
    tag_ids: Vec<u64>,
    primary_view_url: String,
    maintainer_ids: Vec<u64>,
    view_urls: Vec<FilterViewURL>,
    home_url: String,
    onion_url: String,
    policy_url: String,
    submission_url: String,
    issues_url: String,
    forum_url: String,
    chat_url: String,
    email_address: String,
    donate_url: String,
    upstream_filter_list_ids: Vec<u64>,
    include_in_filter_list_ids: Vec<u64>,
    includes_filter_list_ids: Vec<u64>,
    dependency_filter_list_ids: Vec<u64>,
    dependent_filter_list_ids: Vec<u64>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterSoftware {
    id: u64,
    name: String,
    #[serde(default)]
    home_url: Option<String>,
    #[serde(default)]
    download_url: Option<String>,
    supports_abp_url_scheme: bool,
    syntax_ids: Vec<u64>,
}

type FilterSoftwareList = Vec<FilterSoftware>;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilterListSyntax {
    id: u64,
    name: String,
    url: String,
    filter_list_ids: Vec<u64>,
    software_ids: Vec<u64>,
}

type FilterListSyntaxes = Vec<FilterListSyntax>;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilterLicense {
    id: u64,
    name: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    permit_modifications: Option<bool>,
    #[serde(default)]
    permit_distribution: Option<bool>,
    #[serde(default)]
    permit_commercial_use: Option<bool>,
    filter_list_ids: Vec<u64>,
}

type FilterLicenses = Vec<FilterLicense>;

pub enum SearchFilterMessage {
    Open,
    Close,
    Save,
    FilterChanged(String),
    SelectFilter(Option<FilterEntry>),
    LoadFilters,
    FiltersLoaded(FilterEntry),
    Error(String),
    NextPage,
    PreviousPage,
    LanguagesLoaded(FilterLanguages),
    LicensesLoaded(FilterLicenses),
}

pub struct SearchFilterList {
    link: yew::html::Scope<Self>,
    is_open: bool,
    filters: FilterLists,
    selected_filter: Option<FilterEntry>,
    filter_query: String,
    loading: bool,
    languages: FilterLanguages,
    licenses: FilterLicenses,
    current_page: usize,
    results_per_page: usize,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilterLanguage {
    id: u64,
    iso6391: String,
    name: String,
    filter_list_ids: Vec<u64>
}

type FilterLanguages = Vec<FilterLanguage>;



impl Component for SearchFilterList {
    type Message = SearchFilterMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            link: _ctx.link().clone(),
            is_open: false,
            filters: FilterLists::new(),
            languages: FilterLanguages::new(),
            licenses: FilterLicenses::new(),
            selected_filter: None,
            filter_query: String::new(),
            loading: true,
            current_page: 1,
            results_per_page: 10,
        }
    }
    

    fn update(&mut self, _ctx: &Context<Self>, msg: SearchFilterMessage) -> bool {
        match msg {
            SearchFilterMessage::Open => {
                self.is_open = true;
                self.link.send_message(SearchFilterMessage::LoadFilters);
            },
            SearchFilterMessage::Close => self.is_open = false,
            SearchFilterMessage::FilterChanged(query) => self.filter_query = query,
            SearchFilterMessage::SelectFilter(filter) => self.selected_filter = filter,
            SearchFilterMessage::LoadFilters => {
                if self.loading {
                    let link = self.link.clone();
                spawn_local(async move {
                    match fetch_languages(link.clone()).await {
                        Ok(_) => log::info!("Languages loaded successfully"),
                        Err(err) => link.send_message(SearchFilterMessage::Error(err)),
                    };
                    match fetch_filter_lists(link.clone()).await {
                        Ok(_) => log::info!("Filters loaded successfully"),
                        Err(err) => link.send_message(SearchFilterMessage::Error(err)),
                    };
                    match fetch_licenses(link.clone()).await {
                        Ok(_) => log::info!("Licenses loaded successfully"),
                        Err(err) => link.send_message(SearchFilterMessage::Error(err)),
                    };
                });
            }
            },
            SearchFilterMessage::FiltersLoaded(filter) => {
                self.filters.push(filter);
                self.loading = false;
            },
            SearchFilterMessage::LanguagesLoaded(langs ) => {
                self.languages = langs.clone();
            },
            SearchFilterMessage::LicensesLoaded(licenses) => {
                self.licenses = licenses.clone();
            },
            SearchFilterMessage::Error(error) => {
                log::error!("Error loading filters: {}", error);
                self.loading = false;
            },
            SearchFilterMessage::Save => {
                if let Some(filter) = &self.selected_filter {
                    // todo: add logic to save the selected filter to the database using the API
                    // similar to the `AddFilterComponent` save logic
                }
                self.is_open = false;
            },
            SearchFilterMessage::NextPage => {
                if self.current_page < (self.filters.len() as f64 / self.results_per_page as f64).ceil() as usize {
                    self.current_page += 1;
                }
            },
            SearchFilterMessage::PreviousPage => {
                if self.current_page > 1 {
                    self.current_page -= 1;
                }
            },
        }
        true
    }
    

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let save_button_classes = classes!(
            BASE_BUTTON_CSS.clone().to_vec(),
            "focus:ring-green-500",
            "bg-green-600",
            "hover:bg-green-700",
        );
        let search_button_classes = classes!(
            BASE_BUTTON_CSS.clone().to_vec(),
            "bg-blue-600",
            "hover:bg-blue-700",
        );
    
        
        let filtered_filters: Vec<&FilterEntry> = self.filters.iter()
        .filter(|filter| filter.name.contains(&self.filter_query))
        .collect();
        let total_pages = (filtered_filters.len() as f64 / self.results_per_page as f64).ceil() as usize;
        let start_index = (self.current_page - 1) * self.results_per_page;
        let paginated_filters = filtered_filters.into_iter().skip(start_index).take(self.results_per_page);
        let cancel_button_classes = classes!(
            BASE_BUTTON_CSS.clone().to_vec(),
            "focus:ring-red-500",
            "bg-red-600",
            "hover:bg-red-700",
        );
        let prev_button_classes = classes!(
            "bg-gray-500",
            "hover:bg-gray-700",
            "text-white",
            "font-bold",
            "py-2",
            "px-4",
            "rounded",
            if self.current_page == 1 { "opacity-50 cursor-not-allowed" } else { "" },
        );
    
        let next_button_classes = classes!(
            "bg-gray-500",
            "hover:bg-gray-700",
            "text-white",
            "font-bold",
            "py-2",
            "px-4",
            "rounded",
            if self.current_page == total_pages { "opacity-50 cursor-not-allowed" } else { "" },
        );
        let document = gloo_utils::document();
        if let Some(body) = document.body() {
            body.set_class_name(if self.is_open { "modal-open" } else { "" });
        }
        
        html! {
            <>
            <button onclick={self.link.callback(|_| SearchFilterMessage::Open)} class={classes!(search_button_classes, "mt-5")}>
                <svg xmlns="http://www.w3.org/2000/svg" class="-ml-0.5 mr-2 h-5 w-5" fill="none" viewBox="0 0 490.4 490.4" stroke="currentColor">
                    <path fill="white" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M484.1,454.796l-110.5-110.6c29.8-36.3,47.6-82.8,47.6-133.4c0-116.3-94.3-210.6-210.6-210.6S0,94.496,0,210.796 s94.3,210.6,210.6,210.6c50.8,0,97.4-18,133.8-48l110.5,110.5c12.9,11.8,25,4.2,29.2,0C492.5,475.596,492.5,463.096,484.1,454.796z M41.1,210.796c0-93.6,75.9-169.5,169.5-169.5s169.6,75.9,169.6,169.5s-75.9,169.5-169.5,169.5S41.1,304.396,41.1,210.796z"/>
                </svg>
            {"Search filterlist.com"}
            </button>
                { if self.is_open {
                    html! {
                        <div class="fixed inset-0 bg-gray-600 bg-opacity-75 flex items-center justify-center z-50">
                            <div class="bg-white p-6 rounded-lg shadow-lg z-60" style="width: 50vw; height: 80vh; overflow: hidden;">
                                <div class="flex flex-col space-y-4" style="height: 100%;">
                                    <input type="text" placeholder="Search by name" class="border border-gray-300 p-2 rounded"
                                        value={self.filter_query.clone()}
                                        oninput={_ctx.link().callback(|e: InputEvent| {
                                            let input = e.target_dyn_into::<HtmlInputElement>().expect("input element");
                                            SearchFilterMessage::FilterChanged(input.value())
                                        })}
                                    />
                                    <div style="flex-grow: 1; overflow: auto;">
                                        <table class="table-fixed bg-white">
                                            <thead>
                                                <tr style="height: 5vh;">
                                                    <th class="py-2" style="width: 5vw;">{"Name"}</th>
                                                    <th class="py-2" style="width: 10vw;">{"Description"}</th>
                                                    <th class="py-2" style="width: 8vw;">{"Language"}</th>
                                                    <th class="py-2" style="width: 8vw;">{"License"}</th>
                                                    <th class="py-2" style="width: 2vw;">{"Select"}</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                { for paginated_filters.map(|filter| self.view_filter_row(filter)) }
                                            </tbody>
                                        </table>
                                    </div>
                                    <div class="flex justify-between mt-4">
                                        <button 
                                            onclick={self.link.callback(|_| SearchFilterMessage::PreviousPage)} 
                                            class={prev_button_classes}
                                            disabled={self.current_page == 1}>
                                            {"Previous"}
                                        </button>
                                        <span>{"Page "} {self.current_page} {" of "} {total_pages}</span>
                                        <button 
                                            onclick={self.link.callback(|_| SearchFilterMessage::NextPage)} 
                                            class={next_button_classes}
                                            disabled={self.current_page == total_pages}>
                                            {"Next"}
                                        </button>
                                    </div>
                                    <div class="flex space-x-4">
                                        <button onclick={_ctx.link().callback(|_| SearchFilterMessage::Save)} class={save_button_classes.clone()} disabled={self.selected_filter.is_none()}>
                                            {"Save"}
                                        </button>
                                        <button onclick={_ctx.link().callback(|_| SearchFilterMessage::Close)} class={cancel_button_classes}>
                                            {"Cancel"}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </>
        }
    }
    
    
}


impl SearchFilterList {
    fn view_filter_row(&self, filter: &FilterEntry) -> Html {
        let selected = self.selected_filter.as_ref().map(|f| f.id) == Some(filter.id);
        let filter_clone = filter.clone();

        html! {
            <tr>
                <td class="border px-4 py-2 overflow-hidden" style="height: 5vh; white-space: normal; text-overflow: ellipsis;">
                    { if let Some(url) = &filter.primary_view_url {
                        html! { <a href={url.clone()} target="_blank" class="text-blue-600 underline"> { &filter.name } </a> }
                    } else {
                        html! { &filter.name }
                    }}
                </td>
                <td class="border px-4 py-2 overflow-auto" style="height: 5vh; max-width: 10vw; white-space: normal;">
                    { &filter.description }
                </td>
                <td class="border px-4 py-2 overflow-hidden" style="height: 5vh; white-space: nowrap; text-overflow: ellipsis;">
                    { self.get_language_name(filter.language_ids.clone()) }
                </td>
                <td class="border px-4 py-2 overflow-hidden" style="height: 5vh; white-space: nowrap; text-overflow: ellipsis;">
                    { self.get_license_name(filter.license_id) }
                </td>
                <td class="border px-4 py-2 text-center overflow-hidden" style="height: 5vh; white-space: nowrap; text-overflow: ellipsis;">
                    <input type="checkbox" checked={selected} onclick={self.link.callback(move |_| SearchFilterMessage::SelectFilter(Some(filter_clone.clone())))}/>
                </td>
            </tr>
        }
    }

    fn get_language_name(&self, language_ids: Vec<u64>) -> String {
        self.languages.iter()
            .filter(|lang| language_ids.contains(&lang.id))
            .map(|lang| lang.name.clone())
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn get_license_name(&self, license_id: u64) -> String {
        // Retrieve the license(s) name from the licenses list, given the license ID
        // and join them into a single string separated by commas
        self.licenses.iter()
            .filter(|license| license.id == license_id)
            .map(|license| license.name.clone())
            .collect::<Vec<String>>()
            .join(", ")
    }
}


async fn fetch_filter_lists(link: Scope<SearchFilterList>) -> Result<(), String> {
    // dont know what else is supported
    let software_list = fetch_software().await?;
    let ublock_origin = software_list.iter().find(|software| software.name == "uBlock Origin")
        .ok_or("no clue")?;
    match _fetch::<FilterLists>("https://filterlists.com/api/directory/lists").await {
        Ok(filter_lists) => {
            for filter in filter_lists.iter().filter(|filter| {
                !filter.syntax_ids.is_empty() && filter.syntax_ids.iter().any(|id| ublock_origin.syntax_ids.contains(id))
            }) {
                link.send_message(SearchFilterMessage::FiltersLoaded(filter.clone()));
            }
        },
        Err(err) => return Err(err),
    };
    
    Ok(())
   
    }

async fn fetch_licenses(link: Scope<SearchFilterList>) -> Result<(), String> {
    match _fetch::<FilterLicenses>("https://filterlists.com/api/directory/licenses").await {
        Ok(licenses) =>  {
            link.send_message(SearchFilterMessage::LicensesLoaded(licenses));
            Ok(())
        },
        Err(err) => Err(err),

    }
}

async fn fetch_software() -> Result<FilterSoftwareList, String> {
    match _fetch::<FilterSoftwareList>("https://filterlists.com/api/directory/software").await {
        Ok(software_list) => Ok(software_list),
        Err(err) => Err(err),
    
    }
}

async fn fetch_languages(link: Scope<SearchFilterList>) -> Result<(), String> {
   match _fetch::<FilterLanguages>("https://filterlists.com/api/directory/languages").await {
    Ok(languages) => {
            link.send_message(SearchFilterMessage::LanguagesLoaded(languages));
            Ok(())
    },
    Err(err) => return Err(err),
   }
}

async fn _fetch<T>(url: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let response = Request::get(url)
        .send()
        .await
        .map_err(|err| err.to_string())?;
    
    if response.ok() {
        let body = response.text().await.map_err(|err| err.to_string())?;
        log::debug!("Raw response body: {}", body);

        let syntaxes: T = serde_json::from_str(&body).map_err(|err| err.to_string())?;
        return Ok(syntaxes)
    } else {
        Err(format!("Failed to fetch from {}: {}", url, response.status_text()))
    }
}