//! Menus: items for context menus and dropdowns (Rabbit's `ActionMenu` on shadcn `DropdownMenu` /
//! `ContextMenu`), the select options of `Select`, plus the popup helpers that give every menu the family
//! frame (popover bg, 1 px line border, 8 px radius, 4 px padding, overlay shadow).
//!
//! ```ignore
//! menu::context(&row_response, |ui| {
//!     if menu::item(ui, Some(icons::COPY), "Copy value", Some("Ctrl C")).clicked() { .. }
//!     menu::sep(ui);
//!     menu::danger(ui, Some(icons::TRASH_2), "Remove device");
//! });
//! ```

use egui::{Color32, Frame, Margin, Popup, Response, Sense, Stroke, Ui, Vec2};

use crate::icons;
use crate::theme::{self, Theme};

/// Padding inside a menu (`p-1`).
const PAD: f32 = 4.0;
/// Minimum width of a menu (`min-w-[220px]`, border included), content box.
const MIN_W: f32 = 220.0 - 2.0 * PAD - 2.0;
/// Height of a row (`h-7`).
const ROW: f32 = 28.0;

/// The frame of menus and select popups (`rounded-lg border border-line bg-popover p-1 shadow-overlay`).
pub fn frame(t: &Theme) -> Frame {
    Frame::NONE
        .fill(t.raised)
        .stroke(Stroke::new(1.0, t.line))
        .corner_radius(8)
        .shadow(t.shadow)
        .inner_margin(Margin::same(PAD as i8))
}

/// Width of a menu row: its own `natural` width while the popup measures itself (sizing pass), then the
/// popup's width so every row's highlight spans it.
fn fill(ui: &Ui, natural: f32) -> f32 {
    if ui.is_sizing_pass() {
        natural
    } else {
        ui.available_width().max(natural)
    }
}

#[allow(clippy::too_many_arguments)]
fn row(
    ui: &mut Ui,
    icon: Option<&str>,
    label: &str,
    kbd: Option<&str>,
    danger: bool,
    enabled: bool,
    check_right: Option<bool>,
) -> Response {
    let t = theme::theme(ui);
    let lg = super::galley(ui, label, theme::body(), t.text);
    let kg = kbd.map(|k| super::galley(ui, k, theme::ui_font(theme::FS_XS), t.text_3));
    let natural = 8.0
        // an action without an icon keeps its place (`<span className="w-3.5" />`); select options have none
        + if check_right.is_none() { 22.0 } else { 0.0 }
        + lg.size().x
        + kg.as_ref().map_or(0.0, |k| 16.0 + k.size().x)
        + if check_right.is_some() { 32.0 } else { 8.0 };
    // select popups are as wide as their trigger (`min-w-[8rem]`), menus at least 220 px
    let min_w = if check_right.is_some() {
        128.0 - 2.0 * PAD - 2.0
    } else {
        MIN_W
    };
    let w = fill(ui, natural.max(min_w));
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, theme::CONTROL), sense);
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let hot = enabled && highlighted(ui, &resp);
    let (fg, icon_c, kbd_c) = if hot {
        (Color32::WHITE, Color32::WHITE, Color32::WHITE)
    } else if danger {
        (t.danger, t.danger, t.text_3)
    } else {
        (t.text, t.text_2, t.text_3)
    };
    let mut p = ui.painter().clone();
    if !enabled {
        // `data-[disabled]:opacity-50`
        p.set_opacity(0.5);
    }
    if hot {
        p.rect_filled(rect, 5, if danger { t.danger } else { t.accent });
        if enabled {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }
    let cy = rect.center().y;
    let mut x = rect.left() + 8.0;
    if let Some(icon) = icon {
        theme::paint_icon(&p, egui::pos2(x + 7.0, cy), icon, theme::ICON_SM, icon_c);
    }
    if check_right.is_none() {
        x += 22.0;
    }
    p.galley_with_override_text_color(egui::pos2(x, cy - lg.size().y / 2.0), lg, fg);
    if let Some(k) = kg {
        let pos = egui::pos2(rect.right() - 8.0 - k.size().x, cy - k.size().y / 2.0);
        p.galley_with_override_text_color(pos, k, kbd_c);
    }
    if check_right == Some(true) {
        theme::paint_icon(
            &p,
            egui::pos2(rect.right() - 15.0, cy),
            icons::CHECK,
            theme::ICON,
            if hot { Color32::WHITE } else { t.text_2 },
        );
    }
    resp
}

/// Radix's one highlighted item: the pointer moves the focus to the item under it, and the focused item
/// is highlighted unless the pointer is on another one.
fn highlighted(ui: &Ui, resp: &Response) -> bool {
    if resp.hovered() && ui.input(|i| i.pointer.is_moving()) && !resp.has_focus() {
        resp.request_focus();
    }
    resp.hovered() || (resp.has_focus() && !ui.ui_contains_pointer())
}

/// A 28 px menu item; hover fills accent. `kbd` is the shortcut hint at the right (11 px, faint).
pub fn item(ui: &mut Ui, icon: Option<&str>, label: &str, kbd: Option<&str>) -> Response {
    row(ui, icon, label, kbd, false, true, None)
}

/// A disabled menu item (50 % opacity).
pub fn item_disabled(ui: &mut Ui, icon: Option<&str>, label: &str) -> Response {
    row(ui, icon, label, None, false, false, None)
}

/// A destructive item: danger text, danger fill on hover.
pub fn danger(ui: &mut Ui, icon: Option<&str>, label: &str) -> Response {
    row(ui, icon, label, None, true, true, None)
}

/// An option of a select popup (Rabbit `SelectItem`): 28 px, the chosen one checked at the right. The
/// chosen one takes the focus (and so the highlight) when the popup opens, as Radix does.
pub fn option(ui: &mut Ui, label: &str, selected: bool) -> Response {
    let resp = row(ui, None, label, None, false, true, Some(selected));
    if selected && ui.data_mut(|d| d.remove_temp::<bool>(focus_chosen_id()).is_some()) {
        resp.request_focus();
        resp.scroll_to_me(None);
    }
    resp
}

fn focus_chosen_id() -> egui::Id {
    egui::Id::new("studio-ui-select-focus-chosen")
}

/// A connection in a menu (Rabbit's connection switcher): 14 px chip, name, faint detail, and the ok dot
/// at the right when it is connected.
pub fn conn_item(
    ui: &mut Ui,
    color: Color32,
    initials: &str,
    label: &str,
    detail: &str,
    live: bool,
) -> Response {
    let t = theme::theme(ui);
    let lg = super::galley(ui, label, theme::body(), t.text);
    let dg = super::galley(ui, detail, theme::ui_font(theme::FS_XS), t.text_3);
    let natural = 8.0 + 22.0 + lg.size().x + 6.0 + dg.size().x + 24.0;
    let w = fill(ui, natural.max(288.0 - 2.0 * PAD - 2.0));
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, theme::CONTROL), Sense::click());
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let hot = highlighted(ui, &resp);
    let p = ui.painter();
    if hot {
        p.rect_filled(rect, 5, t.accent);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let cy = rect.center().y;
    let x = rect.left() + 8.0;
    super::chip::paint_chip(
        p,
        egui::Rect::from_center_size(egui::pos2(x + 7.0, cy), Vec2::splat(14.0)),
        color,
        initials,
        super::chip::ChipSize::Sm,
    );
    let lw = lg.size().x;
    let fg = if hot { Color32::WHITE } else { t.text };
    p.galley_with_override_text_color(egui::pos2(x + 22.0, cy - lg.size().y / 2.0), lg, fg);
    let dc = if hot { Color32::WHITE } else { t.text_3 };
    p.galley_with_override_text_color(
        egui::pos2(x + 22.0 + lw + 6.0, cy - dg.size().y / 2.0),
        dg,
        dc,
    );
    if live {
        p.circle_filled(egui::pos2(rect.right() - 11.0, cy), 3.0, t.ok);
    }
    resp
}

/// An open connection in a menu (Rabbit `OpenConnectionsItems`, `pr-1`): 14 px chip, name, a check on the
/// active one, and its own 20 px disconnect button at the right. Returns the row's response and the
/// button's (the button does not count as a click on the row).
pub fn open_conn_item(
    ui: &mut Ui,
    color: Color32,
    initials: &str,
    label: &str,
    active: bool,
) -> (Response, Response) {
    let t = theme::theme(ui);
    let lg = super::galley(ui, label, theme::body(), t.text);
    let tail = if active { 16.0 + 8.0 } else { 0.0 } + 20.0 + 4.0;
    let natural = 8.0 + 22.0 + lg.size().x + 8.0 + 8.0 + tail;
    let w = fill(ui, natural.max(MIN_W));
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, theme::CONTROL), Sense::click());
    let cy = rect.center().y;
    let btn =
        egui::Rect::from_center_size(egui::pos2(rect.right() - 4.0 - 10.0, cy), Vec2::splat(20.0));
    let bresp = ui.interact(btn, resp.id.with("unplug"), Sense::click());
    if !ui.is_rect_visible(rect) {
        return (resp, bresp);
    }
    let hot = highlighted(ui, &resp) || bresp.hovered();
    let p = ui.painter();
    if hot {
        p.rect_filled(rect, 5, t.accent);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let x = rect.left() + 8.0;
    super::chip::paint_chip(
        p,
        egui::Rect::from_center_size(egui::pos2(x + 7.0, cy), Vec2::splat(14.0)),
        color,
        initials,
        super::chip::ChipSize::Sm,
    );
    let fg = if hot { Color32::WHITE } else { t.text };
    // `truncate`: the name gives way to the check and the button
    let room = rect.right() - tail - 16.0 - (x + 22.0);
    let lg = if lg.size().x > room {
        super::galley_truncated(ui, label, theme::body(), t.text, room)
    } else {
        lg
    };
    p.galley_with_override_text_color(egui::pos2(x + 22.0, cy - lg.size().y / 2.0), lg, fg);
    if active {
        theme::paint_icon(
            p,
            egui::pos2(btn.left() - 8.0 - 8.0, cy),
            icons::CHECK,
            theme::ICON,
            if hot { Color32::WHITE } else { t.text_2 },
        );
    }
    // `text-fg-3 hover:bg-fg/10 hover:text-fg`, kept on the highlighted row too
    if bresp.hovered() {
        p.rect_filled(btn, theme::R_SM, theme::alpha(t.text, 0.1));
    }
    theme::paint_icon(
        p,
        btn.center(),
        icons::UNPLUG,
        theme::ICON_SM,
        if bresp.hovered() { t.text } else { t.text_3 },
    );
    (resp, bresp)
}

/// Separator between groups: a 1 px line edge to edge through the menu padding (`-mx-1 my-1`).
pub fn sep(ui: &mut Ui) {
    let t = theme::theme(ui);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(fill(ui, 0.0), 9.0), Sense::hover());
    super::hline(
        ui.painter(),
        rect.x_range().expand(PAD),
        rect.center().y,
        t.line,
    );
}

/// Small faint group head inside a menu (`px-2 pt-1.5 pb-1 text-xs text-fg-3`), truncated.
pub fn head(ui: &mut Ui, s: &str) {
    let t = theme::theme(ui);
    let w = fill(ui, MIN_W);
    let g = super::galley_truncated(ui, s, theme::ui_font(theme::FS_XS), t.text_3, w - 16.0);
    let line = super::lh(theme::FS_XS);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 6.0 + line + 4.0), Sense::hover());
    // the text on the middle of its 1.45 line, under the 6 px top padding
    ui.painter().galley(
        egui::pos2(
            rect.left() + 8.0,
            (rect.top() + 6.0 + line / 2.0 - g.size().y / 2.0).round(),
        ),
        g,
        t.text_3,
    );
}

fn styled<R>(popup: Popup<'_>, add: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    let t = theme::Theme::of(popup.ctx().theme());
    popup
        .frame(frame(t))
        .show(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            add(ui)
        })
        .map(|r| r.inner)
}

/// Attach a right-click menu to a response (closes on any click inside).
pub fn context(resp: &Response, add: impl FnOnce(&mut Ui)) {
    styled(Popup::context_menu(resp), add);
}

/// Attach a dropdown menu to a button response (toggles on click), 4 px under it (`sideOffset={4}`).
pub fn dropdown(resp: &Response, add: impl FnOnce(&mut Ui)) {
    styled(Popup::menu(resp).gap(4.0), add);
}

/// A select's popup (Rabbit `SelectContent`, Radix's `item-aligned` position): over its trigger, as wide
/// as it, the chosen option (`selected`, the index among the `rows` that `add` adds) on the trigger's
/// middle.
pub fn select_popup(
    resp: &Response,
    selected: Option<usize>,
    rows: usize,
    add: impl FnOnce(&mut Ui),
) {
    select_popup_scroll(resp, selected, rows, f32::INFINITY, add);
}

/// [`select_popup`] with a list of at most `max_h` points; a longer one scrolls, its first row on the
/// trigger and the chosen one scrolled into view.
pub fn select_popup_scroll(
    resp: &Response,
    selected: Option<usize>,
    rows: usize,
    max_h: f32,
    add: impl FnOnce(&mut Ui),
) {
    let was_open = is_open(resp);
    let scrolls = rows as f32 * ROW > max_h;
    let k = if scrolls {
        0.0
    } else {
        selected.unwrap_or(0) as f32
    };
    // the popup's top: its border and padding, then the rows above the chosen one
    let top = resp.rect.center().y - ROW / 2.0 - k * ROW - PAD - 1.0;
    let anchor = egui::Rect::from_min_size(egui::pos2(resp.rect.left() - 1.0, top), Vec2::ZERO);
    let inner_w = resp.rect.width() + 1.0 - 2.0 * PAD - 2.0;
    let popup = Popup::menu(resp)
        .anchor(anchor)
        .align(egui::RectAlign::BOTTOM_START)
        .align_alternatives(&[])
        .gap(0.0)
        .width(inner_w);
    if resp.clicked() && !was_open {
        resp.ctx
            .data_mut(|d| d.insert_temp(focus_chosen_id(), true));
    }
    styled(popup, |ui| {
        ui.set_min_width(inner_w);
        if scrolls {
            egui::ScrollArea::vertical()
                .max_height(max_h)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    add(ui)
                });
        } else {
            add(ui)
        }
    });
}

/// Whether the popup attached to `resp` with [`dropdown`] / [`select_popup`] is open.
pub fn is_open(resp: &Response) -> bool {
    Popup::is_id_open(&resp.ctx, Popup::default_response_id(resp))
}

/// Development hook for screenshots and end-to-end scripts, where synthetic input does not reach the
/// window: with the `dev-hooks` feature in a debug build, `STUDIO_UI_OPEN_MENU=<key>` is true once for the
/// caller that asks with that key. The crate asks with `palette` and `qselect:<label>`; apps choose their
/// own keys for their own triggers. Without the feature it is always false and costs nothing.
pub fn dev_once(key: impl FnOnce() -> String) -> bool {
    #[cfg(all(debug_assertions, feature = "dev-hooks"))]
    {
        use std::sync::LazyLock;
        use std::sync::atomic::{AtomicBool, Ordering};
        static WANT: LazyLock<Option<String>> =
            LazyLock::new(|| std::env::var("STUDIO_UI_OPEN_MENU").ok());
        static DONE: AtomicBool = AtomicBool::new(false);
        if let Some(want) = WANT.as_deref()
            && !DONE.load(Ordering::Relaxed)
            && want == key()
        {
            DONE.store(true, Ordering::Relaxed);
            return true;
        }
        false
    }
    #[cfg(not(all(debug_assertions, feature = "dev-hooks")))]
    {
        let _ = key;
        false
    }
}

/// [`dev_once`] for a menu trigger: opens its popup.
pub fn dev_open(resp: &Response, key: impl FnOnce() -> String) {
    if dev_once(key) {
        Popup::open_id(&resp.ctx, Popup::default_response_id(resp));
    }
}
