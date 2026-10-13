//! Design tokens of the family ("The Data IDE"), dark and light, and their mapping onto egui's
//! [`egui::Style`]. This module is the canonical source of the tokens (they first came from Rabbit Studio's
//! `DESIGN.md` and the family's HTML mockups); widgets never carry a hex of their own except the identity
//! chip's initials.

use std::sync::LazyLock;

use egui::epaint::Shadow;
use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, Ui, Vec2,
};

/// Every color token of one theme. Two `const` instances exist ([`DARK`], [`LIGHT`]); fetch the active one with
/// [`Theme::of`] or [`theme`], which costs nothing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub dark: bool,
    pub editor: Color32,
    pub panel: Color32,
    pub raised: Color32,
    pub seam: Color32,
    pub line: Color32,
    pub line_soft: Color32,
    pub hover: Color32,
    pub row_hover: Color32,
    pub sel: Color32,
    pub sel_soft: Color32,
    pub field: Color32,
    pub field_line: Color32,
    pub text: Color32,
    pub text_2: Color32,
    pub text_3: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_text: Color32,
    pub on_accent: Color32,
    pub ok: Color32,
    pub warn: Color32,
    pub danger: Color32,
    pub syn_key: Color32,
    pub syn_str: Color32,
    pub syn_num: Color32,
    pub syn_kw: Color32,
    pub syn_special: Color32,
    pub syn_punct: Color32,
    pub comment: Color32,
    /// Chart series: blue, orange, green, violet.
    pub s1: Color32,
    pub s2: Color32,
    pub s3: Color32,
    pub s4: Color32,
    /// Scrim behind dialogs.
    pub scrim: Color32,
    /// Shadow of floating layers (menus, palette, dialogs, toasts).
    pub shadow: Shadow,
}

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub const DARK: Theme = Theme {
    dark: true,
    editor: hex(0x1E1F22),
    panel: hex(0x2B2D30),
    raised: hex(0x2B2D30),
    seam: hex(0x1A1B1E),
    line: hex(0x393B40),
    line_soft: hex(0x2F3135),
    hover: hex(0x393B40),
    row_hover: hex(0x26282C),
    sel: hex(0x2E436E),
    sel_soft: hex(0x25324A),
    field: hex(0x1E1F22),
    field_line: hex(0x4E5157),
    text: hex(0xDFE1E5),
    text_2: hex(0xB4B8BF),
    text_3: hex(0x959AA2),
    accent: hex(0x3574F0),
    accent_hover: hex(0x4682FA),
    accent_text: hex(0x6B9BFA),
    on_accent: Color32::WHITE,
    ok: hex(0x5FB865),
    warn: hex(0xE2B45C),
    danger: hex(0xE5675F),
    syn_key: hex(0xC77DBB),
    syn_str: hex(0x6AAB73),
    syn_num: hex(0x2AACB8),
    syn_kw: hex(0xCF8E6D),
    syn_special: hex(0x56A8F5),
    syn_punct: hex(0x8C8F96),
    comment: hex(0x7A7E85),
    s1: hex(0x3987E5),
    s2: hex(0xD95926),
    s3: hex(0x199E70),
    s4: hex(0x8B6FE0),
    // Rabbit's dialog overlay: `bg-black/40`
    scrim: Color32::from_black_alpha(102),
    // `--t-shadow-overlay: 0 12px 32px rgba(0,0,0,.45), 0 2px 6px rgba(0,0,0,.3)` in one egui shadow: a
    // CSS blur radius is a Gaussian reaching about twice as far as egui's linear blur of the same
    // width, so the width is 48 and the alpha takes in the second, tight layer (measured under a menu).
    shadow: Shadow {
        offset: [0, 12],
        blur: 48,
        spread: 0,
        color: Color32::from_black_alpha(128),
    },
};

pub const LIGHT: Theme = Theme {
    dark: false,
    editor: hex(0xFFFFFF),
    panel: hex(0xF7F8FA),
    raised: hex(0xFFFFFF),
    seam: hex(0xE4E6EB),
    line: hex(0xE4E6EB),
    line_soft: hex(0xEFF0F3),
    hover: hex(0xE9EBEF),
    row_hover: hex(0xF4F6FA),
    sel: hex(0xD5E1FF),
    sel_soft: hex(0xE8EEFD),
    field: hex(0xFFFFFF),
    field_line: hex(0xC9CCD6),
    text: hex(0x1E1F22),
    text_2: hex(0x4F5361),
    text_3: hex(0x6B6F7C),
    accent: hex(0x3574F0),
    accent_hover: hex(0x2A63D6),
    accent_text: hex(0x2A63D6),
    on_accent: Color32::WHITE,
    ok: hex(0x3F8F46),
    warn: hex(0xA36B00),
    danger: hex(0xC9372C),
    syn_key: hex(0x871094),
    syn_str: hex(0x067D17),
    syn_num: hex(0x1750EB),
    syn_kw: hex(0x0033B3),
    syn_special: hex(0x00627A),
    syn_punct: hex(0x6E7180),
    comment: hex(0x8C8C8C),
    s1: hex(0x2A78D6),
    s2: hex(0xEB6834),
    s3: hex(0x1BAF7A),
    s4: hex(0x7B61D1),
    // Rabbit's dialog overlay in light: `#1e1f22` at 30 % (premultiplied)
    scrim: Color32::from_rgba_premultiplied(9, 9, 10, 77),
    // `0 12px 32px rgba(20,24,35,.16), 0 2px 6px rgba(20,24,35,.08)`, fitted as the dark one
    shadow: Shadow {
        offset: [0, 12],
        blur: 48,
        spread: 0,
        color: Color32::from_rgba_premultiplied(4, 4, 6, 46),
    },
};

/// Identity palette: the colour a user gives a connection, site or server, carried by its chip, tab underline
/// and status bar. The only identity hue in the apps.
pub mod identity {
    use super::{Color32, hex};
    pub const GREEN: Color32 = hex(0x4FA25A);
    pub const BLUE: Color32 = hex(0x3574F0);
    pub const AMBER: Color32 = hex(0xC9912C);
    pub const RED: Color32 = hex(0xD6453D);
    pub const MAGENTA: Color32 = hex(0xB4448F);
    pub const TEAL: Color32 = hex(0x1E9AA6);
    pub const VIOLET: Color32 = hex(0x7B61D1);
    pub const SLATE: Color32 = hex(0x6E7380);
    pub const ALL: [Color32; 8] = [GREEN, BLUE, AMBER, RED, MAGENTA, TEAL, VIOLET, SLATE];
    /// Initials on a connection chip (Rabbit's `text-[#15161A]`).
    pub const ON_CHIP: Color32 = super::hex(0x15161A);
}

impl Theme {
    pub fn of(theme: egui::Theme) -> &'static Theme {
        match theme {
            egui::Theme::Dark => &DARK,
            egui::Theme::Light => &LIGHT,
        }
    }

    /// The series color by index (wraps around).
    pub fn series(&self, i: usize) -> Color32 {
        [self.s1, self.s2, self.s3, self.s4][i % 4]
    }
}

/// The active theme of a ui (follows `ui.visuals().dark_mode`, so a light popup inside a dark app still works).
pub fn theme(ui: &Ui) -> &'static Theme {
    if ui.visuals().dark_mode {
        &DARK
    } else {
        &LIGHT
    }
}

/// CSS `color-mix(in srgb, a t, b)`: `t` of `a` over `b`, opaque result when `b` is opaque.
pub fn mix(a: Color32, t: f32, b: Color32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 * t + y as f32 * (1.0 - t)).round() as u8;
    Color32::from_rgba_unmultiplied(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
        l(a.a(), b.a()),
    )
}

/// `a` at opacity `t` (premultiplied), like `color-mix(in srgb, a t, transparent)`.
pub fn alpha(a: Color32, t: f32) -> Color32 {
    a.gamma_multiply(t)
}

// ------------------------------------------------------------------ metrics

pub const FS: f32 = 13.0;
pub const FS_SM: f32 = 12.0;
pub const FS_XS: f32 = 11.0;
pub const FS_DATA: f32 = 12.5;
pub const ROW: f32 = 24.0;
pub const CONTROL: f32 = 28.0;
pub const FIELD: f32 = 32.0;
pub const BAR: f32 = 40.0;
pub const TAB_ROW: f32 = 36.0;
pub const STATUS: f32 = 26.0;
pub const INDENT: f32 = 16.0;
pub const R_XS: u8 = 3;
pub const R_SM: u8 = 4;
pub const R: u8 = 6;
pub const R_LG: u8 = 10;
pub const ICON: f32 = 16.0;
pub const ICON_SM: f32 = 14.0;
pub const ICON_XS: f32 = 12.0;

// ------------------------------------------------------------------ fonts

pub const INTER_MEDIUM: &str = "inter-medium";
pub const INTER_SEMIBOLD: &str = "inter-semibold";
pub const MONO_MEDIUM: &str = "mono-medium";
pub const ICONS: &str = "icons";

static F_MEDIUM: LazyLock<FontFamily> = LazyLock::new(|| FontFamily::Name(INTER_MEDIUM.into()));
static F_SEMIBOLD: LazyLock<FontFamily> = LazyLock::new(|| FontFamily::Name(INTER_SEMIBOLD.into()));
static F_MONO_MEDIUM: LazyLock<FontFamily> = LazyLock::new(|| FontFamily::Name(MONO_MEDIUM.into()));
static F_ICONS: LazyLock<FontFamily> = LazyLock::new(|| FontFamily::Name(ICONS.into()));

/// Inter Regular (chrome text).
pub fn ui_font(size: f32) -> FontId {
    FontId::proportional(size)
}
/// Inter Medium (labels, connection names, buttons).
pub fn medium(size: f32) -> FontId {
    FontId::new(size, F_MEDIUM.clone())
}
/// Inter SemiBold (titles, panel heads).
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, F_SEMIBOLD.clone())
}
/// JetBrains Mono Regular (all data).
pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}
/// JetBrains Mono Medium (metrics, values that need weight).
pub fn mono_medium(size: f32) -> FontId {
    FontId::new(size, F_MONO_MEDIUM.clone())
}
/// The Lucide icon font; glyphs are in [`crate::icons`].
pub fn icon_font(size: f32) -> FontId {
    FontId::new(size, F_ICONS.clone())
}
/// Body chrome text, 13 px.
pub fn body() -> FontId {
    ui_font(FS)
}
/// Data text, JetBrains Mono 12.5 px.
pub fn data() -> FontId {
    mono(FS_DATA)
}

/// Font definitions: Inter and JetBrains Mono first, egui's defaults behind them as glyph fallbacks.
pub fn fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let add = |fonts: &mut FontDefinitions, name: &str, bytes: &'static [u8]| {
        fonts.font_data.insert(
            name.to_owned(),
            std::sync::Arc::new(FontData::from_static(bytes)),
        );
    };
    add(
        &mut fonts,
        "Inter-Regular",
        include_bytes!("../assets/fonts/Inter-Regular.ttf"),
    );
    add(
        &mut fonts,
        "Inter-Medium",
        include_bytes!("../assets/fonts/Inter-Medium.ttf"),
    );
    add(
        &mut fonts,
        "Inter-SemiBold",
        include_bytes!("../assets/fonts/Inter-SemiBold.ttf"),
    );
    add(
        &mut fonts,
        "JetBrainsMono-Regular",
        include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    );
    add(
        &mut fonts,
        "JetBrainsMono-Medium",
        include_bytes!("../assets/fonts/JetBrainsMono-Medium.ttf"),
    );
    fonts.font_data.insert(
        "lucide".to_owned(),
        std::sync::Arc::new(FontData {
            tweak: egui::FontTweak {
                y_offset_factor: 0.0,
                ..Default::default()
            },
            ..FontData::from_static(include_bytes!("../assets/fonts/lucide.ttf"))
        }),
    );

    let fallbacks: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let with = |first: &str| {
        let mut v = vec![first.to_owned(), "Inter-Regular".to_owned()];
        v.extend(fallbacks.iter().cloned());
        v.push("lucide".to_owned());
        v
    };
    fonts
        .families
        .insert(FontFamily::Proportional, with("Inter-Regular"));
    fonts
        .families
        .insert(FontFamily::Name(INTER_MEDIUM.into()), with("Inter-Medium"));
    fonts.families.insert(
        FontFamily::Name(INTER_SEMIBOLD.into()),
        with("Inter-SemiBold"),
    );
    let mono_with = |first: &str| {
        let mut v = vec![
            first.to_owned(),
            "JetBrainsMono-Regular".to_owned(),
            "Inter-Regular".to_owned(),
        ];
        v.extend(fallbacks.iter().cloned());
        v
    };
    fonts
        .families
        .insert(FontFamily::Monospace, mono_with("JetBrainsMono-Regular"));
    fonts.families.insert(
        FontFamily::Name(MONO_MEDIUM.into()),
        mono_with("JetBrainsMono-Medium"),
    );
    fonts
        .families
        .insert(FontFamily::Name(ICONS.into()), vec!["lucide".to_owned()]);
    fonts
}

// ------------------------------------------------------------------ egui style

/// The egui style of one theme: visuals from the tokens, IDE density spacing, family type scale.
pub fn style(t: &Theme) -> egui::Style {
    use egui::{FontFamily as F, TextStyle};

    let mut s = egui::Style {
        text_styles: [
            (TextStyle::Small, FontId::new(FS_XS, F::Proportional)),
            (TextStyle::Body, FontId::new(FS, F::Proportional)),
            (TextStyle::Button, FontId::new(FS, F::Proportional)),
            (TextStyle::Heading, FontId::new(15.0, F_SEMIBOLD.clone())),
            (TextStyle::Monospace, FontId::new(FS_DATA, F::Monospace)),
        ]
        .into(),
        ..Default::default()
    };
    s.animation_time = 0.12;
    s.spacing.item_spacing = Vec2::new(8.0, 4.0);
    s.spacing.button_padding = Vec2::new(10.0, 4.0);
    s.spacing.interact_size = Vec2::new(24.0, ROW);
    s.spacing.menu_margin = egui::Margin::same(4);
    s.spacing.menu_spacing = 2.0;
    s.spacing.window_margin = egui::Margin::same(12);
    s.spacing.icon_width = 14.0;
    s.spacing.icon_width_inner = 8.0;
    // WebKitGTK's overlay bars under `scrollbar-width: thin`: hidden while idle, a 3 px thumb 3 px from the
    // edge once the pointer is over the area, in `fg-3` at 50 % (the text color at 31 % dark, 33 % light),
    // no track; wider under the pointer.
    s.spacing.scroll = egui::style::ScrollStyle {
        bar_width: 8.0,
        floating_width: 3.0,
        bar_inner_margin: 2.0,
        bar_outer_margin: 3.0,
        handle_min_length: 24.0,
        dormant_background_opacity: 0.0,
        active_background_opacity: 0.0,
        interact_background_opacity: 0.0,
        dormant_handle_opacity: 0.0,
        active_handle_opacity: if t.dark { 0.31 } else { 0.33 },
        interact_handle_opacity: 0.45,
        ..egui::style::ScrollStyle::floating()
    };
    s.interaction.tooltip_delay = 0.4;
    s.interaction.selectable_labels = false;

    let v = &mut s.visuals;
    *v = if t.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    // Glyph weight: measured against the WebKit-rendered mockup (mean ink of a data grid's name and
    // type columns at 1x). egui's defaults draw dark text ~35 % heavier and light text ~15 % heavier.
    v.text_options.color_transfer_function =
        egui::epaint::FontColorTransferFunction::Gamma(if t.dark { 1.15 } else { 1.2 });
    // Rabbit dims disabled controls to 45-50 %.
    v.disabled_alpha = 0.5;
    v.text_cursor.blink = false;
    // Rabbit sets no `caret-color`: WebKit draws the caret in the text color, 1 px wide
    v.text_cursor.stroke = Stroke::new(1.0, t.text);
    v.weak_text_color = Some(t.text_3);
    v.hyperlink_color = t.accent_text;
    v.faint_bg_color = t.row_hover;
    v.extreme_bg_color = t.field;
    v.text_edit_bg_color = Some(t.field);
    v.code_bg_color = t.panel;
    v.warn_fg_color = t.warn;
    v.error_fg_color = t.danger;
    v.panel_fill = t.editor;
    v.window_fill = t.raised;
    v.window_stroke = Stroke::new(1.0, t.line);
    v.window_corner_radius = CornerRadius::same(R_LG);
    v.menu_corner_radius = CornerRadius::same(8);
    v.window_shadow = t.shadow;
    v.popup_shadow = t.shadow;
    v.window_highlight_topmost = false;
    v.selection.bg_fill = t.sel;
    // `::selection { background: sel; color: fg }`: egui recolors selected text with this stroke
    v.selection.stroke = Stroke::new(1.0, t.text);
    v.striped = false;
    v.indent_has_left_vline = false;
    v.collapsing_header_frame = false;
    v.button_frame = true;
    v.resize_corner_size = 10.0;

    let w = &mut v.widgets;
    let r = CornerRadius::same(R);
    w.noninteractive.bg_fill = t.panel;
    w.noninteractive.weak_bg_fill = t.panel;
    w.noninteractive.bg_stroke = Stroke::new(1.0, t.line);
    w.noninteractive.fg_stroke = Stroke::new(1.0, t.text);
    w.noninteractive.corner_radius = CornerRadius::same(R_SM);
    w.inactive.bg_fill = t.field;
    w.inactive.weak_bg_fill = Color32::TRANSPARENT;
    w.inactive.bg_stroke = Stroke::new(1.0, t.field_line);
    w.inactive.fg_stroke = Stroke::new(1.0, t.text);
    w.inactive.corner_radius = r;
    w.inactive.expansion = 0.0;
    w.hovered.bg_fill = t.hover;
    w.hovered.weak_bg_fill = t.hover;
    w.hovered.bg_stroke = Stroke::new(1.0, t.field_line);
    w.hovered.fg_stroke = Stroke::new(1.0, t.text);
    w.hovered.corner_radius = r;
    w.hovered.expansion = 0.0;
    w.active.bg_fill = t.hover;
    w.active.weak_bg_fill = t.hover;
    w.active.bg_stroke = Stroke::new(1.0, t.accent);
    w.active.fg_stroke = Stroke::new(1.0, t.text);
    w.active.corner_radius = r;
    w.active.expansion = 0.0;
    w.open = w.hovered;
    s
}

/// Install fonts and both styles, dark first (the family default).
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(fonts());
    ctx.set_style_of(egui::Theme::Dark, style(&DARK));
    ctx.set_style_of(egui::Theme::Light, style(&LIGHT));
    ctx.set_theme(egui::Theme::Dark);
}

// ------------------------------------------------------------------ icon helpers

/// Paint a Lucide glyph centered on `center`.
pub fn paint_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    glyph: &str,
    size: f32,
    color: Color32,
) {
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        glyph,
        icon_font(size),
        color,
    );
}

/// A non-interactive icon taking `size × size`.
pub fn icon(ui: &mut Ui, glyph: &str, size: f32, color: Color32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        paint_icon(ui.painter(), rect.center(), glyph, size, color);
    }
    resp
}
