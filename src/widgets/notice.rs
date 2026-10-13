//! `.notice`: tinted strips at the top of a view (info / warn / crit), never cards.

use egui::text::{LayoutJob, TextFormat};
use egui::{Frame, Margin, Response, Shape, Ui};

use crate::icons;
use crate::theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Info,
    Warn,
    Crit,
}

impl Level {
    pub fn icon(self) -> &'static str {
        match self {
            Level::Info => icons::INFO,
            Level::Warn => icons::TRIANGLE_ALERT,
            Level::Crit => icons::CIRCLE_X,
        }
    }
}

/// Below this width the actions go under the text instead of beside it.
const NARROW: f32 = 480.0;

/// A notice: icon, **bold lead** in the text color, the rest in text-2 (wraps), actions on the right
/// (`Btn::new(..).small()`). Full width, bottom hairline.
pub fn notice(
    ui: &mut Ui,
    level: Level,
    bold: &str,
    rest: &str,
    actions: impl FnOnce(&mut Ui),
) -> Response {
    let t = theme::theme(ui);
    let (bg, ic) = match level {
        Level::Info => (None, t.accent_text),
        Level::Warn => (Some(theme::mix(t.warn, 0.09, t.editor)), t.warn),
        Level::Crit => (Some(theme::mix(t.danger, 0.09, t.editor)), t.danger),
    };
    let idx = ui.painter().add(Shape::Noop);
    let width = ui.available_width();
    let resp = Frame::NONE
        .inner_margin(Margin {
            left: 12,
            right: 8,
            top: 4,
            bottom: 4,
        })
        .show(ui, |ui| {
            ui.set_width(width - 20.0);
            ui.set_min_height(28.0);
            let text = |ui: &mut Ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                theme::icon(ui, level.icon(), theme::ICON_SM, ic);
                let mut job = LayoutJob::default();
                if !bold.is_empty() {
                    job.append(
                        bold,
                        0.0,
                        TextFormat::simple(theme::medium(theme::FS_SM), t.text),
                    );
                }
                job.append(
                    rest,
                    if bold.is_empty() { 0.0 } else { 4.0 },
                    TextFormat::simple(theme::ui_font(theme::FS_SM), t.text_2),
                );
                ui.add(egui::Label::new(job).wrap());
            };
            if width < NARROW {
                // a narrow pane: the actions go under the text, which keeps the full width
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    ui.horizontal_top(text);
                    let below = ui.available_rect_before_wrap();
                    let mut row = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(egui::Rect::from_min_size(
                                below.min + egui::vec2(22.0, 0.0),
                                egui::vec2(below.width() - 22.0, 28.0),
                            ))
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    row.spacing_mut().item_spacing.x = 6.0;
                    actions(&mut row);
                    let used = row.min_rect();
                    if used.width() > 0.0 {
                        ui.allocate_rect(used, egui::Sense::hover());
                    }
                });
            } else {
                egui::Sides::new()
                    .shrink_left()
                    .wrap()
                    .show(ui, text, |ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        actions(ui);
                    });
            }
        })
        .response;
    let rect = resp.rect;
    if let Some(bg) = bg {
        ui.painter().set(idx, Shape::rect_filled(rect, 0, bg));
    }
    super::hline(ui.painter(), rect.x_range(), rect.bottom() - 0.5, t.line);
    resp
}
