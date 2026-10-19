//! The document view on two made-up project files, in both themes.
//!
//! `cargo run --example docview`. Left, a register map as Mb Studio edits it: Table, JSON and Diff, Add
//! register, Import and Export CSV (they move into "⋯" when the pane is narrow), Revert and Save on a working
//! copy that starts with one unsaved edit, so the dot and the diff show. Right, a narrow MQTT topic map: Table and
//! YAML, Import from births, the checks strip with a warning and an error.
//!
//! Environment, for captures: `STUDIO_UI_DOCVIEW_THEME=light` starts in the light theme;
//! `STUDIO_UI_DOCVIEW_VIEW=source|diff` opens the map on its JSON or its diff.

use std::ops::Range;
use std::sync::Arc;

use egui::{Ui, Vec2};

use studio_ui::icons;
use studio_ui::theme::{self, Theme};
use studio_ui::widgets::docview::{self, Action, Check, Document, Level, Line, Tok, WorkingCopy};
use studio_ui::widgets::grid::Column;
use studio_ui::widgets::rich_grid::RichGrid;
use studio_ui::widgets::{self, SegItem, seg};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum View {
    Table,
    Source,
    Diff,
}

#[derive(Clone, Debug, PartialEq)]
struct Register {
    addr: u16,
    name: String,
    dtype: &'static str,
    scale: f32,
    unit: &'static str,
}

fn reg(addr: u16, name: &str, dtype: &'static str, scale: f32, unit: &'static str) -> Register {
    Register {
        addr,
        name: name.into(),
        dtype,
        scale,
        unit,
    }
}

fn to_json(regs: &[Register]) -> String {
    let mut s = String::from("{\n  \"name\": \"Cold room controller\",\n  \"registers\": [\n");
    for (i, r) in regs.iter().enumerate() {
        s.push_str(&format!(
            "    {{ \"addr\": {}, \"name\": \"{}\", \"type\": \"{}\", \"scale\": {}, \"unit\": \"{}\" }}{}\n",
            r.addr,
            r.name,
            r.dtype,
            r.scale,
            r.unit,
            if i + 1 < regs.len() { "," } else { "" }
        ));
    }
    s.push_str("  ]\n}\n");
    s
}

/// A small JSON colouring for the example (the apps keep their own).
fn json_spans(text: &str) -> Vec<(Range<usize>, Tok)> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'"' => {
                let s = i;
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    i += 1;
                }
                i += 1;
                let key = text[i.min(b.len())..].trim_start().starts_with(':');
                out.push((s..i.min(b.len()), if key { Tok::Key } else { Tok::Str }));
            }
            b'0'..=b'9' | b'-' => {
                let s = i;
                while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.' || b[i] == b'-') {
                    i += 1;
                }
                out.push((s..i, Tok::Num));
            }
            b'{' | b'}' | b'[' | b']' | b',' | b':' => {
                out.push((i..i + 1, Tok::Punct));
                i += 1;
            }
            _ => i += 1,
        }
    }
    out
}

fn summary(r: &Register) -> String {
    format!(
        "{} · {} · {} ×{} {}",
        r.name, r.addr, r.dtype, r.scale, r.unit
    )
}

/// Registers removed or changed (−) and added or changed (+), by address.
fn diff_lines(before: &[Register], after: &[Register]) -> Vec<(bool, String)> {
    let mut out = Vec::new();
    for b in before {
        if !after.contains(b) {
            out.push((false, summary(b)));
        }
    }
    for a in after {
        if !before.contains(a) {
            out.push((true, summary(a)));
        }
    }
    out
}

const MQ_YAML: &str = "# cold-storage.mqmap.yaml · topics and metrics → project tags
version: 1
broker: broker   # Plant broker
sparkplug: { group: ColdStorage, resolveAliases: birth }
entries:
  - topic: spBv1.0/ColdStorage/DDATA/gw-line-a/cold-room-2
    metric: temp_c
    tag: cold_room_2.temp
    unit: \"°C\"
  - topic: plant/line-a/compressor/1
    path: \"$.suction_bar\"
    tag: compressor_1.suction
";

#[derive(Debug)]
struct Demo {
    view: View,
    copy: WorkingCopy<Vec<Register>>,
    sel: Option<usize>,
    json: Option<(u64, bool, Arc<egui::Galley>)>,
    mq_view: View,
    mq_galley: Option<(bool, Arc<egui::Galley>)>,
    toast: String,
}

impl Demo {
    fn new() -> Self {
        let saved = vec![
            reg(0, "temp_room", "int16", 0.1, "°C"),
            reg(1, "temp_evaporator", "int16", 0.1, "°C"),
            reg(2, "setpoint", "int16", 0.1, "°C"),
            reg(3, "defrost_max_time", "uint16", 1.0, "min"),
            reg(4, "compressor_hours", "uint32", 1.0, "h"),
        ];
        let mut copy = WorkingCopy::new(saved);
        // one unsaved edit: a moved register
        if let Some(r) = copy.edit().get_mut(3) {
            r.addr = 9;
        }
        let view = match std::env::var("STUDIO_UI_DOCVIEW_VIEW").as_deref() {
            Ok("source") => View::Source,
            Ok("diff") => View::Diff,
            _ => View::Table,
        };
        Self {
            view,
            copy,
            sel: Some(3),
            json: None,
            mq_view: View::Table,
            mq_galley: None,
            toast: String::new(),
        }
    }

    fn map(&mut self, ui: &mut Ui, t: &Theme) {
        let diff = diff_lines(self.copy.base(), self.copy.work());
        let badge = if self.copy.dirty() {
            "\u{2022}".to_owned()
        } else {
            String::new()
        };
        let mut diff_item = SegItem::new(View::Diff, "Diff").icon(icons::FILE_DIFF);
        if !badge.is_empty() {
            diff_item = diff_item.badge(&badge);
        }
        let views = [
            SegItem::new(View::Table, "Table").icon(icons::TABLE),
            SegItem::new(View::Source, "JSON").icon(icons::FILE_BRACES),
            diff_item,
        ];
        let actions = [
            Action::new(icons::PLUS, "Add register").ghost(),
            Action::new(icons::FILE_INPUT, "Import CSV")
                .ghost()
                .overflow(),
            Action::new(icons::DOWNLOAD, "Export CSV")
                .ghost()
                .overflow(),
        ];
        let n = self.copy.work().len();
        let resolved = format!("{n} registers, no overlaps");
        let checks = [
            Check::new(Level::Ok, "JSON valid"),
            Check::new(Level::Ok, &resolved),
            Check::new(Level::Info, "addresses are protocol addresses (0-based)"),
        ];
        let out = Document::new("map", &mut self.view, &views)
            .path("~/plant-floor/maps/cold-room-controller.mbmap.json")
            .dirty(self.copy.dirty())
            .actions(&actions)
            .checks(&checks)
            .show(ui);
        if out.save {
            self.copy.saved();
            self.toast = "Saved".into();
        }
        if out.revert {
            self.copy.revert();
            self.toast = "Edits reverted".into();
        }
        match out.action {
            Some(0) => {
                let next = self
                    .copy
                    .work()
                    .iter()
                    .map(|r| r.addr + 1)
                    .max()
                    .unwrap_or(0);
                self.copy
                    .edit()
                    .push(reg(next, &format!("register_{next}"), "uint16", 1.0, ""));
                self.view = View::Table;
            }
            Some(i) => self.toast = format!("{} (not in this example)", actions[i].label),
            None => {}
        }
        match self.view {
            View::Table => self.table(ui, t),
            View::Source => {
                let (rev, dark) = (self.copy.rev(), t.dark);
                if self
                    .json
                    .as_ref()
                    .is_none_or(|(r, d, _)| *r != rev || *d != dark)
                {
                    let text = to_json(self.copy.work());
                    let job = docview::source_job(
                        &text,
                        &json_spans(&text),
                        theme::data(),
                        Some(20.0),
                        t,
                    );
                    self.json = Some((rev, dark, ui.painter().layout_job(job)));
                }
                if let Some((_, _, g)) = &self.json {
                    docview::source(ui, "map-json", g);
                }
            }
            View::Diff => {
                let lines: Vec<Line<'_>> = diff
                    .iter()
                    .map(|(add, s)| if *add { Line::add(s) } else { Line::del(s) })
                    .collect();
                if !docview::diff(
                    ui,
                    "map-diff",
                    "unsaved edits -- cold-room-controller.mbmap.json",
                    &lines,
                ) {
                    studio_ui::chrome::welcome::pane_empty(
                        ui,
                        icons::GIT_COMMIT_HORIZONTAL,
                        "No changes",
                        "The map has no unsaved edits and no committed version to compare with.",
                    );
                }
            }
        }
    }

    fn table(&mut self, ui: &mut Ui, t: &Theme) {
        let cols = [
            Column::new("address", 80.0).right(),
            Column::new("name", 180.0).flex(),
            Column::new("type", 90.0),
            Column::new("scale", 70.0).right(),
            Column::new("unit", 70.0),
        ];
        let regs = self.copy.work();
        let base = self.copy.base();
        let data = theme::data();
        let out = RichGrid::new("map-grid", &cols, regs.len())
            .selected(self.sel)
            .show(ui, |c| {
                let Some(r) = regs.get(c.row) else { return };
                let changed = !base.contains(r);
                match c.col {
                    0 => {
                        let color = if changed { t.warn } else { t.syn_num };
                        c.text(&r.addr.to_string(), data.clone(), color);
                    }
                    1 => {
                        c.text(&r.name, data.clone(), t.text);
                    }
                    2 => {
                        c.text(r.dtype, data.clone(), t.syn_kw);
                    }
                    3 => {
                        c.text(&format!("×{}", r.scale), data.clone(), t.syn_num);
                    }
                    _ => {
                        c.text(r.unit, data.clone(), t.text_2);
                    }
                }
            });
        if let Some(i) = out.clicked {
            self.sel = Some(i);
        }
    }

    fn topic_map(&mut self, ui: &mut Ui, t: &Theme) {
        let views = [
            SegItem::new(View::Table, "Table").icon(icons::TABLE),
            SegItem::new(View::Source, "YAML").icon(icons::FILE_TEXT),
        ];
        let actions = [Action::new(icons::FILE_INPUT, "Import from births")
            .ghost()
            .overflow()];
        let checks = [
            Check::new(Level::Ok, "YAML valid"),
            Check::new(Level::Ok, "5 of 6 entries resolve on Plant broker"),
            Check::new(Level::Warn, "compressor_1.suction: no message for 4 min"),
            Check::new(Level::Crit, "door_2.open: $.door is not in the payload"),
            Check::new(
                Level::Info,
                "aliases are resolved from the births, never stored",
            ),
        ];
        Document::new("mq-map", &mut self.mq_view, &views)
            .path("~/plant-floor/cold-storage.mqmap.yaml · git main")
            .actions(&actions)
            .checks(&checks)
            .show(ui);
        match self.mq_view {
            View::Source => {
                if self.mq_galley.as_ref().is_none_or(|(d, _)| *d != t.dark) {
                    let job = docview::source_job(
                        MQ_YAML,
                        &docview::yaml_spans(MQ_YAML),
                        theme::mono(theme::FS_SM),
                        Some(20.0),
                        t,
                    );
                    self.mq_galley = Some((t.dark, ui.painter().layout_job(job)));
                }
                if let Some((_, g)) = &self.mq_galley {
                    docview::source(ui, "mq-yaml", g);
                }
            }
            _ => {
                studio_ui::chrome::welcome::pane_empty(
                    ui,
                    icons::TABLE,
                    "The app's table",
                    "Topic, metric or path, tag, value, unit and check: a rich grid of the app's own.",
                );
            }
        }
    }
}

impl eframe::App for Demo {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let t: &Theme = theme::theme(ui);
        egui::Panel::top("top-bar")
            .exact_size(theme::BAR)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(t.panel))
            .show(ui, |ui| {
                widgets::strip(ui, theme::BAR, None, Some(t.line), (12.0, 12.0), |ui| {
                    widgets::text(ui, "studio-ui docview", theme::semibold(theme::FS), t.text);
                    if !self.toast.is_empty() {
                        widgets::text(ui, &self.toast, theme::ui_font(theme::FS_SM), t.text_3);
                    }
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
            .frame(egui::Frame::NONE.fill(t.editor))
            .show(ui, |ui| {
                let all = ui.max_rect();
                let split = (all.left() + all.width() * 0.62).round();
                let left = egui::Rect::from_min_max(all.min, egui::pos2(split, all.bottom()));
                let right = egui::Rect::from_min_max(egui::pos2(split + 1.0, all.top()), all.max);
                widgets::vline(ui.painter(), split + 0.5, all.y_range(), t.line);
                for (k, rect) in [left, right].into_iter().enumerate() {
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                    );
                    child.set_clip_rect(rect);
                    child.spacing_mut().item_spacing = Vec2::ZERO;
                    if k == 0 {
                        self.map(&mut child, t);
                    } else {
                        self.topic_map(&mut child, t);
                    }
                }
            });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("studio-ui docview")
            .with_inner_size([1400.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "studio-ui docview",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            if std::env::var("STUDIO_UI_DOCVIEW_THEME").as_deref() == Ok("light") {
                cc.egui_ctx.set_theme(egui::Theme::Light);
            }
            Ok(Box::new(Demo::new()))
        }),
    )
}
