//! The timeline widget drawing two made-up recordings, in both themes.
//!
//! `cargo run --example timeline`. The first is bus-like: one lane per unit plus an aggregate `bus` lane, every
//! transaction as TX, wait and RX, the t3.5 silence between frames, a unit that times out and one that answers
//! with an exception; Ctrl + wheel zooms it, a horizontal scroll pans it, a click selects a transaction. The
//! second is publish-like: a lane of requests and one per subscription, data bars that grow with their
//! notification count, keep-alive rings, an overflow, a republished message and a status change. The data is
//! generated from the clock; the example asks for a repaint 30 times a second, the widget never does.

use std::time::Instant;

use egui::{Frame, Margin, Ui, Vec2};

use studio_ui::icons;
use studio_ui::theme::{self, Theme};
use studio_ui::widgets::timeline::{
    Axis, Glyph, Item, Kind, Lane, Layout, Micros, Swatch, Tick, Timeline, Window, bar_height,
    legend_item,
};
use studio_ui::widgets::{self, Btn, SegItem, sec, seg};

/// 57600 Bd 8N1: ten bits a byte.
const BYTE_US: u64 = 10 * 1_000_000 / 57_600;
/// The silence between two frames, 3.5 characters.
const T35_US: u64 = 7 * BYTE_US / 2;
const CYCLE_US: u64 = 100_000;
const TIMEOUT_US: u64 = 300_000;
const UNITS: [(u8, &str); 3] = [(10, "Boiler"), (21, "Chiller 2"), (41, "Meter, rooftop")];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outcome {
    Ok,
    Exception,
    Timeout,
}

#[derive(Clone, Copy, Debug)]
struct Txn {
    /// Index into `UNITS`.
    unit: usize,
    start: Micros,
    tx: u64,
    wait: u64,
    rx: u64,
    outcome: Outcome,
}

impl Txn {
    fn end(&self) -> Micros {
        self.start + self.tx + self.wait + self.rx
    }
}

/// A cheap, repeatable scramble of `k`.
fn noise(k: u64) -> u64 {
    let mut x = k.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03;
    x ^= x >> 29;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^ (x >> 32)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Zoom {
    TwoSeconds,
    LastCycle,
}

#[derive(Debug)]
struct Demo {
    start: Instant,
    /// Transactions in start order, made up as the clock runs.
    frames: Vec<Txn>,
    zoom: Zoom,
    /// A window the user zoomed or scrolled to; the live window when `None`.
    held: Option<Window>,
    selected: Option<u64>,
}

impl Demo {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            frames: Vec::new(),
            zoom: Zoom::TwoSeconds,
            held: None,
            selected: None,
        }
    }

    /// Microseconds since start, offset so the first window is already full.
    fn now(&self) -> Micros {
        10_000_000 + self.start.elapsed().as_micros() as Micros
    }

    /// Poll the units one after the other, a cycle every 100 ms, until `now`.
    fn run_bus(&mut self, now: Micros) {
        let mut next = self.frames.last().map_or(0, |t| t.end() + T35_US + 300);
        while next < now {
            let k = self.frames.len() as u64;
            let unit = (k % UNITS.len() as u64) as usize;
            if unit == 0 {
                next = next.max(k / UNITS.len() as u64 * CYCLE_US);
            }
            let n = noise(k);
            let outcome = match unit {
                2 if n.is_multiple_of(5) => Outcome::Timeout,
                1 if n.is_multiple_of(7) => Outcome::Exception,
                _ => Outcome::Ok,
            };
            let tx = 8 * BYTE_US;
            let (wait, rx) = match outcome {
                Outcome::Timeout => (TIMEOUT_US, 0),
                Outcome::Exception => (6_000 + n % 4_000, 5 * BYTE_US),
                Outcome::Ok => (14_000 + n % 9_000, 17 * BYTE_US),
            };
            let t = Txn {
                unit,
                start: next,
                tx,
                wait,
                rx,
                outcome,
            };
            next = t.end() + T35_US + 300;
            self.frames.push(t);
        }
        // keep a minute
        if self.frames.len() > 4_000 {
            self.frames.drain(..2_000);
        }
    }

    fn bus_window(&self, now: Micros) -> Window {
        if let Some(w) = self.held {
            return w;
        }
        match self.zoom {
            Zoom::TwoSeconds => Window::last(now, 2_000_000),
            Zoom::LastCycle => {
                let Some(last) = self.frames.iter().rev().find(|t| t.end() <= now) else {
                    return Window::last(now, 2_000_000);
                };
                let first = self
                    .frames
                    .iter()
                    .rev()
                    .find(|t| t.unit == 0 && t.start <= last.start)
                    .map_or(last.start, |t| t.start);
                let pad = ((last.end() - first) * 3 / 100).max(2_000);
                Window::range(first.saturating_sub(pad), last.end() + pad)
            }
        }
    }

    fn show_bus(&mut self, ui: &mut Ui, now: Micros) {
        let t = theme::theme(ui);
        sec::sec_h_with(ui, "Bus timeline", Some("57600 8N1"), true, |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            if self.held.is_some() && ui.add(Btn::new("Back to live").small()).clicked() {
                self.held = None;
            }
            for (sw, label) in [
                (Swatch::Span(Kind::Gap), "t3.5"),
                (Swatch::Span(Kind::Danger), "CRC"),
                (Swatch::Span(Kind::Warn), "exception"),
                (Swatch::Span(Kind::Timeout), "timeout"),
                (Swatch::Span(Kind::Receive), "RX"),
                (Swatch::Span(Kind::Wait), "wait"),
                (Swatch::Span(Kind::Send), "TX"),
            ] {
                legend_item(ui, sw, label);
            }
            ui.add_space(8.0);
            let mut zoom = self.zoom;
            let items = [
                SegItem::new(Zoom::LastCycle, "Last cycle"),
                SegItem::new(Zoom::TwoSeconds, "2 s"),
            ];
            if seg(ui, &mut zoom, &items).changed() {
                self.zoom = zoom;
                self.held = None;
            }
        });

        let w = self.bus_window(now);
        let mut lanes = vec![Lane::new("bus", "all units")];
        let labels: Vec<String> = UNITS.iter().map(|(u, _)| format!("unit {u}")).collect();
        for ((_, name), label) in UNITS.iter().zip(&labels) {
            lanes.push(Lane::new(label, name));
        }
        // only the frames that may overlap the window: the bus lane, its t3.5, then the unit's lane
        let from = self
            .frames
            .partition_point(|f| f.start < w.t0.saturating_sub(TIMEOUT_US));
        let mut items = Vec::new();
        for (i, f) in self.frames.iter().enumerate().skip(from) {
            if f.start > w.t1 {
                break;
            }
            let id = i as u64;
            let a = f.start;
            let (b, c, d) = (a + f.tx, a + f.tx + f.wait, f.end());
            let (wait, rx) = match f.outcome {
                Outcome::Ok => (Kind::Wait, Kind::Receive),
                Outcome::Exception => (Kind::Wait, Kind::Warn),
                Outcome::Timeout => (Kind::Timeout, Kind::Receive),
            };
            for lane in [0, f.unit + 1] {
                items.push(Item::span(id, lane, a, b, Kind::Send));
                items.push(Item::span(id, lane, b, c, wait));
                if d > c {
                    items.push(Item::span(id, lane, c, d, rx));
                }
                if lane == 0 {
                    items.push(
                        Item::span(1 << 63 | id, 0, d, d + T35_US, Kind::Gap)
                            .tip(format!("t3.5 \u{b7} {:.3} ms", T35_US as f64 / 1000.0)),
                    );
                }
            }
        }

        let frame = Frame::NONE.inner_margin(Margin {
            left: 12,
            right: 12,
            top: 2,
            bottom: 8,
        });
        let r = frame
            .show(ui, |ui| {
                let out = Timeline::new(&lanes, &items, w)
                    .selected(self.selected)
                    .zoomable(true)
                    .show(ui);
                if let Some(nw) = out.window {
                    self.held = Some(nw);
                }
                if let Some(id) = out.clicked {
                    self.selected = Some(id);
                }
                // built only for the frame under the pointer, at the pointer like Mb Studio's
                if let Some(f) = out
                    .hovered
                    .filter(|id| id >> 63 == 0)
                    .and_then(|id| self.frames.get(id as usize))
                {
                    let result = match f.outcome {
                        Outcome::Ok => "ok",
                        Outcome::Exception => "exception 02",
                        Outcome::Timeout => "timeout",
                    };
                    let ms = |us: u64| us as f64 / 1000.0;
                    out.response.on_hover_text_at_pointer(format!(
                        "unit {} \u{b7} {result} \u{b7} {:.1} ms (tx {:.2}, wait {:.1}, rx {:.2})",
                        UNITS[f.unit].0,
                        ms(f.end() - f.start),
                        ms(f.tx),
                        ms(f.wait),
                        ms(f.rx),
                    ));
                }
            })
            .response;
        widgets::hline(
            ui.painter(),
            r.rect.x_range(),
            r.rect.bottom() - 0.5,
            t.line,
        );
        let note = match self.selected.and_then(|id| self.frames.get(id as usize)) {
            Some(f) => format!(
                "selected: unit {} at +{:.1} ms",
                UNITS[f.unit].0,
                f.start as f64 / 1000.0
            ),
            None => "click a transaction to select it".to_owned(),
        };
        Frame::NONE
            .inner_margin(Margin::symmetric(12, 6))
            .show(ui, |ui| {
                widgets::text(ui, note, theme::mono(theme::FS_XS), t.text_3);
            });
    }

    fn show_publish(&self, ui: &mut Ui, now: Micros) {
        let t = theme::theme(ui);
        sec::sec_h_with(ui, "Publish timeline", Some("last 10 s"), false, |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for (sw, label) in [
                (Swatch::Mark(Glyph::Tick, Kind::Faint), "PublishRequest"),
                (Swatch::Mark(Glyph::Dot, Kind::Danger), "status change"),
                (Swatch::Mark(Glyph::Diamond, Kind::Send), "republished"),
                (Swatch::Mark(Glyph::Bar(0.0), Kind::Warn), "overflow"),
                (Swatch::Mark(Glyph::Ring, Kind::Idle), "keep-alive"),
                (Swatch::Mark(Glyph::Bar(0.0), Kind::Send), "data change"),
            ] {
                legend_item(ui, sw, label);
            }
        });

        let w = Window::last(now, 10_000_000);
        let ticks: Vec<Tick<'_>> = (0..=10u64)
            .map(|k| {
                let label = if k == 0 {
                    "now".to_owned()
                } else {
                    format!("-{k} s")
                };
                Tick::new(now - k * 1_000_000, label)
            })
            .collect();
        let lanes = [
            Lane::new("session", "requests").sub_x(66.0),
            Lane::new("sub 1", "Intake overview").focus(true),
            Lane::new("sub 2", "Quiet tags"),
            Lane::new("sub 3", "Alarms").color(theme::identity::AMBER),
        ];
        let mut items = Vec::new();
        // every `period` µs in the window, with the k-th event's scramble
        let each = |period: u64, f: &mut dyn FnMut(u64, Micros, u64)| {
            for k in w.t0 / period..=w.t1 / period {
                let at = k * period;
                if at >= w.t0 && at <= w.t1 {
                    f(k, at, noise(k ^ period));
                }
            }
        };
        each(250_000, &mut |k, at, n| {
            let jitter = n % 4_000;
            items.push(
                Item::mark(k, 0, at + jitter, Glyph::Tick, Kind::Faint).tip("PublishRequest"),
            );
            let values = 1 + (n % 12) as u32;
            let seq = 18_000 + k;
            let latency = 2 + n % 5;
            let item = if n.is_multiple_of(41) {
                Item::mark(k, 1, at + 3_000, Glyph::Diamond, Kind::Send).tip(format!(
                    "republished \u{b7} seq {seq} \u{b7} {values} values"
                ))
            } else if n.is_multiple_of(23) {
                Item::mark(k, 1, at + 3_000, Glyph::Bar(bar_height(values)), Kind::Warn).tip(
                    format!(
                        "seq {seq} \u{b7} {values} values \u{b7} 1 overflow \u{b7} {latency} ms"
                    ),
                )
            } else {
                Item::mark(k, 1, at + 3_000, Glyph::Bar(bar_height(values)), Kind::Send).tip(
                    format!("seq {seq} \u{b7} {values} values \u{b7} {latency} ms"),
                )
            };
            items.push(item);
        });
        each(2_000_000, &mut |k, at, _| {
            items.push(
                Item::mark(k, 2, at + 5_000, Glyph::Ring, Kind::Idle)
                    .tip(format!("keep-alive \u{b7} next seq {}", 412 + k)),
            );
        });
        each(7_000_000, &mut |k, at, _| {
            items.push(
                Item::mark(k, 3, at + 9_000, Glyph::Dot, Kind::Danger)
                    .tip(format!("status change \u{b7} seq {}", 90 + k)),
            );
        });

        let frame = Frame::NONE.inner_margin(Margin {
            left: 12,
            right: 12,
            top: 2,
            bottom: 8,
        });
        let r = frame
            .show(ui, |ui| {
                Timeline::new(&lanes, &items, w)
                    .layout(Layout::SPACED)
                    .axis(Axis::Ticks(&ticks))
                    .show(ui);
            })
            .response;
        widgets::hline(
            ui.painter(),
            r.rect.x_range(),
            r.rect.bottom() - 0.5,
            t.line,
        );
    }
}

impl eframe::App for Demo {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let t: &Theme = theme::theme(ui);
        let now = self.now();
        self.run_bus(now);

        egui::Panel::top("top-bar")
            .exact_size(theme::BAR)
            .show_separator_line(false)
            .frame(Frame::NONE.fill(t.panel))
            .show(ui, |ui| {
                widgets::strip(ui, theme::BAR, None, Some(t.line), (12.0, 12.0), |ui| {
                    widgets::text(ui, "studio-ui timeline", theme::semibold(theme::FS), t.text);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut pref = ctx.theme();
                        let items = [
                            SegItem::new(egui::Theme::Dark, "Dark").icon(icons::MOON),
                            SegItem::new(egui::Theme::Light, "Light").icon(icons::SUN),
                        ];
                        if seg(ui, &mut pref, &items).changed() {
                            ctx.set_theme(pref);
                        }
                    });
                });
            });
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(t.editor))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        self.show_bus(ui, now);
                        self.show_publish(ui, now);
                    });
            });
        // the recordings move with the clock: this example's choice, not the widget's
        ctx.request_repaint_after(std::time::Duration::from_millis(33));
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("studio-ui timeline")
            .with_inner_size([1100.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "studio-ui timeline",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(Demo::new()))
        }),
    )
}
