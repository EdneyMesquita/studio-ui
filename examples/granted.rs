//! The requested → granted list on two made-up negotiations, in both themes.
//!
//! `cargo run --example granted`. Left, an OPC UA CreateSubscription as the publishing inspector shows it: the
//! inline style, a revised publishing interval, a count granted as asked, one still pending. Right, an MQTT 5
//! CONNECT answered by a CONNACK in the columns style: a revised keep-alive, a session expiry the broker left
//! out, limits of the broker's own, a Maximum QoS it did not send (the default applies) and shared
//! subscriptions it does not offer.
//!
//! Environment, for captures: `STUDIO_UI_GRANTED_THEME=light` starts in the light theme.

use egui::{Frame, Ui, Vec2};

use studio_ui::icons;
use studio_ui::theme::{self, Theme};
use studio_ui::widgets::granted::{Grant, GrantList, State, Style};
use studio_ui::widgets::sec::sec_h;
use studio_ui::widgets::{self, SegItem, seg};

#[derive(Debug, Default)]
struct Demo {
    selected: Option<usize>,
}

fn opc(ui: &mut Ui) {
    sec_h(
        ui,
        "CreateSubscription",
        Some("requested \u{2192} revised"),
        true,
    );
    let rows = [
        Grant::new("PublishingInterval", State::Revised)
            .requested("100")
            .granted("250")
            .unit(" ms"),
        Grant::new("MaxKeepAliveCount", State::AsRequested)
            .requested("10")
            .granted("10"),
        Grant::new("LifetimeCount", State::Pending).requested("30"),
    ];
    GrantList::new("opc-contract", &rows).show(ui);
}

fn mqtt(ui: &mut Ui, selected: &mut Option<usize>) {
    sec_h(
        ui,
        "CONNECT \u{2192} CONNACK",
        Some("what this client asked, what the broker granted, what is in effect"),
        true,
    );
    let rows = [
        Grant::new("Protocol", State::AsRequested)
            .requested("MQTT 5.0 · level 5")
            .granted("accepted")
            .effect("5.0"),
        Grant::new("Clean Start", State::Revised)
            .requested("false")
            .granted("Session Present 1")
            .effect("resumed")
            .note("subscriptions and queued QoS 1 kept"),
        Grant::new("Session Expiry Interval", State::AsRequested)
            .requested("3600")
            .unit(" s"),
        Grant::new("Keep Alive", State::Revised)
            .requested("60")
            .granted("30")
            .unit(" s")
            .note("the client must use the server's value"),
        Grant::new("Receive Maximum", State::Granted)
            .requested("20")
            .granted("10")
            .effect("10 out · 20 in")
            .note("unacknowledged QoS 1 and 2 per direction"),
        Grant::new("Maximum QoS", State::Default)
            .requested("client accepts 2")
            .faint_requested()
            .granted("2"),
        Grant::new("Topic Alias Maximum", State::Granted)
            .requested("16")
            .granted("10")
            .effect("10 out · 16 in"),
        Grant::new("Shared Subscription", State::Unsupported)
            .effect("not available")
            .note("$share/ filters are refused (0x9E)"),
        Grant::new("Maximum Packet Size", State::Pending).requested("1 MB"),
    ];
    let out = GrantList::new("mq-connack", &rows)
        .style(Style::Columns)
        .heads([
            "property".into(),
            ("CONNECT", "this client").into(),
            ("CONNACK", "broker").into(),
            "in effect".into(),
            "note".into(),
        ])
        .selected(*selected)
        .show(ui);
    if let Some(i) = out.clicked {
        *selected = Some(i);
    }
}

impl eframe::App for Demo {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let t: &Theme = theme::theme(ui);
        egui::Panel::top("top-bar")
            .exact_size(theme::BAR)
            .show_separator_line(false)
            .frame(Frame::NONE.fill(t.panel))
            .show(ui, |ui| {
                widgets::strip(ui, theme::BAR, None, Some(t.line), (12.0, 12.0), |ui| {
                    widgets::text(ui, "studio-ui granted", theme::semibold(theme::FS), t.text);
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
                let all = ui.max_rect();
                let left = egui::Rect::from_min_size(all.min, Vec2::new(380.0, all.height()));
                let right =
                    egui::Rect::from_min_max(egui::pos2(left.right() + 1.0, all.top()), all.max);
                ui.painter().rect_filled(left, 0, t.panel);
                for (rect, k) in [(left, 0), (right, 1)] {
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                    );
                    child.spacing_mut().item_spacing = Vec2::ZERO;
                    if k == 0 {
                        opc(&mut child);
                    } else {
                        mqtt(&mut child, &mut self.selected);
                    }
                }
            });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("studio-ui granted")
            .with_inner_size([1400.0, 420.0]),
        ..Default::default()
    };
    eframe::run_native(
        "studio-ui granted",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            if std::env::var("STUDIO_UI_GRANTED_THEME").as_deref() == Ok("light") {
                cc.egui_ctx.set_theme(egui::Theme::Light);
            }
            Ok(Box::new(Demo::default()))
        }),
    )
}
