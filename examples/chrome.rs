//! The whole IDE frame of `studio_ui::chrome` with fake data: toolbar, tool stripe, side panels with an
//! explorer that mixes three kinds of connection, the tab row, the status bar, and the welcome screen once
//! every tab is closed.
//!
//! `cargo run --example chrome`. Environment, for captures: `STUDIO_UI_CHROME_SCREEN=welcome` starts with no
//! tab open, `=narrow-crumbs` opens on a tab with long breadcrumbs, `=ua` on one of a live plant; `STUDIO_UI_CHROME_THEME=light` starts in
//! the light theme; `STUDIO_UI_CHROME_SIDE=<0..5>` picks the side panel; `STUDIO_UI_CHROME_SIZE=1100x700`
//! sets the window's size.
//!
//! Everything app-specific stays here, as it would in an app: the model, what a click means, the menus and
//! the tooltips' wording.

use std::collections::HashSet;

use egui::{Color32, Frame, Id, Painter, Rect, Ui, Vec2};

use studio_ui::chrome::{
    self, panel, statusbar,
    statusbar::{Gives, Seg, Shrink},
    stripe::{self, StripeItem},
    tabs::{Dirty, More, Tab, TabRow},
    toolbar::{self, ConnLead, ConnWidget, Crumb, Toolbar},
    tree::{self, After, Lead, Row},
    welcome::{self, Trail, Welcome, WelcomeRow},
};
use studio_ui::icons;
use studio_ui::theme::{self, identity};
use studio_ui::widgets::{self, menu};

/// A made-up connection of one of three kinds.
struct Conn {
    name: &'static str,
    initials: &'static str,
    color: Color32,
    site: usize,
    /// The faint kind after its name.
    kind: &'static str,
    up: bool,
    /// The status bar's detail and the welcome screen's address.
    detail: &'static str,
    address: &'static str,
}

const SITES: [&str; 3] = ["Plant A · cold rooms", "Water treatment", "Bench"];

fn conns() -> Vec<Conn> {
    vec![
        Conn {
            name: "Cold rooms, line A",
            initials: "CR",
            color: identity::GREEN,
            site: 0,
            kind: "mb",
            up: true,
            detail: "RTU · 19200 8E1",
            address: "ttyUSB0 · 19200 8E1",
        },
        Conn {
            name: "Line A broker",
            initials: "LB",
            color: identity::VIOLET,
            site: 0,
            kind: "mq",
            up: true,
            detail: "MQTT 5 · TLS",
            address: "broker.example:8883",
        },
        Conn {
            name: "Treatment SCADA",
            initials: "TS",
            color: identity::TEAL,
            site: 1,
            kind: "ua",
            up: true,
            detail: "Sign & encrypt",
            address: "opc.tcp://scada.example:4840",
        },
        Conn {
            name: "Filter gallery PLC",
            initials: "FG",
            color: identity::AMBER,
            site: 1,
            kind: "ua",
            up: false,
            detail: "Untrusted",
            address: "opc.tcp://192.0.2.40:4840",
        },
        Conn {
            name: "Bench simulator",
            initials: "BS",
            color: identity::SLATE,
            site: 2,
            kind: "mb",
            up: false,
            detail: "Simulator",
            address: "127.0.0.1:5020",
        },
    ]
}

/// An open tab: what it shows and what it belongs to.
#[derive(Clone)]
struct OpenTab {
    name: &'static str,
    icon: &'static str,
    /// Index into the connections; none for the workspace.
    conn: Option<usize>,
    dirty: Option<Dirty>,
    crumbs: &'static [(&'static str, &'static str, bool)],
    /// The status bar's view segment.
    status: &'static str,
}

fn all_tabs() -> Vec<OpenTab> {
    vec![
        OpenTab {
            name: "Cold room 1 controller",
            icon: icons::CPU,
            conn: Some(0),
            dirty: None,
            crumbs: &[
                (icons::ACTIVITY, "ttyUSB0", true),
                (icons::CPU, "unit 10", true),
            ],
            status: "unit 10 · cold-room-controller.map.json",
        },
        OpenTab {
            name: "Bus",
            icon: icons::ACTIVITY,
            conn: Some(0),
            dirty: None,
            crumbs: &[(icons::ACTIVITY, "ttyUSB0", true)],
            status: "4 units · 38 % busy",
        },
        OpenTab {
            name: "Fast trend",
            icon: icons::LIST,
            conn: Some(2),
            dirty: None,
            crumbs: &[
                (icons::GLOBE, "scada.example:4840", true),
                (icons::LIST, "Fast trend", false),
            ],
            status: "18 items · 250 ms",
        },
        OpenTab {
            name: "plant/line-a/#",
            icon: icons::INBOX,
            conn: Some(1),
            dirty: None,
            crumbs: &[
                (
                    icons::GLOBE,
                    "mqtts://broker.example:8883/a-rather-long-path-for-the-crumbs",
                    true,
                ),
                (
                    icons::INBOX,
                    "plant/line-a/cold-room-1/evaporator/fans/speed",
                    true,
                ),
            ],
            status: "612 messages · 3 retained",
        },
        OpenTab {
            name: "line-a.topics.yaml",
            icon: icons::FILE_BRACES,
            conn: None,
            dirty: Some(Dirty::Pending),
            crumbs: &[(icons::FILE_BRACES, "line-a.topics.yaml", true)],
            status: "YAML · 64 lines",
        },
        OpenTab {
            name: "Tags",
            icon: icons::TAG,
            conn: None,
            dirty: Some(Dirty::Changed),
            crumbs: &[(icons::TAG, "Tags", false)],
            status: "42 tags",
        },
    ]
}

const SIDES: [(&str, &str); 6] = [
    (icons::NETWORK, "Explorer"),
    (icons::LIST_TREE, "Browse: address space and topics"),
    (
        icons::FOLDER_TREE,
        "Project: maps, watch lists, topic maps, tags",
    ),
    (icons::SHIELD_CHECK, "Certificates"),
    (icons::CIRCLE_DOT, "Recordings"),
    (icons::BELL, "Problems"),
];

struct Example {
    conns: Vec<Conn>,
    tabs: Vec<OpenTab>,
    active: Option<usize>,
    side: usize,
    side_visible: bool,
    filter: String,
    open: HashSet<String>,
    recording: Option<f64>,
    polling: bool,
    notation: usize,
    problems: usize,
}

impl Example {
    fn new() -> Self {
        let screen = std::env::var("STUDIO_UI_CHROME_SCREEN").unwrap_or_default();
        let tabs = if screen == "welcome" {
            Vec::new()
        } else {
            all_tabs()
        };
        let active = match screen.as_str() {
            "welcome" => None,
            "narrow-crumbs" => Some(3),
            "ua" => Some(2),
            _ => Some(0),
        };
        let side = std::env::var("STUDIO_UI_CHROME_SIDE")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&s: &usize| s < SIDES.len())
            .unwrap_or(0);
        let open = ["s:0", "s:1", "c:0", "c:2"].map(str::to_owned).into();
        Self {
            conns: conns(),
            tabs,
            active,
            side,
            side_visible: true,
            filter: String::new(),
            open,
            recording: None,
            polling: true,
            notation: 0,
            problems: 2,
        }
    }

    fn active_tab(&self) -> Option<&OpenTab> {
        self.active.and_then(|i| self.tabs.get(i))
    }

    fn active_conn(&self) -> Option<&Conn> {
        self.active_tab()
            .and_then(|t| t.conn)
            .and_then(|c| self.conns.get(c))
    }

    /// Select the tab named `name`, opening it when it is not.
    fn open_tab(&mut self, name: &str) {
        if let Some(i) = self.tabs.iter().position(|t| t.name == name) {
            self.active = Some(i);
        } else if let Some(t) = all_tabs().into_iter().find(|t| t.name == name) {
            self.tabs.push(t);
            self.active = Some(self.tabs.len() - 1);
        }
    }

    fn close_tab(&mut self, i: usize) {
        if i >= self.tabs.len() {
            return;
        }
        self.tabs.remove(i);
        self.active = match self.active {
            _ if self.tabs.is_empty() => None,
            Some(a) if a > i || a == self.tabs.len() => Some(a - 1),
            a => a,
        };
    }

    fn toggle(&mut self, key: &str) {
        if !self.open.remove(key) {
            self.open.insert(key.to_owned());
        }
    }
}

/// The example's mark: a rounded tile with a small device on it.
fn mark(p: &Painter, r: Rect) {
    let k = r.width() / 18.0;
    p.rect_filled(r, 4.0 * k, Color32::from_rgb(0x24, 0x28, 0x37));
    let body = Rect::from_center_size(r.center() - Vec2::new(0.0, k), Vec2::new(11.0 * k, 7.0 * k));
    p.rect_filled(body, 1.5 * k, Color32::from_rgb(0x5F, 0xD0, 0x8A));
    p.circle_filled(
        body.left_center() + Vec2::new(2.5 * k, -k),
        0.9 * k,
        Color32::from_rgb(0x0B, 0x3B, 0x2A),
    );
    for dx in [-3.5, 0.0, 3.5] {
        p.circle_filled(
            r.center() + Vec2::new(dx * k, 5.0 * k),
            1.2 * k,
            Color32::from_rgb(0x5F, 0xD0, 0x8A),
        );
    }
}

impl eframe::App for Example {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        widgets::track_focus_visible(&ctx);
        let window = chrome::window_width(ui);
        egui::Panel::top("chrome-toolbar")
            .exact_size(theme::BAR)
            .show_separator_line(false)
            .frame(Frame::NONE)
            .show(ui, |ui| self.toolbar(ui, window));
        egui::Panel::bottom("chrome-status")
            .exact_size(theme::STATUS)
            .show_separator_line(false)
            .frame(Frame::NONE)
            .show(ui, |ui| self.status(ui, window));
        egui::Panel::left("chrome-stripe")
            .exact_size(chrome::STRIPE_W)
            .resizable(false)
            .show_separator_line(false)
            .frame(Frame::NONE)
            .show(ui, |ui| self.stripe(ui));
        if self.side_visible {
            let width = chrome::clamp_side_width(chrome::SIDE_DEFAULT, window);
            egui::Panel::left("chrome-side")
                .exact_size(width)
                .resizable(false)
                .show_separator_line(false)
                .frame(Frame::NONE)
                .show(ui, |ui| self.side_panel(ui));
        }
        let t = theme::theme(ui);
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(t.editor))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                if !self.tabs.is_empty() {
                    self.tab_row(ui);
                }
                match self.active_tab() {
                    Some(tab) => {
                        let title = format!(
                            "{} \u{b7} {}",
                            tab.name,
                            self.active_conn().map_or("Workspace", |c| c.name)
                        );
                        welcome::pane_empty(
                            ui,
                            tab.icon,
                            &title,
                            "A view would draw here. Close every tab for the welcome screen.",
                        );
                    }
                    None => self.welcome(ui),
                }
            });
    }
}

impl Example {
    fn toolbar(&mut self, ui: &mut Ui, window: f32) {
        let conn = self.active_conn();
        let up = self.conns.iter().filter(|c| c.up).count();
        let count = format!("{up} connected");
        let (lead, label, strong) = match (self.active_tab(), conn) {
            (Some(_), Some(c)) => (ConnLead::Chip(c.color, c.initials), c.name, true),
            (Some(_), None) => (ConnLead::Icon(icons::LAYOUT_GRID), "Workspace", true),
            (None, _) => (ConnLead::None, count.as_str(), false),
        };
        let crumbs: Vec<Crumb<'_>> = self
            .active_tab()
            .map(|t| {
                t.crumbs
                    .iter()
                    .map(|&(icon, label, mono)| Crumb { icon, label, mono })
                    .collect()
            })
            .unwrap_or_default();
        let tb = Toolbar {
            tint: conn.map(|c| c.color),
            mark: &mark,
            conn: ConnWidget {
                lead,
                label,
                strong,
            },
            crumbs: &crumbs,
            show_crumbs: window >= chrome::NARROW,
            search_hint: "Search devices, nodes, topics, tags",
            search_kbd: "Ctrl K",
        };
        let now = ui.input(|i| i.time);
        let mut toggle_theme = false;
        let mut record = false;
        let out = toolbar::toolbar(ui, &tb, |ui| {
            let settings = widgets::icon_button(ui, icons::SETTINGS, theme::CONTROL, "Settings");
            menu::dropdown(&settings, |ui| {
                let _ = menu::item(ui, Some(icons::INFO), "About", None);
            });
            toggle_theme = toolbar::theme_toggle(ui).clicked();
            record = toolbar::record_button(
                ui,
                self.recording.map(|start| now - start),
                "Record every value and every frame of this connection",
            )
            .clicked();
        });
        if toggle_theme {
            let next = if theme::theme(ui).dark {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            };
            ui.ctx().set_theme(next);
        }
        if record {
            self.recording = match self.recording {
                Some(_) => None,
                None => Some(now),
            };
        }
        let mut pick = None;
        menu::dropdown(&out.conn, |ui| {
            menu::head(ui, "Switch connection");
            for (i, c) in self.conns.iter().enumerate() {
                if menu::conn_item(ui, c.color, c.initials, c.name, SITES[c.site], c.up).clicked() {
                    pick = Some(i);
                }
            }
        });
        if let Some(i) = pick
            && let Some(t) = self.tabs.iter().position(|t| t.conn == Some(i))
        {
            self.active = Some(t);
        }
    }

    fn stripe(&mut self, ui: &mut Ui) {
        let tip = format!("Problems ({})", self.problems);
        let top: Vec<StripeItem<'_>> = SIDES
            .iter()
            .enumerate()
            .map(|(k, &(icon, label))| {
                let label = if k == 5 { tip.as_str() } else { label };
                let it = StripeItem::new(icon, label).pressed(self.side_visible && self.side == k);
                match k {
                    0 => it.shortcut("Ctrl B"),
                    5 => it.attention(self.problems > 0),
                    _ => it,
                }
            })
            .collect();
        let bottom = [StripeItem::new(
            icons::ARROW_LEFT_RIGHT,
            "Import / export connections",
        )];
        let out = stripe::stripe(ui, &top, &bottom);
        if let Some(k) = out.clicked {
            if self.side == k {
                self.side_visible = !self.side_visible;
            } else {
                self.side = k;
                self.side_visible = true;
            }
        }
    }

    fn side_panel(&mut self, ui: &mut Ui) {
        panel::background(ui);
        match self.side {
            0 => self.explorer(ui),
            1 => {
                panel::head(ui, "Browse", |_| {});
                panel::note(
                    ui,
                    &panel::Note {
                        text: "Treatment SCADA \u{b7} opc.tcp://scada.example:4840",
                        mono: false,
                        muted: true,
                        suffix: None,
                    },
                );
                panel::body(
                    ui,
                    &panel::Body::tree("browse")
                        .foot("Right-click a variable to add it to a subscription."),
                    |ui| {
                        for (d, open, label) in [
                            (0, Some(true), "Objects"),
                            (1, Some(true), "Intake"),
                            (2, Some(false), "P101"),
                            (2, None, "Level"),
                            (1, Some(false), "Filters"),
                        ] {
                            let icon = if open.is_some() {
                                icons::FOLDER
                            } else {
                                icons::VARIABLE
                            };
                            let _ = tree::node(
                                ui,
                                &Row {
                                    open,
                                    guides: true,
                                    ..Row::new(d, Lead::Icon(icon), label)
                                },
                            );
                        }
                    },
                );
            }
            2 => {
                panel::head(ui, "Project", |ui| {
                    let _ = widgets::icon_button(ui, icons::PLUS, theme::CONTROL, "New file");
                    let _ = widgets::icon_button(ui, icons::FILE_INPUT, theme::CONTROL, "Import");
                });
                panel::note(
                    ui,
                    &panel::Note {
                        text: "~/plant-floor",
                        mono: true,
                        muted: false,
                        suffix: Some("\u{b7} git main \u{b7} 2 changed"),
                    },
                );
                ui.add_space(4.0);
                panel::body(ui, &panel::Body::new("project"), |ui| {
                    for (file, color, count, sel) in [
                        ("plant-a.tags.yaml", identity::SLATE, "42", true),
                        (
                            "cold-room-controller.map.json",
                            identity::GREEN,
                            "18",
                            false,
                        ),
                        ("fast-trend.watch.yaml", identity::TEAL, "12", false),
                        ("line-a.topics.yaml", identity::VIOLET, "9", false),
                    ] {
                        let _ = tree::node(
                            ui,
                            &Row {
                                mono: true,
                                selected: sel,
                                after: &[After::Counted(color, count)],
                                ..Row::new(0, Lead::Icon(icons::FILE_BRACES), file)
                            },
                        );
                    }
                });
            }
            5 => {
                let n = self.problems.to_string();
                panel::head(ui, "Problems", |ui| panel::head_count(ui, &n, true));
                panel::body(ui, &panel::Body::new("problems"), |ui| {
                    let _ = tree::node(
                        ui,
                        &Row {
                            after: &[After::Hot(icons::CIRCLE_X, "no reply")],
                            ..Row::new(0, Lead::Dot(identity::GREEN), "Evaporator fans")
                        },
                    );
                    let _ = tree::node(
                        ui,
                        &Row {
                            after: &[After::Warm(icons::SHIELD_ALERT, "untrusted")],
                            ..Row::new(0, Lead::Dot(identity::AMBER), "Filter gallery PLC")
                        },
                    );
                });
            }
            k => {
                panel::head(ui, SIDES[k].1.split(':').next().unwrap_or_default(), |_| {});
                panel::empty(ui, "Nothing here yet. This panel is the app's to fill.");
            }
        }
    }

    fn explorer(&mut self, ui: &mut Ui) {
        let t = theme::theme(ui);
        panel::head(ui, "Explorer", |ui| {
            let _ =
                widgets::icon_button(ui, icons::PLUS, theme::CONTROL, "New connection (Ctrl N)");
            let _ = widgets::icon_button(ui, icons::RADAR, theme::CONTROL, "Discover");
            let _ = widgets::icon_button(ui, icons::FOLDER_PLUS, theme::CONTROL, "New site");
        });
        panel::search(ui, &mut self.filter, "Search devices, nodes, topics, tags");
        let filter = self.filter.trim().to_ascii_lowercase();
        let selected = self.active_tab().map(|t| t.name);
        let mut toggles: Vec<String> = Vec::new();
        let mut opens: Vec<&'static str> = Vec::new();
        panel::body(ui, &panel::Body::tree("explorer-tree"), |ui| {
            let mut shown = 0;
            if filter.is_empty() {
                let r = tree::node(
                    ui,
                    &Row {
                        selected: selected == Some("Tags"),
                        after: &[After::Mono("42")],
                        ..Row::new(0, Lead::Icon(icons::TAG), "Tags")
                    },
                );
                if r.row.clicked() {
                    opens.push("Tags");
                }
                let _ = tree::node(
                    ui,
                    &Row {
                        after: &[After::Text("paste hex")],
                        ..Row::new(0, Lead::Icon(icons::BINARY), "Decoder")
                    },
                );
            }
            for (si, site) in SITES.iter().enumerate() {
                let conns: Vec<(usize, &Conn)> = self
                    .conns
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| {
                        c.site == si
                            && (filter.is_empty() || c.name.to_ascii_lowercase().contains(&filter))
                    })
                    .collect();
                if conns.is_empty() {
                    continue;
                }
                shown += 1;
                let key = format!("s:{si}");
                let s_open = !filter.is_empty() || self.open.contains(&key);
                let count = conns.len().to_string();
                let r = tree::node(
                    ui,
                    &Row {
                        open: Some(s_open),
                        after: &[After::Text(&count)],
                        more: true,
                        highlight: &filter,
                        ..Row::new(
                            0,
                            Lead::Icon(if s_open {
                                icons::FOLDER_OPEN
                            } else {
                                icons::FOLDER
                            }),
                            site,
                        )
                    },
                );
                if r.row.clicked() {
                    toggles.push(key);
                }
                tree::row_menu(
                    &r,
                    || format!("site:{si}"),
                    |ui| {
                        menu::head(ui, site);
                        let _ =
                            menu::item(ui, Some(icons::PLUS), "New connection here\u{2026}", None);
                        let _ = menu::item(ui, Some(icons::PENCIL), "Rename site\u{2026}", None);
                    },
                );
                if !s_open {
                    continue;
                }
                for (ci, c) in conns {
                    let key = format!("c:{ci}");
                    let c_open = c.up && (!filter.is_empty() || self.open.contains(&key));
                    let after: &[After<'_>] = match (c.up, ci) {
                        (true, 0) => &[After::Icon(icons::LOCK, t.text_3), After::Live(t.ok)],
                        (true, 2) => &[
                            After::Icon(icons::TRIANGLE_ALERT, t.warn),
                            After::Live(t.ok),
                        ],
                        (true, _) => &[After::Live(t.ok)],
                        (false, 3) => &[After::Warm(icons::SHIELD_ALERT, "untrusted")],
                        (false, _) => &[After::Text("offline")],
                    };
                    let r = tree::node(
                        ui,
                        &Row {
                            open: c.up.then_some(c_open),
                            strong: true,
                            tag: Some(c.kind),
                            after,
                            more: true,
                            link: (ci == 3).then_some("Review"),
                            highlight: &filter,
                            ..Row::new(1, Lead::Chip(c.color, c.initials), c.name)
                        },
                    );
                    if r.row.clicked() && c.up {
                        toggles.push(key);
                    }
                    tree::row_menu(
                        &r,
                        || format!("conn:{ci}"),
                        |ui| {
                            menu::head(ui, c.name);
                            let _ = menu::item(
                                ui,
                                Some(if c.up { icons::UNPLUG } else { icons::PLUG }),
                                if c.up { "Disconnect" } else { "Connect" },
                                None,
                            );
                            menu::sep(ui);
                            let _ =
                                menu::danger(ui, Some(icons::TRASH_2), "Delete connection\u{2026}");
                        },
                    );
                    if !c_open {
                        continue;
                    }
                    let leaves: &[(Option<&str>, &str, &str, After<'_>)] = match ci {
                        0 => &[
                            (None, icons::ACTIVITY, "Bus", After::Mono("ttyUSB0")),
                            (None, icons::ARROW_LEFT_RIGHT, "Traffic", After::Text("")),
                            (
                                Some("10"),
                                icons::CPU,
                                "Cold room 1 controller",
                                After::Mono("12 ms"),
                            ),
                            (
                                Some("11"),
                                icons::CPU,
                                "Evaporator fans",
                                After::Hot(icons::CIRCLE_X, "no reply"),
                            ),
                        ],
                        1 => &[(
                            None,
                            icons::INBOX,
                            "plant/line-a/#",
                            After::Text("612 msgs"),
                        )],
                        _ => &[
                            (None, icons::LIST_TREE, "Address space", After::Text("4 ns")),
                            (
                                None,
                                icons::BELL,
                                "Alarms & events",
                                After::WarmText("2 unacked"),
                            ),
                            (Some("2"), icons::LIST, "Fast trend", After::Mono("250 ms")),
                        ],
                    };
                    for &(badge, icon, label, after) in leaves {
                        let after = [after];
                        let r = tree::node(
                            ui,
                            &Row {
                                badge,
                                selected: selected == Some(label),
                                after: if matches!(after[0], After::Text("")) {
                                    &[]
                                } else {
                                    &after
                                },
                                more: badge.is_some(),
                                ..Row::new(2, Lead::Icon(icon), label)
                            },
                        );
                        if r.row.clicked() {
                            opens.push(label);
                        }
                    }
                }
            }
            if !filter.is_empty() && shown == 0 {
                tree::nothing_matches(ui, self.filter.trim());
            }
        });
        for k in toggles {
            self.toggle(&k);
        }
        for name in opens {
            self.open_tab(name);
        }
    }

    fn tab_row(&mut self, ui: &mut Ui) {
        let slate = identity::SLATE;
        let tabs: Vec<Tab<'_>> = self
            .tabs
            .iter()
            .map(|t| {
                let (sub, color) = t
                    .conn
                    .and_then(|c| self.conns.get(c))
                    .map_or(("Workspace", slate), |c| (c.name, c.color));
                Tab::new(Id::new(t.name), t.icon, t.name, sub, color).dirty(t.dirty)
            })
            .collect();
        let out = TabRow::new(&tabs, self.active)
            .more(More::OnOverflow)
            .draggable(true)
            .show(ui);
        let mut close = out.close;
        for (i, t) in out.tabs.iter().enumerate() {
            menu::context(&t.row, |ui| {
                if menu::item(ui, Some(icons::X), "Close", Some("Ctrl W")).clicked() {
                    close = Some(i);
                }
            });
        }
        if let Some(i) = out.select {
            self.active = Some(i);
        }
        if let Some((from, to)) = out.moved {
            let t = self.tabs.remove(from);
            self.tabs.insert(to, t);
            self.active = Some(to);
        }
        if let Some(i) = close {
            self.close_tab(i);
        }
    }

    fn status(&mut self, ui: &mut Ui, window: f32) {
        let t = theme::theme(ui);
        let conn = self.active_conn();
        let up = self.conns.iter().filter(|c| c.up).count();
        let count = format!("{up} of {} connected", self.conns.len());
        let live = if self.polling {
            "Polling \u{b7} 1s"
        } else {
            "Paused"
        };
        let problems = format!("{} problems", self.problems);
        let beat = (ui.input(|i| i.time) / 5.0) as u64;
        let ring = statusbar::live_ring(ui, Id::new("chrome-live-beat"), self.polling, beat);
        let notations = ["Modicon 4xxxx", "Zero-based", "Hex"];
        let view = self.active_tab().map(|t| t.status);
        let mut segs = vec![match conn {
            Some(c) => Seg::identity(Some(c.color), c.name, Some(c.detail))
                .gives(Gives::DetailFirst)
                .shrink(Shrink { tier: 1, min: 0.0 }),
            None => Seg::identity(None, "Workspace", Some("plant-a.studio.yaml")),
        }];
        segs.push(Seg::button("count", &count).icon(icons::PLUG));
        segs.push(Seg::live(self.polling, ring, live));
        segs.push(
            Seg::button("problems", &problems)
                .icon(icons::TRIANGLE_ALERT)
                .icon_color(t.warn),
        );
        if conn.is_some_and(|c| c.kind == "ua") && window > chrome::NARROW {
            segs.push(
                Seg::text("Live plant: writes ask for the name")
                    .icon(icons::LOCK)
                    .icon_color(t.danger)
                    .color(theme::mix(t.danger, 0.70, t.text))
                    .tint(theme::alpha(t.danger, 0.11))
                    .extra_width(2.0),
            );
        }
        segs.push(Seg::spacer());
        if let Some(v) = view {
            segs.push(Seg::text(v).mono().shrink(Shrink { tier: 0, min: 64.0 }));
        }
        segs.push(Seg::button("notation", notations[self.notation]).icon(icons::HASH));
        let out = statusbar::statusbar(ui, &segs);
        let last = segs.len() - 1;
        if let Some(r) = &out[last]
            && let Some(n) =
                statusbar::pick_menu(r, "notation", "Address notation", &notations, self.notation)
        {
            self.notation = n;
        }
        if out[2].as_ref().is_some_and(|r| r.clicked()) {
            self.polling = !self.polling;
        }
        if out[3].as_ref().is_some_and(|r| r.clicked()) {
            self.side = 5;
            self.side_visible = true;
        }
        if let Some(r) = &out[0] {
            widgets::tip(r.clone(), "What the active tab belongs to");
        }
    }

    fn welcome(&mut self, ui: &mut Ui) {
        let start = [
            WelcomeRow::new(welcome::Lead::Icon(icons::PLUS), "New connection")
                .trail(Trail::Kbd("Ctrl N")),
            WelcomeRow::new(welcome::Lead::Icon(icons::FOLDER_OPEN), "Open a project")
                .trail(Trail::Kbd("Ctrl O")),
            WelcomeRow::new(
                welcome::Lead::Icon(icons::BINARY),
                "Decode a frame, chunk or packet",
            ),
            WelcomeRow::new(welcome::Lead::Icon(icons::SEARCH), "Search everything")
                .trail(Trail::Kbd("Ctrl K")),
            WelcomeRow::new(
                welcome::Lead::Icon(icons::FLASK_CONICAL),
                "Start the three simulators",
            ),
        ];
        let details: Vec<String> = self
            .conns
            .iter()
            .map(|c| format!("{} \u{b7} {} \u{b7} ", c.kind, SITES[c.site]))
            .collect();
        let all = format!("All {} connections", self.conns.len());
        let mut connect: Vec<WelcomeRow<'_>> = self
            .conns
            .iter()
            .zip(&details)
            .map(|(c, d)| {
                let trail = if c.site == 2 {
                    Trail::Spinner
                } else {
                    Trail::Detail {
                        text: d,
                        data: Some(c.address),
                        color: None,
                    }
                };
                WelcomeRow::new(welcome::Lead::Chip(c.color, c.initials), c.name)
                    .trail(trail)
                    .enabled(c.site != 2)
            })
            .collect();
        connect.push(
            WelcomeRow::new(welcome::Lead::Icon(icons::NETWORK), &all).trail(Trail::Kbd("Ctrl B")),
        );
        let w = Welcome {
            mark: &mark,
            title: "Studio",
            version: "Version 0.1.0 \u{b7} a chrome example with fake data",
            start: &start,
            connect: &connect,
            connect_empty: "No saved connections yet.",
            promises: [
                (
                    icons::ACTIVITY,
                    "The wire, visible: bus, session and broker",
                ),
                (icons::LOCK, "Live plants ask before any write or publish"),
                (
                    icons::GIT_COMMIT_HORIZONTAL,
                    "Projects are files you can keep in git",
                ),
            ],
        };
        let out = welcome::show(ui, &w);
        if out.start == Some(1) {
            self.open_tab("Tags");
        }
        if let Some(k) = out.connect {
            if k == self.conns.len() {
                self.side = 0;
                self.side_visible = true;
            } else if let Some(name) = all_tabs()
                .iter()
                .find(|t| t.conn == Some(k))
                .map(|t| t.name)
            {
                self.open_tab(name);
            }
        }
    }
}

fn main() -> eframe::Result {
    let size = std::env::var("STUDIO_UI_CHROME_SIZE")
        .ok()
        .and_then(|s| {
            let (w, h) = s.split_once('x')?;
            Some([w.parse().ok()?, h.parse().ok()?])
        })
        .unwrap_or([1440.0, 900.0]);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("studio-ui chrome")
            .with_inner_size(size),
        ..Default::default()
    };
    eframe::run_native(
        "studio-ui chrome",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            if std::env::var("STUDIO_UI_CHROME_THEME").as_deref() == Ok("light") {
                cc.egui_ctx.set_theme(egui::Theme::Light);
            }
            Ok(Box::new(Example::new()))
        }),
    )
}
