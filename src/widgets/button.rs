//! Buttons: `.btn` (default, primary, ghost, danger, small), `.tb-btn` (toolbar) and icon buttons.

use egui::{Color32, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget};

use crate::theme::{self, Theme, alpha};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BtnKind {
    /// Hairline border, text color.
    Default,
    /// Accent fill, the one primary action of a view.
    Primary,
    /// No border, text-2; turns text on hover.
    Ghost,
    /// Danger fill (destructive confirm).
    Danger,
}

/// `.btn`. Build with [`Btn::new`] and chain modifiers, then `ui.add(btn)`.
#[derive(Clone, Copy, Debug)]
pub struct Btn<'a> {
    label: &'a str,
    icon: Option<&'a str>,
    kind: BtnKind,
    small: bool,
    kbd: Option<&'a str>,
    enabled: bool,
    pressed: bool,
    icon_only: bool,
    icon_size: f32,
}

impl<'a> Btn<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            kind: BtnKind::Default,
            small: false,
            kbd: None,
            enabled: true,
            pressed: false,
            icon_only: false,
            icon_size: theme::ICON_SM,
        }
    }
    pub fn icon(mut self, glyph: &'a str) -> Self {
        self.icon = Some(glyph);
        self
    }
    /// A 16 px icon: Rabbit's `Button` default (`[&_svg:not([class*='size-'])]:size-4`) where its source
    /// gives the icon no size of its own; 14 px otherwise.
    pub fn icon16(mut self) -> Self {
        self.icon_size = theme::ICON;
        self
    }
    pub fn kind(mut self, kind: BtnKind) -> Self {
        self.kind = kind;
        self
    }
    pub fn primary(self) -> Self {
        self.kind(BtnKind::Primary)
    }
    pub fn ghost(self) -> Self {
        self.kind(BtnKind::Ghost)
    }
    pub fn danger(self) -> Self {
        self.kind(BtnKind::Danger)
    }
    /// 24 px high, 8 px padding.
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }
    /// A keycap after the label (`Read now ⏎`).
    pub fn kbd(mut self, k: &'a str) -> Self {
        self.kbd = Some(k);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// In a narrow toolbar: the icon alone (no label, no keycap); the label becomes the tooltip.
    pub fn icon_only(mut self, yes: bool) -> Self {
        self.icon_only = yes && self.icon.is_some();
        self
    }
    /// The width the button takes (galleys are cached, so this is cheap).
    pub fn width(&self, ui: &Ui) -> f32 {
        self.layout(ui).0
    }
    /// Width, height, padding and the label and keycap galleys.
    #[allow(clippy::type_complexity)]
    fn layout(
        &self,
        ui: &Ui,
    ) -> (
        f32,
        f32,
        f32,
        Option<std::sync::Arc<egui::Galley>>,
        Option<std::sync::Arc<egui::Galley>>,
    ) {
        let t = theme::theme(ui);
        let (h, pad) = if self.small {
            (24.0, 8.0)
        } else {
            (theme::CONTROL, 10.0)
        };
        let font = match self.kind {
            // `size="sm"`: `text-sm`
            BtnKind::Primary | BtnKind::Danger if self.small => theme::medium(theme::FS_SM),
            BtnKind::Primary | BtnKind::Danger => theme::medium(theme::FS),
            _ if self.small => theme::ui_font(theme::FS_SM),
            _ => theme::ui_font(theme::FS),
        };
        let (fg, ..) = colors(t, self.kind);
        let label = if self.icon_only { "" } else { self.label };
        let g = (!label.is_empty()).then(|| super::galley(ui, label, font, fg));
        let kg = self
            .kbd
            .filter(|_| !self.icon_only)
            .map(|k| super::galley(ui, k, theme::medium(10.5), fg));
        let gap = self.gap();
        let mut w = pad * 2.0;
        let mut parts = 0;
        if self.icon.is_some() {
            w += self.icon_size;
            parts += 1;
        }
        if let Some(g) = &g {
            w += g.size().x;
            parts += 1;
        }
        if let Some(k) = &kg {
            w += super::kbd_size(k).x;
            parts += 1;
        }
        w += gap * (parts.max(1) - 1) as f32;
        if g.is_none() && self.icon.is_some() && kg.is_none() {
            w = h;
        }
        (w, h, pad, g, kg)
    }
    /// `gap-1.5`, and `gap-1` for `size="sm"`.
    fn gap(&self) -> f32 {
        if self.small { 4.0 } else { 6.0 }
    }
    /// Pressed look (toggle buttons such as `Pause` while paused).
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }
}

impl Widget for Btn<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme::theme(ui);
        let (fg, bg, border) = colors(t, self.kind);
        let (w, h, pad, g, kg) = self.layout(ui);
        let gap = self.gap();
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(w, h), sense);
        if self.icon_only {
            resp = super::tip(resp, self.label);
        }
        if !ui.is_rect_visible(rect) {
            return resp;
        }
        let hovered = self.enabled && (resp.hovered() || resp.is_pointer_button_down_on());
        let (mut fg, mut bg, mut border) = (fg, bg, border);
        if hovered || self.pressed {
            match self.kind {
                BtnKind::Default => bg = t.hover,
                BtnKind::Ghost => {
                    bg = t.hover;
                    fg = t.text;
                }
                BtnKind::Primary => {
                    bg = t.accent_hover;
                    border = t.accent_hover;
                }
                // `hover:bg-destructive/90`: 90 % of the fill over the dialog behind it
                BtnKind::Danger => {
                    bg = theme::mix(t.danger, 0.9, t.editor);
                    border = bg;
                }
            }
        }
        let opacity = if self.enabled { 1.0 } else { 0.45 };
        let p = ui.painter();
        let mut p = p.clone();
        p.set_opacity(opacity);
        p.rect(
            rect,
            theme::R,
            bg,
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );
        let mut x = if w == h && g.is_none() {
            rect.center().x - self.icon_size / 2.0
        } else {
            rect.left() + pad
        };
        let cy = rect.center().y;
        if let Some(icon) = self.icon {
            theme::paint_icon(
                &p,
                egui::pos2(x + self.icon_size / 2.0, cy),
                icon,
                self.icon_size,
                fg,
            );
            x += self.icon_size + gap;
        }
        if let Some(g) = g {
            let gw = g.size().x;
            p.galley_with_override_text_color(egui::pos2(x, cy - g.size().y / 2.0), g, fg);
            x += gw + gap;
        }
        if let Some(k) = kg {
            let ks = super::kbd_size(&k);
            let r = egui::Rect::from_min_size(egui::pos2(x, (cy - ks.y / 2.0).round()), ks);
            let (kb, kborder, kfg) = match self.kind {
                BtnKind::Primary | BtnKind::Danger => (
                    Color32::TRANSPARENT,
                    alpha(Color32::WHITE, 0.35),
                    alpha(Color32::WHITE, 0.85),
                ),
                _ => (t.editor, t.line, t.text_2),
            };
            super::paint_kbd(&p, r, k, kb, kborder, kfg);
        }
        if resp.has_focus() {
            super::focus_ring(ui.painter(), rect, theme::R, t);
        }
        if hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        resp
    }
}

fn colors(t: &Theme, kind: BtnKind) -> (Color32, Color32, Color32) {
    match kind {
        BtnKind::Default => (t.text, Color32::TRANSPARENT, t.field_line),
        BtnKind::Ghost => (t.text_2, Color32::TRANSPARENT, Color32::TRANSPARENT),
        BtnKind::Primary => (t.on_accent, t.accent, t.accent),
        BtnKind::Danger => (Color32::WHITE, t.danger, t.danger),
    }
}

/// `.tb-btn.icon`: a square flat icon button (28, 24 or 20 px) with a tooltip.
pub fn icon_button(ui: &mut Ui, glyph: &str, side: f32, tooltip: &str) -> Response {
    let t = theme::theme(ui);
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
    if ui.is_rect_visible(rect) {
        let hot = resp.hovered() || resp.is_pointer_button_down_on();
        if hot {
            ui.painter().rect_filled(
                rect,
                if side <= 20.0 { theme::R_SM } else { theme::R },
                t.hover,
            );
        }
        let size = if side <= 20.0 {
            theme::ICON_XS
        } else if side <= 24.0 {
            theme::ICON_SM
        } else {
            theme::ICON
        };
        theme::paint_icon(
            ui.painter(),
            rect.center(),
            glyph,
            size,
            if hot { t.text } else { t.text_2 },
        );
    }
    if resp.has_focus() {
        super::focus_ring(ui.painter(), rect, theme::R, t);
    }
    super::tip(resp, tooltip)
}
