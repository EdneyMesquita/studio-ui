//! `.dialog.one`: a modal over a scrim (title with icon, body, footer buttons) and the typed-guard input.
//!
//! ```ignore
//! let r = Dialog::new("write", "Write the setpoint on Line 2?").icon(icons::SEND, t.danger).show(ctx, |ui| {
//!     dialog::field_row(ui, "Device", |ui| { .. });
//!     ok = dialog::guard(ui, conn.color, &conn.initials, &conn.name, "is a live plant", "setpoint", &mut buf).0;
//! }, |ui| {
//!     if ui.add(Btn::new("Write setpoint").danger().enabled(ok)).clicked() { .. }
//!     if ui.add(Btn::new("Cancel").ghost()).clicked() { .. }
//! });
//! if r.close { state.dialog = None; }
//! ```

use egui::{Color32, Frame, Id, Margin, Stroke, Ui, Vec2, WidgetText};

use crate::icons;
use crate::theme;

#[derive(Debug)]
pub struct Dialog<'a> {
    id: Id,
    title: WidgetText,
    icon: Option<(&'a str, Color32)>,
    width: f32,
    footer_start: bool,
    close_tip: &'a str,
}

#[derive(Debug)]
pub struct DialogResponse<R> {
    pub inner: R,
    /// Esc, scrim click or the close button: the caller should drop the dialog.
    pub close: bool,
}

impl<'a> Dialog<'a> {
    /// A dialog titled in Inter SemiBold 15.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug, title: &str) -> Self {
        Self::with_title(
            id_salt,
            egui::RichText::new(title).font(theme::semibold(15.0)),
        )
    }

    /// A dialog with a rich title (e.g. a `LayoutJob` mixing mono identifiers).
    pub fn with_title(
        id_salt: impl std::hash::Hash + std::fmt::Debug,
        title: impl Into<WidgetText>,
    ) -> Self {
        Self {
            id: Id::new(id_salt),
            title: title.into(),
            icon: None,
            // Rabbit's `Modal`: `max-w-md`
            width: 448.0,
            footer_start: false,
            close_tip: "Close",
        }
    }
    pub fn icon(mut self, glyph: &'a str, color: Color32) -> Self {
        self.icon = Some((glyph, color));
        self
    }
    /// Rabbit's `Modal danger`: the danger triangle before the title.
    pub fn danger(self, t: &theme::Theme) -> Self {
        self.icon(icons::TRIANGLE_ALERT, t.danger)
    }
    /// Footer buttons from the left, in the order added: Rabbit's task modals (`New virtual host`,
    /// `Delete queue`) put no spacer before them. Without it they sit at the right, the primary action
    /// added first (the `ConfirmDialog` way).
    pub fn footer_start(mut self) -> Self {
        self.footer_start = true;
        self
    }
    /// The close button's tooltip (`Close`), in the app's language.
    pub fn close_tip(mut self, tip: &'a str) -> Self {
        self.close_tip = tip;
        self
    }
    /// The whole width, border included: 448 (default, `max-w-md`), 460, 560, 640.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Footer buttons are laid out right to left: add the primary action first.
    pub fn show<R>(
        self,
        ctx: &egui::Context,
        body: impl FnOnce(&mut Ui) -> R,
        footer: impl FnOnce(&mut Ui),
    ) -> DialogResponse<R> {
        let t = theme::Theme::of(ctx.theme());
        let width = self.width.min(ctx.content_rect().width() - 32.0);
        let mut close_btn = false;
        let modal = egui::Modal::new(self.id)
            .backdrop_color(t.scrim)
            .frame(
                Frame::NONE
                    .fill(t.editor)
                    .stroke(Stroke::new(1.0, t.line))
                    .corner_radius(theme::R_LG)
                    .shadow(t.shadow),
            )
            .show(ctx, |ui| {
                // the frame's 1 px border is outside the content
                let width = width - 2.0;
                ui.set_width(width);
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                // head
                super::strip(ui, 48.0, None, Some(t.line), (18.0, 10.0), |ui| {
                    // `DialogTitle flex items-center gap-2`
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if let Some((icon, color)) = self.icon {
                        theme::icon(ui, icon, theme::ICON, color);
                    }
                    let title = self.title.clone();
                    ui.add(egui::Label::new(title).truncate());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if super::icon_button(ui, icons::X, 28.0, self.close_tip).clicked() {
                            close_btn = true;
                        }
                    });
                });
                let inner = Frame::NONE
                    // `DialogBody`: px-[22px] py-[18px]
                    .inner_margin(Margin::symmetric(22, 18))
                    .show(ui, |ui| {
                        ui.set_width(width - 44.0);
                        ui.spacing_mut().item_spacing = Vec2::new(8.0, 12.0);
                        body(ui)
                    })
                    .inner;
                // `DialogFooter`: `min-h-14 border-t pr-3.5 pl-[18px]`, edge to edge rule on top
                let (foot, _) =
                    ui.allocate_exact_size(Vec2::new(width, 56.0), egui::Sense::hover());
                super::hline(ui.painter(), foot.x_range(), foot.top() + 0.5, t.line);
                let frect = egui::Rect::from_min_max(
                    egui::pos2(foot.left() + 18.0, foot.top() + 1.0),
                    egui::pos2(foot.right() - 14.0, foot.bottom()),
                );
                let layout = if self.footer_start {
                    egui::Layout::left_to_right(egui::Align::Center)
                } else {
                    egui::Layout::right_to_left(egui::Align::Center)
                };
                let mut fui = ui.new_child(egui::UiBuilder::new().max_rect(frect).layout(layout));
                fui.spacing_mut().item_spacing.x = 8.0;
                footer(&mut fui);
                inner
            });
        let close = close_btn || modal.should_close();
        DialogResponse {
            inner: modal.inner,
            close,
        }
    }
}

/// A label/value row of a dialog summary (`Server`, `Node`, `Value`): label 84 px wide in text-2 12 px.
pub fn field_row(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui)) {
    let t = theme::theme(ui);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let (r, _) = ui.allocate_exact_size(Vec2::new(84.0, 20.0), egui::Sense::hover());
        let g = super::galley(ui, label, theme::ui_font(theme::FS_SM), t.text_2);
        ui.painter().galley(
            egui::pos2(r.left(), r.center().y - g.size().y / 2.0),
            g,
            t.text_2,
        );
        add(ui);
    });
}

/// The typed guard of a dangerous action: a danger-tinted box naming the target (identity chip and
/// name), why it is guarded (`is a live plant`, `is production`), and an input that must hold `word`
/// exactly. Returns whether it does, and the input's response (to focus it or submit on Enter).
pub fn guard(
    ui: &mut Ui,
    color: Color32,
    initials: &str,
    name: &str,
    reason: &str,
    word: &str,
    buf: &mut String,
) -> (bool, egui::Response) {
    use egui::text::{LayoutJob, TextFormat};
    let t = theme::theme(ui);
    let mut input = None;
    Frame::NONE
        .fill(theme::alpha(t.danger, 0.08))
        .corner_radius(theme::R)
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                super::chip::chip(ui, color, initials, super::chip::ChipSize::Sm);
                ui.add_space(6.0);
                let f = theme::ui_font(theme::FS_SM);
                let mut j = LayoutJob::default();
                j.append(
                    name,
                    0.0,
                    TextFormat::simple(theme::medium(theme::FS_SM), t.text),
                );
                j.append(
                    &format!(" {reason}. Type "),
                    0.0,
                    TextFormat::simple(f.clone(), t.text_2),
                );
                j.append(
                    word,
                    0.0,
                    TextFormat::simple(theme::mono_medium(theme::FS_DATA), t.text),
                );
                j.append(" to confirm.", 0.0, TextFormat::simple(f, t.text_2));
                ui.add(egui::Label::new(j).wrap());
            });
            input = Some(super::field::text_input(ui, buf, "", true, f32::INFINITY));
        });
    let ok = buf.trim() == word;
    let resp = input.unwrap_or_else(|| ui.label(""));
    (ok, resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_tip_defaults_to_english_and_can_be_replaced() {
        assert_eq!(Dialog::new("d", "t").close_tip, "Close");
        assert_eq!(
            Dialog::new("d", "t").close_tip("Fechar").close_tip,
            "Fechar"
        );
    }
}
