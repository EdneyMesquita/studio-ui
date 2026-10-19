//! A protocol-neutral timeline: swimlanes of spans and marks on a time axis, the "now" edge at the right.
//!
//! The widget takes plain data and knows no protocol. [`Lane`]s carry a label, an optional sub-label, an optional
//! identity colour and a focus tint; [`Item`]s are spans (`start..end`) or marks (`at`, a [`Glyph`]) placed in a
//! lane, each with a [`Kind`] that picks its theme colour and an optional tooltip text. A [`Window`] says which
//! time is shown; [`Axis`] how the grid and its labels are made; [`Layout`] the geometry. Times are integer
//! microseconds ([`Micros`]) from any origin the caller likes.
//!
//! ```ignore
//! let lanes = [Lane::new("bus", "all units"), Lane::new("unit 10", "Boiler")];
//! let items = [
//!     Item::span(7, 1, t, t + 1_389, Kind::Send),
//!     Item::span(7, 1, t + 1_389, t + 24_000, Kind::Wait),
//!     Item::span(7, 1, t + 24_000, t + 26_952, Kind::Receive).tip("unit 10 · ok · 27.0 ms"),
//! ];
//! let out = Timeline::new(&lanes, &items, Window::last(now, 2_000_000)).show(ui);
//! if let Some(id) = out.clicked { selected = Some(id); }
//! ```
//!
//! Spans are clipped to the plot and never thinner than 0.8 px; marks are drawn whole and dropped when they
//! fall outside the window. Consecutive spans of one lane that share an id form one *run* (a transaction's TX,
//! wait and RX): a run is hovered, clicked and outlined as a single thing. The widget asks for no repaint of its
//! own, so a still timeline costs nothing; it returns before painting when it is scrolled out of view. The
//! geometry ([`Geom`], [`hit`], [`auto_ticks`], [`zoom_window`], [`pan_window`]) is pure and tested.
//!
//! # Kinds and colours
//!
//! | [`Kind`]  | colour                              | used for                                         |
//! |-----------|-------------------------------------|--------------------------------------------------|
//! | `Send`    | accent                              | TX, a data change, a request on the wire          |
//! | `Wait`    | accent at 24 %                      | turnaround, latency, a handshake in flight        |
//! | `Receive` | accent 62 % over the editor tone    | RX, the reply                                     |
//! | `Ok`      | ok                                  | an acknowledgement                                |
//! | `Warn`    | warn                                | an exception, a queue overflow                    |
//! | `Danger`  | danger                              | a CRC error, a status change                      |
//! | `Idle`    | text 3                              | keep-alives, pings                                |
//! | `Faint`   | text at 30 %                        | requests, minor ticks                             |
//! | `Timeout` | danger at 18 %, hatched in danger   | a request nobody answered                         |
//! | `Gap`     | text 3 at 10 %, hatched in text 3   | silence that is part of the protocol (t3.5)       |
//!
//! # How the family's timelines map onto it
//!
//! **Mb Studio's RS-485 bus timeline** (`app/src/bus_timeline.rs`): [`Layout::DEFAULT`] (24 px lanes, 168 px
//! label column, 20 px axis, 10 px right pad, bands inset 1 px over the plot only, grid over the bands, a rule
//! at the plot's left edge) and [`Axis::Auto`]. Lane 0 is `Lane::new("bus", "all units")`, then
//! `Lane::new("unit 10", device_name)` per device. Its `Window { t0, t1, relative }` is
//! [`Window`]` { t0, t1, live: relative }` (`Window::last(now, 2_000_000)`, or the last cycle with
//! `live: false`). Each visible transaction is pushed twice, first into the bus lane and then into its unit's
//! lane, as up to three consecutive spans with the frame index as id: TX `Kind::Send`, wait `Kind::Wait` (or
//! `Kind::Timeout` for a timeout), RX `Kind::Receive` (`Kind::Warn` for an exception, `Kind::Danger` for a CRC
//! or malformed reply); zero-length parts may be left out. The three spans are one run, so the hover region is
//! the whole transaction, 1.5 px wider on each side and at least 3 px wide, exactly as before. Mb shows its
//! tooltip with egui's `on_hover_text_at_pointer`, built only for the hovered frame: leave `tip` empty and use
//! [`TimelineResponse::hovered`]. The legend is [`legend_item`] with `Swatch::Span(kind)`; `bus_timeline::hatch`
//! is [`hatch`].
//!
//! **OPC Studio's publishing timeline** (`app/src/views/publishing/timeline.rs`): [`Layout::SPACED`] (26 px
//! lane pitch with a 24 px band and a 2 px gap, bands across the label column, 150 px label column, 4 px top,
//! 22 px axis with its labels 5 px under the lanes, 8 px right pad, bands over the grid, no edge rule). The
//! clock is microseconds: `t1` is the frame's clock, `Window::last(t1, 10_000_000)`, a mark `age` old sits at
//! `t1 - age`. The axis is [`Axis::Ticks`] with eleven ticks, `"now"` at `t1` and `"-k s"` every second back.
//! Lane 0 is `Lane::new("session", "requests").sub_x(66.0)`; each subscription is
//! `Lane::new("sub 7", name).focus(id == focus)` with the name cut to 14 characters by the caller as today.
//! A PublishRequest is `Glyph::Tick` in `Kind::Faint`; a data message `Glyph::Bar(bar_height(values))` in
//! `Kind::Send`, or `Kind::Warn` when it carried the overflow bit; a keep-alive `Glyph::Ring` in `Kind::Idle`; a
//! status change `Glyph::Dot` in `Kind::Danger`; a republished message `Glyph::Diamond` in `Kind::Send`. The
//! tooltip is the item's `tip` (`describe(m)`, `"PublishRequest"`), shown with [`super::tip`] like today: the
//! nearest mark within 5 px wins. The legend is [`paint_swatch`] with `Swatch::Mark(glyph, kind)` in its own
//! right-to-left row. The caller's frame keeps the 12 px sides, 2 px top and 8 px bottom it has now; OPC's
//! tooltip hung under that frame, so it passes `tip_side(TipSide { gap: TipSide::BOTTOM.gap + 8.0,
//! ..TipSide::BOTTOM })`.
//!
//! **MQTT** (IIoT Studio, to come): a lane per topic or per client, with the client's identity colour.
//! A QoS 1 publish is a `Kind::Send` span from PUBLISH to PUBACK; a QoS 2 handshake three consecutive spans
//! with one id (PUBLISH→PUBREC `Kind::Send`, PUBREC→PUBREL `Kind::Wait`, PUBREL→PUBCOMP `Kind::Receive`) that
//! hover as one; an unanswered one `Kind::Timeout`. PINGREQ / PINGRESP are `Glyph::Ring` marks in
//! `Kind::Idle`; a Sparkplug sequence gap is a `Glyph::Diamond` in `Kind::Warn` (or a `Kind::Gap` span over the
//! missing stretch).
//!
//! # What is not pixel-identical
//!
//! - OPC drew its grid with unsnapped `line_segment`s; every grid line here is a crisp [`super::vline`], so a
//!   line can sit up to half a pixel from where OPC put it.
//! - Sub-labels are truncated with an ellipsis at the label column (Mb's rule); OPC only cut names at 14
//!   characters, so a 14-character name of wide glyphs that used to run into the plot is now cut.
//! - A span that ends less than 0.8 px left of the plot is dropped per span; Mb dropped whole transactions, so a
//!   TX that ends a hair before `t0` of a transaction that is still visible no longer leaves a 0.8 px sliver.

use std::borrow::Cow;

use egui::{
    Align2, Color32, Pos2, Rect, Response, Sense, Shape as EShape, Stroke, StrokeKind, Ui, Vec2,
};

use super::TipSide;
use crate::theme::{self, Theme, alpha, mix};

/// Time on the axis, integer microseconds from an origin of the caller's choice.
pub type Micros = u64;

/// The narrowest window [`zoom_window`] zooms into.
pub const MIN_SPAN: Micros = 100;

/// The time window shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub t0: Micros,
    pub t1: Micros,
    /// The right edge is "now": the axis counts back from it (`−500 ms … now`) and its tick is right-aligned.
    /// Otherwise the axis counts forward from `t0` (`+20 ms`).
    pub live: bool,
}

impl Window {
    /// The last `span` up to `now` (live).
    pub fn last(now: Micros, span: Micros) -> Self {
        Self {
            t0: now.saturating_sub(span),
            t1: now,
            live: true,
        }
    }

    /// A fixed stretch `t0..t1` (not live).
    pub fn range(t0: Micros, t1: Micros) -> Self {
        Self {
            t0,
            t1,
            live: false,
        }
    }

    /// Its length, at least 1 µs.
    pub fn span(&self) -> Micros {
        self.t1.saturating_sub(self.t0).max(1)
    }
}

/// x of time `t` in a plot spanning `x0..x1` for the window (not clamped).
pub fn x_of(t: Micros, w: &Window, x0: f32, x1: f32) -> f32 {
    let f = (t as f64 - w.t0 as f64) / w.span() as f64;
    x0 + (x1 - x0) * f as f32
}

/// The time under `x` in a plot spanning `x0..x1` (the inverse of [`x_of`], clamped at 0).
pub fn t_of(x: f32, w: &Window, x0: f32, x1: f32) -> Micros {
    let f = f64::from((x - x0) / (x1 - x0).max(f32::EPSILON));
    (w.t0 as f64 + f * w.span() as f64).round().max(0.0) as Micros
}

/// Grid step in ms so that a span shows at most 9 lines.
pub fn axis_step_ms(span_ms: f64) -> u64 {
    const STEPS: [u64; 14] = [
        1, 2, 5, 10, 20, 25, 50, 100, 200, 250, 500, 1000, 2000, 5000,
    ];
    STEPS
        .iter()
        .copied()
        .find(|s| span_ms / *s as f64 <= 9.0)
        .unwrap_or(10_000)
}

/// Ticks of [`Axis::Auto`]: (time, label value in ms). Live windows count back from `t1` (`-500`), the others
/// forward from `t0` (`+20`).
pub fn auto_ticks(w: &Window) -> Vec<(Micros, i64)> {
    let span_ms = w.span() as f64 / 1000.0;
    let step = axis_step_ms(span_ms) as i64;
    let mut out = Vec::new();
    if w.live {
        let first = -((span_ms as i64) / step) * step;
        let mut v = first;
        while v <= 0 {
            let t = w.t1 as i64 + v * 1000;
            if t >= w.t0 as i64 {
                out.push((t as Micros, v));
            }
            v += step;
        }
    } else {
        let mut v = 0i64;
        while (v as f64) <= span_ms {
            out.push((w.t0 + (v as u64) * 1000, v));
            v += step;
        }
    }
    out
}

/// The text of an [`Axis::Auto`] tick: `now`, `−500 ms` (a real minus sign) or `+20 ms`.
pub fn auto_label(live: bool, v: i64) -> String {
    if live {
        if v == 0 {
            "now".to_owned()
        } else {
            format!("\u{2212}{} ms", -v)
        }
    } else {
        format!("+{v} ms")
    }
}

/// Label every n-th of `n_ticks` grid lines across `plot_w` so labels `label_w` wide never touch.
pub fn label_every(n_ticks: usize, plot_w: f32, label_w: f32) -> usize {
    if n_ticks < 2 || plot_w <= 0.0 {
        return 1;
    }
    let spacing = plot_w / (n_ticks - 1) as f32;
    ((label_w / spacing).ceil() as usize).max(1)
}

/// What an item means, and so its colour (see the table in the module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Send,
    Wait,
    Receive,
    Ok,
    Warn,
    Danger,
    Idle,
    Faint,
    Timeout,
    Gap,
}

/// How a span of a [`Kind`] is filled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fill {
    Solid(Color32),
    /// A tinted block with diagonal lines: [`hatch`].
    Hatch {
        bg: Color32,
        line: Color32,
    },
}

impl Kind {
    /// The fill of a span of this kind.
    pub fn fill(self, t: &Theme) -> Fill {
        match self {
            Kind::Send => Fill::Solid(t.accent),
            Kind::Wait => Fill::Solid(alpha(t.accent, 0.24)),
            Kind::Receive => Fill::Solid(mix(t.accent, 0.62, t.editor)),
            Kind::Ok => Fill::Solid(t.ok),
            Kind::Warn => Fill::Solid(t.warn),
            Kind::Danger => Fill::Solid(t.danger),
            Kind::Idle => Fill::Solid(t.text_3),
            Kind::Faint => Fill::Solid(alpha(t.text, 0.30)),
            Kind::Timeout => Fill::Hatch {
                bg: alpha(t.danger, 0.18),
                line: t.danger,
            },
            Kind::Gap => Fill::Hatch {
                bg: alpha(t.text_3, 0.10),
                line: alpha(t.text_3, 0.45),
            },
        }
    }

    /// The colour of a mark of this kind: the fill, or the line of a hatched kind.
    pub fn color(self, t: &Theme) -> Color32 {
        match self.fill(t) {
            Fill::Solid(c) => c,
            Fill::Hatch { line, .. } => line,
        }
    }
}

/// The shape of a mark, centred on its time and on the lane's band.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Glyph {
    /// A 1.5 px stroke 7 px in from the band's top and bottom.
    Tick,
    /// A 4 px bar standing 2 px above the band's bottom, this tall (at most the band less 6 px).
    Bar(f32),
    /// A hollow circle, radius 4, 1.5 px stroke.
    Ring,
    /// A filled circle, radius 4.
    Dot,
    /// A diamond 11 px across, filled with the colour at 25 % and outlined in it.
    Diamond,
}

/// The height of a [`Glyph::Bar`] that grows with a count: 5 px and 1.2 px per unit (clamped when painted).
pub fn bar_height(count: u32) -> f32 {
    5.0 + count as f32 * 1.2
}

/// Where an item sits on the axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Span { start: Micros, end: Micros },
    Mark { at: Micros, glyph: Glyph },
}

/// One span or mark in a lane.
#[derive(Clone, Debug, PartialEq)]
pub struct Item<'a> {
    /// The caller's id, returned by hover and click; consecutive spans of a lane with the same id are one run.
    pub id: u64,
    /// Index into the lanes.
    pub lane: usize,
    pub kind: Kind,
    pub shape: Shape,
    /// Shown with the family tooltip while the item is hovered (the first non-empty one of a run).
    pub tip: Option<Cow<'a, str>>,
}

impl<'a> Item<'a> {
    pub fn span(id: u64, lane: usize, start: Micros, end: Micros, kind: Kind) -> Self {
        Self {
            id,
            lane,
            kind,
            shape: Shape::Span { start, end },
            tip: None,
        }
    }
    pub fn mark(id: u64, lane: usize, at: Micros, glyph: Glyph, kind: Kind) -> Self {
        Self {
            id,
            lane,
            kind,
            shape: Shape::Mark { at, glyph },
            tip: None,
        }
    }
    pub fn tip(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.tip = Some(text.into());
        self
    }
}

/// One swimlane: a label (mono medium, text) and a sub-label (ui, text 3) in the label column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lane<'a> {
    pub label: &'a str,
    pub sub: &'a str,
    /// x of the sub-label from the widget's left edge; [`Layout::sub_x`] when `None`.
    pub sub_x: Option<f32>,
    /// An identity colour: a 2 px stripe at the band's left edge in the label column.
    pub color: Option<Color32>,
    /// Tint the band with the accent (the lane the view is about).
    pub focus: bool,
}

impl<'a> Lane<'a> {
    pub fn new(label: &'a str, sub: &'a str) -> Self {
        Self {
            label,
            sub,
            sub_x: None,
            color: None,
            focus: false,
        }
    }
    pub fn sub_x(mut self, x: f32) -> Self {
        self.sub_x = Some(x);
        self
    }
    pub fn color(mut self, c: Color32) -> Self {
        self.color = Some(c);
        self
    }
    pub fn focus(mut self, on: bool) -> Self {
        self.focus = on;
        self
    }
}

/// A grid line with its label, for [`Axis::Ticks`].
#[derive(Clone, Debug, PartialEq)]
pub struct Tick<'a> {
    pub at: Micros,
    pub label: Cow<'a, str>,
}

impl<'a> Tick<'a> {
    pub fn new(at: Micros, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            at,
            label: label.into(),
        }
    }
}

/// How the grid lines and their labels are made.
#[derive(Clone, Copy, Debug, Default)]
pub enum Axis<'a> {
    /// Every 1, 2, 5, 10 … ms so the window shows at most 9 lines ([`auto_ticks`]), labelled with
    /// [`auto_label`]; a narrow plot labels every 2nd, 3rd … line so the labels never touch.
    #[default]
    Auto,
    /// These lines, every one labelled.
    Ticks(&'a [Tick<'a>]),
}

/// Where the axis labels sit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AxisText {
    /// Bottom-aligned this far above the widget's bottom.
    Bottom(f32),
    /// Top-aligned this far under the last lane.
    Top(f32),
}

/// The geometry of a timeline. [`Layout::DEFAULT`] is the family's 24 px row; [`Layout::SPACED`] separates the
/// lanes by a 2 px gap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// The label column at the left.
    pub label_w: f32,
    /// The gap right of the plot.
    pub pad_r: f32,
    /// Space above the first lane.
    pub top: f32,
    /// Lane pitch.
    pub lane_h: f32,
    /// The band's inset from the lane's top and bottom.
    pub band_top: f32,
    pub band_bottom: f32,
    /// Bands run across the label column too, not only the plot.
    pub band_full: bool,
    /// Spans are the band less this much at the top and bottom.
    pub span_inset: f32,
    /// Height of the axis under the lanes.
    pub axis_h: f32,
    pub axis_text: AxisText,
    /// Grid lines are painted over the bands (otherwise the bands over the grid).
    pub grid_on_top: bool,
    /// A grid rule along the plot's left edge.
    pub edge_line: bool,
    /// x of the sub-labels from the widget's left edge.
    pub sub_x: f32,
}

impl Layout {
    /// 24 px lanes with 1 px between bands, a 168 px label column, a 20 px axis.
    pub const DEFAULT: Self = Self {
        label_w: 168.0,
        pad_r: 10.0,
        top: 0.0,
        lane_h: theme::ROW,
        band_top: 1.0,
        band_bottom: 1.0,
        band_full: false,
        span_inset: 4.0,
        axis_h: 20.0,
        axis_text: AxisText::Bottom(4.0),
        grid_on_top: true,
        edge_line: true,
        sub_x: 62.0,
    };

    /// 24 px bands 26 px apart across the whole width, a 150 px label column, the axis labels under the lanes.
    pub const SPACED: Self = Self {
        label_w: 150.0,
        pad_r: 8.0,
        top: 4.0,
        lane_h: theme::ROW + 2.0,
        band_top: 0.0,
        band_bottom: 2.0,
        band_full: true,
        span_inset: 4.0,
        axis_h: 22.0,
        axis_text: AxisText::Top(5.0),
        grid_on_top: false,
        edge_line: false,
        sub_x: 56.0,
    };

    /// Height of a timeline of `lanes` lanes.
    pub fn height(&self, lanes: usize) -> f32 {
        self.top + lanes as f32 * self.lane_h + self.axis_h
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The pure geometry of one timeline in its rect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geom {
    pub rect: Rect,
    pub layout: Layout,
    pub lanes: usize,
}

impl Geom {
    pub fn new(rect: Rect, layout: Layout, lanes: usize) -> Self {
        Self {
            rect,
            layout,
            lanes,
        }
    }
    /// Left edge of the plot.
    pub fn x0(&self) -> f32 {
        self.rect.left() + self.layout.label_w
    }
    /// Right edge of the plot.
    pub fn x1(&self) -> f32 {
        self.rect.right() - self.layout.pad_r
    }
    pub fn lanes_top(&self) -> f32 {
        self.rect.top() + self.layout.top
    }
    pub fn lanes_bottom(&self) -> f32 {
        self.lanes_top() + self.lanes as f32 * self.layout.lane_h
    }
    /// The plot: the lanes right of the label column.
    pub fn plot(&self) -> Rect {
        Rect::from_min_max(
            Pos2::new(self.x0(), self.lanes_top()),
            Pos2::new(self.x1(), self.lanes_bottom()),
        )
    }
    pub fn x(&self, t: Micros, w: &Window) -> f32 {
        x_of(t, w, self.x0(), self.x1())
    }
    pub fn t(&self, x: f32, w: &Window) -> Micros {
        t_of(x, w, self.x0(), self.x1())
    }
    /// The tinted band of lane `i`.
    pub fn band(&self, i: usize) -> Rect {
        let l = &self.layout;
        let y = self.lanes_top() + i as f32 * l.lane_h;
        let (left, right) = if l.band_full {
            (self.rect.left(), self.rect.right())
        } else {
            (self.x0(), self.x1())
        };
        Rect::from_min_max(
            Pos2::new(left, y + l.band_top),
            Pos2::new(right, y + l.lane_h - l.band_bottom),
        )
    }
    /// Top and bottom of the spans of lane `i`.
    pub fn span_y(&self, i: usize) -> (f32, f32) {
        let b = self.band(i);
        (
            b.top() + self.layout.span_inset,
            b.bottom() - self.layout.span_inset,
        )
    }
    /// The painted rect of a span, clamped to the plot and at least 0.8 px wide; `None` when it is empty or
    /// out of view.
    pub fn span_rect(&self, lane: usize, start: Micros, end: Micros, w: &Window) -> Option<Rect> {
        let (x0, x1) = (self.x0(), self.x1());
        let (yt, yb) = self.span_y(lane);
        let xa = self.x(start, w).max(x0);
        let xb = self.x(end, w).min(x1);
        (xb > xa - 0.8 && end > start && end >= w.t0 && start <= w.t1)
            .then(|| Rect::from_min_max(Pos2::new(xa, yt), Pos2::new(xb.max(xa + 0.8), yb)))
    }
    /// The centre of a mark, `None` outside the window.
    pub fn mark_pos(&self, lane: usize, at: Micros, w: &Window) -> Option<Pos2> {
        (at >= w.t0 && at <= w.t1).then(|| Pos2::new(self.x(at, w), self.band(lane).center().y))
    }
}

/// The exclusive end of the run that starts at `i`: the spans after it in the same lane with the same id.
pub fn run_end(items: &[Item<'_>], i: usize) -> usize {
    let Some(first) = items.get(i) else {
        return i;
    };
    let mut e = i + 1;
    if matches!(first.shape, Shape::Span { .. }) {
        while let Some(it) = items.get(e) {
            if it.id != first.id || it.lane != first.lane || !matches!(it.shape, Shape::Span { .. })
            {
                break;
            }
            e += 1;
        }
    }
    e
}

/// The time a run of spans covers.
fn run_extent(run: &[Item<'_>]) -> Option<(Micros, Micros)> {
    run.iter().fold(None, |acc, it| match it.shape {
        Shape::Span { start, end } => Some(match acc {
            Some((a, b)) => (start.min(a), end.max(b)),
            None => (start, end),
        }),
        Shape::Mark { .. } => acc,
    })
}

/// The item under `pos`: a run of spans whose time is under the pointer (1.5 px either side, at least 3 px
/// wide, within the span height), else the nearest mark within 5 px across and half a lane up or down. The
/// first run hit wins, and the first of equally near marks. Returns the index of the run's first item.
pub fn hit(g: &Geom, w: &Window, items: &[Item<'_>], pos: Pos2) -> Option<usize> {
    let mut best: Option<(f32, usize)> = None;
    let mut i = 0;
    while i < items.len() {
        let end = run_end(items, i);
        let it = &items[i];
        if it.lane < g.lanes {
            match it.shape {
                Shape::Span { .. } => {
                    if let Some((a, b)) = run_extent(&items[i..end])
                        && b >= w.t0
                        && a <= w.t1
                    {
                        let (yt, yb) = g.span_y(it.lane);
                        let xa = g.x(a, w) - 1.5;
                        let xb = g.x(b, w).max(xa + 3.0) + 1.5;
                        if pos.y >= yt
                            && pos.y <= yb
                            && pos.x >= xa
                            && pos.x <= xb
                            && best.is_none_or(|(d, _)| d > 0.0)
                        {
                            best = Some((0.0, i));
                        }
                    }
                }
                Shape::Mark { at, .. } => {
                    if let Some(c) = g.mark_pos(it.lane, at, w) {
                        let dx = (pos.x - c.x).abs();
                        if dx <= 5.0
                            && (pos.y - c.y).abs() <= g.layout.lane_h / 2.0
                            && best.is_none_or(|(d, _)| dx < d)
                        {
                            best = Some((dx, i));
                        }
                    }
                }
            }
        }
        i = end;
    }
    best.map(|(_, i)| i)
}

/// `w` zoomed by `factor` (above 1 zooms in) around time `anchor`, which stays where it is; never narrower
/// than [`MIN_SPAN`]. The result is not live.
pub fn zoom_window(w: &Window, factor: f32, anchor: Micros) -> Window {
    let span = w.span() as f64;
    let new_span = (span / f64::from(factor.max(f32::EPSILON))).max(MIN_SPAN as f64);
    let k = new_span / span;
    let t0 = (anchor as f64 - (anchor as f64 - w.t0 as f64) * k)
        .round()
        .max(0.0);
    Window {
        t0: t0 as Micros,
        t1: (t0 + new_span).round() as Micros,
        live: false,
    }
}

/// `w` moved by a scroll of `dx` points across a plot `plot_w` wide (content follows the pointer: a positive
/// `dx` shows earlier time); never before 0. The result is not live.
pub fn pan_window(w: &Window, dx: f32, plot_w: f32) -> Window {
    let span = w.span();
    let dt = (f64::from(dx / plot_w.max(1.0)) * span as f64).round() as i64;
    let t0 = (w.t0 as i64 - dt).max(0) as Micros;
    Window {
        t0,
        t1: t0 + span,
        live: false,
    }
}

/// Diagonal hatching on a tinted block (the timeout pattern): 1.5 px lines 5 px apart.
pub fn hatch(p: &egui::Painter, r: Rect, bg: Color32, line: Color32) {
    p.rect_filled(r, 0, bg);
    let pc = p.with_clip_rect(r.intersect(p.clip_rect()));
    let h = r.height();
    let mut x = r.left() - h;
    while x < r.right() {
        pc.line_segment(
            [Pos2::new(x, r.bottom()), Pos2::new(x + h, r.top())],
            Stroke::new(1.5, line),
        );
        x += 5.0;
    }
}

/// What [`Timeline::show`] reports.
#[derive(Clone, Debug)]
pub struct TimelineResponse {
    pub response: Response,
    /// The id of the item under the pointer.
    pub hovered: Option<u64>,
    /// The id of the item clicked this frame.
    pub clicked: Option<u64>,
    /// The window after this frame's zoom or scroll, when [`Timeline::zoomable`] and it changed.
    pub window: Option<Window>,
}

/// The timeline widget; see the module docs.
#[derive(Clone, Copy, Debug)]
pub struct Timeline<'a> {
    lanes: &'a [Lane<'a>],
    items: &'a [Item<'a>],
    window: Window,
    axis: Axis<'a>,
    layout: Layout,
    selected: Option<u64>,
    zoomable: bool,
    tip_side: TipSide,
}

impl<'a> Timeline<'a> {
    pub fn new(lanes: &'a [Lane<'a>], items: &'a [Item<'a>], window: Window) -> Self {
        Self {
            lanes,
            items,
            window,
            axis: Axis::Auto,
            layout: Layout::DEFAULT,
            selected: None,
            zoomable: false,
            tip_side: TipSide::BOTTOM,
        }
    }
    pub fn axis(mut self, axis: Axis<'a>) -> Self {
        self.axis = axis;
        self
    }
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
    /// Outline the item (or run) with this id.
    pub fn selected(mut self, id: Option<u64>) -> Self {
        self.selected = id;
        self
    }
    /// Where the items' tooltips sit: under the widget by default ([`super::tip`]).
    pub fn tip_side(mut self, side: TipSide) -> Self {
        self.tip_side = side;
        self
    }
    /// Ctrl + wheel (or a pinch) zooms around the pointer, a horizontal scroll pans; the new window comes back
    /// in [`TimelineResponse::window`] for the caller to keep.
    pub fn zoomable(mut self, on: bool) -> Self {
        self.zoomable = on;
        self
    }

    /// Paint the timeline across the available width.
    pub fn show(self, ui: &mut Ui) -> TimelineResponse {
        let t = theme::theme(ui);
        let width = ui.available_width();
        let h = self.layout.height(self.lanes.len());
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, h), Sense::click());
        let mut out = TimelineResponse {
            response,
            hovered: None,
            clicked: None,
            window: None,
        };
        if !ui.is_rect_visible(rect) {
            return out;
        }
        let g = Geom::new(rect, self.layout, self.lanes.len());
        let w = &self.window;
        let p = ui.painter_at(rect);

        if !self.layout.grid_on_top {
            self.paint_grid(ui, &p, &g, t);
        }
        self.paint_lanes(ui, &p, &g, t);
        if self.layout.grid_on_top {
            self.paint_grid(ui, &p, &g, t);
        }
        self.paint_items(&p, &g, t);

        let pointer = out.response.hover_pos();
        let hit = pointer.and_then(|pos| hit(&g, w, self.items, pos));
        out.hovered = hit.map(|i| self.items[i].id);
        if out.response.clicked() {
            out.clicked = out.hovered;
        }
        if let Some(i) = hit {
            let tip = self.items[i..run_end(self.items, i)]
                .iter()
                .find_map(|it| it.tip.as_deref().filter(|s| !s.is_empty()));
            if let Some(tip) = tip {
                out.response = super::tip_at(out.response, tip, self.tip_side, None);
            }
        }
        if self.zoomable
            && let Some(pos) = pointer.filter(|pos| g.plot().contains(*pos))
        {
            let (zoom, dx) = ui.input(|i| (i.zoom_delta(), i.smooth_scroll_delta.x));
            let mut nw = *w;
            if zoom != 1.0 {
                nw = zoom_window(&nw, zoom, g.t(pos.x, &nw));
            }
            if dx != 0.0 {
                nw = pan_window(&nw, dx, g.x1() - g.x0());
            }
            if nw != *w {
                out.window = Some(nw);
            }
        }
        out
    }

    fn paint_lanes(&self, ui: &Ui, p: &egui::Painter, g: &Geom, t: &Theme) {
        let left = g.rect.left();
        for (i, ln) in self.lanes.iter().enumerate() {
            let band = g.band(i);
            p.rect_filled(band, 0, alpha(t.text, if i % 2 == 1 { 0.05 } else { 0.03 }));
            if ln.focus {
                p.rect_filled(band, 0, alpha(t.accent, 0.07));
            }
            if let Some(c) = ln.color {
                p.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(left, band.top()),
                        Pos2::new(left + 2.0, band.bottom()),
                    ),
                    0,
                    c,
                );
            }
            let cy = band.center().y;
            let lg = super::galley(ui, ln.label, theme::mono_medium(11.5), t.text);
            p.galley(Pos2::new(left + 8.0, cy - lg.size().y / 2.0), lg, t.text);
            let sub_x = ln.sub_x.unwrap_or(self.layout.sub_x);
            let sg = super::galley_truncated(
                ui,
                ln.sub,
                theme::ui_font(11.0),
                t.text_3,
                self.layout.label_w - sub_x - 8.0,
            );
            p.galley(
                Pos2::new(left + sub_x, cy - sg.size().y / 2.0),
                sg,
                t.text_3,
            );
        }
    }

    fn paint_grid(&self, ui: &Ui, p: &egui::Painter, g: &Geom, t: &Theme) {
        let w = &self.window;
        let (x0, x1) = (g.x0(), g.x1());
        let lanes = g.lanes_top()..=g.lanes_bottom();
        let font = theme::mono(10.5);
        let label = |x: f32, at: Micros, text: &str| {
            let right = w.live && at == w.t1;
            let (y, anchor) = match self.layout.axis_text {
                AxisText::Bottom(d) => (
                    g.rect.bottom() - d,
                    if right {
                        Align2::RIGHT_BOTTOM
                    } else {
                        Align2::CENTER_BOTTOM
                    },
                ),
                AxisText::Top(d) => (
                    g.lanes_bottom() + d,
                    if right {
                        Align2::RIGHT_TOP
                    } else {
                        Align2::CENTER_TOP
                    },
                ),
            };
            p.text(Pos2::new(x, y), anchor, text, font.clone(), t.text_3);
        };
        let in_plot = |x: f32| x >= x0 - 0.5 && x <= x1 + 0.5;
        match self.axis {
            Axis::Auto => {
                let ticks = auto_ticks(w);
                let label_w = super::galley(ui, "\u{2212}2000 ms", font.clone(), t.text_3)
                    .size()
                    .x
                    + 10.0;
                let every = label_every(ticks.len(), x1 - x0, label_w);
                let last = ticks.len().saturating_sub(1);
                for (k, (at, v)) in ticks.into_iter().enumerate() {
                    let x = g.x(at, w);
                    if !in_plot(x) {
                        continue;
                    }
                    super::vline(p, x, lanes.clone(), t.line_soft);
                    // live windows keep "now", the others the start
                    let from_anchor = if w.live { last - k } else { k };
                    if from_anchor % every == 0 {
                        label(x, at, &auto_label(w.live, v));
                    }
                }
            }
            Axis::Ticks(ticks) => {
                for tick in ticks {
                    let x = g.x(tick.at, w);
                    if !in_plot(x) {
                        continue;
                    }
                    super::vline(p, x, lanes.clone(), t.line_soft);
                    label(x, tick.at, &tick.label);
                }
            }
        }
        if self.layout.edge_line {
            super::vline(p, x0, lanes, t.line_soft);
        }
    }

    fn paint_items(&self, p: &egui::Painter, g: &Geom, t: &Theme) {
        let w = &self.window;
        let pc = p.with_clip_rect(g.plot().intersect(p.clip_rect()));
        for it in self.items.iter().filter(|it| it.lane < g.lanes) {
            match it.shape {
                Shape::Span { start, end } => {
                    if let Some(r) = g.span_rect(it.lane, start, end, w) {
                        match it.kind.fill(t) {
                            Fill::Solid(c) => {
                                pc.rect_filled(r, 0, c);
                            }
                            Fill::Hatch { bg, line } => hatch(&pc, r, bg, line),
                        }
                    }
                }
                Shape::Mark { at, glyph } => {
                    if let Some(c) = g.mark_pos(it.lane, at, w) {
                        paint_glyph(p, c, g.band(it.lane), glyph, it.kind.color(t));
                    }
                }
            }
        }
        let Some(sel) = self.selected else {
            return;
        };
        let stroke = Stroke::new(1.0, t.text);
        let mut i = 0;
        while i < self.items.len() {
            let end = run_end(self.items, i);
            let it = &self.items[i];
            if it.id == sel && it.lane < g.lanes {
                match it.shape {
                    Shape::Span { .. } => {
                        if let Some((a, b)) = run_extent(&self.items[i..end])
                            && let Some(r) = g.span_rect(it.lane, a, b, w)
                        {
                            pc.rect_stroke(r, 0, stroke, StrokeKind::Outside);
                        }
                    }
                    Shape::Mark { at, .. } => {
                        if let Some(c) = g.mark_pos(it.lane, at, w) {
                            p.circle_stroke(c, 7.5, stroke);
                        }
                    }
                }
            }
            i = end;
        }
    }
}

/// Paint a mark centred on `c` in a lane whose band is `band`.
fn paint_glyph(p: &egui::Painter, c: Pos2, band: Rect, glyph: Glyph, color: Color32) {
    match glyph {
        Glyph::Tick => {
            p.rect_filled(
                Rect::from_min_size(
                    Pos2::new(c.x - 0.5, band.top() + 7.0),
                    Vec2::new(1.5, band.height() - 14.0),
                ),
                0,
                color,
            );
        }
        Glyph::Bar(h) => {
            let h = h.min(band.height() - 6.0);
            p.rect_filled(
                Rect::from_min_size(
                    Pos2::new(c.x - 2.0, band.bottom() - 2.0 - h),
                    Vec2::new(4.0, h),
                ),
                1,
                color,
            );
        }
        Glyph::Ring => {
            p.circle_stroke(c, 4.0, Stroke::new(1.5, color));
        }
        Glyph::Dot => {
            p.circle_filled(c, 4.0, color);
        }
        Glyph::Diamond => diamond(p, c, 5.5, color),
    }
}

fn diamond(p: &egui::Painter, c: Pos2, r: f32, color: Color32) {
    p.add(EShape::convex_polygon(
        vec![
            c + Vec2::new(0.0, -r),
            c + Vec2::new(r, 0.0),
            c + Vec2::new(0.0, r),
            c + Vec2::new(-r, 0.0),
        ],
        alpha(color, 0.25),
        Stroke::new(1.5, color),
    ));
}

/// A legend swatch: a span's 10 × 8 block or a mark's glyph.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Swatch {
    Span(Kind),
    Mark(Glyph, Kind),
}

/// Paint a legend swatch centred on `c`, 10 px wide and at most 9 px tall: a span or a bar is a 10 × 8 block
/// (hatched for hatched kinds), a tick 3 × 8, a ring of radius 3.25, a dot of 4, a diamond 9 px across.
pub fn paint_swatch(p: &egui::Painter, c: Pos2, sw: Swatch, t: &Theme) {
    let block = Rect::from_center_size(c, Vec2::new(10.0, 8.0));
    match sw {
        Swatch::Span(kind) | Swatch::Mark(Glyph::Bar(_), kind) => match kind.fill(t) {
            Fill::Solid(color) => {
                p.rect_filled(block, 2, color);
            }
            Fill::Hatch { bg, line } => hatch(p, block, bg, line),
        },
        Swatch::Mark(Glyph::Tick, kind) => {
            p.rect_filled(
                Rect::from_center_size(c, Vec2::new(3.0, 8.0)),
                2,
                kind.color(t),
            );
        }
        Swatch::Mark(Glyph::Ring, kind) => {
            p.circle_stroke(c, 3.25, Stroke::new(1.5, kind.color(t)));
        }
        Swatch::Mark(Glyph::Dot, kind) => {
            p.circle_filled(c, 4.0, kind.color(t));
        }
        Swatch::Mark(Glyph::Diamond, kind) => diamond(p, c, 4.5, kind.color(t)),
    }
}

/// One legend entry, left to right: the swatch, then its label (11 px, text 3), 16 px tall.
pub fn legend_item(ui: &mut Ui, sw: Swatch, label: &str) -> Response {
    let t = theme::theme(ui);
    let g = super::galley(ui, label, theme::ui_font(theme::FS_XS), t.text_3);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(14.0 + g.size().x, 16.0), Sense::hover());
    let p = ui.painter();
    paint_swatch(p, Pos2::new(rect.left() + 5.0, rect.center().y), sw, t);
    p.galley(
        Pos2::new(rect.left() + 14.0, rect.center().y - g.size().y / 2.0),
        g,
        t.text_3,
    );
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A timeline 1000 px wide: Mb's plot runs 168..990, OPC's 150..992.
    fn geom(layout: Layout, lanes: usize) -> Geom {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, layout.height(lanes)));
        Geom::new(rect, layout, lanes)
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn x_maps_window_linearly() {
        let w = Window::last(10_000_000, 2_000_000);
        assert_eq!(x_of(8_000_000, &w, 100.0, 300.0), 100.0);
        assert_eq!(x_of(10_000_000, &w, 100.0, 300.0), 300.0);
        assert_eq!(x_of(9_000_000, &w, 100.0, 300.0), 200.0);
        assert_eq!(t_of(200.0, &w, 100.0, 300.0), 9_000_000);
        // before the origin the window is narrower, never negative
        assert_eq!(Window::last(500, 2_000_000).t0, 0);
    }

    #[test]
    fn a_mark_slides_from_the_right_edge_to_the_left_in_ten_seconds() {
        // OPC's publishing timeline: t1 is the frame's clock, a mark `age` old sits at t1 - age
        let g = geom(Layout::SPACED, 2);
        let now = 60_000_000;
        let w = Window::last(now, 10_000_000);
        let at = |age_ms: u64| g.mark_pos(1, now - age_ms * 1000, &w).map(|p| p.x);
        assert_eq!(at(0), Some(992.0));
        assert_eq!(at(10_000), Some(150.0));
        assert_eq!(at(5_000), Some(571.0));
        // a mark older than the window is dropped, not clamped or wrapped
        assert_eq!(at(10_001), None);
        // two frames 100 ms apart move a mark by a hundredth of the plot
        let (Some(a), Some(b)) = (at(2000), at(2100)) else {
            panic!("both marks are in the window");
        };
        assert!(near(a - b, 8.42));
    }

    #[test]
    fn axis_steps_and_ticks() {
        assert_eq!(axis_step_ms(2000.0), 250);
        assert_eq!(axis_step_ms(90.0), 10);
        let w = Window::last(10_000_000, 2_000_000);
        let t = auto_ticks(&w);
        assert_eq!(t.first(), Some(&(8_000_000, -2000)));
        assert_eq!(t.last(), Some(&(10_000_000, 0)));
        assert_eq!(t.len(), 9);
        let abs = Window::range(0, 90_000);
        assert_eq!(auto_ticks(&abs).last(), Some(&(90_000, 90)));
        assert_eq!(auto_label(true, 0), "now");
        assert_eq!(auto_label(true, -500), "\u{2212}500 ms");
        assert_eq!(auto_label(false, 20), "+20 ms");
    }

    #[test]
    fn narrow_axes_label_fewer_lines() {
        assert_eq!(label_every(9, 800.0, 60.0), 1);
        assert_eq!(label_every(9, 300.0, 60.0), 2);
        assert_eq!(label_every(9, 150.0, 60.0), 4);
        assert_eq!(label_every(1, 100.0, 60.0), 1);
    }

    #[test]
    fn the_layouts_keep_each_apps_geometry() {
        // Mb: lanes × 24 + a 20 px axis, bands inset 1 px over the plot only, spans y+5..y+19
        let mb = geom(Layout::DEFAULT, 3);
        assert_eq!(Layout::DEFAULT.height(3), 3.0 * 24.0 + 20.0);
        assert_eq!((mb.x0(), mb.x1()), (168.0, 990.0));
        assert_eq!(
            mb.band(1),
            Rect::from_min_max(Pos2::new(168.0, 25.0), Pos2::new(990.0, 47.0))
        );
        assert_eq!(mb.span_y(1), (29.0, 43.0));
        assert_eq!(mb.band(1).center().y, 36.0);
        // OPC: 4 px top, a 26 px pitch with 24 px bands across the width, a 22 px axis; marks 12 px into a lane
        let opc = geom(Layout::SPACED, 3);
        assert_eq!(Layout::SPACED.height(3), 4.0 + 3.0 * 26.0 + 22.0);
        assert_eq!((opc.x0(), opc.x1()), (150.0, 992.0));
        assert_eq!(
            opc.band(1),
            Rect::from_min_max(Pos2::new(0.0, 30.0), Pos2::new(1000.0, 54.0))
        );
        assert_eq!(opc.band(1).center().y, 30.0 + 12.0);
        assert_eq!(opc.lanes_bottom(), 82.0);
    }

    #[test]
    fn spans_clip_to_the_plot_and_keep_a_visible_width() {
        let g = geom(Layout::DEFAULT, 1);
        let w = Window::range(1_000_000, 1_822_000); // 1 px per ms
        let Some(r) = g.span_rect(0, 900_000, 1_010_000, &w) else {
            panic!("a span that crosses the left edge is drawn");
        };
        assert_eq!((r.left(), r.right()), (168.0, 178.0));
        let Some(r) = g.span_rect(0, 1_800_000, 1_900_000, &w) else {
            panic!("a span that crosses the right edge is drawn");
        };
        assert!(near(r.left(), 968.0) && near(r.right(), 990.0));
        // a 10 µs span is still 0.8 px wide
        let Some(r) = g.span_rect(0, 1_100_000, 1_100_010, &w) else {
            panic!("a short span is drawn");
        };
        assert!(near(r.width(), 0.8));
        assert_eq!(g.span_rect(0, 1_100_000, 1_100_000, &w), None);
        assert_eq!(g.span_rect(0, 100_000, 200_000, &w), None);
        assert_eq!(g.span_rect(0, 1_900_000, 2_000_000, &w), None);
    }

    #[test]
    fn a_run_of_spans_hovers_as_one_transaction() {
        let g = geom(Layout::DEFAULT, 2);
        let w = Window::range(1_000_000, 1_822_000); // 1 px per ms
        let items = [
            Item::span(7, 1, 1_100_000, 1_102_000, Kind::Send),
            Item::span(7, 1, 1_102_000, 1_120_000, Kind::Wait),
            Item::span(7, 1, 1_120_000, 1_123_000, Kind::Receive),
            Item::span(8, 1, 1_200_000, 1_200_500, Kind::Send),
        ];
        assert_eq!(run_end(&items, 0), 3);
        assert_eq!(run_end(&items, 3), 4);
        let y = g.band(1).center().y;
        // x 268..291 is the transaction, 1.5 px either side counts
        for x in [266.6, 268.0, 280.0, 291.0, 292.4] {
            assert_eq!(hit(&g, &w, &items, Pos2::new(x, y)), Some(0), "x {x}");
        }
        assert_eq!(hit(&g, &w, &items, Pos2::new(266.4, y)), None);
        assert_eq!(hit(&g, &w, &items, Pos2::new(292.6, y)), None);
        // a 0.5 ms transaction reaches at least 3 px right of its start: 366.5..371
        assert_eq!(hit(&g, &w, &items, Pos2::new(370.9, y)), Some(3));
        assert_eq!(hit(&g, &w, &items, Pos2::new(371.1, y)), None);
        // only within the span height, and only in its own lane
        let (yt, yb) = g.span_y(1);
        assert_eq!(hit(&g, &w, &items, Pos2::new(280.0, yt - 0.1)), None);
        assert_eq!(hit(&g, &w, &items, Pos2::new(280.0, yb + 0.1)), None);
        assert_eq!(
            hit(&g, &w, &items, Pos2::new(280.0, g.band(0).center().y)),
            None
        );
    }

    #[test]
    fn the_nearest_mark_within_five_px_is_hovered() {
        let g = geom(Layout::SPACED, 2);
        let w = Window::last(10_000_000, 10_000_000);
        let x = |t: Micros| g.x(t, &w);
        let items = [
            Item::mark(1, 1, 5_000_000, Glyph::Ring, Kind::Idle),
            Item::mark(2, 1, 5_040_000, Glyph::Bar(bar_height(3)), Kind::Send),
            Item::mark(3, 0, 5_000_000, Glyph::Tick, Kind::Faint),
        ];
        let cy = g.band(1).center().y;
        assert_eq!(
            hit(&g, &w, &items, Pos2::new(x(5_000_000) + 1.0, cy)),
            Some(0)
        );
        assert_eq!(
            hit(&g, &w, &items, Pos2::new(x(5_040_000) - 1.0, cy)),
            Some(1)
        );
        assert_eq!(hit(&g, &w, &items, Pos2::new(x(5_000_000) - 5.5, cy)), None);
        // half a lane up or down
        assert_eq!(
            hit(&g, &w, &items, Pos2::new(x(5_000_000), cy + 12.9)),
            Some(0)
        );
        assert_eq!(
            hit(&g, &w, &items, Pos2::new(x(5_000_000), cy - 13.1)),
            Some(2)
        );
    }

    #[test]
    fn a_span_wins_over_a_nearby_mark() {
        let g = geom(Layout::DEFAULT, 1);
        let w = Window::range(0, 822_000);
        let items = [
            Item::mark(1, 0, 100_000, Glyph::Dot, Kind::Danger),
            Item::span(2, 0, 101_000, 110_000, Kind::Send),
        ];
        let y = g.band(0).center().y;
        assert_eq!(hit(&g, &w, &items, Pos2::new(g.x(101_000, &w), y)), Some(1));
    }

    #[test]
    fn zoom_keeps_the_time_under_the_pointer_and_pan_moves_the_window() {
        let w = Window::last(10_000_000, 2_000_000);
        let z = zoom_window(&w, 2.0, 9_000_000);
        assert_eq!((z.t0, z.t1, z.live), (8_500_000, 9_500_000, false));
        assert_eq!(zoom_window(&w, 1e9, 9_000_000).span(), MIN_SPAN);
        // a 100 px scroll over a 1000 px plot shows a tenth of the span earlier
        let p = pan_window(&w, 100.0, 1000.0);
        assert_eq!((p.t0, p.t1, p.live), (7_800_000, 9_800_000, false));
        assert_eq!(pan_window(&w, 10_000.0, 1000.0).t0, 0);
    }

    #[test]
    fn bars_grow_with_their_count() {
        assert!(bar_height(1) < bar_height(5));
        assert!(near(bar_height(0), 5.0));
        // OPC caps a bar at its lane less 8 px: the 24 px band less 6
        let band = geom(Layout::SPACED, 1).band(0);
        assert!(near(bar_height(1000).min(band.height() - 6.0), 26.0 - 8.0));
    }
}
