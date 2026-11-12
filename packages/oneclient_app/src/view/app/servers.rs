use freya::prelude::*;
use oneclient_core::servers::ServerEntry;

use crate::components::{Button, Icon, IconType, ScrollArea};
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

        let body: Element = if list.servers.is_empty() {
            empty_state()
        } else {
            let mut column = rect().vertical().width(Size::fill()).spacing(10.);
            for entry in list.servers {
                column = column.child(ServerRow { entry });
            }
            column.into_element()
        };

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
                            .child(body),
                    ),
            )
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

#[derive(PartialEq)]
struct ServerRow {
    entry: ServerEntry,
}

impl Component for ServerRow {
    fn render(&self) -> impl IntoElement {
        let dispatch = use_dispatch();
        let mut ping = use_state(|| None::<Option<u32>>);

        if ping.read().is_none() {
            let entry = self.entry.clone();
            use_hook(move || {
                spawn(async move {
                    let result = oneclient_core::servers::ping(&entry).await;
                    ping.set(Some(result));
                });
            });
        }

        let address = format!("{}:{}", self.entry.address, self.entry.port);
        let address_for_copy = address.clone();
        let copy = {
            let dispatch = dispatch.clone();
            move |_: Event<PressEventData>| {
                if freya::text_edit::Clipboard::set(address_for_copy.clone()).is_ok() {
                    dispatch.notify(t("oneclient.servers.copied")).send();
                }
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
            .cross_align(Alignment::Center)
            .spacing(14.)
            .padding(Gaps::new_all(14.))
            .corner_radius(rounded(12.))
            .background(colors::page_elevated())
            .border(border_all_color(1., colors::component_border()))
            .child(
                rect()
                    .vertical()
                    .width(Size::flex(1.0))
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
                            .maybe_child(self.entry.promoted.then(|| {
                                badge(
                                    t("oneclient.servers.promoted").to_string(),
                                    colors::code_warn(),
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
                    .color(ping_color),
            )
            .child(
                Button::new()
                    .secondary()
                    .small()
                    .on_press(copy)
                    .child(Icon::new(IconType::Link03).size(13.))
                    .text(t("oneclient.servers.copy")),
            )
            .maybe_child(self.entry.url.clone().map(|url| {
                Button::new()
                    .ghost()
                    .small()
                    .on_press(move |_| crate::platform::open_url(&url))
                    .child(Icon::new(IconType::LinkExternal01).size(13.))
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
