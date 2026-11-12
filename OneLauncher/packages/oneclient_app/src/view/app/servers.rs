use freya::prelude::*;
use oneclient_core::servers::ServerEntry;

use crate::components::{Button, Icon, IconType, ScrollArea, TextInput};
use crate::hooks::{settled_or_loading, use_dispatch, use_servers};
use crate::i18n::t;
use crate::theme::{colors, rounded, text_size};
use crate::ui::border_all_color;

#[derive(PartialEq)]
pub struct Servers;

impl Component for Servers {
    fn render(&self) -> impl IntoElement {
        let query = use_servers();
        let list = settled_or_loading(&query).unwrap_or_default();
        let custom_servers = use_state(Vec::<ServerEntry>::new);
        let show_add_modal = use_state(|| false);
        let name_input = use_state(String::new);
        let address_input = use_state(String::new);
        let port_input = use_state(|| "25565".to_string());

        // Load custom servers on mount
        if custom_servers.read().is_empty() {
            let mut cs = custom_servers;
            use_hook(move || {
                spawn(async move {
                    let loaded = oneclient_core::servers::load_custom_servers().await;
                    cs.set(loaded);
                });
            });
        }

        let custom_list = custom_servers.read().clone();
        let mut combined_servers = custom_list;
        combined_servers.extend(list.servers);

        let body: Element = if combined_servers.is_empty() {
            empty_state()
        } else {
            let mut column = rect()
                .vertical()
                .width(Size::fill())
                .height(Size::auto())
                .spacing(10.);
            for entry in combined_servers {
                let is_custom = entry.id.starts_with("custom-");
                column = column.child(ServerRow {
                    entry,
                    is_custom,
                    custom_servers,
                });
            }
            column.into_element()
        };

        let add_server_click = {
            let mut show_add_modal = show_add_modal;
            move |_| {
                show_add_modal.set(true);
            }
        };

        let modal_bg = colors::page_elevated();

        rect()
            .vertical()
            .width(Size::fill())
            .height(Size::fill())
            .overflow(Overflow::Clip)
            .child(
                ScrollArea::new()
                    .width(Size::fill())
                    .height(Size::fill())
                    .child(
                        rect()
                            .vertical()
                            .width(Size::fill())
                            .padding(Gaps::new(0., 40., 40., 40.))
                            .spacing(20.)
                            .child(
                                rect()
                                    .horizontal()
                                    .width(Size::fill())
                                    .main_align(Alignment::SpaceBetween)
                                    .cross_align(Alignment::Center)
                                    .child(
                                        rect()
                                            .vertical()
                                            .spacing(2.)
                                            .child(
                                                label()
                                                    .text(t("oneclient.servers.title"))
                                                    .font_size(text_size(32.))
                                                    .font_weight(FontWeight::BOLD)
                                                    .color(colors::fg_primary()),
                                            )
                                            .child(
                                                label()
                                                    .text(t("oneclient.servers.subtitle"))
                                                    .font_size(text_size(13.))
                                                    .color(colors::fg_secondary()),
                                            ),
                                    )
                                    .child(
                                        Button::new()
                                            .primary()
                                            .small()
                                            .on_press(add_server_click)
                                            .child(Icon::new(IconType::Plus).size(13.))
                                            .text(t("oneclient.servers.add_custom")),
                                    ),
                            )
                            .child(body),
                    ),
            )
            .maybe_child((*show_add_modal.read()).then(|| {
                let show_add_modal = show_add_modal;
                let custom_servers = custom_servers;
                let name_input = name_input;
                let address_input = address_input;
                let port_input = port_input;

                let save_click = move |_| {
                    let name = name_input.read().trim().to_string();
                    let address = address_input.read().trim().to_string();
                    let port = port_input.read().trim().parse::<u16>().unwrap_or(25565);
                    if !name.is_empty() && !address.is_empty() {
                        let id = format!("custom-{}", uuid::Uuid::new_v4());
                        let entry = ServerEntry {
                            id,
                            name,
                            summary: "Custom user server".to_string(),
                            address,
                            port,
                            region: "CUSTOM".to_string(),
                            tags: vec!["custom".to_string()],
                            is_promoted: false,
                            sponsor_url: None,
                        };
                        let mut cs = custom_servers;
                        let mut sm = show_add_modal;
                        let mut ni = name_input;
                        let mut ai = address_input;
                        spawn(async move {
                            let _ = oneclient_core::servers::save_custom_server(entry).await;
                            let loaded = oneclient_core::servers::load_custom_servers().await;
                            cs.set(loaded);
                        });
                        sm.set(false);
                        ni.set(String::new());
                        ai.set(String::new());
                    }
                };

                let cancel_click = move |_| {
                    let mut show_add_modal = show_add_modal;
                    show_add_modal.set(false);
                };

                rect()
                    .position(Position::new_absolute().top(0.).left(0.).right(0.).bottom(0.))
                    .layer(Layer::OverlayLevel(10))
                    .background(Color::BLACK.with_a(150))
                    .center()
                    .child(
                        rect()
                            .vertical()
                            .width(Size::px(420.))
                            .padding(Gaps::new_all(24.))
                            .spacing(16.)
                            .corner_radius(rounded(16.))
                            .background(modal_bg)
                            .border(border_all_color(1., colors::component_border()))
                            .child(
                                label()
                                    .text(t("oneclient.servers.add_modal.title"))
                                    .font_size(text_size(18.))
                                    .font_weight(FontWeight::BOLD)
                                    .color(colors::fg_primary()),
                            )
                            .child(
                                rect()
                                    .vertical()
                                    .spacing(6.)
                                    .child(
                                        label()
                                            .text(t("oneclient.servers.add_modal.name"))
                                            .font_size(text_size(12.))
                                            .color(colors::fg_secondary()),
                                    )
                                    .child(
                                        TextInput::new(name_input)
                                            .placeholder("My Survival Server"),
                                    ),
                            )
                            .child(
                                rect()
                                    .vertical()
                                    .spacing(6.)
                                    .child(
                                        label()
                                            .text(t("oneclient.servers.add_modal.address"))
                                            .font_size(text_size(12.))
                                            .color(colors::fg_secondary()),
                                    )
                                    .child(
                                        TextInput::new(address_input)
                                            .placeholder("play.example.com"),
                                    ),
                            )
                            .child(
                                rect()
                                    .vertical()
                                    .spacing(6.)
                                    .child(
                                        label()
                                            .text(t("oneclient.servers.add_modal.port"))
                                            .font_size(text_size(12.))
                                            .color(colors::fg_secondary()),
                                    )
                                    .child(
                                        TextInput::new(port_input)
                                            .placeholder("25565"),
                                    ),
                            )
                            .child(
                                rect()
                                    .horizontal()
                                    .width(Size::fill())
                                    .main_align(Alignment::End)
                                    .spacing(10.)
                                    .child(
                                        Button::new()
                                            .ghost()
                                            .small()
                                            .on_press(cancel_click)
                                            .text(t("oneclient.servers.add_modal.cancel")),
                                    )
                                    .child(
                                        Button::new()
                                            .primary()
                                            .small()
                                            .on_press(save_click)
                                            .text(t("oneclient.servers.add_modal.save")),
                                    ),
                            ),
                    )
                    .into_element()
            }))
    }
}

fn empty_state() -> Element {
    rect()
        .vertical()
        .width(Size::fill())
        .padding(Gaps::new_all(36.))
        .center()
        .spacing(8.)
        .corner_radius(rounded(14.))
        .background(colors::page_elevated())
        .border(border_all_color(1., colors::component_border()))
        .child(
            Icon::new(IconType::Globe01)
                .size(26.)
                .color(colors::fg_secondary()),
        )
        .child(
            label()
                .text(t("oneclient.servers.empty.title"))
                .font_size(text_size(15.))
                .font_weight(FontWeight::MEDIUM)
                .color(colors::fg_primary()),
        )
        .child(
            label()
                .text(t("oneclient.servers.empty.body"))
                .font_size(text_size(12.))
                .text_align(TextAlign::Center)
                .color(colors::fg_secondary()),
        )
        .into_element()
}

#[derive(Clone)]
struct ServerRow {
    entry: ServerEntry,
    is_custom: bool,
    custom_servers: State<Vec<ServerEntry>>,
}

impl PartialEq for ServerRow {
    fn eq(&self, other: &Self) -> bool {
        self.entry == other.entry && self.is_custom == other.is_custom
    }
}

impl Component for ServerRow {
    fn render(&self) -> impl IntoElement {
        let dispatch = use_dispatch();
        let ping = use_state(|| None::<Option<u32>>);

        if ping.read().is_none() {
            let entry = self.entry.clone();
            let mut ping = ping;
            use_hook(move || {
                spawn(async move {
                    let result = oneclient_core::servers::ping(&entry).await;
                    ping.set(Some(result));
                });
            });
        }

        let address = format!("{}:{}", self.entry.address, self.entry.port);
        let address_for_copy = address.clone();
        let server_id_for_click = self.entry.id.clone();
        let copy = {
            let dispatch = dispatch.clone();
            move |_: Event<PressEventData>| {
                let sid = server_id_for_click.clone();
                spawn(async move {
                    oneclient_core::servers::record_server_click(&sid).await;
                });
                if freya::text_edit::Clipboard::set(address_for_copy.clone()).is_ok() {
                    dispatch.notify(t("oneclient.servers.copied")).send();
                }
            }
        };

        let remove_custom = {
            let id = self.entry.id.clone();
            let cs = self.custom_servers;
            move |_| {
                let id = id.clone();
                let mut cs = cs;
                spawn(async move {
                    let _ = oneclient_core::servers::remove_custom_server(&id).await;
                    let loaded = oneclient_core::servers::load_custom_servers().await;
                    cs.set(loaded);
                });
            }
        };

        let ping_label = match *ping.read() {
            None => t("oneclient.servers.pinging").to_string(),
            Some(None) => t("oneclient.servers.offline").to_string(),
            Some(Some(ms)) => format!("{ms} ms"),
        };
        let ping_color = match *ping.read() {
            None => colors::fg_secondary(),
            Some(None) => colors::danger(),
            Some(Some(ms)) if ms < 80 => colors::success(),
            Some(Some(_)) => colors::code_warn(),
        };

        rect()
            .horizontal()
            .width(Size::fill())
            .height(Size::auto())
            .cross_align(Alignment::Center)
            .spacing(14.)
            .padding(Gaps::new_all(14.))
            .corner_radius(rounded(12.))
            .background(colors::page_elevated())
            .border(border_all_color(1., colors::component_border()))
            .child(
                rect()
                    .vertical()
                    .width(Size::percent(58.))
                    .spacing(4.)
                    .child(
                        rect()
                            .horizontal()
                            .cross_align(Alignment::Center)
                            .spacing(8.)
                            .child(
                                label()
                                    .text(self.entry.name.clone())
                                    .font_size(text_size(15.))
                                    .font_weight(FontWeight::SEMI_BOLD)
                                    .color(colors::fg_primary()),
                            )
                            .maybe_child(
                                (!self.entry.region.is_empty())
                                    .then(|| badge(self.entry.region.clone(), colors::brand())),
                            )
                            .maybe_child(self.entry.is_promoted.then(|| {
                                badge(
                                    t("oneclient.servers.promoted").to_string(),
                                    colors::code_warn(),
                                )
                            }))
                            .maybe_child(self.is_custom.then(|| {
                                badge(
                                    t("oneclient.servers.custom").to_string(),
                                    colors::success(),
                                )
                            })),
                    )
                    .child(
                        label()
                            .text(self.entry.summary.clone())
                            .font_size(text_size(12.))
                            .max_lines(2)
                            .color(colors::fg_secondary()),
                    )
                    .child(
                        label()
                            .text(address)
                            .font_size(text_size(11.))
                            .color(colors::fg_secondary().with_a(180)),
                    ),
            )
            .child(
                label()
                    .text(ping_label)
                    .font_size(text_size(12.))
                    .width(Size::px(72.))
                    .color(ping_color),
            )
            .child(
                Button::new()
                    .secondary()
                    .small()
                    .width(Size::px(160.))
                    .on_press(copy)
                    .child(Icon::new(IconType::Link03).size(13.))
                    .text(t("oneclient.servers.copy")),
            )
            .maybe_child(self.entry.sponsor_url.clone().map(|url| {
                Button::new()
                    .ghost()
                    .small()
                    .width(Size::px(34.))
                    .on_press(move |_| crate::platform::open_url(&url))
                    .child(Icon::new(IconType::LinkExternal01).size(13.))
            }))
            .maybe_child(self.is_custom.then(|| {
                Button::new()
                    .ghost()
                    .small()
                    .width(Size::px(34.))
                    .on_press(remove_custom)
                    .child(Icon::new(IconType::Trash01).size(13.))
            }))
    }
}

fn badge(text: String, color: Color) -> Element {
    rect()
        .padding(Gaps::new_symmetric(2., 8.))
        .corner_radius(rounded(6.))
        .background(color.with_a(40))
        .child(
            label()
                .text(text)
                .font_size(text_size(10.))
                .font_weight(FontWeight::SEMI_BOLD)
                .color(color),
        )
        .into_element()
}
