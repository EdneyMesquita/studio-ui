//! A protocol-neutral requested → granted list: what a client asked for, what the server granted, and a mark
//! where they differ.
//!
//! Every protocol of the family negotiates: OPC UA's CreateSubscription and CreateMonitoredItems come back with
//! *revised* values, MQTT's CONNACK answers CONNECT's properties, sets limits of its own and leaves some out. The
//! widget takes rows of plain text, [`Grant`]s, each with a [`State`] the application worked out, and draws them
//! as `requested → granted` lines ([`Style::Inline`], an inspector's rows) or as a table with a column for each
//! side, what is in effect and a note ([`Style::Columns`]). It never parses or compares values: the state says
//! whether the server revised a value, and [`State::compare`] works it out for the simple cases.
//!
//! ```ignore
//! let rows = [
//!     Grant::new("PublishingInterval", State::Revised).requested("100").granted("250").unit(" ms"),
//!     Grant::new("MaxKeepAliveCount", State::AsRequested).requested("10").granted("10"),
//!     Grant::new("LifetimeCount", State::Pending).requested("30"),
//! ];
//! GrantList::new("contract", &rows).show(ui);
//! ```
//!
//! # States
//!
//! | [`State`]       | means                                                     | inline granted          | columns: granted        | columns: in effect   |
//! |-----------------|-----------------------------------------------------------|-------------------------|-------------------------|----------------------|
//! | `Pending`       | no answer yet                                             | `-`, faint              | `—`, faint              | `—`, faint           |
//! | `AsRequested`   | granted as asked, or the answer left it out               | the value               | the value, or `not sent: as requested`, faint | faint |
//! | `Revised`       | the server changed it: **the mark**                       | warn, medium            | the value               | warn, medium         |
//! | `Granted`       | a value of the server's own, not a revision (a limit)     | the value               | the value               | text                 |
//! | `Default`       | not sent (by either side): the server's default applies   | the value, ` server default` faint | `absent: ` + the value, faint | faint |
//! | `Unsupported`   | the server does not offer it                              | `unsupported`, warn     | the value, or `0`       | warn, medium         |
//!
//! A requested value of `None` means the client did not send it: inline it reads `-` (OPC's), in the columns
//! `—`, faint. [`Grant::faint_requested`] draws a requested text faint, for an explanation that is not a value
//! ("client accepts 2"). What is in effect is [`Grant::effect`] when given (`10 out · 20 in`), else the granted
//! value, else the requested one, with the unit.
//!
//! # How the family maps onto it
//!
//! **OPC Studio's publishing inspector** (`app/src/views/publishing/mod.rs`, `inspector`): its `row` +
//! `arrow_job` lines are [`Style::Inline`]` { label_w: 176.0 }` (the default). `arrow_job(asked, got, unit,
//! differs)` is `Grant::new(label, state).requested(asked).granted(got).unit(unit)` with the state `Pending` when
//! `got` is `None`, `Revised` when `differs`, else `AsRequested`; `Contract::revised` (half a millisecond of
//! tolerance) stays the app's test. The rows that are not negotiated (`MaxNotificationsPerPublish`, `Priority`,
//! the session's) stay `plain_job` rows in the app, or are `Granted` rows with no requested value. **Its items
//! table** keeps its own `sampling`/`rev`, `queue`/`rev` columns; [`granted_color`] gives the `rev` cell's
//! colour (warn when revised, text otherwise) and `—` stays the app's dash.
//!
//! **IIoT Studio's MQTT session** (the mockup's `negRows`, `table.grid.mq-neg`): [`Style::Columns`] with
//! [`GrantList::heads`]` (["property", ("CONNECT", "this client"), ("CONNACK", "broker"), "in effect",
//! "note"])`. Session Expiry Interval answered with another value is `Revised`, left out of CONNACK is
//! `AsRequested` with no granted value (`not sent: as requested`); Server Keep Alive is `Revised`; Receive
//! Maximum, Topic Alias Maximum and Maximum Packet Size are `Granted` with `effect("10 out · 20 in")`; an absent
//! Maximum QoS or Retain Available is `Default` with the default as the granted value and no requested one; a
//! present Maximum QoS is `Revised`; Shared Subscription Available 0 is `Unsupported` with
//! `effect("not available")`. The SUBACK's granted QoS per filter is the same list. OPC UA's session
//! (RevisedSessionTimeout, the revised sampling intervals) fits the same rows when it is shown.
//!
//! # Not matched exactly
//!
//! - The mockup's note column wraps; the columns style's rows are 24 px and truncate a long note (the inline
//!   style wraps the value, as OPC's rows did).
//! - The mockup writes free text in the cells (`Session Present 1`, `Assigned Client Identifier "…"`): pass it
//!   as the granted text. Faint free text in the granted column is not offered; only the states' own words are
//!   faint there.

use std::hash::Hash;

use egui::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Color32, FontId, Id, Sense, Ui, Vec2};

use super::grid::{Column, HEADER_H, ROW_H};
use super::rich_grid::RichGrid;
use crate::theme::{self, Theme};

/// Pending, inline.
pub const PENDING_INLINE: &str = "-";
/// Pending or not sent, in the columns.
pub const DASH: &str = "\u{2014}";
/// The answer left the value out, so the request stands.
pub const AS_REQUESTED: &str = "not sent: as requested";
/// Before a server default in the columns.
pub const ABSENT: &str = "absent: ";
/// After a server default, inline.
pub const SERVER_DEFAULT: &str = " server default";
/// An unsupported value, inline.
pub const UNSUPPORTED: &str = "unsupported";

/// Where a negotiated value stands. See the module docs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum State {
    #[default]
    Pending,
    AsRequested,
    Revised,
    Granted,
    Default,
    Unsupported,
}

impl State {
    /// The state of a value asked as `requested` and answered as `granted`: `Pending` with no answer,
    /// `AsRequested` when they are equal, `Revised` when not, `Granted` when nothing was asked.
    pub fn compare<T: PartialEq>(requested: Option<T>, granted: Option<T>) -> State {
        match (requested, granted) {
            (_, None) => State::Pending,
            (None, Some(_)) => State::Granted,
            (Some(a), Some(b)) if a == b => State::AsRequested,
            (Some(_), Some(_)) => State::Revised,
        }
    }

    /// The server's answer differs from what was asked (the mark).
    pub fn marked(self) -> bool {
        matches!(self, State::Revised | State::Unsupported)
    }
}

/// One negotiated value. Borrowed text, built each frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Grant<'a> {
    pub key: &'a str,
    /// What the client asked; `None` when it did not send it.
    pub requested: Option<&'a str>,
    /// What the server answered; `None` when it has not answered or left it out.
    pub granted: Option<&'a str>,
    pub state: State,
    /// After the value (`" ms"`, `" s"`).
    pub unit: &'a str,
    /// What is in force, when it is not simply the granted (or requested) value.
    pub effect: Option<&'a str>,
    pub note: &'a str,
    /// Draw the requested text faint: an explanation, not a value.
    pub faint_requested: bool,
}

impl<'a> Grant<'a> {
    pub fn new(key: &'a str, state: State) -> Self {
        Self {
            key,
            state,
            ..Self::default()
        }
    }
    pub fn requested(mut self, value: &'a str) -> Self {
        self.requested = Some(value);
        self
    }
    pub fn granted(mut self, value: &'a str) -> Self {
        self.granted = Some(value);
        self
    }
    pub fn unit(mut self, unit: &'a str) -> Self {
        self.unit = unit;
        self
    }
    pub fn effect(mut self, effect: &'a str) -> Self {
        self.effect = Some(effect);
        self
    }
    pub fn note(mut self, note: &'a str) -> Self {
        self.note = note;
        self
    }
    pub fn faint_requested(mut self) -> Self {
        self.faint_requested = true;
        self
    }
}

/// How a piece of text is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink {
    Text,
    Faint,
    /// The mark: warn, in the medium weight.
    Mark,
    /// Warn, regular weight (a grid cell).
    Warn,
}

/// A run of text and its ink.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub text: String,
    pub ink: Ink,
}

fn piece(text: impl Into<String>, ink: Ink) -> Piece {
    Piece {
        text: text.into(),
        ink,
    }
}

/// The inline line of a row: `requested → granted unit`.
pub fn inline_pieces(g: &Grant<'_>) -> Vec<Piece> {
    let asked_ink = if g.faint_requested {
        Ink::Faint
    } else {
        Ink::Text
    };
    let mut out = vec![
        match g.requested {
            Some(r) => piece(r, asked_ink),
            None => piece(PENDING_INLINE, Ink::Faint),
        },
        piece(" \u{2192} ", Ink::Faint),
    ];
    let value = g.granted.or(g.requested);
    let mut unit = true;
    match (g.state, g.granted) {
        (State::Pending, _) => {
            out.push(piece(PENDING_INLINE, Ink::Faint));
            unit = false;
        }
        (State::Revised, Some(v)) => out.push(piece(v, Ink::Mark)),
        (State::Unsupported, v) => {
            out.push(piece(v.unwrap_or(UNSUPPORTED), Ink::Mark));
            unit = false;
        }
        (State::Default, Some(v)) => {
            out.push(piece(v, Ink::Text));
            if !g.unit.is_empty() {
                out.push(piece(g.unit, Ink::Text));
            }
            out.push(piece(SERVER_DEFAULT, Ink::Faint));
            unit = false;
        }
        _ => match value {
            Some(v) => out.push(piece(v, Ink::Text)),
            None => {
                out.push(piece(PENDING_INLINE, Ink::Faint));
                unit = false;
            }
        },
    }
    if unit && !g.unit.is_empty() {
        out.push(piece(g.unit, Ink::Text));
    }
    out
}

/// The three value cells of a row in the columns style: requested, granted, in effect.
pub fn column_pieces(g: &Grant<'_>) -> [Piece; 3] {
    let with_unit = |v: &str| {
        if g.unit.is_empty() {
            v.to_owned()
        } else {
            format!("{v}{}", g.unit)
        }
    };
    let requested = match g.requested {
        Some(r) if g.faint_requested => piece(r, Ink::Faint),
        Some(r) => piece(with_unit(r), Ink::Text),
        None => piece(DASH, Ink::Faint),
    };
    let granted = match (g.state, g.granted) {
        (State::Pending, _) => piece(DASH, Ink::Faint),
        (State::AsRequested, None) => piece(AS_REQUESTED, Ink::Faint),
        (State::Default, Some(v)) => piece(format!("{ABSENT}{}", with_unit(v)), Ink::Faint),
        (State::Default, None) => piece(ABSENT.trim_end_matches([' ', ':']), Ink::Faint),
        (State::Unsupported, None) => piece("0", Ink::Text),
        (_, Some(v)) => piece(with_unit(v), Ink::Text),
        (_, None) => piece(DASH, Ink::Faint),
    };
    let ink = match g.state {
        State::Pending | State::AsRequested | State::Default => Ink::Faint,
        State::Granted => Ink::Text,
        State::Revised | State::Unsupported => Ink::Mark,
    };
    let effect = match (g.effect, g.state) {
        (Some(e), _) => piece(e, ink),
        (None, State::Pending) => piece(DASH, Ink::Faint),
        (None, State::Unsupported) => piece(UNSUPPORTED, ink),
        (None, _) => match g.granted.or(g.requested) {
            Some(v) => piece(with_unit(v), ink),
            None => piece(DASH, Ink::Faint),
        },
    };
    [requested, granted, effect]
}

/// The colour of a granted value in a grid cell: warn when the server revised it (or does not offer it), text
/// otherwise.
pub fn granted_color(state: State, t: &Theme) -> Color32 {
    if state.marked() { t.warn } else { t.text }
}

fn ink_format(ink: Ink, size: f32, t: &Theme) -> TextFormat {
    let (font, color) = match ink {
        Ink::Text => (theme::mono(size), t.text),
        Ink::Faint => (theme::mono(size), t.text_3),
        Ink::Mark => (theme::mono_medium(size), t.warn),
        Ink::Warn => (theme::mono(size), t.warn),
    };
    TextFormat::simple(font, color)
}

/// Pieces as one layout job in the data size (mono 12.5).
pub fn job(pieces: &[Piece], t: &Theme) -> LayoutJob {
    let mut job = LayoutJob::default();
    for p in pieces {
        job.append(&p.text, 0.0, ink_format(p.ink, theme::FS_DATA, t));
    }
    job
}

/// Height of an inline row whose value is `value_h` tall: 4 px above and below, at least 23 px.
pub fn inline_height(value_h: f32) -> f32 {
    (value_h + 8.0).max(23.0)
}

/// Height of the columns style for `rows` rows: the header and one grid row each.
pub fn columns_height(rows: usize) -> f32 {
    HEADER_H + rows as f32 * ROW_H
}

/// How the list looks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Style {
    /// An inspector's rows: the key in a `label_w` column (11 px, text 2), then `requested → granted unit`,
    /// wrapping. OPC's publishing inspector uses 176 px.
    Inline { label_w: f32 },
    /// A grid: key, requested, granted, in effect, note.
    Columns,
}

impl Default for Style {
    fn default() -> Self {
        Style::Inline { label_w: 176.0 }
    }
}

/// A column head of the columns style: a title and an optional faint hint after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Head<'a> {
    pub title: &'a str,
    pub hint: Option<&'a str>,
}

impl<'a> From<&'a str> for Head<'a> {
    fn from(title: &'a str) -> Self {
        Head { title, hint: None }
    }
}

impl<'a> From<(&'a str, &'a str)> for Head<'a> {
    fn from((title, hint): (&'a str, &'a str)) -> Self {
        Head {
            title,
            hint: Some(hint),
        }
    }
}

/// What the list hands back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GrantResponse {
    pub hovered: Option<usize>,
    pub clicked: Option<usize>,
}

/// The list.
pub struct GrantList<'a> {
    id: Id,
    rows: &'a [Grant<'a>],
    style: Style,
    heads: [Head<'a>; 5],
    selected: Option<usize>,
}

impl std::fmt::Debug for GrantList<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GrantList")
            .field("id", &self.id)
            .field("rows", &self.rows.len())
            .field("style", &self.style)
            .finish()
    }
}

impl<'a> GrantList<'a> {
    pub fn new(id_salt: impl Hash + std::fmt::Debug, rows: &'a [Grant<'a>]) -> Self {
        Self {
            id: Id::new(id_salt),
            rows,
            style: Style::default(),
            heads: [
                "property".into(),
                "requested".into(),
                "granted".into(),
                "in effect".into(),
                "note".into(),
            ],
            selected: None,
        }
    }
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    /// The columns style's heads: key, requested, granted, in effect, note.
    pub fn heads(mut self, heads: [Head<'a>; 5]) -> Self {
        self.heads = heads;
        self
    }
    /// The columns style: a selected row.
    pub fn selected(mut self, row: Option<usize>) -> Self {
        self.selected = row;
        self
    }

    pub fn show(self, ui: &mut Ui) -> GrantResponse {
        let t = theme::theme(ui);
        match self.style {
            Style::Inline { label_w } => self.inline(ui, label_w, t),
            Style::Columns => self.columns(ui, t),
        }
    }

    fn inline(&self, ui: &mut Ui, label_w: f32, t: &Theme) -> GrantResponse {
        let mut out = GrantResponse::default();
        for (i, g) in self.rows.iter().enumerate() {
            let width = ui.available_width();
            let mut job = job(&inline_pieces(g), t);
            job.wrap.max_width = (width - label_w - 12.0).max(40.0);
            let vg = ui.painter().layout_job(job);
            let (rect, resp) = ui
                .allocate_exact_size(Vec2::new(width, inline_height(vg.size().y)), Sense::hover());
            if resp.hovered() {
                out.hovered = Some(i);
            }
            super::hline(ui.painter(), rect.x_range(), rect.top() + 0.5, t.line_soft);
            let lg = super::galley(ui, g.key, theme::ui_font(theme::FS_XS), t.text_2);
            ui.painter().galley(
                egui::pos2(rect.left() + 10.0, rect.top() + 11.5 - lg.size().y / 2.0),
                lg,
                t.text_2,
            );
            ui.painter().galley(
                egui::pos2(rect.left() + label_w, rect.center().y - vg.size().y / 2.0),
                vg,
                Color32::PLACEHOLDER,
            );
        }
        out
    }

    fn columns(&self, ui: &mut Ui, t: &Theme) -> GrantResponse {
        let h = &self.heads;
        let col = |head: &Head<'a>, w: f32| {
            let c = Column::new(head.title, w);
            match head.hint {
                Some(hint) => c.hint(hint),
                None => c,
            }
        };
        let cols = [
            col(&h[0], 180.0),
            col(&h[1], 170.0),
            col(&h[2], 190.0).flex(),
            col(&h[3], 160.0),
            col(&h[4], 200.0).flex(),
        ];
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), columns_height(self.rows.len())),
            Sense::hover(),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        child.set_clip_rect(rect.intersect(ui.clip_rect()));
        let rows = self.rows;
        let resp = RichGrid::new(self.id, &cols, rows.len())
            .selected(self.selected)
            .show(&mut child, |c| {
                let Some(g) = rows.get(c.row) else {
                    return;
                };
                match c.col {
                    0 => {
                        c.text(g.key, theme::ui_font(theme::FS_SM), t.text);
                    }
                    4 => {
                        c.text(g.note, theme::ui_font(theme::FS_SM), t.text_3);
                    }
                    k => {
                        let p = &column_pieces(g)[k - 1];
                        let mut job = LayoutJob::default();
                        job.append(&p.text, 0.0, ink_format(p.ink, theme::FS_DATA, t));
                        job.wrap = TextWrapping::truncate_at_width(c.inner().width().max(0.0));
                        c.job(job);
                    }
                }
            });
        GrantResponse {
            hovered: resp.hovered,
            clicked: resp.clicked,
        }
    }
}

/// A font for callers that paint a granted value themselves: the mark's weight when `state` is marked.
pub fn granted_font(state: State, size: f32) -> FontId {
    if state.marked() {
        theme::mono_medium(size)
    } else {
        theme::mono(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(p: &[Piece]) -> String {
        p.iter().map(|p| p.text.as_str()).collect()
    }

    #[test]
    fn compare_works_out_the_simple_states() {
        assert_eq!(State::compare(Some(100), Some(250)), State::Revised);
        assert_eq!(State::compare(Some(10), Some(10)), State::AsRequested);
        assert_eq!(State::compare(Some(10), None), State::Pending);
        assert_eq!(State::compare::<u32>(None, None), State::Pending);
        assert_eq!(State::compare(None, Some(65_535)), State::Granted);
        assert!(State::Revised.marked() && State::Unsupported.marked());
        assert!(!State::AsRequested.marked() && !State::Default.marked());
    }

    /// OPC's `arrow_job`: `100 → 250 ms` with 250 in the mark, `10 → 10`, `30 → -`.
    #[test]
    fn inline_reads_like_opcs_arrow_rows() {
        let g = Grant::new("PublishingInterval", State::Revised)
            .requested("100")
            .granted("250")
            .unit(" ms");
        let p = inline_pieces(&g);
        assert_eq!(texts(&p), "100 \u{2192} 250 ms");
        assert_eq!(p[0].ink, Ink::Text);
        assert_eq!(p[1].ink, Ink::Faint);
        assert_eq!(p[2].ink, Ink::Mark);
        assert_eq!(p[3].ink, Ink::Text);
        let same = Grant::new("MaxKeepAliveCount", State::AsRequested)
            .requested("10")
            .granted("10");
        assert_eq!(texts(&inline_pieces(&same)), "10 \u{2192} 10");
        assert_eq!(inline_pieces(&same)[2].ink, Ink::Text);
        let pending = Grant::new("LifetimeCount", State::Pending)
            .requested("30")
            .unit(" ms");
        let p = inline_pieces(&pending);
        assert_eq!(texts(&p), "30 \u{2192} -");
        assert_eq!(p[2].ink, Ink::Faint);
        let unasked = Grant::new("LifetimeCount", State::Pending);
        assert_eq!(texts(&inline_pieces(&unasked)), "- \u{2192} -");
    }

    #[test]
    fn inline_default_and_unsupported() {
        let d = Grant::new("Maximum QoS", State::Default).granted("2");
        let p = inline_pieces(&d);
        assert_eq!(texts(&p), "- \u{2192} 2 server default");
        assert_eq!(p.last().map(|p| p.ink), Some(Ink::Faint));
        let u = Grant::new("Shared Subscription", State::Unsupported).requested("1");
        let p = inline_pieces(&u);
        assert_eq!(texts(&p), "1 \u{2192} unsupported");
        assert_eq!(p[2].ink, Ink::Mark);
        // a left-out answer reads as the request
        let left = Grant::new("Session Expiry Interval", State::AsRequested)
            .requested("3600")
            .unit(" s");
        assert_eq!(texts(&inline_pieces(&left)), "3600 \u{2192} 3600 s");
    }

    #[test]
    fn columns_carry_the_mqtt_wording() {
        let pc = |g: Grant<'_>| column_pieces(&g).map(|p| (p.text, p.ink));
        // CONNACK left Session Expiry out
        let [r, g, e] = pc(Grant::new("Session Expiry Interval", State::AsRequested)
            .requested("3600")
            .unit(" s"));
        assert_eq!(r, ("3600 s".into(), Ink::Text));
        assert_eq!(g, (AS_REQUESTED.into(), Ink::Faint));
        assert_eq!(e, ("3600 s".into(), Ink::Faint));
        // Server Keep Alive
        let [_, g, e] = pc(Grant::new("Keep Alive", State::Revised)
            .requested("60")
            .granted("30")
            .unit(" s"));
        assert_eq!(g, ("30 s".into(), Ink::Text));
        assert_eq!(e, ("30 s".into(), Ink::Mark));
        // Maximum QoS absent: the default
        let [r, g, e] = pc(Grant::new("Maximum QoS", State::Default).granted("2"));
        assert_eq!(r, (DASH.into(), Ink::Faint));
        assert_eq!(g, ("absent: 2".into(), Ink::Faint));
        assert_eq!(e, ("2".into(), Ink::Faint));
        // a limit of the server's own, with what is in force spelled out
        let [_, g, e] = pc(Grant::new("Receive Maximum", State::Granted)
            .requested("20")
            .granted("10")
            .effect("10 out · 20 in"));
        assert_eq!(g, ("10".into(), Ink::Text));
        assert_eq!(e, ("10 out · 20 in".into(), Ink::Text));
        // Shared Subscription Available 0
        let [_, g, e] =
            pc(Grant::new("Shared Subscription", State::Unsupported).effect("not available"));
        assert_eq!(g, ("0".into(), Ink::Text));
        assert_eq!(e, ("not available".into(), Ink::Mark));
        // pending, and an explanation in the requested column
        let [r, g, e] = pc(Grant::new("Maximum QoS", State::Pending)
            .requested("client accepts 2")
            .faint_requested());
        assert_eq!(r, ("client accepts 2".into(), Ink::Faint));
        assert_eq!(g, (DASH.into(), Ink::Faint));
        assert_eq!(e, (DASH.into(), Ink::Faint));
    }

    #[test]
    fn heights() {
        assert_eq!(inline_height(15.0), 23.0);
        assert_eq!(inline_height(32.0), 40.0);
        assert_eq!(columns_height(0), HEADER_H);
        assert_eq!(columns_height(13), HEADER_H + 13.0 * ROW_H);
    }
}
