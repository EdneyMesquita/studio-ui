//! Identity chips and dots, live dots and status pills.

use egui::{Align2, Color32, Rect, Response, Sense, Ui, Vec2};

use crate::icons;
use crate::theme::{self, alpha, identity};

/// Chip sizes of the family: 18 px with initials, 14 px small with initials, 8 px dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipSize {
    Md,
    Sm,
    Dot,
}

/// Connection chip: a rounded square in the connection color with its initials.
pub fn chip(ui: &mut Ui, color: Color32, initials: &str, size: ChipSize) -> Response {
    let side = match size {
        ChipSize::Md => 18.0,
        ChipSize::Sm => 14.0,
        ChipSize::Dot => 8.0,
    };
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(side), Sense::hover());
    paint_chip(ui.painter(), rect, color, initials, size);
    resp
}

pub fn paint_chip(
    painter: &egui::Painter,
    rect: Rect,
    color: Color32,
    initials: &str,
    size: ChipSize,
) {
    match size {
        ChipSize::Md => {
            painter.rect_filled(rect, 4, color);
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                initials,
                theme::semibold(9.0),
                identity::ON_CHIP,
            );
        }
        ChipSize::Sm => {
            painter.rect_filled(rect, 3, color);
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                initials,
                theme::semibold(7.5),
                identity::ON_CHIP,
            );
        }
        ChipSize::Dot => {
            painter.rect_filled(rect, 2, color);
        }
    }
}

/// A connection dot (8 px rounded square), the tab and status bar marker.
pub fn dot(ui: &mut Ui, color: Color32) -> Response {
    chip(ui, color, "", ChipSize::Dot)
}

pub fn paint_live_dot(painter: &egui::Painter, center: egui::Pos2, color: Color32) {
    painter.circle_filled(center, 5.0, alpha(color, 0.25));
    painter.circle_filled(center, 3.0, color);
}

/// Status of a value, a device or a check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ok,
    Warn,
    Crit,
    Idle,
}

impl Status {
    pub fn color(self, t: &theme::Theme) -> Color32 {
        match self {
            Status::Ok => t.ok,
            Status::Warn => t.warn,
            Status::Crit => t.danger,
            Status::Idle => t.text_3,
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Status::Ok => icons::CIRCLE_CHECK,
            Status::Warn => icons::TRIANGLE_ALERT,
            Status::Crit => icons::CIRCLE_X,
            Status::Idle => icons::CLOCK,
        }
    }
}

/// `.st` pill: a status icon in the status color and an optional word in the text color.
/// A state is always an icon plus a word, never color alone (pass `None` only where a tooltip says it).
pub fn status_pill(ui: &mut Ui, status: Status, word: Option<&str>) -> Response {
    let t = theme::theme(ui);
    let color = status.color(t);
    let g = word.map(|w| super::galley(ui, w, theme::ui_font(theme::FS_SM), t.text));
    let w = 12.0 + g.as_ref().map_or(0.0, |g| 5.0 + g.size().x);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 16.0), Sense::hover());
    let p = ui.painter();
    theme::paint_icon(
        p,
        egui::pos2(rect.left() + 6.0, rect.center().y),
        status.icon(),
        12.0,
        color,
    );
    if let Some(g) = g {
        let pos = egui::pos2(rect.left() + 17.0, rect.center().y - g.size().y / 2.0);
        p.galley(pos, g, t.text);
    }
    resp
}
