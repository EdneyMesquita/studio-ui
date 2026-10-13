//! Gallery of every public widget of `studio-ui`, in both themes.
//!
//! `cargo run --example gallery`. Each section shows a widget next to the calls that build it, so this file is
//! also the usage reference. Sample data is generic: `.example` hosts and made-up numbers.

use std::cell::Cell;
use std::fmt::Write as _;

use egui::text::{LayoutJob, TextFormat};
use egui::{Frame, Key, Margin, Modifiers, Sense, Ui, Vec2};

use studio_ui::icons;
use studio_ui::theme::{self, Theme};
use studio_ui::widgets::{
    self, Btn, Fit, SegItem, Status, TipSide,
    button::BtnKind,
    chart::Chart,
    chart::Series,
    check,
    chip::{self, ChipSize},
    dialog::{self, Dialog},
    field, fit,
    grid::{CellFont, Column, Grid},
    icon_button,
    inspector::{self, Inspector},
    menu,
    metrics::{self, Metric},
    notice::{self, Level},
    palette::{self, PaletteConfig, PaletteItem, PaletteState},
    rich_grid::RichGrid,
    sec, seg, seg_fit, seg_stretch, seg_width, switch,
    toast::{self, Toast},
};

/// Made-up hosts in the grid, enough to show that only the rows in view are laid out.
const SERVERS: usize = 5_000;
const HOST: &str = "node-0001.example";
const RANGES: [&str; 4] = ["1 s", "5 s", "1 min", "5 min"];
const OPTIONS: [&str; 12] = [
    "1 s", "2 s", "5 s", "10 s", "30 s", "1 min", "5 min", "15 min", "1 h", "6 h", "1 day",
    "1 week",
];
/// Initials of the eight connection colors of `theme::identity`.
const CONN_INITIALS: [&str; 8] = ["WE", "DB", "CA", "MQ", "LG", "CI", "ST", "BK"];

/// The sections of the left list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Buttons,
    Fields,
    Status,
    Chart,
    Grid,
    Inspector,
    Overlays,
    Text,
    Icons,
}

impl Section {
    const ALL: [Section; 9] = [
        Section::Buttons,
        Section::Fields,
        Section::Status,
        Section::Chart,
        Section::Grid,
        Section::Inspector,
        Section::Overlays,
        Section::Text,
        Section::Icons,
    ];

    fn label(self) -> &'static str {
        match self {
            Section::Buttons => "Buttons & controls",
            Section::Fields => "Fields",
            Section::Status => "Status & notices",
            Section::Chart => "Chart & metrics",
            Section::Grid => "Data grid",
            Section::Inspector => "Inspector",
            Section::Overlays => "Overlays",
            Section::Text => "Text & helpers",
            Section::Icons => "Icons",
        }
    }

    fn intro(self) -> &'static str {
        match self {
            Section::Buttons => {
                "Buttons, icon buttons, segmented controls and a toolbar that collapses as its width shrinks."
            }
            Section::Fields => {
                "Text inputs and query-bar fields with their labels, a select, a checkbox and a switch."
            }
            Section::Status => {
                "Status pills, chips, dots and notices. A state is always an icon and a word, never color alone."
            }
            Section::Chart => {
                "A line chart with a legend, dashed limits and a hover tooltip, and a metrics strip."
            }
            Section::Grid => {
                "Virtualized tables: only the rows in view are laid out. The rich grid paints bars, pills and buttons."
            }
            Section::Inspector => {
                "A resizable side pane for the selected row. In a window 960 px wide or less it moves under its table."
            }
            Section::Overlays => {
                "Command palette (Ctrl K), menus, select popups, dialogs and toasts."
            }
            Section::Text => {
                "Text, truncation, rules, keycaps, tooltips and focus helpers from widgets/mod.rs."
            }
            Section::Icons => {
                "Every bundled Lucide glyph with its name. Draw one with theme::icon(ui, glyph, size, color)."
            }
        }
    }
}

/// The view a segmented control switches between.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum View {
    Table,
    Chart,
    Raw,
}

fn view_items() -> Vec<SegItem<'static, View>> {
    vec![
        SegItem::new(View::Table, "Table")
            .icon(icons::TABLE)
            .short("TB"),
        SegItem::new(View::Chart, "Chart")
            .icon(icons::CHART_LINE)
            .short("CH"),
        SegItem::new(View::Raw, "Raw")
            .icon(icons::BINARY)
            .short("RW"),
    ]
}

/// Segment with a badge: a count after the label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Open,
    Closed,
    All,
    Archived,
}

fn kind_items() -> [SegItem<'static, Kind>; 4] {
    [
        SegItem::new(Kind::Open, "Open").badge("3").short("O"),
        SegItem::new(Kind::Closed, "Closed").badge("12").short("C"),
        SegItem::new(Kind::All, "All").badge("15").short("A"),
        // a segment the app cannot offer yet: dimmed, not clickable, tooltip says why
        SegItem::new(Kind::Archived, "Archived")
            .badge("0")
            .short("R")
            .enabled(false)
            .tooltip("Needs the archive, not in this demo"),
    ]
}

/// Which modal dialog is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Modal {
    Closed,
    Delete,
    New,
}

/// One sample row of the grid: a made-up host with made-up numbers.
#[derive(Debug)]
struct Server {
    name: String,
    zone: &'static str,
    cpu: f32,
    mem: f32,
    state: Status,
}

fn status_word(s: Status) -> &'static str {
    match s {
        Status::Ok => "healthy",
        Status::Warn => "degraded",
        Status::Crit => "failing",
        Status::Idle => "idle",
    }
}

/// Deterministic pseudo-random hosts (no rand crate needed for a gallery).
fn sample_servers(n: usize) -> Vec<Server> {
    const ZONES: [&str; 3] = ["zone-a", "zone-b", "zone-c"];
    let mut seed = 0x5EED_u64;
    let mut next = move || {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (seed >> 33) as f32 / (1u64 << 31) as f32
    };
    (0..n)
        .map(|i| {
            let cpu = next() * 100.0;
            let state = if cpu > 90.0 {
                Status::Crit
            } else if cpu > 75.0 {
                Status::Warn
            } else {
                Status::Ok
            };
            Server {
                name: format!("node-{:04}.example", i + 1),
                zone: ZONES[i % ZONES.len()],
                cpu,
                mem: 2.0 + next() * 30.0,
                state,
            }
        })
        .collect()
}

/// The command palette's items: actions, hosts and one typed-only action.
fn palette_items() -> Vec<PaletteItem> {
    vec![
        PaletteItem::new(
            "Actions",
            icons::REFRESH_CW,
            "Refresh all hosts".into(),
            String::new(),
            Some("F5"),
        ),
        PaletteItem::new(
            "Actions",
            icons::DOWNLOAD,
            "Export table".into(),
            "CSV".into(),
            Some("Ctrl E"),
        ),
        PaletteItem::new(
            "Actions",
            icons::SETTINGS,
            "Open settings".into(),
            String::new(),
            None,
        ),
        PaletteItem::new(
            "Actions",
            icons::TRASH_2,
            "Clear the history".into(),
            String::new(),
            None,
        )
        .typed_only(),
        PaletteItem::new(
            "Hosts",
            icons::NETWORK,
            "web-01.example".into(),
            "zone-a · 12 ms".into(),
            None,
        )
        .chip(theme::identity::GREEN, "WE"),
        PaletteItem::new(
            "Hosts",
            icons::NETWORK,
            "db-primary.example".into(),
            "zone-b".into(),
            None,
        )
        .chip(theme::identity::BLUE, "DB"),
        PaletteItem::new(
            "Hosts",
            icons::CPU,
            "node-0042.example".into(),
            "zone-c · cpu 42 %".into(),
            None,
        )
        .mono()
        .dot(theme::identity::TEAL),
        PaletteItem::new(
            "Dashboards",
            icons::LAYOUT_GRID,
            "Overview".into(),
            "dashboard".into(),
            None,
        ),
    ]
}

struct Gallery {
    section: Section,
    /// The last action, shown under the section title.
    last: String,
    // buttons & controls
    running: bool,
    kind: Kind,
    view: View,
    toolbar_w: f32,
    // fields
    host: String,
    empty: String,
    secret: String,
    query: String,
    unit: String,
    every: String,
    range: usize,
    check_a: bool,
    check_b: bool,
    select_all: bool,
    switch_on: bool,
    // chart
    cpu: Vec<f32>,
    temp: Vec<f32>,
    // grid & inspector
    servers: Vec<Server>,
    grid_sel: Option<usize>,
    rich_sel: Option<usize>,
    scroll_to: Option<usize>,
    laid_out: usize,
    // overlays
    palette_state: PaletteState,
    palette_items: Vec<PaletteItem>,
    palette_pick: Option<String>,
    option: usize,
    modal: Modal,
    confirm: String,
    new_host: String,
    toast: Option<Toast>,
}

impl Gallery {
    fn new() -> Self {
        // Per-app wording: the palette's hint and the groups listed before anything is typed.
        let mut palette_state = PaletteState::default();
        palette_state.config = PaletteConfig {
            hint: "Search hosts, dashboards and actions".to_owned(),
            start_groups: vec!["Actions", "Hosts"],
            ..PaletteConfig::default()
        };
        // Demo series: smooth made-up curves, 120 samples, oldest first.
        let cpu = (0..120)
            .map(|i| 50.0 + 28.0 * (i as f32 * 0.12).sin() + 6.0 * (i as f32 * 0.45).cos())
            .collect();
        let temp = (0..120)
            .map(|i| 21.5 + 1.2 * (i as f32 * 0.05).sin())
            .collect();
        Self {
            section: Section::Buttons,
            last: "nothing yet".to_owned(),
            running: false,
            kind: Kind::Open,
            view: View::Table,
            toolbar_w: 520.0,
            host: HOST.to_owned(),
            empty: String::new(),
            secret: String::new(),
            query: String::new(),
            unit: "ms".to_owned(),
            every: "1".to_owned(),
            range: 1,
            check_a: true,
            check_b: false,
            select_all: false,
            switch_on: false,
            cpu,
            temp,
            servers: sample_servers(SERVERS),
            grid_sel: None,
            rich_sel: None,
            scroll_to: None,
            laid_out: 0,
            palette_state,
            palette_items: palette_items(),
            palette_pick: None,
            option: 2,
            modal: Modal::Closed,
            confirm: String::new(),
            new_host: String::new(),
            toast: None,
        }
    }

    fn show_buttons(&mut self, ui: &mut Ui) {
        // `Btn` is the button: builder calls pick the kind, size, icon and keycap.
        sec::sec_h(
            ui,
            "Kinds",
            Some(
                "Btn::new(label) with .primary(), .ghost(), .danger(); plain is the hairline default",
            ),
            true,
        );
        ui.horizontal_wrapped(|ui| {
            ui.add(Btn::new("Default"));
            ui.add(Btn::new("Primary").primary());
            ui.add(Btn::new("Ghost").ghost());
            ui.add(Btn::new("Delete").danger());
            ui.add(Btn::new("Disabled").enabled(false));
            // the same kinds, chosen by value
            ui.add(Btn::new("By kind").kind(BtnKind::Primary));
        });

        sec::sec_h(
            ui,
            "Sizes, icons, keycaps",
            Some(".small() is 24 px; .icon(glyph) leads; .kbd(text) adds a keycap"),
            false,
        );
        ui.horizontal_wrapped(|ui| {
            ui.add(Btn::new("Small").small());
            ui.add(Btn::new("Refresh").icon(icons::REFRESH_CW));
            ui.add(Btn::new("Save").icon(icons::SAVE).primary().small());
            ui.add(Btn::new("Save as").icon(icons::FILE_TEXT).icon16());
            ui.add(Btn::new("Run").kbd("Ctrl R").primary());
            ui.add(Btn::new("Find").kbd("Ctrl F").ghost());
        });

        sec::sec_h(
            ui,
            "Toggles and icon-only",
            Some(
                ".pressed(on) holds the pressed look; .icon_only(true) keeps the glyph, the label becomes a tooltip",
            ),
            false,
        );
        ui.horizontal_wrapped(|ui| {
            let (label, glyph) = if self.running {
                ("Pause", icons::PAUSE)
            } else {
                ("Start", icons::PLAY)
            };
            if ui
                .add(Btn::new(label).icon(glyph).pressed(self.running))
                .clicked()
            {
                self.running = !self.running;
                self.last = format!("{label} pressed");
            }
            ui.add(Btn::new("Settings").icon(icons::SETTINGS).icon_only(true));
            ui.add(
                Btn::new("Delete")
                    .icon(icons::TRASH_2)
                    .icon_only(true)
                    .danger(),
            );
            // `icon_button` is the flat square one, with its tooltip built in
            if icon_button(ui, icons::COPY, 28.0, "Copy").clicked() {
                self.last = "Copy (demo)".to_owned();
            }
            icon_button(ui, icons::X, 24.0, "Close");
            icon_button(ui, icons::ELLIPSIS, 20.0, "More");
        });

        sec::sec_h(
            ui,
            "Segmented control",
            Some("seg(ui, &mut value, &items); a SegItem takes an icon, a badge and a short label"),
            false,
        );
        ui.horizontal_wrapped(|ui| {
            if seg(ui, &mut self.kind, &kind_items()).changed() {
                self.last = format!("kind = {:?}", self.kind);
            }
            if seg(ui, &mut self.view, &view_items()).changed() {
                self.last = format!("view = {:?}", self.view);
            }
        });

        sec::sec_h(
            ui,
            "Toolbar that fits its width",
            Some(
                "Fit::pick(avail, full, icons): labels collapse, then the segment compacts and the rest move into the overflow menu",
            ),
            false,
        );
        ui.add(egui::Slider::new(&mut self.toolbar_w, 240.0..=900.0).text("width"));
        self.toolbar(ui);

        sec::sec_h(
            ui,
            "Wrapping row",
            Some("fit::flow(widths, gap, avail) gives each item its row"),
            false,
        );
        self.flow_row(ui);
    }

    /// A toolbar that picks its level from the width it has: the widths of each level come from the same
    /// widgets it draws.
    fn toolbar(&mut self, ui: &mut Ui) {
        const GAP: f32 = 6.0;
        let t = theme::theme(ui);
        let views = view_items();
        let start = Btn::new("Start").icon(icons::PLAY).primary();
        let refresh = Btn::new("Refresh").icon(icons::REFRESH_CW);
        let export = Btn::new("Export").icon(icons::DOWNLOAD);
        let avail = self.toolbar_w - 16.0;
        let full = fit::row(
            &[
                start.width(ui),
                refresh.width(ui),
                seg_width(ui, &views, false),
                export.width(ui),
            ],
            GAP,
        );
        let icons_w = fit::row(
            &[
                start.icon_only(true).width(ui),
                refresh.icon_only(true).width(ui),
                seg_width(ui, &views, false),
                export.icon_only(true).width(ui),
            ],
            GAP,
        );
        let level = Fit::pick(avail, full, icons_w);

        ui.allocate_ui(Vec2::new(self.toolbar_w, theme::BAR), |ui| {
            // `strip`: a fixed-height row with fill, bottom rule and inner padding
            widgets::strip(
                ui,
                theme::BAR,
                Some(t.panel),
                Some(t.line),
                (8.0, 8.0),
                |ui| {
                    ui.spacing_mut().item_spacing.x = GAP;
                    if ui.add(start.icon_only(level.icons())).clicked() {
                        self.last = "Start (toolbar)".to_owned();
                    }
                    if level.compact() {
                        seg_fit(ui, &mut self.view, &views, true);
                        // the secondary items move into the ⋯ menu
                        let more = icon_button(ui, icons::ELLIPSIS, 28.0, "More");
                        menu::dropdown(&more, |ui| {
                            if menu::item(ui, Some(icons::REFRESH_CW), "Refresh", Some("F5"))
                                .clicked()
                            {
                                self.last = "Refresh (menu)".to_owned();
                            }
                            if menu::item(ui, Some(icons::DOWNLOAD), "Export", Some("Ctrl E"))
                                .clicked()
                            {
                                self.last = "Export (menu)".to_owned();
                            }
                        });
                    } else {
                        if ui.add(refresh.icon_only(level.icons())).clicked() {
                            self.last = "Refresh (toolbar)".to_owned();
                        }
                        seg_fit(ui, &mut self.view, &views, false);
                        if ui.add(export.icon_only(level.icons())).clicked() {
                            self.last = "Export (toolbar)".to_owned();
                        }
                    }
                },
            );
        });
        widgets::text(
            ui,
            format!("level: {level:?} (avail {avail:.0}, full {full:.0}, icons {icons_w:.0})"),
            theme::mono(theme::FS_XS),
            t.text_3,
        );
    }

    /// `fit::flow` wraps items onto rows; each row is then laid out in a horizontal strip.
    fn flow_row(&mut self, ui: &mut Ui) {
        let labels = [
            "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta", "Iota",
        ];
        let buttons: Vec<Btn<'_>> = labels.iter().map(|&l| Btn::new(l).small()).collect();
        let widths: Vec<f32> = buttons.iter().map(|b| b.width(ui)).collect();
        let (rows, n) = fit::flow(&widths, 6.0, ui.available_width().min(260.0));
        for row in 0..n as u8 {
            ui.horizontal(|ui| {
                for (i, b) in buttons.iter().enumerate() {
                    if rows[i] == row && ui.add(*b).clicked() {
                        self.last = format!("{} (row {})", labels[i], row + 1);
                    }
                }
            });
        }
    }

    fn show_fields(&mut self, ui: &mut Ui) {
        let t = theme::theme(ui);
        sec::sec_h(
            ui,
            "Text inputs",
            Some(
                "text_input, text_input_invalid, password_input, search_field: 32 px, search 28 px",
            ),
            true,
        );
        form_label(ui, "Host, monospace");
        field::text_input(ui, &mut self.host, HOST, true, 320.0);
        form_label(ui, "Invalid (aria-invalid look)");
        field::text_input_invalid(ui, &mut self.empty, "Required", false, 320.0);
        form_label(ui, "Secret, shown as dots");
        field::password_input(ui, &mut self.secret, "Type anything", 320.0, false);
        form_label(ui, "Search");
        field::search_field(ui, &mut self.query, "Search nodes", 320.0);

        sec::sec_h(
            ui,
            "Query-bar fields",
            Some(
                "qfield: label inside the box, value right-aligned if asked; qselect: a menu of options",
            ),
            false,
        );
        ui.horizontal(|ui| {
            field::qfield(ui, "unit", &mut self.unit, 130.0, false);
            field::qfield(ui, "every", &mut self.every, 110.0, true);
            if field::qselect(ui, "range", &RANGES, &mut self.range, 170.0) {
                self.last = format!("range = {}", RANGES[self.range]);
            }
        });

        sec::sec_h(
            ui,
            "Form control",
            Some("seg_stretch: a segmented control that spans its field"),
            false,
        );
        form_label(ui, "View");
        if seg_stretch(ui, &mut self.view, &view_items()).changed() {
            self.last = format!("view = {:?}", self.view);
        }

        sec::sec_h(
            ui,
            "Checkbox and switch",
            Some("check::checkbox / checkbox_sm, switch::switch(ui, &mut on, title, description)"),
            false,
        );
        check::checkbox(ui, &mut self.check_a, "Record history", true);
        check::checkbox(ui, &mut self.check_b, "Disabled", false);
        check::checkbox_sm(
            ui,
            &mut self.select_all,
            "Select all (small, muted)",
            true,
            true,
        );
        switch::switch(
            ui,
            &mut self.switch_on,
            "Auto-reconnect",
            "Reconnect after a dropped link, up to five tries.",
        );
        widgets::text(
            ui,
            format!("record {} · auto {}", self.check_a, self.switch_on),
            theme::mono(theme::FS_XS),
            t.text_3,
        );
    }

    fn show_status(&mut self, ui: &mut Ui) {
        let t = theme::theme(ui);
        sec::sec_h(
            ui,
            "Status pills",
            Some("chip::status_pill(ui, Status, Some(word)); pass None for the icon alone"),
            true,
        );
        ui.horizontal_wrapped(|ui| {
            for s in [Status::Ok, Status::Warn, Status::Crit, Status::Idle] {
                chip::status_pill(ui, s, Some(status_word(s)));
            }
            chip::status_pill(ui, Status::Ok, None);
        });

        sec::sec_h(
            ui,
            "Chips and dots",
            Some("chip::chip(ui, color, initials, ChipSize::Md | Sm); widgets::dot(ui, color)"),
            false,
        );
        ui.horizontal_wrapped(|ui| {
            for (c, initials) in theme::identity::ALL.iter().zip(CONN_INITIALS) {
                chip::chip(ui, *c, initials, ChipSize::Md);
            }
        });
        ui.horizontal_wrapped(|ui| {
            for (c, initials) in theme::identity::ALL.iter().zip(CONN_INITIALS) {
                chip::chip(ui, *c, initials, ChipSize::Sm);
            }
            for c in theme::identity::ALL {
                widgets::dot(ui, c);
            }
        });
        ui.horizontal(|ui| {
            // a live dot is painted by hand: allocate its box, then paint at the center
            let (r, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
            chip::paint_live_dot(ui.painter(), r.center(), t.ok);
            widgets::text(ui, "Live", theme::ui_font(theme::FS_SM), t.text);
        });

        sec::sec_h(
            ui,
            "Notices",
            Some("notice(ui, Level, bold, rest, actions): a tinted strip, actions at the right"),
            false,
        );
        notice::notice(
            ui,
            Level::Info,
            "Read-only view.",
            "Edits are off until the link is writable.",
            |ui| {
                if ui.add(Btn::new("Learn more").small().ghost()).clicked() {
                    self.last = "Learn more (demo)".to_owned();
                }
            },
        );
        notice::notice(
            ui,
            Level::Warn,
            "Clock drift.",
            "The local clock is 4 s behind the example server.",
            |ui| {
                if ui.add(Btn::new("Sync now").small()).clicked() {
                    self.last = "Sync now (demo)".to_owned();
                }
            },
        );
        notice::notice(
            ui,
            Level::Crit,
            "Link lost.",
            "Retrying in 5 s. The last value is 18 minutes old.",
            |ui| {
                if ui.add(Btn::new("Retry").small().danger()).clicked() {
                    self.last = "Retry (demo)".to_owned();
                }
            },
        );
    }

    fn show_chart(&mut self, ui: &mut Ui) {
        let t = theme::theme(ui);
        sec::sec_h(
            ui,
            "Metrics strip",
            Some(
                "metrics(ui, &cells) returns the clicked cell; a Metric with link: true reacts to clicks",
            ),
            true,
        );
        let cells = [
            Metric::new("Uptime", "3d 04h", "since restart").icon(icons::CLOCK),
            Metric {
                link: true,
                ..Metric::new("Errors", "12", "last hour")
            }
            .icon(icons::TRIANGLE_ALERT)
            .color(t.warn),
            Metric::new("p95 latency", "18.4 ms", "5 min window"),
            Metric::new("Throughput", "4,153 /s", "average"),
        ];
        if let Some(i) = metrics::metrics(ui, &cells) {
            self.last = format!("metric '{}' clicked", cells[i].key);
        }

        sec::sec_h(
            ui,
            "Line chart",
            Some("Chart::new(id, &series).sample_secs(1.0).height(160.0).decimals(1).show(ui)"),
            false,
        );
        let series = [
            Series {
                name: "CPU",
                color: t.series(0),
                values: &self.cpu,
                unit: "%",
                limit: Some(80.0),
            },
            Series {
                name: "Temp",
                color: t.series(1),
                values: &self.temp,
                unit: "°C",
                limit: None,
            },
        ];
        Chart::new("gallery-chart", &series)
            .sample_secs(1.0)
            .height(160.0)
            .decimals(1)
            .show(ui);

        let mut num = String::new();
        widgets::chart::fmt_num(&mut num, 4153.5, 1);
        widgets::text(
            ui,
            format!(
                "chart::fmt_num(4153.5, 1) = {num} · chart::nice_step(34.0) = {} · metrics::grid_shape(4, 600) = {:?}",
                widgets::chart::nice_step(34.0),
                metrics::grid_shape(4, 600.0)
            ),
            theme::mono(theme::FS_XS),
            t.text_3,
        );
    }

    fn show_grid(&mut self, ui: &mut Ui) {
        let t = theme::theme(ui);
        sec::sec_h(
            ui,
            "Virtualized table",
            Some(
                "Grid::new(id, &cols, rows).selected(i).sticky_first(true).scroll_to(i).show(ui, |c| ..)",
            ),
            true,
        );
        ui.horizontal(|ui| {
            if ui.add(Btn::new("Jump to node 4,000").small()).clicked() {
                self.scroll_to = Some(3_999);
            }
            widgets::text(
                ui,
                format!(
                    "{} rows, {} laid out this frame",
                    self.servers.len(),
                    self.laid_out
                ),
                theme::mono(theme::FS_XS),
                t.text_3,
            );
        });
        let scroll = self.scroll_to.take();
        let half = (ui.available_height() * 0.5).floor();

        let cols = [
            Column::new("#", 64.0).right(),
            Column::new("host", 210.0).flex(),
            Column::new("zone", 90.0),
            Column::new("cpu", 80.0).hint("%").right(),
            Column::new("mem", 90.0).hint("GiB").right(),
            Column::new("state", 120.0),
        ];
        let servers = &self.servers;
        ui.allocate_ui(Vec2::new(ui.available_width(), half), |ui| {
            let out = Grid::new("gallery-grid", &cols, servers.len())
                .selected(self.grid_sel)
                .sticky_first(true)
                .scroll_to(scroll)
                .show(ui, |c| {
                    let s = &servers[c.row];
                    match c.col {
                        0 => {
                            let _ = write!(c.text, "{}", c.row + 1);
                            c.color = t.text_3;
                        }
                        1 => c.text.push_str(&s.name),
                        2 => c.text.push_str(s.zone),
                        3 => {
                            let _ = write!(c.text, "{:.1}", s.cpu);
                        }
                        4 => {
                            let _ = write!(c.text, "{:.1}", s.mem);
                        }
                        // a status cell: an icon in its color, then the word
                        _ => {
                            c.text.push_str(status_word(s.state));
                            c.icon = Some((s.state.icon(), s.state.color(t)));
                            c.font = CellFont::Ui;
                        }
                    }
                });
            self.laid_out = out.laid_out;
            if let Some(r) = out.clicked {
                self.grid_sel = Some(r);
            }
            if let Some(r) = out.secondary_clicked {
                self.last = format!("right-clicked {}", servers[r].name);
            }
            if let Some((r, c)) = out.double_clicked {
                self.last = format!("double-clicked row {} column {c}", r + 1);
            }
        });

        ui.add_space(6.0);
        sec::sec_h(
            ui,
            "Cells that paint themselves",
            Some("RichGrid: c.text, c.pill, c.bar, c.button; row_bg tints rows"),
            false,
        );
        let rcols = [
            Column::new("host", 220.0),
            Column::new("state", 130.0),
            Column::new("cpu", 190.0).flex(),
            Column::new("action", 110.0).right(),
        ];
        // rows that are failing get a faint danger tint under hover and selection
        let row_bg =
            |row: usize| (servers[row].state == Status::Crit).then(|| theme::alpha(t.danger, 0.08));
        let mut copied = None;
        let out = RichGrid::new("gallery-rich", &rcols, servers.len())
            .selected(self.rich_sel)
            .row_bg(&row_bg)
            .show(ui, |c| {
                let s = &servers[c.row];
                match c.col {
                    0 => {
                        // `text2`: the host in text, its domain in a faint second run
                        let (host, domain) =
                            s.name.split_at(s.name.find('.').unwrap_or(s.name.len()));
                        c.text2(host, t.text, domain, t.text_3, theme::data());
                    }
                    1 => c.pill(
                        s.state.icon(),
                        s.state.color(t),
                        status_word(s.state),
                        t.text,
                    ),
                    2 => {
                        c.bar(s.cpu / 100.0, 120.0, s.state.color(t), t);
                    }
                    _ => {
                        if c.button(Some(icons::COPY), "Copy", true).clicked() {
                            copied = Some(c.row);
                        }
                    }
                }
            });
        if let Some(r) = out.clicked {
            self.rich_sel = Some(r);
        }
        if let Some(r) = copied {
            self.last = format!("copied {}", servers[r].name);
        }
    }

    fn show_inspector(&mut self, ui: &mut Ui) {
        let t = theme::theme(ui);
        sec::sec_h(
            ui,
            "Side inspector",
            Some("Inspector::new(id, title).id_text(..).copy(text).show_with_foot(ui, body, foot)"),
            true,
        );
        ui.horizontal(|ui| {
            // the width is shared by every inspector and kept by the app: read it, set it
            let ctx = ui.ctx().clone();
            let mut w = inspector::width(&ctx);
            let slider = egui::Slider::new(&mut w, inspector::MIN_WIDTH..=inspector::MAX_WIDTH)
                .text("width");
            if ui.add(slider).changed() {
                inspector::set_width(&ctx, w);
            }
            widgets::text(
                ui,
                "drag the inspector's left edge; double-click resets",
                theme::ui_font(theme::FS_XS),
                t.text_3,
            );
        });
        // a window up to 960 px wide stacks every inspector under its table (this one is 1280 px)
        let stacked = inspector::stacked(ui.ctx().content_rect().width());
        widgets::text(
            ui,
            format!("stacked under the table: {stacked}"),
            theme::mono(theme::FS_XS),
            t.text_3,
        );
        ui.add_space(6.0);

        let sel = self
            .grid_sel
            .unwrap_or(0)
            .min(self.servers.len().saturating_sub(1));
        let s = &self.servers[sel];
        let id_text = format!("#{}", sel + 1);
        let cpu = format!("{:.1} %", s.cpu);
        let mem = format!("{:.1} GiB", s.mem);
        // The side inspector takes the right of this area; the table takes what is left.
        let resp = Inspector::new("gallery-inspector", &s.name)
            .id_text(&id_text)
            .copy(&s.name)
            .show_with_foot(
                ui,
                |ui| {
                    sec::sec_h(ui, "Host", None, true);
                    sec::kv(ui, 0, "name", &s.name, t.text, None);
                    sec::kv(ui, 0, "zone", s.zone, t.text, Some("region"));
                    sec::kv(
                        ui,
                        0,
                        "state",
                        status_word(s.state),
                        s.state.color(t),
                        Some("status"),
                    );
                    sec::sec_h_with(ui, "Load", Some("sampled"), false, |ui| {
                        ui.add(Btn::new("Refresh").small().ghost());
                    });
                    // indented rows, one level deeper
                    sec::kv(ui, 1, "cpu", &cpu, t.text, None);
                    sec::kv(ui, 1, "mem", &mem, t.text, None);
                    sec::sec_h(ui, "Notes", None, false);
                    sec::kv(
                        ui,
                        0,
                        "tags",
                        "env=lab role=web rack=r-07 (example data; a long value wraps)",
                        t.text_2,
                        None,
                    );
                },
                |ui| {
                    if ui.add(Btn::new("Drain").small()).clicked() {
                        self.last = "Drain (demo)".to_owned();
                    }
                    if ui.add(Btn::new("Restart").small().ghost()).clicked() {
                        self.last = "Restart (demo)".to_owned();
                    }
                },
            );
        if resp.copied {
            self.last = "copied host name".to_owned();
        }

        // the table beside the inspector
        let cols = [
            Column::new("host", 200.0).flex(),
            Column::new("zone", 90.0),
            Column::new("state", 110.0),
        ];
        let servers = &self.servers;
        let out = Grid::new("inspector-grid", &cols, servers.len())
            .selected(Some(sel))
            .show(ui, |c| {
                let s = &servers[c.row];
                match c.col {
                    0 => c.text.push_str(&s.name),
                    1 => c.text.push_str(s.zone),
                    _ => {
                        c.text.push_str(status_word(s.state));
                        c.icon = Some((s.state.icon(), s.state.color(t)));
                    }
                }
            });
        if let Some(r) = out.clicked {
            self.grid_sel = Some(r);
        }
    }

    fn show_overlays(&mut self, ui: &mut Ui) {
        let t = theme::theme(ui);
        sec::sec_h(
            ui,
            "Command palette",
            Some(
                "palette::palette(ctx, &mut state, &items) each frame; Ctrl K opens it, Esc closes",
            ),
            true,
        );
        ui.horizontal(|ui| {
            if ui
                .add(Btn::new("Open palette").icon(icons::SEARCH).kbd("Ctrl K"))
                .clicked()
            {
                self.palette_state.open();
            }
            let pick = self.palette_pick.as_deref().unwrap_or("none yet");
            widgets::text(
                ui,
                format!("last pick: {pick}"),
                theme::mono(theme::FS_XS),
                t.text_3,
            );
        });
        // the matcher behind the search: every typed word must match, the label start ranks first
        let s = palette::score("cold", "cold room", "cold room bench");
        widgets::text(
            ui,
            format!("palette::score(\"cold\", \"cold room\") = {s}"),
            theme::mono(theme::FS_XS),
            t.text_3,
        );

        sec::sec_h(
            ui,
            "Menus",
            Some("menu::context(&resp, ..) on right-click; menu::dropdown(&resp, ..) on a click"),
            false,
        );
        ui.horizontal(|ui| {
            let target = ui.add(Btn::new("Right-click me").ghost());
            menu::context(&target, |ui| {
                menu::head(ui, "Host");
                if menu::item(ui, Some(icons::COPY), "Copy name", Some("Ctrl C")).clicked() {
                    self.last = "copy name (menu)".to_owned();
                }
                if menu::item(ui, Some(icons::PENCIL), "Rename", None).clicked() {
                    self.last = "rename (menu)".to_owned();
                }
                menu::item_disabled(ui, Some(icons::LOCK), "Lock (needs admin)");
                menu::sep(ui);
                if menu::danger(ui, Some(icons::TRASH_2), "Delete host").clicked() {
                    self.last = "delete (menu)".to_owned();
                }
            });

            let trigger = ui.add(Btn::new("Actions").icon(icons::CHEVRON_DOWN).ghost());
            menu::dropdown(&trigger, |ui| {
                menu::head(ui, "Connections");
                if menu::conn_item(
                    ui,
                    theme::identity::GREEN,
                    "WE",
                    "web-01.example",
                    "12 ms",
                    true,
                )
                .clicked()
                {
                    self.last = "web-01 (conn item)".to_owned();
                }
                let (row, unplug) = menu::open_conn_item(
                    ui,
                    theme::identity::BLUE,
                    "DB",
                    "db-primary.example",
                    true,
                );
                if row.clicked() {
                    self.last = "db-primary (open item)".to_owned();
                }
                if unplug.clicked() {
                    self.last = "unplug db-primary (demo)".to_owned();
                }
                menu::sep(ui);
                if menu::item(ui, Some(icons::PLUS), "New connection", Some("Ctrl N")).clicked() {
                    self.last = "new connection (menu)".to_owned();
                }
            });
        });

        sec::sec_h(
            ui,
            "Select popup",
            Some(
                "select_popup_scroll(&trigger, Some(i), rows, max_h, |ui| menu::option(..)): taller lists scroll",
            ),
            false,
        );
        let trigger = ui.add(Btn::new(OPTIONS[self.option]));
        menu::select_popup_scroll(&trigger, Some(self.option), OPTIONS.len(), 140.0, |ui| {
            for (i, o) in OPTIONS.iter().enumerate() {
                if menu::option(ui, o, i == self.option).clicked() {
                    self.option = i;
                }
            }
        });
        let state = if menu::is_open(&trigger) {
            "open"
        } else {
            "closed"
        };
        widgets::text(
            ui,
            format!("popup {state}"),
            theme::mono(theme::FS_XS),
            t.text_3,
        );

        sec::sec_h(
            ui,
            "Dialog and toast",
            Some(
                "Dialog::new(id, title).show(ctx, body, footer) and Toast::new(ctx, title, sub) into a slot",
            ),
            false,
        );
        ui.horizontal(|ui| {
            if ui.add(Btn::new("Delete host").danger()).clicked() {
                self.modal = Modal::Delete;
                self.confirm.clear();
            }
            if ui.add(Btn::new("New host")).clicked() {
                self.modal = Modal::New;
            }
            if ui
                .add(Btn::new("Saved toast").icon(icons::CIRCLE_CHECK))
                .clicked()
            {
                self.toast = Some(Toast::new(
                    ui.ctx(),
                    "Saved",
                    "3 changes written to the example config.",
                ));
            }
            if ui
                .add(Btn::new("Warning toast").icon(icons::TRIANGLE_ALERT))
                .clicked()
            {
                self.toast = Some(
                    Toast::new(ui.ctx(), "Write failed", "Check the link and retry.")
                        .icon(icons::TRIANGLE_ALERT, Some(t.warn)),
                );
            }
        });
    }

    fn show_text(&mut self, ui: &mut Ui) {
        let t = theme::theme(ui);
        sec::sec_h(
            ui,
            "Text",
            Some(
                "text(ui, s, font, color) on one line; text_lh uses the 1.45 line box; para wraps",
            ),
            true,
        );
        widgets::text(ui, "One line, no wrap: Inter 13 px", theme::body(), t.text);
        widgets::text_lh(
            ui,
            "The same text on a 1.45 line box",
            theme::body(),
            t.text_2,
        );
        widgets::para(
            ui,
            "A paragraph wraps at the available width. Its rows sit 1.45 line heights apart, as in the family's \
             paragraphs, and each glyph run is centered in its row.",
            theme::ui_font(theme::FS_SM),
            t.text_2,
        );
        let job = LayoutJob::simple(
            "css_text lays a job out with the family's line box.".to_owned(),
            theme::ui_font(theme::FS_SM),
            t.text_2,
            f32::INFINITY,
        );
        let (g, dy) = widgets::css_text(ui, job, widgets::lh(theme::FS_SM));
        let (r, _) = ui.allocate_exact_size(g.size(), Sense::hover());
        ui.painter().galley(r.min + Vec2::new(0.0, dy), g, t.text_2);
        widgets::text(
            ui,
            format!("lh(13) = {:.2} px", widgets::lh(theme::FS)),
            theme::mono(theme::FS_XS),
            t.text_3,
        );

        sec::sec_h(
            ui,
            "Truncation and rules",
            Some("galley_truncated ends a line with an ellipsis; hline and vline draw 1 px rules"),
            false,
        );
        let (r, _) = ui.allocate_exact_size(Vec2::new(240.0, 20.0), Sense::hover());
        let g = widgets::galley_truncated(
            ui,
            "a host name much too long for this cell",
            theme::mono(theme::FS_DATA),
            t.text,
            r.width(),
        );
        ui.painter().galley(r.left_top(), g, t.text);
        ui.horizontal(|ui| {
            let (rule, _) = ui.allocate_exact_size(Vec2::new(120.0, 8.0), Sense::hover());
            widgets::hline(ui.painter(), rule.x_range(), rule.center().y, t.line);
            let (rule, _) = ui.allocate_exact_size(Vec2::new(8.0, 24.0), Sense::hover());
            widgets::vline(ui.painter(), rule.center().x, rule.y_range(), t.line);
            widgets::text(ui, "rule", theme::ui_font(theme::FS_XS), t.text_3);
        });

        sec::sec_h(
            ui,
            "Keycaps",
            Some("kbd_size and paint_kbd draw a keycap; kbd_rect_right puts one at a right edge"),
            false,
        );
        ui.horizontal(|ui| {
            for k in ["Ctrl", "K", "Esc"] {
                let g = widgets::galley(ui, k, theme::medium(10.5), t.text_2);
                let (r, _) = ui.allocate_exact_size(widgets::kbd_size(&g), Sense::hover());
                widgets::paint_kbd(ui.painter(), r, g, t.editor, t.line, t.text_2);
            }
        });
        let (row, _) = ui.allocate_exact_size(Vec2::new(320.0, 24.0), Sense::hover());
        widgets::text(
            ui,
            "Search the palette",
            theme::ui_font(theme::FS_SM),
            t.text_2,
        );
        let g = widgets::galley(ui, "Esc", theme::medium(10.5), t.text_2);
        let r = widgets::kbd_rect_right(&g, row.right(), row.center().y);
        widgets::paint_kbd(ui.painter(), r, g, t.editor, t.line, t.text_2);

        sec::sec_h(
            ui,
            "Tooltips",
            Some("tip(resp, text) below the trigger; tip_at(resp, text, side, suffix) elsewhere"),
            false,
        );
        ui.horizontal(|ui| {
            let r = ui.add(Btn::new("Below (default)"));
            widgets::tip(r, "Shown below the trigger");
            let r = ui.add(Btn::new("Right, with a shortcut").ghost());
            widgets::tip_at(r, "Shown to the right", TipSide::RIGHT, Some("F2"));
            let r = ui.add(Btn::new("Above, wide").ghost());
            widgets::tip_at(r, "Shown above, up to 448 px wide", TipSide::TOP_MD, None);
        });

        sec::sec_h(
            ui,
            "Focus",
            Some("track_focus_visible runs once a frame; the ring shows after Tab or arrow keys"),
            false,
        );
        let visible = widgets::focus_is_visible(ui.ctx());
        widgets::text(
            ui,
            if visible {
                "keyboard focus ring: shown"
            } else {
                "keyboard focus ring: hidden (press Tab to see it)"
            },
            theme::ui_font(theme::FS_SM),
            t.text_2,
        );
    }

    fn show_icons(&mut self, ui: &mut Ui) {
        let t = theme::theme(ui);
        sec::sec_h(
            ui,
            "Lucide glyphs",
            Some("icons::ALL: every bundled glyph as (lucide name, glyph); hover for the name"),
            true,
        );
        ui.horizontal_wrapped(|ui| {
            for &(name, glyph) in icons::ALL {
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(104.0, 60.0), Sense::hover());
                if resp.hovered() {
                    ui.painter().rect_filled(rect, theme::R, t.row_hover);
                }
                theme::paint_icon(
                    ui.painter(),
                    egui::pos2(rect.center().x, rect.top() + 22.0),
                    glyph,
                    20.0,
                    t.text,
                );
                let g = widgets::galley_truncated(ui, name, theme::mono(10.5), t.text_3, 96.0);
                let x = rect.center().x - g.size().x / 2.0;
                ui.painter()
                    .galley(egui::pos2(x, rect.top() + 42.0), g, t.text_3);
                widgets::tip(resp, name);
            }
        });
    }

    /// Modal and floating layers: the palette, the dialogs and the toast, drawn over the window.
    fn show_floating(&mut self, ctx: &egui::Context) {
        if let Some(i) = palette::palette(ctx, &mut self.palette_state, &self.palette_items) {
            self.palette_pick = Some(self.palette_items[i].label.clone());
        }
        match self.modal {
            Modal::Closed => {}
            Modal::Delete => self.delete_dialog(ctx),
            Modal::New => self.new_dialog(ctx),
        }
        toast::show(ctx, &mut self.toast);
    }

    /// The danger dialog with a typed guard: Delete stays off until the host name is typed.
    fn delete_dialog(&mut self, ctx: &egui::Context) {
        let t = Theme::of(ctx.theme());
        // the footer reads what the body typed this frame
        let typed_ok = Cell::new(false);
        let mut delete = false;
        let mut cancel = false;
        // a rich title: the bold verb in the title font, the host in mono
        let mut title = LayoutJob::default();
        title.append(
            "Delete ",
            0.0,
            TextFormat::simple(theme::semibold(15.0), t.text),
        );
        title.append(HOST, 0.0, TextFormat::simple(theme::mono(13.0), t.text_2));
        let r = Dialog::with_title("gallery-delete", title).danger(t).show(
            ctx,
            |ui| {
                dialog::field_row(ui, "Host", |ui| {
                    widgets::text(ui, HOST, theme::data(), t.text);
                });
                dialog::field_row(ui, "Effect", |ui| {
                    widgets::text(
                        ui,
                        "Removes the host and its history.",
                        theme::ui_font(theme::FS_SM),
                        t.text_2,
                    );
                });
                widgets::para(
                    ui,
                    "Type the host name to confirm.",
                    theme::ui_font(theme::FS_SM),
                    t.text_2,
                );
                field::text_input(ui, &mut self.confirm, HOST, true, f32::INFINITY);
                typed_ok.set(self.confirm == HOST);
            },
            |ui| {
                if ui
                    .add(Btn::new("Delete host").danger().enabled(typed_ok.get()))
                    .clicked()
                {
                    delete = true;
                }
                if ui.add(Btn::new("Cancel").ghost()).clicked() {
                    cancel = true;
                }
            },
        );
        if delete {
            self.toast = Some(Toast::new(
                ctx,
                "Host removed (demo)",
                "Nothing was deleted: this gallery only shows the dialog.",
            ));
        }
        if delete || cancel || r.close {
            self.modal = Modal::Closed;
        }
    }

    /// A dialog whose footer buttons sit at the left (`footer_start`), in the order added.
    fn new_dialog(&mut self, ctx: &egui::Context) {
        let t = Theme::of(ctx.theme());
        let mut create = false;
        let mut cancel = false;
        let r = Dialog::new("gallery-new", "New host")
            .width(460.0)
            .footer_start()
            .show(
                ctx,
                |ui| {
                    widgets::text(ui, "Host name", theme::ui_font(theme::FS_SM), t.text_2);
                    field::text_input(
                        ui,
                        &mut self.new_host,
                        "node-0200.example",
                        true,
                        f32::INFINITY,
                    );
                },
                |ui| {
                    if ui.add(Btn::new("Create").primary()).clicked() {
                        create = true;
                    }
                    if ui.add(Btn::new("Cancel").ghost()).clicked() {
                        cancel = true;
                    }
                },
            );
        if create {
            self.last = format!("create '{}' (demo)", self.new_host);
        }
        if create || cancel || r.close {
            self.modal = Modal::Closed;
        }
    }

    fn show_section(&mut self, section: Section, ui: &mut Ui) {
        match section {
            Section::Buttons => self.show_buttons(ui),
            Section::Fields => self.show_fields(ui),
            Section::Status => self.show_status(ui),
            Section::Chart => self.show_chart(ui),
            Section::Overlays => self.show_overlays(ui),
            Section::Text => self.show_text(ui),
            Section::Icons => self.show_icons(ui),
            // the grid and the inspector lay out their own panes: they are not scrolled
            Section::Grid | Section::Inspector => {}
        }
    }
}

/// A form label above its control: 12 px in text-2.
fn form_label(ui: &mut Ui, s: &str) {
    widgets::text(ui, s, theme::ui_font(theme::FS_SM), theme::theme(ui).text_2);
}

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Focus rings show after Tab or the arrow keys only (see `widgets::track_focus_visible`).
        widgets::track_focus_visible(&ctx);
        // Ctrl K (cmd on macOS) opens the palette.
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::K)) {
            self.palette_state.open();
        }
        let t = theme::theme(ui);

        egui::Panel::top("top-bar")
            .exact_size(theme::BAR)
            .show_separator_line(false)
            .frame(Frame::NONE.fill(t.panel))
            .show(ui, |ui| {
                widgets::strip(ui, theme::BAR, None, Some(t.line), (12.0, 12.0), |ui| {
                    widgets::text(ui, "studio-ui gallery", theme::semibold(theme::FS), t.text);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // the theme toggle: both styles are installed, switching is one call
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

        egui::Panel::left("sections")
            .exact_size(200.0)
            .resizable(false)
            .frame(Frame::NONE.fill(t.panel).inner_margin(Margin::same(8)))
            .show(ui, |ui| {
                for s in Section::ALL {
                    ui.selectable_value(&mut self.section, s, s.label());
                }
            });

        let section = self.section;
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(t.editor).inner_margin(Margin::same(16)))
            .show(ui, |ui| {
                widgets::text(ui, section.label(), theme::semibold(15.0), t.text);
                widgets::para(ui, section.intro(), theme::ui_font(theme::FS_SM), t.text_3);
                widgets::text(
                    ui,
                    format!("last action: {}", self.last),
                    theme::mono(theme::FS_XS),
                    t.text_3,
                );
                ui.add_space(4.0);
                match section {
                    // these lay out their own fixed panes, so they are not put in a scroll area
                    Section::Grid => self.show_grid(ui),
                    Section::Inspector => self.show_inspector(ui),
                    _ => {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| self.show_section(section, ui));
                    }
                }
            });

        self.show_floating(&ctx);
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("studio-ui gallery")
            .with_inner_size([1280.0, 820.0]),
        ..Default::default()
    };
    eframe::run_native(
        "studio-ui gallery",
        options,
        Box::new(|cc| {
            // fonts, both styles, dark first: once, at startup
            theme::install(&cc.egui_ctx);
            Ok(Box::new(Gallery::new()))
        }),
    )
}
