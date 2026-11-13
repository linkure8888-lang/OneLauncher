use freya::prelude::*;
use oneclient_social::Friend;

use crate::components::{Button, Icon, IconType, ScrollArea, TextInput};
use crate::hooks::use_dispatch;
use crate::i18n::t;
use crate::theme::{colors, rounded, text_size};
use crate::ui::border_all_color;

#[derive(PartialEq)]
pub struct Friends;

impl Component for Friends {
    fn render(&self) -> impl IntoElement {
        let dispatch = use_dispatch();
        let friends_list = use_state(|| match oneclient_social::load_friends() {
            Ok(list) => list,
            Err(_) => Vec::new(),
        });

        let nickname_input = use_state(String::new);
        let uuid_input = use_state(String::new);

        let add_friend_action = {
            let mut nickname_input = nickname_input;
            let mut uuid_input = uuid_input;
            let mut friends_list = friends_list;
            let dispatch = dispatch.clone();
            move |_: Event<PressEventData>| {
                let nick = nickname_input.read().trim().to_string();
                let id = uuid_input.read().trim().to_string();
                if nick.is_empty() || id.is_empty() {
                    dispatch.notify(t("oneclient.friends.error.empty")).send();
                    return;
                }
                match oneclient_social::add_friend(&nick, &id) {
                    Ok(()) => {
                        nickname_input.set(String::new());
                        uuid_input.set(String::new());
                        if let Ok(list) = oneclient_social::load_friends() {
                            friends_list.set(list);
                        }
                        dispatch.notify(t("oneclient.friends.added")).send();
                    }
                    Err(err) => {
                        dispatch
                            .notify(t("oneclient.friends.error.add"))
                            .body(err.to_string())
                            .error()
                            .send();
                    }
                }
            }
        };

        let list = friends_list.read();

        let body: Element = if list.is_empty() {
            empty_state()
        } else {
            let mut column = rect()
                .vertical()
                .width(Size::fill())
                .height(Size::auto())
                .spacing(10.);
            for entry in list.iter() {
                let friends_list_row = friends_list;
                column = column.child(FriendRow {
                    entry: entry.clone(),
                    on_removed: std::rc::Rc::new(move || {
                        let mut fl = friends_list_row;
                        if let Ok(list) = oneclient_social::load_friends() {
                            fl.set(list);
                        }
                    }),
                });
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
                                            .text(t("oneclient.friends.title"))
                                            .font_size(text_size(32.))
                                            .font_weight(FontWeight::BOLD)
                                            .color(colors::fg_primary()),
                                    )
                                    .child(
                                        label()
                                            .text(t("oneclient.friends.subtitle"))
                                            .font_size(text_size(13.))
                                            .color(colors::fg_secondary()),
                                    ),
                            )
                            .child(
                                rect()
                                    .horizontal()
                                    .width(Size::fill())
                                    .cross_align(Alignment::Center)
                                    .spacing(12.)
                                    .child(rect().width(Size::flex(1.)).child(
                                        TextInput::new(nickname_input).placeholder(t(
                                            "oneclient.friends.nickname_placeholder",
                                        )),
                                    ))
                                    .child(
                                        rect().width(Size::flex(1.)).child(
                                            TextInput::new(uuid_input).placeholder(t(
                                                "oneclient.friends.uuid_placeholder",
                                            )),
                                        ),
                                    )
                                    .child(
                                        Button::new()
                                            .primary()
                                            .on_press(add_friend_action)
                                            .child(Icon::new(IconType::Users01).size(15.))
                                            .text(t("oneclient.friends.add")),
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
            Icon::new(IconType::Users01)
                .size(26.)
                .color(colors::fg_secondary()),
        )
        .child(
            label()
                .text(t("oneclient.friends.empty.title"))
                .font_size(text_size(15.))
                .font_weight(FontWeight::MEDIUM)
                .color(colors::fg_primary()),
        )
        .child(
            label()
                .text(t("oneclient.friends.empty.body"))
                .font_size(text_size(12.))
                .text_align(TextAlign::Center)
                .color(colors::fg_secondary()),
        )
        .into_element()
}

#[derive(Clone)]
struct FriendRow {
    entry: Friend,
    on_removed: std::rc::Rc<dyn Fn()>,
}

impl PartialEq for FriendRow {
    fn eq(&self, other: &Self) -> bool {
        self.entry == other.entry
    }
}

impl Component for FriendRow {
    fn render(&self) -> impl IntoElement {
        let dispatch = use_dispatch();
        let entry = self.entry.clone();
        let on_removed = self.on_removed.clone();

        let formatted_time = match chrono::DateTime::from_timestamp(entry.last_seen as i64, 0) {
            Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            None => entry.last_seen.to_string(),
        };

        let uuid_val = entry.uuid.clone();
        let remove_action = {
            let dispatch = dispatch.clone();
            move |_: Event<PressEventData>| match oneclient_social::remove_friend(&uuid_val) {
                Ok(()) => {
                    on_removed();
                    dispatch.notify(t("oneclient.friends.removed")).send();
                }
                Err(err) => {
                    dispatch
                        .notify(t("oneclient.friends.error.remove"))
                        .body(err.to_string())
                        .error()
                        .send();
                }
            }
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
                    .width(Size::flex(1.0))
                    .spacing(4.)
                    .child(
                        label()
                            .text(entry.nickname.clone())
                            .font_size(text_size(15.))
                            .font_weight(FontWeight::SEMI_BOLD)
                            .color(colors::fg_primary()),
                    )
                    .child(
                        label()
                            .text(format!("UUID: {}", entry.uuid))
                            .font_size(text_size(12.))
                            .color(colors::fg_secondary()),
                    )
                    .child(
                        label()
                            .text(format!(
                                "{}: {}",
                                t("oneclient.friends.last_seen"),
                                formatted_time
                            ))
                            .font_size(text_size(11.))
                            .color(colors::fg_secondary().with_a(180)),
                    ),
            )
            .child(
                Button::new()
                    .danger()
                    .small()
                    .width(Size::px(120.))
                    .on_press(remove_action)
                    .child(Icon::new(IconType::Trash01).size(13.))
                    .text(t("oneclient.friends.remove")),
            )
    }
}
