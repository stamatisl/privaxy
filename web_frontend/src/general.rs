use crate::save_button;
use regex::Regex;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::{html, Callback, Component, Context, Html};

pub enum Message {
    Load,
    Save,
    NetworkLoadSuccess(NetworkConfig),
    UpdateProxyPort(String),
    UpdateBindAddr(String),
    UpdateWebPort(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// Network configuration for Privaxy
pub struct NetworkConfig {
    /// Bind address for the proxy server.
    pub bind_addr: String,
    /// Port for the proxy server.
    pub proxy_port: u16,
    /// Port for the web server.
    pub web_port: u16,
    /// Enable TLS for the web server.
    pub tls: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct NetworkSettings {
    current_config: NetworkConfig,
    remote_config: NetworkConfig,
    raw_proxy_port: String,
    raw_bind_addr: String,
    raw_web_port: String,
    save_callback: Callback<()>,
    proxy_port_error: Option<String>,
    bind_addr_error: Option<String>,
    web_port_error: Option<String>,
}

enum SettingCategories {
    Network(NetworkSettings),
    Other,
}

pub(crate) struct GeneralSettings {
    changes_saved: bool,
    network_settings: Option<NetworkSettings>,
    loading: bool,
    save_callback: Callback<()>,
}

impl GeneralSettings {
    fn save(&self) {
        self.save_callback.emit(());
    }

    fn config_has_changed(&self) -> bool {
        let net_changed = match &self.network_settings {
            None => return false,
            Some(network_settings) => {
                network_settings.current_config != network_settings.remote_config
            }
        };
        net_changed
    }
}

impl GeneralSettings {
    fn new() -> Self {
        Self {
            changes_saved: true,
            network_settings: None,
            loading: true,
            save_callback: Callback::noop(),
        }
    }
}

impl Component for GeneralSettings {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Message::Load);
        Self {
            changes_saved: true,
            network_settings: None,
            loading: true,
            save_callback: Callback::noop(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Load => {
                let link = ctx.link().clone();
                spawn_local(async move {
                    let request = Request::get("/api/settings/network");
                    match request.send().await {
                        Ok(response) => {
                            if response.ok() {
                                if let Ok(network_config) = response.json::<NetworkConfig>().await {
                                    link.send_message(Message::NetworkLoadSuccess(network_config));
                                }
                            } else {
                                log::error!(
                                    "Failed to load network config: {:?}",
                                    response.status()
                                );
                            }
                        }
                        Err(err) => {
                            log::error!("Request error: {:?}", err);
                        }
                    }
                });
                self.loading = false;
                self.changes_saved = true;
            }
            Message::Save => {
                self.changes_saved = true;
            }
            Message::NetworkLoadSuccess(network_config) => {
                self.network_settings = {
                    Some(NetworkSettings {
                        current_config: network_config.clone(),
                        remote_config: network_config.clone(),
                        raw_proxy_port: network_config.proxy_port.to_string(),
                        raw_bind_addr: network_config.bind_addr.clone(),
                        raw_web_port: network_config.web_port.to_string(),
                        save_callback: ctx.link().callback(|_| Message::Save),
                        proxy_port_error: None,
                        bind_addr_error: None,
                        web_port_error: None,
                    })
                };
                self.loading = false;
            }
            Message::UpdateProxyPort(value) => {
                if let Some(ref mut network_settings) = self.network_settings {
                    network_settings.raw_proxy_port = value.clone();
                    network_settings.proxy_port_error = match value.parse::<u16>() {
                        Ok(p) if p >= 1 => {
                            network_settings.current_config.proxy_port = p;
                            None
                        }
                        _ => Some(
                            "Invalid proxy port. Must be a number between 1 and 65535.".to_string(),
                        ),
                    };
                }
            }
            Message::UpdateBindAddr(value) => {
                if let Some(ref mut network_settings) = self.network_settings {
                    network_settings.raw_bind_addr = value.clone();
                    let re = Regex::new(r"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$").unwrap();
                    network_settings.bind_addr_error = if re.is_match(&value) {
                        network_settings.current_config.bind_addr = value.clone();
                        None
                    } else {
                        Some("Invalid bind address. Must be a valid IP address.".to_string())
                    };
                }
            }
            Message::UpdateWebPort(value) => {
                if let Some(ref mut network_settings) = self.network_settings {
                    network_settings.raw_web_port = value.clone();
                    network_settings.web_port_error = match value.parse::<u16>() {
                        Ok(p) if p >= 1 => {
                            network_settings.current_config.web_port = p;
                            None
                        }
                        _ => Some(
                            "Invalid web port. Must be a number between 1 and 65535.".to_string(),
                        ),
                    };
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let render_setting = |setting_name: &str,
                              setting_value: String,
                              oninput: Callback<InputEvent>,
                              error: Option<&String>,
                              description: &str| {
            html! {
                <div class="mb-4" style="display: flex; flex-direction: column; align-items: flex-start; width: 100%; padding: 2px 0;">
                    <div style="display: flex; align-items: center; width: 100%;">
                        <div class="text-gray-500" style="width: 200px; text-align: right; padding-right: 4px;">{ setting_name }</div>
                        <div style="flex-grow: 1;">
                            <input value={setting_value} class="shadow appearance-none border rounded w-80 py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline" type="text" oninput={oninput} />
                        </div>
                    </div>
                    <div style="width: calc(100% - 200px); margin-left: 200px;">
                        <p class="text-gray-400 text-sm">{description}</p>
                        if let Some(error_msg) = error {
                            <p class="text-red-500 text-xs italic">{error_msg}</p>
                        }
                    </div>
                </div>
            }
        };

        let render_boolean_setting = |setting_name: &str,
                                      setting_value: bool,
                                      description: &str| {
            let checkbox_callback = ctx.link().callback(|_| Message::Load);
            html! {
                <div class="mb-4" style="display: flex; flex-direction: column; align-items: flex-start; width: 100%; padding: 2px 0;">
                    <div style="display: flex; align-items: center; width: 100%;">
                        <div class="text-gray-500" style="width: 200px; text-align: right; padding-right: 4px;">{ setting_name }</div>
                        <div style="flex-grow: 1;">
                            <input checked={setting_value} onchange={checkbox_callback} type="checkbox" class="focus:ring-blue-500 h-4 w-4 text-blue-600 border-gray-300 rounded" />
                        </div>
                    </div>
                    <div style="width: calc(100% - 200px); margin-left: 200px;">
                        <p class="text-gray-400 text-sm">{description}</p>
                    </div>
                </div>
            }
        };

        let render_category = |category_name: &str, category_settings: SettingCategories| {
            html! {
                <fieldset class="mb-8" style="width: 100%;">
                    <legend class="text-lg font-medium text-gray-900">{category_name}</legend>
                    <div class="mt-4 border-t border-b border-gray-200 divide-y divide-gray-200">
                        { match category_settings {
                            SettingCategories::Network(network_settings) => {
                                html! {
                                    <>
                                    { render_setting(
                                        "Bind address",
                                        network_settings.raw_bind_addr.clone(),
                                        ctx.link().callback(|e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            Message::UpdateBindAddr(input.value())
                                        }),
                                        network_settings.bind_addr_error.as_ref(),
                                        "Enter the IP address the proxy should bind to."
                                    ) }
                                    { render_setting(
                                        "Proxy port",
                                        network_settings.raw_proxy_port.clone(),
                                        ctx.link().callback(|e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            Message::UpdateProxyPort(input.value())
                                        }),
                                        network_settings.proxy_port_error.as_ref(),
                                        "Enter the port number for the proxy server (1-65535)."
                                    ) }
                                    { render_setting(
                                        "Web port",
                                        network_settings.raw_web_port.clone(),
                                        ctx.link().callback(|e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            Message::UpdateWebPort(input.value())
                                        }),
                                        network_settings.web_port_error.as_ref(),
                                        "Enter the port number for the web server (1-65535)."
                                    ) }
                                    { render_boolean_setting("TLS", network_settings.current_config.tls, "If the web server uses HTTPS") }
                                    </>
                                }
                            }
                            SettingCategories::Other => html! {<></>},
                        }}
                    </div>
                </fieldset>
            }
        };

        let save_button_state = if !self.changes_saved {
            save_button::SaveButtonState::Disabled
        } else {
            save_button::SaveButtonState::Enabled
        };

        let save_callback = ctx.link().callback(|_| Message::Save);
        let title = html! {
            <div class="pt-1.5 mb-4">
                <h1 class="text-2xl font-bold text-gray-900">{ "General Settings" }</h1>
            </div>
        };

        html! {
            <>
            { title }
                if let Some(network_settings) = &self.network_settings {
                    {render_category("Network", SettingCategories::Network(network_settings.clone()))}
                } else {
                <div>{"Loading..."}</div>}
            <save_button::SaveButton state={save_button_state} onclick={save_callback} />
            </>
        }
    }
}
