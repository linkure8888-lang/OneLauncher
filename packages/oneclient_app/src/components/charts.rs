use freya::prelude::*;

use crate::i18n::t;
use crate::theme::{colors, rounded, rounded_px, text_size};
use crate::utils::format_duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueUnit {
    Duration,
    Count,
}

impl ValueUnit {
    pub fn format(self, value: i64) -> String {
        match self {
            ValueUnit::Duration => format_duration(value),
            ValueUnit::Count => format!("{value}×"),
        }
    }
}

const DEFAULT_HEIGHT: f32 = 150.;
const Y_AXIS_WIDTH: f32 = 44.;

#[derive(PartialEq)]
pub struct BarChart {
    values: Vec<i64>,
    labels: Vec<String>,
    readout_labels: Option<Vec<String>>,
    highlight: Option<usize>,
    unit: ValueUnit,
    height: f32,
    gap: f32,
}

impl BarChart {
    pub fn new(values: Vec<i64>, labels: Vec<String>) -> Self {
        Self {
            values,
            labels,
            readout_labels: None,
            highlight: None,
            unit: ValueUnit::Duration,
            height: DEFAULT_HEIGHT,
            gap: 4.,
        }
    }

    pub fn readout_labels(mut self, labels: Vec<String>) -> Self {
        self.readout_labels = Some(labels);
        self
    }

    pub fn highlight(mut self, highlight: Option<usize>) -> Self {
        self.highlight = highlight;
        self
    }

    pub fn unit(mut self, unit: ValueUnit) -> Self {
        self.unit = unit;
        self
    }

    #[allow(dead_code)]
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    fn readout_name(&self, i: usize) -> String {
        self.readout_labels
            .as_ref()
            .unwrap_or(&self.labels)
            .get(i)
            .cloned()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("#{}", i + 1))
    }
}

impl Component for BarChart {
    fn render(&self) -> impl IntoElement {
        let mut hovered = use_state(|| Option::<usize>::None);
        let active = *hovered.read();
        let mut plot_width = use_state(|| 0f32);

        let unit = self.unit;
        let height = self.height;
        let max = self.values.iter().copied().max().unwrap_or(0);

        let focus = active.or(self.highlight);
        let (readout_label, readout_value) = match focus {
            Some(i) if i < self.values.len() => (self.readout_name(i), unit.format(self.values[i])),
            _ => {
                let total: i64 = self.values.iter().sum();
                (
                    t("oneclient.components.charts.total").to_string(),
                    unit.format(total),
                )
            }
        };

        let mut bars = rect()
            .horizontal()
            .content(Content::Flex)
            .width(Size::fill())
            .height(Size::fill())
            .cross_align(Alignment::End)
            .spacing(self.gap)
            .on_pointer_leave(move |_| *hovered.write() = None);

        for (i, &v) in self.values.iter().enumerate() {
            let frac = if max > 0 { v as f32 / max as f32 } else { 0.0 };
            let h = if v > 0 { (frac * height).max(4.0) } else { 0.0 };
            let color = match active {
                Some(a) if a != i => colors::component_bg_hover(),
                _ => colors::brand(),
            };

            bars = bars.child(
                rect()
                    .width(Size::flex(1.0))
                    .height(Size::fill())
                    .main_align(Alignment::End)
                    .cursor(CursorIcon::Pointer)
                    .on_pointer_enter(move |_| *hovered.write() = Some(i))
                    .child(
                        rect()
                            .width(Size::fill())
                            .height(Size::px(h))
                            .corner_radius(CornerRadius::new(
                                rounded_px(4.),
                                rounded_px(4.),
                                0.,
                                0.,
                            ))
                            .background(color),
                    ),
            );
        }

        let stride = label_stride(&self.labels, *plot_width.read(), self.gap);
        let mut label_row = rect()
            .horizontal()
            .content(Content::Flex)
            .width(Size::fill())
            .height(Size::px(14.))
            .spacing(self.gap);
        for (i, text) in self.labels.iter().enumerate() {
            let shown = i % stride == 0;
            label_row = label_row.child(
                rect()
                    .width(Size::flex(1.0))
                    .horizontal()
                    .main_align(Alignment::Center)
                    .child(
                        label()
                            .text(if shown { text.clone() } else { String::new() })
                            .font_size(text_size(10.))
                            .max_lines(1)
                            .color(colors::fg_secondary()),
                    ),
            );
        }

        let plot = rect()
            .width(Size::fill())
            .height(Size::px(height))
            .overflow(Overflow::Clip)
            .child(gridlines(height))
            .child(bars);

        let plot_row = rect()
            .horizontal()
            .content(Content::Flex)
            .width(Size::fill())
            .spacing(6.)
            .child(y_axis(max, unit, height))
            .child(
                rect()
                    .width(Size::flex(1.0))
                    .on_sized(move |e: Event<SizedEventData>| {
                        let w = e.area.width();
                        if (*plot_width.peek() - w).abs() > 0.5 {
                            plot_width.set(w);
                        }
                    })
                    .child(plot),
            );

        let labels_row = rect()
            .horizontal()
            .content(Content::Flex)
            .width(Size::fill())
            .spacing(6.)
            .child(rect().width(Size::px(Y_AXIS_WIDTH)))
            .child(rect().width(Size::flex(1.0)).child(label_row));

        rect()
            .vertical()
            .width(Size::fill())
            .spacing(8.)
            .child(readout(readout_label, readout_value))
            .child(plot_row)
            .child(labels_row)
    }
}

/// Bars to skip between shown labels derived from measured plot width
fn label_stride(labels: &[String], plot_width: f32, gap: f32) -> usize {
    let n = labels.len();
    if n == 0 {
        return 1;
    }
    // Before the first measurement thin conservatively to avoid an overlap flash
    if plot_width <= 0.0 {
        return (n / 12).max(1);
    }

    let max_chars = labels
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0)
        .max(1);
    // ~6.2px per char at font_size 10
    let slot_px = max_chars as f32 * 6.2 + gap.max(8.0) + 12.0;
    let max_labels = (plot_width / slot_px).floor().max(1.0) as usize;

    if n <= max_labels {
        1
    } else {
        n.div_ceil(max_labels)
    }
}

fn gridlines(height: f32) -> Element {
    let mut overlay = rect()
        .width(Size::fill())
        .height(Size::fill())
        .position(Position::new_absolute().top(0.).left(0.))
        .interactive(false);

    for frac in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
        let top = ((1.0 - frac) * height - if frac == 0.0 { 1.0 } else { 0.0 }).max(0.);
        overlay = overlay.child(
            rect()
                .width(Size::fill())
                .height(Size::px(1.))
                .position(Position::new_absolute().left(0.).top(top))
                .background(colors::component_border().with_a(110)),
        );
    }
    overlay.into_element()
}

fn y_axis(max: i64, unit: ValueUnit, height: f32) -> Element {
    let fmt = |v: i64| match unit {
        ValueUnit::Duration => format_duration(v),
        ValueUnit::Count => v.to_string(),
    };

    let tick = |text: String| {
        label()
            .text(text)
            .font_size(text_size(9.))
            .max_lines(1)
            .width(Size::fill())
            .text_align(TextAlign::Right)
            .color(colors::fg_secondary().with_a(160))
    };

    rect()
        .vertical()
        .width(Size::px(Y_AXIS_WIDTH))
        .height(Size::px(height))
        .main_align(Alignment::SpaceBetween)
        .child(tick(fmt(max)))
        .child(tick(fmt(max / 2)))
        .child(tick(fmt(0)))
        .into_element()
}

fn readout(name: String, value: String) -> Element {
    rect()
        .horizontal()
        .width(Size::fill())
        .cross_align(Alignment::Center)
        .spacing(8.)
        .height(Size::px(20.))
        .child(
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(6.)
                .padding(Gaps::new_symmetric(3., 8.))
                .corner_radius(rounded(6.))
                .background(colors::component_bg())
                .border(crate::ui::border_all_color(1., colors::component_border()))
                .child(
                    rect()
                        .width(Size::px(8.))
                        .height(Size::px(8.))
                        .corner_radius(rounded(2.))
                        .background(colors::brand()),
                )
                .child(
                    label()
                        .text(name)
                        .font_size(text_size(11.))
                        .color(colors::fg_secondary()),
                )
                .child(
                    label()
                        .text(value)
                        .font_size(text_size(11.))
                        .font_weight(FontWeight::SEMI_BOLD)
                        .color(colors::fg_primary()),
                ),
        )
        .into_element()
}
