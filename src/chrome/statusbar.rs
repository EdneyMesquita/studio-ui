//! The 26 px status bar (Rabbit `StatusBar`, the mockup's `.statusbar`): one left-to-right row of segments
//! clipped to the bar. A [`SegKind::Spacer`] splits the left group from the right one. Segments are plain
//! text, buttons (hover fill, a menu or a click), the identity of what the active tab belongs to, or the
//! live dot that beats when data arrives. Short of room, the segments the app marks with a [`Shrink`] give
//! way, tier by tier and with an ellipsis; whatever still does not fit is cut at the right end.
//!
//! ```no_run
//! # use studio_ui::chrome::statusbar::{self, Seg, Shrink};
//! # use studio_ui::{egui, icons, theme::identity};
//! # fn f(ui: &mut egui::Ui, beat: u64, problems: usize) {
//! let p = format!("{problems} problems");
//! let ring = statusbar::live_ring(ui, egui::Id::new("app-live-beat"), true, beat);
//! let segs = [
//!     Seg::identity(Some(identity::TEAL), "Water treatment", Some("Signed"))
//!         .shrink(Shrink { tier: 1, min: 0.0 }),
//!     Seg::live(true, ring, "Live · 500 ms"),
//!     Seg::button("problems", &p).icon(icons::TRIANGLE_ALERT),
//!     Seg::spacer(),
//!     Seg::text("tcp://plc.example:5000").mono().shrink(Shrink { tier: 0, min: 64.0 }),
//! ];
//! let out = statusbar::statusbar(ui, &segs);
//! if out[2].as_ref().is_some_and(|r| r.clicked()) { /* show the problems */ }
//! # }
//! ```
//!
//! # How the apps map onto it
//!
//! | Mb Studio `chrome/statusbar.rs` | This module |
//! |---|---|
//! | `Item::Conn`: dot, name, transport; only it and the first right-hand segment shrink, in proportion | [`Seg::identity`] with [`Gives::Name`] and `Shrink { tier: 0, min: 0.0 }` (only while a connection is active), the first right segment `Shrink { tier: 0, min: 0.0 }` |
//! | `Item::Count` (`N of M connected`, plug icon, the open-connections menu) | [`Seg::button`]`(..).icon(PLUG)`; the menu is the app's, on the response |
//! | `Item::Live` (`Polling · 1s` / `Paused`, `ring`) | [`Seg::live`]`(polling, `[`live_ring`]`(ui, Id::new("mb-live-beat"), polling, beat), text)` |
//! | `Item::Busy` (activity icon, hover only) | [`Seg::button`]`(..).icon(ACTIVITY)` |
//! | `Item::Problems` (icon in danger / warn / text-2) | [`Seg::button`]`(..).icon(TRIANGLE_ALERT).icon_color(c)` |
//! | `Item::Guard`: read-only (lock, warn) or, from 1100 px, live plant (shield, text-2) | [`Seg::text`]`(..).icon(..).color(warn)` / `.icon(SHIELD_ALERT)`; the app decides the 1100 px |
//! | `Seg { text, mono, hide_below }` from the view | [`Seg::text`]`(..)` `.mono()` `.hide_below(w)` |
//! | `Item::Notation` and its "Address notation" menu | [`Seg::button`]`(..).icon(HASH)` + [`pick_menu`] |
//!
//! | OPC Studio `chrome/statusbar.rs` | This module |
//! |---|---|
//! | `Item::Server`: name gives way after the security label, painted clipped to the item | [`Seg::identity`] with [`Gives::DetailFirst`], `Shrink { tier: 1, min: 0.0 }` while a server is active |
//! | `Item::Url`: gives way first, down to a 64 px stub | [`Seg::text`]`(url).mono().shrink(Shrink { tier: 0, min: 64.0 })` |
//! | `Item::Session`, `Item::RoundTrip` (hover tooltips), shown over 960 px | [`Seg::text`]`(..).icon(..)`; the app leaves them out at 960 px and under |
//! | `Item::Guard`: read-only (eye, text-2), live plant (danger-tinted badge, `+2` px) | [`Seg::text`]`(..).icon(EYE)` / [`Seg::text`]`(..).icon(LOCK).icon_color(danger).color(mix(danger, .7, text)).tint(alpha(danger, .11)).extra_width(2.0)` |
//! | `Item::Live` (`Publishing`), its ring on the view model's version | [`Seg::live`] with [`live_ring`]`(ui, Id::new("opc-live-beat"), ..)` |
//! | `Item::Workspace` | [`Seg::text`] |
//! | `Item::Notation` ("Show NodeIds as") | [`Seg::button`]`(..).icon(HASH)` + [`pick_menu`] |
//!
//! Both apps capped the bar at a fixed number of items (16 and 12); the kit takes any number.

use egui::{Color32, Galley, Id, Rect, Response, Sense, Ui, Vec2, pos2};
use std::sync::Arc;

use crate::theme::{self, Theme};
use crate::widgets::{self, menu};

/// Space between items (`gap-0.5`).
pub const GAP: f32 = 2.0;
/// An item's side padding (`px-[7px]`).
pub const PAD: f32 = 7.0;
/// The live dot's ring stays this long after a beat (Rabbit `LiveIndicator`).
const RING_S: f64 = 0.7;
/// The dot beats at most this often, however often data arrives (Rabbit `BEAT_EVERY`).
const BEAT_EVERY_S: f64 = 4.0;

/// How the identity segment gives way when it shrinks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gives {
    /// The name is cut; the detail keeps its width after it (and may spill to the bar's clip).
    Name,
    /// The detail is cut first (and dropped under 24 px), then the name; painted clipped to the segment.
    DetailFirst,
}

/// What a segment is.
#[derive(Clone, Copy, Debug)]
pub enum SegKind<'a> {
    /// Text (and an icon); no hover look. Its response senses hover, for a tooltip.
    Plain,
    /// `ITEM_BUTTON`: `hover:bg-hover hover:text-fg`, also while its menu is open.
    Button,
    /// A button led by the live dot: ok while `on`, faint otherwise, ringed while `ring`.
    Live { on: bool, ring: bool },
    /// What the active tab belongs to: an 8 px identity dot, the name in medium, a detail in text-2.
    Identity {
        color: Option<Color32>,
        detail: Option<&'a str>,
        gives: Gives,
    },
    /// Takes what the others leave: the left group ends here.
    Spacer,
}

/// How a segment shrinks when the bar is short of room: lower tiers first; within a tier in proportion to
/// what each can still give, down to its `min`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shrink {
    pub tier: u8,
    pub min: f32,
}

/// One segment; build with [`Seg::text`], [`Seg::button`], [`Seg::live`], [`Seg::identity`] or
/// [`Seg::spacer`].
#[derive(Clone, Copy, Debug)]
pub struct Seg<'a> {
    pub kind: SegKind<'a>,
    pub text: &'a str,
    /// A 12 px icon before the text.
    pub icon: Option<&'a str>,
    /// The icon's colour; the text's otherwise.
    pub icon_color: Option<Color32>,
    /// The text's colour at rest; text-2 otherwise.
    pub color: Option<Color32>,
    /// JetBrains Mono 12 (addresses, file names); 11 px UI text otherwise.
    pub mono: bool,
    /// A badge fill behind the whole segment.
    pub tint: Option<Color32>,
    /// Width added to the natural one.
    pub extra_w: f32,
    pub shrink: Option<Shrink>,
    /// Left out when the window is narrower than this (Rabbit's `max-[960px]:hidden`).
    pub hide_below: Option<f32>,
    /// Its widget id salt (keeps a menu open while other segments come and go); the position otherwise.
    pub id: Option<&'a str>,
    /// For an identity segment: it senses clicks (and the pointer shows it), answering like the buttons.
    pub clickable: bool,
}

impl<'a> Seg<'a> {
    fn new(kind: SegKind<'a>, text: &'a str) -> Self {
        Self {
            kind,
            text,
            icon: None,
            icon_color: None,
            color: None,
            mono: false,
            tint: None,
            extra_w: 0.0,
            shrink: None,
            hide_below: None,
            id: None,
            clickable: false,
        }
    }

    /// Plain text.
    pub fn text(text: &'a str) -> Self {
        Self::new(SegKind::Plain, text)
    }

    /// A button; `id` names it (its menu stays open while other segments change).
    pub fn button(id: &'a str, text: &'a str) -> Self {
        Self {
            id: Some(id),
            ..Self::new(SegKind::Button, text)
        }
    }

    /// The live segment.
    pub fn live(on: bool, ring: bool, text: &'a str) -> Self {
        Self::new(SegKind::Live { on, ring }, text)
    }

    /// The identity segment: `name` in medium text colour, after the dot when there is a `color`.
    pub fn identity(color: Option<Color32>, name: &'a str, detail: Option<&'a str>) -> Self {
        Self::new(
            SegKind::Identity {
                color,
                detail,
                gives: Gives::Name,
            },
            name,
        )
    }

    pub fn spacer() -> Self {
        Self::new(SegKind::Spacer, "")
    }

    #[must_use]
    pub fn icon(mut self, glyph: &'a str) -> Self {
        self.icon = Some(glyph);
        self
    }

    #[must_use]
    pub fn icon_color(mut self, c: Color32) -> Self {
        self.icon_color = Some(c);
        self
    }

    #[must_use]
    pub fn color(mut self, c: Color32) -> Self {
        self.color = Some(c);
        self
    }

    #[must_use]
    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }

    #[must_use]
    pub fn tint(mut self, c: Color32) -> Self {
        self.tint = Some(c);
        self
    }

    #[must_use]
    pub fn extra_width(mut self, w: f32) -> Self {
        self.extra_w = w;
        self
    }

    #[must_use]
    pub fn shrink(mut self, s: Shrink) -> Self {
        self.shrink = Some(s);
        self
    }

    /// Hidden in a window narrower than `width`.
    #[must_use]
    pub const fn hide_below(mut self, width: f32) -> Self {
        self.hide_below = Some(width);
        self
    }

    /// For an identity segment: how it gives way.
    #[must_use]
    pub fn gives(mut self, how: Gives) -> Self {
        if let SegKind::Identity { gives, .. } = &mut self.kind {
            *gives = how;
        }
        self
    }

    /// For an identity segment: it can be clicked, and its response says so like a button's.
    #[must_use]
    pub const fn clickable(mut self) -> Self {
        self.clickable = true;
        self
    }

    #[must_use]
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    /// Whether it shows in a window this wide.
    pub fn shown(&self, window: f32) -> bool {
        self.hide_below.is_none_or(|w| window >= w)
    }

    fn font(&self) -> egui::FontId {
        if self.mono {
            theme::mono(theme::FS_SM)
        } else {
            theme::ui_font(theme::FS_XS)
        }
    }
}

/// The bar's widths fitted into `avail`: the spacer (if any) takes what is left; short of room, the
/// segments with a [`Shrink`] give way tier by tier, each in proportion to what it can give above its
/// `min`. What is still too wide runs off the right end.
pub fn fit(widths: &mut [f32], spacer: Option<usize>, shrink: &[Option<Shrink>], avail: f32) {
    let len = widths.len();
    if len == 0 {
        return;
    }
    let natural: f32 = widths.iter().sum::<f32>() + GAP * (len - 1) as f32;
    if natural <= avail {
        if let Some(k) = spacer {
            widths[k] = avail - natural;
        }
        return;
    }
    let mut deficit = natural - avail;
    let mut tiers: Vec<u8> = shrink.iter().flatten().map(|s| s.tier).collect();
    tiers.sort_unstable();
    tiers.dedup();
    for tier in tiers {
        if deficit <= 0.0 {
            break;
        }
        let give = |k: usize| match shrink.get(k).copied().flatten() {
            Some(s) if s.tier == tier => (widths[k] - s.min).max(0.0),
            _ => 0.0,
        };
        let can: f32 = (0..len).map(give).sum();
        if can <= 0.0 {
            continue;
        }
        let cut = deficit.min(can);
        let gives: Vec<f32> = (0..len).map(give).collect();
        for (w, g) in widths.iter_mut().zip(gives) {
            *w -= g * cut / can;
        }
        deficit -= cut;
    }
}

/// Width of a plain item: padding, an optional 12 px icon and 6 px gap, the text.
fn item_w(g: &Galley, icon: bool) -> f32 {
    2.0 * PAD + g.size().x + if icon { 18.0 } else { 0.0 }
}

/// Paints a plain item in `r`: optional 12 px icon, then the text.
fn paint_item(ui: &Ui, r: Rect, icon: Option<(&str, Color32)>, g: Arc<Galley>, fg: Color32) {
    let p = ui.painter();
    let cy = r.center().y;
    let mut x = r.left() + PAD;
    if let Some((icon, c)) = icon {
        theme::paint_icon(p, pos2(x + 6.0, cy), icon, theme::ICON_XS, c);
        x += 18.0;
    }
    p.galley_with_override_text_color(pos2(x, cy - g.size().y / 2.0), g, fg);
}

/// A button item: hover fill, also while its menu is open. Returns its response and text colour.
fn button(ui: &mut Ui, r: Rect, id: Id, t: &Theme, fg: Color32) -> (Response, Color32) {
    let resp = ui.interact(r, id, Sense::click());
    let open = menu::is_open(&resp);
    if resp.hovered() || open {
        ui.painter().rect_filled(r, theme::R_SM, t.hover);
        (resp, t.text)
    } else {
        (resp, fg)
    }
}

/// Whether the live dot shows its ring: tracks `beat` (a counter that grows when data arrives) in temp
/// memory under `id` and starts a 700 ms ring on a new one (at most every 4 s), then asks for exactly one
/// repaint at its end. No animation: two paints per beat. Resuming (`on` again) rings the last beat once
/// more, like Rabbit's effect.
pub fn live_ring(ui: &Ui, id: Id, on: bool, beat: u64) -> bool {
    let now = ui.input(|i| i.time);
    let (seen, mut at, was_on) =
        ui.data(|d| d.get_temp::<(u64, f64, bool)>(id))
            .unwrap_or((beat, f64::NEG_INFINITY, on));
    if beat > seen && now - at >= BEAT_EVERY_S {
        at = now;
    }
    if on && !was_on && at.is_finite() {
        at = now;
    }
    ui.data_mut(|d| d.insert_temp(id, (beat, at, on)));
    let left = RING_S - (now - at);
    if on && left > 0.0 {
        crate::chrome::repaint_after(ui.ctx(), left);
        true
    } else {
        false
    }
}

/// The identity segment's natural width and parts.
struct Ident {
    name: Arc<Galley>,
    detail: Option<Arc<Galley>>,
    /// Dot and detail with their gaps.
    fixed: f32,
}

/// The status bar in `ui`'s max rect (a 26 px panel). Returns one response per segment, `None` for a
/// spacer and for a segment left out (`hide_below`) or cut off at the right end.
pub fn statusbar(ui: &mut Ui, segs: &[Seg<'_>]) -> Vec<Option<Response>> {
    let t = theme::theme(ui);
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0, t.panel);
    widgets::hline(ui.painter(), rect.x_range(), rect.top() + 0.5, t.seam);
    // `overflow-hidden`: clipped at the bar's padding box, under the top border
    let clip = Rect::from_min_max(pos2(rect.left(), rect.top() + 1.0), rect.max);
    ui.set_clip_rect(clip.intersect(ui.clip_rect()));
    let window = crate::chrome::window_width(ui);
    let mut out: Vec<Option<Response>> = (0..segs.len()).map(|_| None).collect();

    // Measure: every shown segment's natural width, left to right.
    let shown: Vec<usize> = (0..segs.len()).filter(|&k| segs[k].shown(window)).collect();
    let mut galleys: Vec<Option<Arc<Galley>>> = Vec::with_capacity(shown.len());
    let mut ident: Option<Ident> = None;
    let mut widths: Vec<f32> = Vec::with_capacity(shown.len());
    for &k in &shown {
        let s = &segs[k];
        let (g, w) = match s.kind {
            SegKind::Spacer => (None, 0.0),
            SegKind::Identity { color, detail, .. } => {
                let name = widgets::galley(ui, s.text, theme::medium(theme::FS_XS), t.text);
                let detail =
                    detail.map(|d| widgets::galley(ui, d, theme::ui_font(theme::FS_XS), t.text_2));
                let fixed = if color.is_some() { 8.0 + 6.0 } else { 0.0 }
                    + detail.as_ref().map_or(0.0, |v| 6.0 + v.size().x);
                let w = 2.0 * PAD + fixed + name.size().x;
                ident = Some(Ident {
                    name: name.clone(),
                    detail,
                    fixed,
                });
                (Some(name), w)
            }
            SegKind::Live { .. } => {
                let g = widgets::galley(ui, s.text, s.font(), t.text_2);
                let w = 2.0 * PAD + 6.0 + 6.0 + g.size().x;
                (Some(g), w)
            }
            SegKind::Plain | SegKind::Button => {
                let g = widgets::galley(ui, s.text, s.font(), t.text_2);
                let w = item_w(&g, s.icon.is_some());
                (Some(g), w)
            }
        };
        galleys.push(g);
        widths.push(w + s.extra_w);
    }
    let spacer = shown
        .iter()
        .position(|&k| matches!(segs[k].kind, SegKind::Spacer));
    let shrink: Vec<Option<Shrink>> = shown.iter().map(|&k| segs[k].shrink).collect();
    let natural = widths.clone();
    fit(&mut widths, spacer, &shrink, rect.width() - 16.0);

    // Paint, left to right.
    let mut x = rect.left() + 8.0;
    let cy = rect.center().y + 0.5;
    for (j, &k) in shown.iter().enumerate() {
        let s = &segs[k];
        let w = widths[j];
        let r = Rect::from_min_size(pos2(x, cy - 11.0), Vec2::new(w, 22.0));
        x += w + GAP;
        if r.left() > clip.right() {
            break;
        }
        let id = match s.id {
            Some(salt) => ui.id().with(("status", salt)),
            None => ui.id().with(("status", k)),
        };
        let rest = s.color.unwrap_or(t.text_2);
        if let Some(tint) = s.tint {
            ui.painter().rect_filled(r, theme::R_SM, tint);
        }
        // the text, cut when the segment gave way
        let fitted = |ui: &Ui, g: &Arc<Galley>, icon: bool| {
            if w + 0.01 < natural[j] {
                let room = w - 2.0 * PAD - if icon { 18.0 } else { 0.0 } - s.extra_w;
                widgets::galley_truncated(ui, s.text, s.font(), rest, room)
            } else {
                g.clone()
            }
        };
        match s.kind {
            SegKind::Spacer => {}
            SegKind::Plain => {
                let Some(g) = &galleys[j] else { continue };
                let resp = ui.interact(r, id, Sense::hover());
                let g = fitted(ui, g, s.icon.is_some());
                let icon = s.icon.map(|i| (i, s.icon_color.unwrap_or(rest)));
                paint_item(ui, r, icon, g, rest);
                out[k] = Some(resp);
            }
            SegKind::Button => {
                let Some(g) = &galleys[j] else { continue };
                let (resp, fg) = button(ui, r, id, t, rest);
                let g = fitted(ui, g, s.icon.is_some());
                let icon = s.icon.map(|i| (i, s.icon_color.unwrap_or(fg)));
                paint_item(ui, r, icon, g, fg);
                out[k] = Some(resp);
            }
            SegKind::Live { on, ring } => {
                let Some(g) = &galleys[j] else { continue };
                let (resp, fg) = button(ui, r, id, t, rest);
                let dot = pos2(r.left() + PAD + 3.0, r.center().y);
                if ring {
                    ui.painter()
                        .circle_filled(dot, 5.5, theme::alpha(t.ok, 0.35));
                }
                let c = if on {
                    t.ok
                } else {
                    theme::alpha(t.text_3, 0.6)
                };
                ui.painter().circle_filled(dot, 3.0, c);
                ui.painter().galley_with_override_text_color(
                    pos2(r.left() + PAD + 12.0, r.center().y - g.size().y / 2.0),
                    g.clone(),
                    fg,
                );
                out[k] = Some(resp);
            }
            SegKind::Identity { .. } => {
                let Some(id_parts) = &ident else { continue };
                // clickable, it hovers and answers like a button
                let resp = if s.clickable {
                    button(ui, r, id, t, t.text).0
                } else {
                    ui.interact(r, id, Sense::hover())
                };
                paint_identity(ui, r, s, id_parts, t);
                out[k] = Some(resp);
            }
        }
    }
    out
}

/// The identity segment `s` in `r`.
fn paint_identity(ui: &Ui, r: Rect, s: &Seg<'_>, parts: &Ident, t: &Theme) {
    let SegKind::Identity {
        color,
        detail,
        gives,
    } = s.kind
    else {
        return;
    };
    let name = s.text;
    let w = r.width();
    let cy = r.center().y;
    let p = match gives {
        Gives::Name => ui.painter().clone(),
        Gives::DetailFirst => ui.painter_at(r),
    };
    let mut x = r.left() + PAD;
    if let Some(c) = color {
        p.rect_filled(
            Rect::from_center_size(pos2(x + 4.0, cy), Vec2::splat(8.0)),
            2,
            c,
        );
        x += 8.0 + 6.0;
    }
    let name_font = theme::medium(theme::FS_XS);
    let cut = |room: f32| {
        if parts.name.size().x > room {
            widgets::galley_truncated(ui, name, name_font.clone(), t.text, room)
        } else {
            parts.name.clone()
        }
    };
    match gives {
        Gives::Name => {
            // `truncate`: the name takes what the detail leaves (the detail may spill, like Rabbit's)
            let room = (w - 2.0 * PAD - parts.fixed).max(0.0);
            let ng = cut(room);
            let nw = ng.size().x.min(room);
            p.galley(pos2(x, cy - ng.size().y / 2.0), ng, t.text);
            x += nw + 6.0;
            if let Some(v) = &parts.detail {
                p.galley(pos2(x, cy - v.size().y / 2.0), v.clone(), t.text_2);
            }
        }
        Gives::DetailFirst => {
            let dot = if color.is_some() { 8.0 + 6.0 } else { 0.0 };
            let ng = cut((w - 2.0 * PAD - dot).max(0.0));
            let nw = ng.size().x;
            p.galley(pos2(x, cy - ng.size().y / 2.0), ng, t.text);
            x += nw + 6.0;
            let left = r.right() - PAD - x;
            if let (Some(v), Some(text)) = (&parts.detail, detail)
                && left > 24.0
            {
                let vg = if v.size().x > left {
                    widgets::galley_truncated(
                        ui,
                        text,
                        theme::ui_font(theme::FS_XS),
                        t.text_2,
                        left,
                    )
                } else {
                    v.clone()
                };
                p.galley(pos2(x, cy - vg.size().y / 2.0), vg, t.text_2);
            }
        }
    }
}

/// A segment's choice menu (the address or NodeId notation): `head`, then each option with a check on
/// the `current` one. Opens over `resp`; `key` names it for `menu::dev_open`. Returns the option picked.
pub fn pick_menu(
    resp: &Response,
    key: &str,
    head: &str,
    options: &[&str],
    current: usize,
) -> Option<usize> {
    let mut pick = None;
    menu::dev_open(resp, || key.to_owned());
    menu::dropdown(resp, |ui| {
        menu::head(ui, head);
        for (i, o) in options.iter().enumerate() {
            let icon = (i == current).then_some(crate::icons::CHECK);
            if menu::item(ui, icon, o, None).clicked() {
                pick = Some(i);
            }
        }
    });
    pick
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chrome::{NARROW, shares};

    // Mb Studio's own test, unchanged.
    #[test]
    fn segments_hide_under_their_breakpoint() {
        let s = Seg::text("Erlang").hide_below(NARROW);
        assert!(s.shown(960.0));
        assert!(!s.shown(959.0));
        assert!(Seg::text("unit 10").mono().shown(400.0));
    }

    const T0: Option<Shrink> = Some(Shrink { tier: 0, min: 0.0 });

    #[test]
    fn room_to_spare_goes_to_the_spacer() {
        let mut w = [100.0, 50.0, 0.0, 80.0];
        fit(&mut w, Some(2), &[None; 4], 400.0);
        // 230 of items, 3 gaps of 2
        assert_eq!(w, [100.0, 50.0, 400.0 - 236.0, 80.0]);
    }

    #[test]
    fn mb_policy_is_shares_between_the_identity_and_the_first_right_segment() {
        // identity 200, live 80, spacer, a 150 right segment, notation 70; 120 short
        for avail in [460.0, 400.0, 10.0] {
            let mut w = [200.0, 80.0, 0.0, 150.0, 70.0];
            let shrink = [T0, None, None, T0, None];
            fit(&mut w, Some(2), &shrink, avail);
            let natural = 500.0 + 4.0 * GAP;
            let (a, b) = shares(200.0, 150.0, 350.0 - (natural - avail));
            assert!(
                (w[0] - a).abs() < 1e-3 && (w[3] - b).abs() < 1e-3,
                "{avail}: {w:?}"
            );
            // the others never shrink; the spacer stays empty
            assert_eq!((w[1], w[2], w[4]), (80.0, 0.0, 70.0));
        }
    }

    #[test]
    fn opc_policy_cuts_the_endpoint_to_a_stub_then_the_identity() {
        let url = Some(Shrink { tier: 0, min: 64.0 });
        let server = Some(Shrink { tier: 1, min: 0.0 });
        let shrink = [server, None, None, url];
        // 50 short: only the endpoint gives way
        let mut w = [180.0, 100.0, 0.0, 200.0];
        fit(&mut w, Some(2), &shrink, 486.0 - 50.0);
        assert_eq!(w, [180.0, 100.0, 0.0, 150.0]);
        // 200 short: the endpoint stops at 64, the server gives the other 64
        let mut w = [180.0, 100.0, 0.0, 200.0];
        fit(&mut w, Some(2), &shrink, 486.0 - 200.0);
        assert_eq!(w, [116.0, 100.0, 0.0, 64.0]);
        // far too short: the server to nothing, the rest runs off the end
        let mut w = [180.0, 100.0, 0.0, 200.0];
        fit(&mut w, Some(2), &shrink, 50.0);
        assert_eq!(w, [0.0, 100.0, 0.0, 64.0]);
        // an endpoint already under its stub never grows
        let mut w = [180.0, 100.0, 0.0, 40.0];
        fit(&mut w, Some(2), &shrink, 300.0);
        assert_eq!(w[3], 40.0);
    }

    #[test]
    fn identity_gives_way_by_name_or_detail() {
        let mut s = Seg::identity(None, "Workspace", None);
        assert!(matches!(
            s.kind,
            SegKind::Identity {
                gives: Gives::Name,
                ..
            }
        ));
        s = s.gives(Gives::DetailFirst);
        assert!(matches!(
            s.kind,
            SegKind::Identity {
                gives: Gives::DetailFirst,
                ..
            }
        ));
        // not clickable unless asked
        assert!(!Seg::identity(None, "x", None).clickable);
        assert!(Seg::identity(None, "x", None).clickable().clickable);
        // `gives` leaves other kinds alone
        assert!(matches!(
            Seg::text("x").gives(Gives::DetailFirst).kind,
            SegKind::Plain
        ));
    }

    #[test]
    fn a_clickable_identity_senses_clicks() {
        let ctx = egui::Context::default();
        crate::theme::install(&ctx);
        let mut senses = Vec::new();
        let mut frame = ctx.run_ui(egui::RawInput::default(), |ui| {
            let segs = [
                Seg::identity(None, "A", None),
                Seg::identity(None, "B", None).clickable(),
            ];
            for r in statusbar(ui, &segs).into_iter().flatten() {
                senses.push(r.sense.senses_click());
            }
        });
        frame.textures_delta.clear();
        assert_eq!(senses, [false, true]);
    }
}
