//! Hatch Settings (HATCH / GRADIENT seTtings: fill, angle and scale, origin,
//! boundaries and options for the running command) and the boundary warning
//! an empty-area pick shows.

use iced::widget::{button, column, container, pick_list, row, text, text_input, Space};
use iced::{Element, Fill, Length, Theme};

use crate::app::Message;
use crate::modules::draw::draw::hatch::{HatchFill, HatchSettings};
use crate::scene::model::hatch_model::GradientKind;
use crate::t;
use crate::ui::style::common::muted_style;
use crate::ui::style::form::{button_style, dialog_button, field_style};
use crate::ui::window::attman_dialog::{color_label, parse_color, COLORS};
use crate::ui::window::import_dialogs::warning_badge;
use crate::ui::window::pdf_dialogs::{card, chip, well_style};
use codec::entities::HatchStyleType;

/// The Hatch Settings window: the settings and the typed text of its fields.
#[derive(Debug, Clone)]
pub struct HatchSettingsState {
    pub settings: HatchSettings,
    pub angle: String,
    pub scale: String,
    pub spacing: String,
    pub origin_x: String,
    pub origin_y: String,
    pub gap: String,
    pub transparency: String,
    pub layers: Vec<String>,
    /// A field that does not read: its message, and OK waits.
    pub error: Option<String>,
}

impl HatchSettingsState {
    pub fn new(settings: HatchSettings, layers: Vec<String>) -> Self {
        let number = |value: f64| format!("{value}");
        let spacing = match &settings.fill {
            HatchFill::User { spacing, .. } => *spacing,
            _ => 1.0,
        };
        Self {
            angle: format!("{}", settings.angle_deg),
            scale: number(settings.scale),
            spacing: number(spacing),
            origin_x: number(settings.origin[0]),
            origin_y: number(settings.origin[1]),
            gap: format!("{}", settings.gap),
            transparency: settings.transparency.map(|t| t.to_string()).unwrap_or_default(),
            settings,
            layers,
            error: None,
        }
    }

    /// The settings with the typed fields read; the first field that does
    /// not read is the error.
    pub fn read(&self) -> Result<HatchSettings, String> {
        let number = |text: &str| text.trim().replace(',', ".").parse::<f64>().ok();
        let mut settings = self.settings.clone();
        settings.angle_deg = number(&self.angle)
            .filter(|v| v.is_finite())
            .ok_or_else(|| crate::tr!("hatch-settings", "bad-angle"))?;
        settings.scale = number(&self.scale)
            .filter(|v| *v > 0.0)
            .ok_or_else(|| crate::tr!("hatch-settings", "bad-scale"))?;
        if let HatchFill::User { spacing, .. } = &mut settings.fill {
            *spacing = number(&self.spacing)
                .filter(|v| *v > 0.0)
                .ok_or_else(|| crate::tr!("hatch-settings", "bad-spacing"))?;
        }
        settings.origin = [
            number(&self.origin_x).ok_or_else(|| crate::tr!("hatch-settings", "bad-origin"))?,
            number(&self.origin_y).ok_or_else(|| crate::tr!("hatch-settings", "bad-origin"))?,
        ];
        settings.gap = number(&self.gap)
            .filter(|v| (0.0..=crate::modules::draw::draw::hatch::MAX_GAP_TOLERANCE).contains(v))
            .ok_or_else(|| crate::tr!("hatch-settings", "bad-gap"))?;
        settings.transparency = match self.transparency.trim() {
            "" => None,
            text => Some(
                text.parse::<u8>()
                    .ok()
                    .filter(|v| *v <= 90)
                    .ok_or_else(|| crate::tr!("hatch-settings", "bad-transparency"))?,
            ),
        };
        Ok(settings)
    }
}

/// The warning an empty-area pick shows.
#[derive(Debug, Clone, Default)]
pub struct BoundaryWarningState {
    pub details: bool,
    pub dont_show: bool,
    pub gap: f64,
}

#[derive(Debug, Clone)]
pub enum HatchDialogMsg {
    /// 0 solid, 1 gradient, 2 pattern, 3 user defined.
    Kind(u8),
    Pattern(String),
    Gradient(String),
    Color(String),
    Background(String),
    Angle(String),
    Scale(String),
    Spacing(String),
    Double(bool),
    OriginX(String),
    OriginY(String),
    Associative(bool),
    Separate(bool),
    Keep(bool),
    Island(u8),
    Gap(String),
    Layer(String),
    Transparency(String),
    GradientColor1(String),
    GradientColor2(String),
    /// The draw order's label (see `draw_order_labels`).
    DrawOrder(String),
    Annotative(bool),
    /// Boundaries: OK and continue picking points (0) or selecting (1).
    Boundary(u8),
    Ok,
    WarningDetails(bool),
    WarningDontShow(bool),
    WarningClose,
}

fn msg(m: HatchDialogMsg) -> Message {
    Message::HatchDialog(m)
}

fn kind_of(fill: &HatchFill) -> u8 {
    match fill {
        HatchFill::Solid => 0,
        HatchFill::Gradient(..) => 1,
        HatchFill::Pattern(_) => 2,
        HatchFill::User { .. } => 3,
    }
}

fn island_of(style: HatchStyleType) -> u8 {
    match style {
        HatchStyleType::Normal => 0,
        HatchStyleType::Outer => 1,
        HatchStyleType::Ignore => 2,
    }
}

/// HPDRAWORDER's choices, in its order.
pub fn draw_order_labels() -> Vec<String> {
    ["order-none", "order-back", "order-front", "order-behind", "order-in-front"]
        .iter()
        .map(|key| crate::i18n::loader().get_attr("hatch-settings", key))
        .collect()
}

/// A swatch of the chosen fill.
fn preview<'a>(state: &'a HatchSettingsState) -> Element<'a, Message> {
    let s = &state.settings;
    let swatch: Element<'a, Message> = match &s.fill {
        HatchFill::Gradient(kind, invert) => {
            let angle = state.angle.trim().replace(',', ".").parse::<f64>().unwrap_or(0.0);
            crate::ui::window::gradient_editor::preview_image(
                crate::ui::window::gradient_editor::fill_preview(
                    *kind,
                    *invert,
                    s.gradient_colors[0].clone(),
                    s.gradient_colors[1].clone(),
                    angle,
                ),
            )
        }
        fill => {
            let (_, pattern) = crate::modules::draw::draw::hatch::fill_pattern(
                fill,
                0.0,
                &s.gradient_colors,
            );
            crate::ui::properties::hatch_pattern_preview(pattern, 48.0)
        }
    };
    container(swatch)
        .height(Length::Fixed(48.0))
        .width(Fill)
        .style(well_style)
        .into()
}

/// The "use current" / "none" choice at the top of a list.
pub fn current_label() -> String {
    crate::tr!("hatch-settings", "use-current")
}

pub fn none_label() -> String {
    crate::tr!("hatch-settings", "none")
}

/// One choice out of a few, as joined buttons.
fn segmented<'a>(options: Vec<(u8, String)>, current: u8, on: fn(u8) -> HatchDialogMsg) -> Element<'a, Message> {
    let buttons = options.into_iter().map(|(value, label)| {
        button(text(label).size(11).width(Fill).align_x(iced::Center))
            .on_press(msg(on(value)))
            .style(button_style(value == current))
            .padding([5, 6])
            .width(Fill)
            .into()
    });
    container(iced::widget::Row::with_children(buttons.collect::<Vec<_>>()).spacing(2))
        .padding(2)
        .width(Fill)
        .style(well_style)
        .into()
}

fn labelled<'a>(label: String, control: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    row![
        text(label).size(11).style(muted_style).width(Length::Fixed(110.0)),
        control.into()
    ]
    .spacing(6)
    .align_y(iced::Center)
    .into()
}

fn input<'a>(value: &'a str, enabled: bool, on: fn(String) -> HatchDialogMsg) -> Element<'a, Message> {
    let mut field = text_input("", value)
        .size(11)
        .padding([4, 8])
        .width(Fill)
        .style(field_style);
    if enabled {
        field = field.on_input(move |s| msg(on(s)));
    }
    field.into()
}

fn list<'a>(selected: String, options: Vec<String>, on: fn(String) -> HatchDialogMsg) -> Element<'a, Message> {
    pick_list(Some(selected), options, |s: &String| s.clone())
        .on_select(move |s| msg(on(s)))
        .text_size(11)
        .padding([4, 8])
        .width(Fill)
        .into()
}

pub fn view_settings<'a>(
    state: &'a HatchSettingsState,
    sizing: crate::ui::modal::ModalSizing,
) -> Element<'a, Message> {
    let s = &state.settings;
    let kind = kind_of(&s.fill);
    let tr = |key: &str| crate::i18n::loader().get_attr("hatch-settings", key);
    let mut colors: Vec<String> = vec![current_label()];
    colors.extend(COLORS.iter().map(|c| c.to_string()));
    let mut backgrounds: Vec<String> = vec![none_label()];
    backgrounds.extend(COLORS.iter().map(|c| c.to_string()));

    // ── Fill ─────────────────────────────────────────────────────────────
    let mut fill = column![segmented(
        vec![
            (0, tr("solid")),
            (1, tr("gradient")),
            (2, tr("pattern")),
            (3, tr("user-defined")),
        ],
        kind,
        HatchDialogMsg::Kind,
    )]
    .spacing(8);
    match &s.fill {
        HatchFill::Pattern(name) => {
            let names: Vec<String> = crate::scene::model::hatch_patterns::catalog()
                .iter()
                .map(|entry| entry.name.clone())
                .collect();
            fill = fill.push(labelled(tr("pattern"), list(name.clone(), names, HatchDialogMsg::Pattern)));
        }
        HatchFill::Gradient(gradient, invert) => {
            let names: Vec<String> = GradientKind::CHOICES
                .iter()
                .map(|(kind, inverted)| kind.choice_label(*inverted).to_string())
                .collect();
            fill = fill.push(labelled(
                tr("gradient"),
                list(
                    gradient.choice_label(*invert).to_string(),
                    names,
                    HatchDialogMsg::Gradient,
                ),
            ));
        }
        _ => {}
    }
    if matches!(s.fill, HatchFill::Gradient(..)) {
        let stops: Vec<String> = COLORS[2..].iter().map(|c| c.to_string()).collect();
        fill = fill
            .push(labelled(
                tr("color-1"),
                list(color_label(s.gradient_colors[0].clone()), stops.clone(), HatchDialogMsg::GradientColor1),
            ))
            .push(labelled(
                tr("color-2"),
                list(color_label(s.gradient_colors[1].clone()), stops, HatchDialogMsg::GradientColor2),
            ));
    } else {
        fill = fill.push(labelled(
            tr("color"),
            list(
                s.color.clone().map_or_else(current_label, color_label),
                colors,
                HatchDialogMsg::Color,
            ),
        ));
    }
    fill = fill.push(preview(state));
    if !matches!(s.fill, HatchFill::Gradient(..)) {
        fill = fill.push(labelled(
            tr("background"),
            list(
                s.background.clone().map_or_else(none_label, color_label),
                backgrounds,
                HatchDialogMsg::Background,
            ),
        ));
    }

    // ── Angle and scale ──────────────────────────────────────────────────
    let user = matches!(s.fill, HatchFill::User { .. });
    let pattern = matches!(s.fill, HatchFill::Pattern(_));
    let mut angle_scale = column![
        labelled(tr("angle"), input(&state.angle, kind != 0, HatchDialogMsg::Angle)),
        labelled(tr("scale"), input(&state.scale, pattern, HatchDialogMsg::Scale)),
    ]
    .spacing(6);
    if let HatchFill::User { double, .. } = s.fill {
        angle_scale = angle_scale
            .push(labelled(tr("spacing"), input(&state.spacing, user, HatchDialogMsg::Spacing)))
            .push(row![
                Space::new().width(Length::Fixed(116.0)),
                chip(tr("double"), double, msg(HatchDialogMsg::Double(!double)))
            ]);
    }

    // ── Origin ───────────────────────────────────────────────────────────
    // One field per line: side by side the labels leave the fields no room.
    let origin = column![
        labelled("X".into(), input(&state.origin_x, kind >= 2, HatchDialogMsg::OriginX)),
        labelled("Y".into(), input(&state.origin_y, kind >= 2, HatchDialogMsg::OriginY)),
    ]
    .spacing(8);

    // ── Boundaries ───────────────────────────────────────────────────────
    let boundaries = row![
        button(text(tr("pick-points")).size(11))
            .on_press(msg(HatchDialogMsg::Boundary(0)))
            .style(button_style(false))
            .padding([5, 10]),
        button(text(tr("select-objects")).size(11))
            .on_press(msg(HatchDialogMsg::Boundary(1)))
            .style(button_style(false))
            .padding([5, 10]),
        text(tr("boundaries-note")).size(11).style(muted_style).width(Fill),
    ]
    .spacing(8)
    .align_y(iced::Center);

    // ── Options ──────────────────────────────────────────────────────────
    let mut layers = vec![current_label()];
    layers.extend(state.layers.iter().cloned());
    let options = column![
        row![
            chip(tr("associative"), s.associative, msg(HatchDialogMsg::Associative(!s.associative))),
            chip(tr("separate"), s.separate, msg(HatchDialogMsg::Separate(!s.separate))),
            chip(
                tr("keep-boundaries"),
                s.keep_boundaries,
                msg(HatchDialogMsg::Keep(!s.keep_boundaries))
            ),
        ]
        .spacing(6),
        labelled(
            tr("island-style"),
            segmented(
                vec![(0, tr("island-normal")), (1, tr("island-outer")), (2, tr("island-ignore"))],
                island_of(s.island),
                HatchDialogMsg::Island,
            ),
        ),
        labelled(tr("gap"), input(&state.gap, true, HatchDialogMsg::Gap)),
        labelled(
            tr("draw-order"),
            list(
                draw_order_labels()[s.draw_order.min(4) as usize].clone(),
                draw_order_labels(),
                HatchDialogMsg::DrawOrder,
            ),
        ),
        row![
            Space::new().width(Length::Fixed(116.0)),
            chip(tr("annotative"), s.annotative, msg(HatchDialogMsg::Annotative(!s.annotative))),
        ],
        labelled(
            tr("layer"),
            list(
                s.layer.clone().unwrap_or_else(current_label),
                layers,
                HatchDialogMsg::Layer,
            ),
        ),
        labelled(
            tr("transparency"),
            text_input(&current_label(), &state.transparency)
                .size(11)
                .padding([4, 8])
                .width(Fill)
                .style(field_style)
                .on_input(|s| msg(HatchDialogMsg::Transparency(s))),
        ),
    ]
    .spacing(6);

    let mut body = column![
        card(tr("fill-title"), fill),
        row![
            card(tr("angle-title"), angle_scale),
            card(tr("origin-title"), origin),
        ]
        .spacing(10),
        card(tr("boundaries-title"), boundaries),
        card(tr("options-title"), options),
    ]
    .spacing(10);
    if let Some(error) = &state.error {
        body = body.push(
            text(error.as_str())
                .size(11)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.palette().danger.base.color),
                }),
        );
    }
    column![
        // The options card runs past a short window: the cards scroll, the
        // buttons stay.
        iced::widget::scrollable(body.padding(iced::Padding::ZERO.right(10))).height(Fill),
        row![
            Space::new().width(Fill),
            dialog_button(t!("Cancel"), Message::CloseModal, false),
            dialog_button(t!("OK"), msg(HatchDialogMsg::Ok), true),
        ]
        .spacing(6),
    ]
    .spacing(12)
    .padding([10, 12])
    .width(sizing.width)
    .height(sizing.height)
    .into()
}

pub fn view_warning<'a>(
    state: &'a BoundaryWarningState,
    sizing: crate::ui::modal::ModalSizing,
) -> Element<'a, Message> {
    let tr = |key: &str| crate::i18n::loader().get_attr("hatch-warning", key);
    let toggle = button(
        text(if state.details { tr("hide-details") } else { tr("show-details") }).size(11),
    )
    .on_press(msg(HatchDialogMsg::WarningDetails(!state.details)))
    .style(button::text)
    .padding([2, 0]);
    let mut text_column = column![
        text(tr("heading")).size(13).font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..iced::Font::DEFAULT
        }),
        text(tr("body")).size(11).style(muted_style),
        toggle,
    ]
    .spacing(8);
    if state.details {
        text_column = text_column.push(
            container(
                column![
                    text(crate::tr!(
                        "hatch-warning",
                        "detail-gap",
                        gap = state.gap.to_string()
                    ))
                    .size(11),
                    text(tr("detail-hint")).size(11).style(muted_style),
                ]
                .spacing(4),
            )
            .padding([8, 10])
            .width(Fill)
            .style(well_style),
        );
    }
    column![
        row![warning_badge(), text_column.width(Fill)].spacing(12),
        iced::widget::rule::horizontal(1),
        row![
            chip(
                tr("dont-show"),
                state.dont_show,
                msg(HatchDialogMsg::WarningDontShow(!state.dont_show))
            ),
            Space::new().width(Fill),
            dialog_button(
                std::borrow::Cow::Owned(tr("close")),
                msg(HatchDialogMsg::WarningClose),
                true
            ),
        ]
        .align_y(iced::Center),
    ]
    .spacing(12)
    .padding([12, 14])
    .width(sizing.width)
    .into()
}

/// The settings after one edit.
pub fn edit(state: &mut HatchSettingsState, message: &HatchDialogMsg) {
    let s = &mut state.settings;
    state.error = None;
    match message {
        HatchDialogMsg::Kind(kind) => {
            s.fill = match kind {
                0 => HatchFill::Solid,
                1 => HatchFill::Gradient(GradientKind::Linear, false),
                2 => HatchFill::Pattern(
                    crate::modules::draw::draw::hatch::last_pattern_name("ANSI31"),
                ),
                _ => HatchFill::User {
                    spacing: 1.0,
                    double: false,
                },
            };
        }
        HatchDialogMsg::Pattern(name) => s.fill = HatchFill::Pattern(name.clone()),
        HatchDialogMsg::Gradient(label) => {
            if let Some((kind, invert)) = GradientKind::from_choice_label(label) {
                s.fill = HatchFill::Gradient(kind, invert);
            }
        }
        HatchDialogMsg::Color(label) => {
            s.color = if *label == current_label() { None } else { parse_color(label) };
        }
        HatchDialogMsg::Background(label) => {
            s.background = if *label == none_label() { None } else { parse_color(label) };
        }
        HatchDialogMsg::Angle(text) => state.angle = text.clone(),
        HatchDialogMsg::Scale(text) => state.scale = text.clone(),
        HatchDialogMsg::Spacing(text) => state.spacing = text.clone(),
        HatchDialogMsg::Double(on) => {
            if let HatchFill::User { double, .. } = &mut s.fill {
                *double = *on;
            }
        }
        HatchDialogMsg::OriginX(text) => state.origin_x = text.clone(),
        HatchDialogMsg::OriginY(text) => state.origin_y = text.clone(),
        HatchDialogMsg::Associative(on) => s.associative = *on,
        HatchDialogMsg::Separate(on) => {
            s.separate = *on;
            if *on {
                s.keep_boundaries = false;
            }
        }
        HatchDialogMsg::Keep(on) => {
            s.keep_boundaries = *on;
            if *on {
                s.separate = false;
            }
        }
        HatchDialogMsg::Island(style) => {
            s.island = match style {
                1 => HatchStyleType::Outer,
                2 => HatchStyleType::Ignore,
                _ => HatchStyleType::Normal,
            };
        }
        HatchDialogMsg::Gap(text) => state.gap = text.clone(),
        HatchDialogMsg::Layer(name) => {
            s.layer = (*name != current_label()).then(|| name.clone());
        }
        HatchDialogMsg::Transparency(text) => state.transparency = text.clone(),
        HatchDialogMsg::GradientColor1(label) => {
            if let Some(color) = parse_color(label) {
                s.gradient_colors[0] = color;
            }
        }
        HatchDialogMsg::GradientColor2(label) => {
            if let Some(color) = parse_color(label) {
                s.gradient_colors[1] = color;
            }
        }
        HatchDialogMsg::DrawOrder(label) => {
            if let Some(order) = draw_order_labels().iter().position(|l| l == label) {
                s.draw_order = order as u8;
            }
        }
        HatchDialogMsg::Annotative(on) => s.annotative = *on,
        _ => {}
    }
}
