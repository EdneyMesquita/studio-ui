//! `.seg`: segmented control (view switchers, kinds, ON/OFF).

use egui::{Response, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::theme::{self, alpha};

/// One segment. `badge` is drawn faint after the label (`Holding 18`).
#[derive(Clone, Copy, Debug)]
pub struct SegItem<'a, T> {
    pub value: T,
    pub icon: Option<&'a str>,
    pub label: &'a str,
    /// Label in a compact control (`HR` for `Holding`); none: an item with an icon shows the icon alone.
    pub short: Option<&'a str>,
    pub badge: Option<&'a str>,
    pub enabled: bool,
    pub tooltip: Option<&'a str>,
}

impl<'a, T> SegItem<'a, T> {
    pub fn new(value: T, label: &'a str) -> Self {
        Self {
            value,
            icon: None,
            label,
            short: None,
            badge: None,
            enabled: true,
            tooltip: None,
        }
    }
    pub fn icon(mut self, glyph: &'a str) -> Self {
        self.icon = Some(glyph);
        self
    }
    /// The label of a compact control.
    pub fn short(mut self, short: &'a str) -> Self {
        self.short = Some(short);
        self
    }
    pub fn badge(mut self, badge: &'a str) -> Self {
        self.badge = Some(badge);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn tooltip(mut self, tip: &'a str) -> Self {
        self.tooltip = Some(tip);
        self
    }
}

/// Segmented control; returns the response of the whole group, `changed()` when `current` changed.
pub fn seg<T: PartialEq + Copy>(
    ui: &mut Ui,
    current: &mut T,
    items: &[SegItem<'_, T>],
) -> Response {
    seg_fit(ui, current, items, false)
}

/// The label an item shows: in a compact control its short label, or nothing when it has an icon.
fn shown_label<'a, T>(it: &SegItem<'a, T>, compact: bool) -> &'a str {
    match (compact, it.short, it.icon) {
        (false, ..) => it.label,
        (true, Some(s), _) => s,
        (true, None, Some(_)) => "",
        (true, None, None) => it.label,
    }
}

const PAD: f32 = 10.0;
const GAP: f32 = 6.0;

/// One segment's width and its laid-out label and badge.
fn measure<T>(
    ui: &Ui,
    it: &SegItem<'_, T>,
    compact: bool,
    t: &theme::Theme,
) -> (
    f32,
    std::sync::Arc<egui::Galley>,
    Option<std::sync::Arc<egui::Galley>>,
) {
    let font = theme::ui_font(theme::FS);
    let pad = if compact { 8.0 } else { PAD };
    let g = super::galley(ui, shown_label(it, compact), font.clone(), t.text);
    let b = it.badge.map(|b| super::galley(ui, b, font, t.text_3));
    let mut w = pad * 2.0 + g.size().x;
    if it.icon.is_some() {
        w += theme::ICON_SM + GAP;
    }
    if let Some(b) = &b {
        w += 5.0 + b.size().x;
    }
    if g.size().x == 0.0 && it.icon.is_some() {
        // the icon alone, its badge (`Faults 2`) right after it
        w = 12.0 + theme::ICON_SM + b.as_ref().map_or(0.0, |b| 4.0 + b.size().x);
    }
    (w, g, b)
}

/// The width [`seg_fit`] takes for these items (galleys are cached, so this is cheap).
pub fn seg_width<T>(ui: &Ui, items: &[SegItem<'_, T>], compact: bool) -> f32 {
    let t = theme::theme(ui);
    let n = items.len().min(16);
    items
        .iter()
        .take(n)
        .map(|it| measure(ui, it, compact, t).0)
        .sum::<f32>()
        + 2.0 * (n.saturating_sub(1)) as f32
        + 4.0
}

/// [`seg`] as a form field's control: Rabbit's `SegmentedControl` is `inline-flex`, but a `Field`'s
/// flex column stretches it, so its track spans the field with the segments at the left.
pub fn seg_stretch<T: PartialEq + Copy>(
    ui: &mut Ui,
    current: &mut T,
    items: &[SegItem<'_, T>],
) -> Response {
    seg_impl(ui, current, items, false, true)
}

/// [`seg`] that can be compact for a narrow pane: items with an icon show it alone (their label becomes the
/// tooltip), others their short label.
pub fn seg_fit<T: PartialEq + Copy>(
    ui: &mut Ui,
    current: &mut T,
    items: &[SegItem<'_, T>],
    compact: bool,
) -> Response {
    seg_impl(ui, current, items, compact, false)
}

fn seg_impl<T: PartialEq + Copy>(
    ui: &mut Ui,
    current: &mut T,
    items: &[SegItem<'_, T>],
    compact: bool,
    stretch: bool,
) -> Response {
    let t = theme::theme(ui);
    let pad = if compact { 8.0 } else { PAD };
    let gap = GAP;
    // measure
    let mut widths = [0.0f32; 16];
    let mut galleys = Vec::with_capacity(items.len());
    for (i, it) in items.iter().enumerate() {
        let (w, g, b) = measure(ui, it, compact, t);
        if let Some(slot) = widths.get_mut(i) {
            *slot = w;
        }
        galleys.push((g, b));
    }
    let n = items.len().min(widths.len());
    let total: f32 = widths[..n].iter().sum::<f32>() + 2.0 * (n.saturating_sub(1)) as f32 + 4.0;
    let total = if stretch {
        total.max(ui.available_width())
    } else {
        total
    };
    let (rect, mut group) =
        ui.allocate_exact_size(Vec2::new(total, theme::CONTROL), Sense::hover());
    let painter = ui.painter().clone();
    painter.rect_filled(rect, theme::R, alpha(t.text, 0.06));
    let mut x = rect.left() + 2.0;
    for (i, (it, (g, b))) in items.iter().zip(galleys).enumerate().take(n) {
        let w = widths[i];
        let r = egui::Rect::from_min_size(egui::pos2(x, rect.top() + 2.0), Vec2::new(w, 24.0));
        x += w + 2.0;
        let id = group.id.with(i);
        let resp = ui.interact(
            r,
            id,
            if it.enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        let on = *current == it.value;
        if resp.clicked() && !on && it.enabled {
            *current = it.value;
            group.mark_changed();
        }
        match it.tooltip {
            Some(tip) => {
                super::tip(resp, tip);
            }
            // a compact segment without its label names itself in the tooltip
            None if g.size().x == 0.0 || shown_label(it, compact) != it.label => {
                super::tip(resp, it.label);
            }
            None => {}
        }
        let hot = it.enabled && ui.rect_contains_pointer(r);
        if on {
            let shadow = egui::epaint::Shadow {
                offset: [0, 1],
                blur: 2,
                spread: 0,
                color: if t.dark {
                    egui::Color32::from_black_alpha(64)
                } else {
                    egui::Color32::from_black_alpha(30)
                },
            };
            painter.add(shadow.as_shape(r, theme::R_SM));
            painter.rect(
                r,
                theme::R_SM,
                t.editor,
                Stroke::new(1.0, t.line),
                StrokeKind::Outside,
            );
        }
        let fg = if !it.enabled {
            alpha(t.text_2, 0.45)
        } else if on || hot {
            t.text
        } else {
            t.text_2
        };
        let cy = r.center().y;
        let mut cx = r.left() + pad;
        if g.size().x == 0.0 && it.icon.is_some() {
            cx = r.left() + 6.0;
        }
        if let Some(icon) = it.icon {
            theme::paint_icon(&painter, egui::pos2(cx + 7.0, cy), icon, theme::ICON_SM, fg);
            cx += theme::ICON_SM + gap;
        }
        let gw = g.size().x;
        painter.galley_with_override_text_color(egui::pos2(cx, cy - g.size().y / 2.0), g, fg);
        cx += if gw == 0.0 { 4.0 - gap } else { gw + 5.0 };
        if let Some(b) = b {
            let bc = if it.enabled {
                t.text_3
            } else {
                alpha(t.text_3, 0.45)
            };
            painter.galley_with_override_text_color(egui::pos2(cx, cy - b.size().y / 2.0), b, bc);
        }
    }
    group
}
