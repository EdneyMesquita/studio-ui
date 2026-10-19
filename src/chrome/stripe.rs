//! The 40 px tool stripe (Rabbit `ToolStripe`, the mockup's `.stripe`): one 30 px button per tool window
//! from the top, 34 px apart, the pressed one with an accent bar at the edge; an attention dot on a
//! button that needs it; and a slot at the bottom for buttons that open something else.
//!
//! ```no_run
//! # use studio_ui::chrome::stripe::{self, StripeItem};
//! # use studio_ui::{egui, icons};
//! # fn f(ui: &mut egui::Ui, side: &mut usize, problems: usize) {
//! let tip = format!("Problems ({problems})");
//! let top = [
//!     StripeItem::new(icons::NETWORK, "Explorer").pressed(*side == 0).shortcut("Ctrl B"),
//!     StripeItem::new(icons::BELL, &tip).pressed(*side == 1).attention(problems > 0),
//! ];
//! let bottom = [StripeItem::new(icons::ARROW_LEFT_RIGHT, "Import / export connections")];
//! let out = stripe::stripe(ui, &top, &bottom);
//! if let Some(k) = out.clicked { *side = k; }
//! # }
//! ```
//!
//! Which tool windows exist is the app's: the stripe paints the items it is given.
//!
//! # How the apps map onto it
//!
//! | Mb Studio `chrome/stripe.rs` | This module |
//! |---|---|
//! | Explorer, Register maps, Recordings, Problems; pressed = `side_visible && side == k` | four [`StripeItem`]s, `.pressed(..)`, Explorer `.shortcut("Ctrl B")` |
//! | the import / export button 36 px above the bottom | `bottom: &[StripeItem::new(icons::ARROW_LEFT_RIGHT, "Import / export connections")]` |
//! | `Action::Side(side)` on a click | [`StripeResponse::clicked`] (index into `top`) |
//!
//! | OPC Studio `chrome/stripe.rs` | This module |
//! |---|---|
//! | six items, the Problems tip `Problems (N)` | the same, `.attention(problems > 0)` for the red dot on the bell |
//! | the bottom import / export button | `bottom` as for Mb; [`StripeResponse::bottom_clicked`] |

use egui::{Rect, Response, Sense, Ui, Vec2, pos2};

use crate::theme;
use crate::widgets::{self, TipSide};

/// The top of the first button, from the stripe's top.
const TOP: f32 = 6.0;
/// One button's pitch.
const PITCH: f32 = 34.0;
/// A button's side.
const BTN: f32 = 30.0;

/// One button of the stripe.
#[derive(Clone, Copy, Debug)]
pub struct StripeItem<'a> {
    pub icon: &'a str,
    /// The tooltip (right of the button).
    pub tip: &'a str,
    /// Faint after the tip (`Ctrl B`).
    pub shortcut: Option<&'a str>,
    /// Its tool window shows: hover fill and the accent bar.
    pub pressed: bool,
    /// The danger dot over the icon's top right (Problems while there are any).
    pub attention: bool,
}

impl<'a> StripeItem<'a> {
    pub fn new(icon: &'a str, tip: &'a str) -> Self {
        Self {
            icon,
            tip,
            shortcut: None,
            pressed: false,
            attention: false,
        }
    }

    #[must_use]
    pub fn pressed(mut self, yes: bool) -> Self {
        self.pressed = yes;
        self
    }

    #[must_use]
    pub fn attention(mut self, yes: bool) -> Self {
        self.attention = yes;
        self
    }

    #[must_use]
    pub fn shortcut(mut self, s: &'a str) -> Self {
        self.shortcut = Some(s);
        self
    }
}

/// What the stripe reports: every button's response, and which one was clicked.
#[derive(Debug, Default)]
pub struct StripeResponse {
    pub top: Vec<Response>,
    pub bottom: Vec<Response>,
    /// Index into `top`.
    pub clicked: Option<usize>,
    /// Index into `bottom`.
    pub bottom_clicked: Option<usize>,
}

/// The `k`th top button of a stripe in `rect`.
pub fn top_slot(rect: Rect, k: usize) -> Rect {
    Rect::from_min_size(
        pos2(
            rect.center().x - BTN / 2.0,
            rect.top() + TOP + k as f32 * PITCH,
        ),
        Vec2::splat(BTN),
    )
}

/// The `k`th bottom button of a stripe in `rect`, counted upward from the bottom.
pub fn bottom_slot(rect: Rect, k: usize) -> Rect {
    Rect::from_min_size(
        pos2(
            rect.center().x - BTN / 2.0,
            rect.bottom() - 36.0 - k as f32 * PITCH,
        ),
        Vec2::splat(BTN),
    )
}

/// The button under `p`: `Some((false, k))` for the `k`th top button, `Some((true, k))` for a bottom one.
pub fn hit(rect: Rect, top: usize, bottom: usize, p: egui::Pos2) -> Option<(bool, usize)> {
    (0..top)
        .find(|&k| top_slot(rect, k).contains(p))
        .map(|k| (false, k))
        .or_else(|| {
            (0..bottom)
                .find(|&k| bottom_slot(rect, k).contains(p))
                .map(|k| (true, k))
        })
}

/// The stripe in `ui`'s max rect (a 40 px panel): `panel` fill, the `seam` on its right, `top` from the top
/// and `bottom` stacked upward from the bottom.
pub fn stripe(ui: &mut Ui, top: &[StripeItem<'_>], bottom: &[StripeItem<'_>]) -> StripeResponse {
    let t = theme::theme(ui);
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0, t.panel);
    widgets::vline(ui.painter(), rect.right() - 0.5, rect.y_range(), t.seam);
    let mut out = StripeResponse::default();
    for (k, it) in top.iter().enumerate() {
        let r = top_slot(rect, k);
        let resp = button(ui, r, ui.id().with(("stripe", k)), it, true);
        if resp.clicked() {
            out.clicked = Some(k);
        }
        out.top.push(resp);
    }
    for (k, it) in bottom.iter().enumerate() {
        let r = bottom_slot(rect, k);
        let resp = button(ui, r, ui.id().with(("stripe-io", k)), it, false);
        if resp.clicked() {
            out.bottom_clicked = Some(k);
        }
        out.bottom.push(resp);
    }
    out
}

fn button(ui: &mut Ui, r: Rect, id: egui::Id, it: &StripeItem<'_>, bar: bool) -> Response {
    let t = theme::theme(ui);
    let resp = ui.interact(r, id, Sense::click());
    let pressed = it.pressed;
    if pressed || resp.hovered() {
        ui.painter().rect_filled(r, theme::R, t.hover);
    }
    if pressed && bar {
        let bar = Rect::from_min_max(
            pos2(r.left() - 5.0, r.top() + 7.0),
            pos2(r.left() - 3.0, r.bottom() - 7.0),
        );
        ui.painter().rect_filled(bar, 2, t.accent);
    }
    let fg = if pressed || resp.hovered() {
        t.text
    } else {
        t.text_2
    };
    theme::paint_icon(ui.painter(), r.center(), it.icon, theme::ICON, fg);
    if it.attention {
        ui.painter()
            .circle_filled(pos2(r.center().x + 6.0, r.center().y - 7.0), 3.5, t.danger);
    }
    widgets::tip_at(resp, it.tip, TipSide::RIGHT, it.shortcut)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stripe_rect() -> Rect {
        Rect::from_min_size(pos2(0.0, 40.0), Vec2::new(40.0, 800.0))
    }

    #[test]
    fn buttons_stack_from_the_top_and_from_the_bottom() {
        let r = stripe_rect();
        assert_eq!(
            top_slot(r, 0),
            Rect::from_min_size(pos2(5.0, 46.0), Vec2::splat(30.0))
        );
        assert_eq!(top_slot(r, 3).top(), 46.0 + 3.0 * 34.0);
        // the apps' one bottom button: 36 px above the bottom edge
        assert_eq!(bottom_slot(r, 0).top(), 840.0 - 36.0);
        assert_eq!(bottom_slot(r, 1).top(), 840.0 - 36.0 - 34.0);
    }

    #[test]
    fn hit_testing_tells_top_from_bottom_and_gaps() {
        let r = stripe_rect();
        assert_eq!(hit(r, 6, 1, top_slot(r, 5).center()), Some((false, 5)));
        assert_eq!(hit(r, 6, 1, bottom_slot(r, 0).center()), Some((true, 0)));
        // the 4 px between two buttons is nothing
        let gap = pos2(20.0, top_slot(r, 0).bottom() + 2.0);
        assert_eq!(hit(r, 6, 1, gap), None);
        // a slot past the item count is nothing
        assert_eq!(hit(r, 2, 1, top_slot(r, 4).center()), None);
    }
}
