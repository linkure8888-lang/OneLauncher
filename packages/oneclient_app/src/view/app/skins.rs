use freya::prelude::*;
use oneclient_core::{LibrarySkin, SkinModel};
use uuid::Uuid;

use crate::components::{Button, Icon, IconType, LocalImage, ScrollArea};
use crate::hooks::{
    invalidate_skin_library, settled_or_loading, use_current_account, use_dispatch,
    use_skin_library,
};
use crate::i18n::t;
use crate::theme::{colors, rounded, text_size};
use crate::ui::border_all_color;

const CARD_WIDTH: f32 = 190.;
const PREVIEW_EDGE: u32 = 256;

#[derive(PartialEq)]
pub struct Skins;

impl Component for Skins {
    fn render(&self) -> impl IntoElement {
        let library = use_skin_library();
        let skins = settled_or_loading(&library).unwrap_or_default();
        let account = settled_or_loading(&use_current_account()).flatten();
        let dispatch = use_dispatch();

        let add = {
            let dispatch = dispatch.clone();
            move |_: Event<PressEventData>| {
                let dispatch = dispatch.clone();
                spawn(async move {
                    let Some(file) = rfd::AsyncFileDialog::new()
                        .add_filter("PNG", &["png"])
                        .set_title(t("oneclient.skins.pick_file"))
                        .pick_file()
                        .await
                    else {
                        return;
                    };
                    let name = file
                        .file_name()
                        .trim_end_matches(".png")
                        .trim_end_matches(".PNG")
                        .to_string();
                    match oneclient_core::import_library_skin(
                        &name,
                        file.path().to_path_buf(),
                        None,
                        SkinModel::Classic,
                    )
                    .await
                    {
                        Ok(_) => {
                            invalidate_skin_library().await;
                            dispatch.notify(t("oneclient.skins.imported")).send();
                        }
                        Err(err) => {
                            dispatch
                                .notify(t("oneclient.skins.import_failed"))
                                .body(err)
                                .error()
                                .send();
                        }
                    }
                });
            }
        };

        let body: Element = if skins.is_empty() {
            empty_state()
        } else {
            let mut grid = rect()
                .horizontal()
                .width(Size::fill())
                .content(Content::wrap_spacing(16.));
            for (id, skin) in skins {
                grid = grid.child(SkinCard {
                    id,
                    skin,
                    account_id: account.as_ref().map(|account| account.id),
                    account_offline: account.as_ref().is_some_and(|account| account.is_offline()),
                });
            }
            grid.into_element()
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
                            .spacing(24.)
                            .child(
                                rect()
                                    .horizontal()
                                    .width(Size::fill())
                                    .cross_align(Alignment::Center)
                                    .spacing(16.)
                                    .child(
                                        rect()
                                            .vertical()
                                            .width(Size::flex(1.0))
                                            .spacing(2.)
                                            .child(
                                                label()
                                                    .text(t("oneclient.skins.title"))
                                                    .font_size(text_size(32.))
                                                    .font_weight(FontWeight::BOLD)
                                                    .color(colors::fg_primary()),
                                            )
                                            .child(
                                                label()
                                                    .text(t("oneclient.skins.subtitle"))
                                                    .font_size(text_size(13.))
                                                    .color(colors::fg_secondary()),
                                            ),
                                    )
                                    .child(
                                        Button::new()
                                            .primary()
                                            .on_press(add)
                                            .child(Icon::new(IconType::Plus).size(16.))
                                            .text(t("oneclient.skins.add")),
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
        .padding(Gaps::new_all(40.))
        .center()
        .spacing(10.)
        .corner_radius(rounded(14.))
        .background(colors::page_elevated())
        .border(border_all_color(1., colors::component_border()))
        .child(
            Icon::new(IconType::Image01)
                .size(28.)
                .color(colors::fg_secondary()),
        )
        .child(
            label()
                .text(t("oneclient.skins.empty.title"))
                .font_size(text_size(15.))
                .font_weight(FontWeight::MEDIUM)
                .color(colors::fg_primary()),
        )
        .child(
            label()
                .text(t("oneclient.skins.empty.body"))
                .font_size(text_size(12.))
                .text_align(TextAlign::Center)
                .color(colors::fg_secondary()),
        )
        .into_element()
}

#[derive(PartialEq)]
struct SkinCard {
    id: Uuid,
    skin: LibrarySkin,
    account_id: Option<Uuid>,
    account_offline: bool,
}

impl Component for SkinCard {
    fn render(&self) -> impl IntoElement {
        let dispatch = use_dispatch();
        let id = self.id;
        let account_id = self.account_id;
        let can_apply = account_id.is_some() && self.account_offline;

        let apply = {
            let dispatch = dispatch.clone();
            move |_: Event<PressEventData>| {
                let Some(account_id) = account_id else {
                    return;
                };
                let dispatch = dispatch.clone();
                spawn(async move {
                    match oneclient_core::apply_library_skin(account_id, id).await {
                        Ok(_) => {
                            dispatch.notify(t("oneclient.skins.applied")).send();
                        }
                        Err(err) => {
                            dispatch
                                .notify(t("oneclient.skins.apply_failed"))
                                .body(err)
                                .error()
                                .send();
                        }
                    }
                });
            }
        };

        let remove = {
            let dispatch = dispatch.clone();
            move |_: Event<PressEventData>| {
                let dispatch = dispatch.clone();
                spawn(async move {
                    if let Err(err) = oneclient_core::delete_library_skin(id).await {
                        dispatch
                            .notify(t("oneclient.skins.delete_failed"))
                            .body(err)
                            .error()
                            .send();
                        return;
                    }
                    invalidate_skin_library().await;
                });
            }
        };

        let preview = oneclient_core::library_skin_path(id).ok();

        rect()
            .vertical()
            .width(Size::px(CARD_WIDTH))
            .spacing(10.)
            .padding(Gaps::new_all(12.))
            .corner_radius(rounded(12.))
            .background(colors::page_elevated())
            .border(border_all_color(1., colors::component_border()))
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(CARD_WIDTH - 24.))
                    .corner_radius(rounded(8.))
                    .overflow(Overflow::Clip)
                    .background(colors::component_bg())
                    .center()
                    .maybe_child(preview.map(|path| {
                        LocalImage::new(path, PREVIEW_EDGE, true)
                            .width(Size::fill())
                            .height(Size::fill())
                    })),
            )
            .child(
                label()
                    .text(self.skin.name.clone())
                    .font_size(text_size(13.))
                    .font_weight(FontWeight::MEDIUM)
                    .max_lines(1)
                    .color(colors::fg_primary()),
            )
            .child(
                label()
                    .text(match self.skin.model {
                        SkinModel::Classic => t("oneclient.skins.model.classic"),
                        SkinModel::Slim => t("oneclient.skins.model.slim"),
                    })
                    .font_size(text_size(11.))
                    .color(colors::fg_secondary()),
            )
            .child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .spacing(6.)
                    .child(
                        Button::new()
                            .primary()
                            .small()
                            .enabled(can_apply)
                            .on_press(apply)
                            .text(t("oneclient.skins.apply")),
                    )
                    .child(
                        Button::new()
                            .ghost()
                            .small()
                            .on_press(remove)
                            .child(Icon::new(IconType::Trash01).size(13.)),
                    ),
            )
    }
}
