//! Rabbit's `SwitchField`: a 30×18 `Switch` (`mt-px`) and, 10 px after it, a medium title with a faint
//! description under it.

use egui::{Color32, Response, Sense, Ui, Vec2};

use crate::theme;

/// Toggle with title (medium 13 px) and description (faint 12 px, wraps), on Rabbit's 1.45 lines. Click
/// anywhere on it to flip.
pub fn switch(ui: &mut Ui, on: &mut bool, title: &str, desc: &str) -> Response {
    let t = theme::theme(ui);
    let width = ui.available_width();
    let text_w = (width - 40.0).max(60.0);
    let title_font = theme::medium(theme::FS);
    let (tg, tdy) = super::css_text(
        ui,
        egui::text::LayoutJob::simple(title.to_owned(), title_font, t.text, text_w),
        super::lh(theme::FS),
    );
    let dg = (!desc.is_empty()).then(|| {
        super::css_text(
            ui,
            egui::text::LayoutJob::simple(
                desc.to_owned(),
                theme::ui_font(theme::FS_SM),
                t.text_3,
                text_w,
            ),
            super::lh(theme::FS_SM),
        )
    });
    let th = tg.size().y;
    let h = th + dg.as_ref().map_or(0.0, |d| d.0.size().y);
    let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(width, h.max(19.0)), Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    // `transition-colors duration-150` / `transition-transform duration-150 ease-out-expo`
    let k = ui.ctx().animate_bool_with_time_and_easing(
        resp.id,
        *on,
        0.15,
        egui::emath::easing::exponential_out,
    );
    paint_track(ui.painter(), rect.min + Vec2::new(0.0, 1.0), k, t);
    if resp.has_focus() {
        super::focus_ring(
            ui.painter(),
            egui::Rect::from_min_size(rect.min + Vec2::new(0.0, 1.0), Vec2::new(30.0, 18.0)),
            9,
            t,
        );
    }
    let x = rect.left() + 40.0;
    ui.painter()
        .galley(egui::pos2(x, rect.top() + tdy), tg, t.text);
    if let Some((d, ddy)) = dg {
        ui.painter()
            .galley(egui::pos2(x, rect.top() + th + ddy), d, t.text_3);
    }
    resp
}

/// The 30×18 track at `min` with its 14 px thumb, `k` of the way on. Radix's thumb sits 4 px in when
/// off (`translate-x-0.5` past the 1 px border and padding) and 2 px from the right when on
/// (`translate-x-3`): it travels 10 px.
pub fn paint_track(painter: &egui::Painter, min: egui::Pos2, k: f32, t: &theme::Theme) {
    let track = egui::Rect::from_min_size(min, Vec2::new(30.0, 18.0));
    painter.rect_filled(track, 9, theme::mix(t.accent, k, t.field_line));
    let c = egui::pos2(track.left() + 11.0 + 10.0 * k, track.center().y);
    // `shadow-[0_1px_2px_rgba(0,0,0,0.3)]`
    painter.circle_filled(c + Vec2::new(0.0, 1.0), 7.5, Color32::from_black_alpha(40));
    painter.circle_filled(c, 7.0, Color32::WHITE);
}
