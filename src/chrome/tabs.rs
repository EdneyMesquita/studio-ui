//! The 36 px editor tab row (Rabbit `TabStrip`, `EditorTab`; the mockup's `.tabrow`): per tab an identity
//! dot, an icon, the name, the faint name of what it belongs to, an optional pane number and dirty dot, the
//! close button, and the identity-coloured underline on the selected tab. The tabs scroll sideways under a
//! hidden scroll bar; a chevron at the end lists every tab. An optional slot before the chevron holds the
//! app's own control (Mb's layout picker), and tabs may be dragged onto each other.
//!
//! ```no_run
//! # use studio_ui::chrome::tabs::{Tab, TabRow};
//! # use studio_ui::{egui, icons, theme::identity, widgets::menu};
//! # fn f(ui: &mut egui::Ui, active: &mut usize) {
//! let tabs = [
//!     Tab::new(egui::Id::new("bus"), icons::ACTIVITY, "Bus", "Cold rooms", identity::GREEN),
//!     Tab::new(egui::Id::new("tags"), icons::TAG, "Tags", "Workspace", identity::SLATE),
//! ];
//! let out = TabRow::new(&tabs, Some(*active)).show(ui);
//! if let Some(i) = out.select { *active = i; }
//! for (i, t) in out.tabs.iter().enumerate() {
//!     menu::context(&t.row, |ui| { let _ = menu::item(ui, None, "Close", Some("Ctrl W")); });
//! }
//! # }
//! ```
//!
//! # How the apps map onto it
//!
//! | Mb Studio `chrome/tabs.rs` | This module |
//! |---|---|
//! | `tab_where` (connection name and colour, or `Workspace`), `tab_label`, `tab_icon` | [`Tab::new`]`(Id::new(tab), icon, label, where, colour)` |
//! | the pane number of a tab shown in a split, `editor::{pane_number_width, paint_pane_number}` | [`Tab::pane`] with [`PaneMark`]; [`pane_number_width`], [`paint_pane_number`] |
//! | the tab drag (`TabDrag`, `editor::dragged`), the drop on another tab → `Action::MoveTab` | [`TabRow::draggable`], [`TabDrag`], [`dragged`], [`TabRowResponse::moved`] |
//! | the layout picker in a 37 px strip before the chevron | [`TabRow::trailing`]`(37.0, \|ui, rect\| layout_picker(ui, rect, ..))` |
//! | the chevron only while the tabs overflow | [`TabRow::more`]`(`[`More::OnOverflow`]`)` |
//! | the hover tooltip `name\nwhere · transport\nIn pane N` | [`TabRow::tip`] |
//! | the tab's context menu (Close, Close others, Open to the side…) | `menu::context(&out.tabs[i].row, ..)` |
//! | select / close (× or middle-click) | [`TabRowResponse::select`] / [`TabRowResponse::close`] |
//!
//! | OPC Studio `chrome/tabs.rs` | This module |
//! |---|---|
//! | `tab_where` (server or `Workspace`), the always-shown chevron, no drag, no panes | [`TabRow::new`] defaults |
//! | reveal on a change of the active *index* | `Tab::new(Id::new(i), ..)`: the key is the index, so the reveal fires exactly as before |
//! | the context menu (Close, Close others, Close all) | `menu::context(&out.tabs[i].row, ..)` |
//!
//! New and opt-in: [`Tab::dirty`], the mockup's unsaved-changes dot (text-2, or warn while pending).

use std::sync::Arc;

use egui::{
    Align2, Color32, Id, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, pos2, vec2,
};

use crate::icons;
use crate::theme::{self, Theme};
use crate::widgets::{self, menu};

/// The "All open tabs" chevron at the row's end (`w-8`).
pub const MORE_W: f32 = 32.0;
/// A tab's widest (`max-w-[300px]`).
pub const TAB_MAX: f32 = 300.0;
/// The dirty dot and its gap.
const DIRTY_W: f32 = 7.0 + 7.0;

/// Whether the tabs overflow their strip (Rabbit `TabStrip`: `scrollWidth > clientWidth + 1`).
pub fn overflows(content_w: f32, strip_w: f32) -> bool {
    content_w > strip_w + 1.0
}

/// A tab's fixed parts around its name: `pl-3`, dot, gaps, icon, the name of what it belongs to
/// (`where_w`), whatever sits between that and the close button with its gaps (`extra_w`: the pane number,
/// the dirty dot), the close button and `pr-1.5`.
pub fn tab_fixed(where_w: f32, extra_w: f32) -> f32 {
    12.0 + 8.0 + 7.0 + 14.0 + 7.0 + 7.0 + where_w + extra_w + 7.0 + 20.0 + 6.0
}

/// The room a tab's name gets: what is left of `max_w`, down to 0 (`min-w-0 truncate`); the name of what
/// it belongs to never gives way (`shrink-0`).
pub fn name_room(name_w: f32, where_w: f32, extra_w: f32, max_w: f32) -> f32 {
    name_w.min((max_w - tab_fixed(where_w, extra_w)).max(0.0))
}

/// A tab's width for a name `name_w` wide: its fixed parts and the name's room, at most `max_w`.
pub fn tab_width(name_w: f32, where_w: f32, extra_w: f32, max_w: f32) -> f32 {
    (tab_fixed(where_w, extra_w) + name_room(name_w, where_w, extra_w, max_w)).min(max_w)
}

/// The row split into the tabs' strip, the trailing slot (`trailing_w`) and the chevron (`more_w`, 0 when
/// it is hidden), right to left, all above the 1 px bottom rule.
pub fn row_layout(rect: Rect, trailing_w: f32, more_w: f32) -> (Rect, Rect, Rect) {
    let bottom = rect.bottom() - 1.0;
    let more = Rect::from_min_max(
        pos2(rect.right() - more_w, rect.top()),
        pos2(rect.right(), bottom),
    );
    let trailing = Rect::from_min_max(
        pos2(more.left() - trailing_w, rect.top()),
        pos2(more.left(), bottom),
    );
    let tabs = Rect::from_min_max(rect.min, pos2(trailing.left(), bottom));
    (tabs, trailing, more)
}

/// Which pane shows a tab, in a split editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaneMark {
    /// 0-based; painted as `index + 1`.
    pub index: usize,
    /// The focused pane: filled accent.
    pub focused: bool,
}

/// The mockup's unsaved-changes dot (`.tab .dirty`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dirty {
    /// Changed: text-2.
    Changed,
    /// Changed and not yet applied anywhere (`.dirty.pending`): warn.
    Pending,
}

/// One tab.
#[derive(Clone, Copy, Debug)]
pub struct Tab<'a> {
    /// Stable identity: a new key on the selected tab scrolls it into view once.
    pub key: Id,
    pub icon: &'a str,
    pub name: &'a str,
    /// What it belongs to (a connection, `Workspace`), faint after the name; never cut.
    pub sub: &'a str,
    /// The identity colour: the dot and the selected tab's underline.
    pub color: Color32,
    pub pane: Option<PaneMark>,
    pub dirty: Option<Dirty>,
}

impl<'a> Tab<'a> {
    pub fn new(key: Id, icon: &'a str, name: &'a str, sub: &'a str, color: Color32) -> Self {
        Self {
            key,
            icon,
            name,
            sub,
            color,
            pane: None,
            dirty: None,
        }
    }

    #[must_use]
    pub fn pane(mut self, pane: Option<PaneMark>) -> Self {
        self.pane = pane;
        self
    }

    #[must_use]
    pub fn dirty(mut self, dirty: Option<Dirty>) -> Self {
        self.dirty = dirty;
        self
    }
}

/// When the "All open tabs" chevron shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum More {
    Always,
    /// Only while the tabs overflow their strip (the trailing slot then sits flush right).
    OnOverflow,
}

/// A tab being dragged, the drag-and-drop payload (`egui::DragAndDrop`). A view that takes a dropped tab
/// (a split pane) reads it with [`dragged`] and takes it with `DragAndDrop::take_payload::<TabDrag>`.
#[derive(Clone, Debug)]
pub struct TabDrag {
    /// Index into the row's tabs.
    pub index: usize,
    /// Its name, for a drag ghost.
    pub label: String,
}

/// The tab being dragged, if any.
pub fn dragged(ctx: &egui::Context) -> Option<Arc<TabDrag>> {
    egui::DragAndDrop::payload::<TabDrag>(ctx)
}

/// A slot's painter: called with the row's `Ui` and the slot's rect.
type Slot<'a> = Box<dyn FnOnce(&mut Ui, Rect) + 'a>;

/// The tab row; configure, then [`TabRow::show`].
pub struct TabRow<'a> {
    tabs: &'a [Tab<'a>],
    selected: Option<usize>,
    more: More,
    trailing: Option<(f32, Slot<'a>)>,
    draggable: bool,
    max_w: f32,
    tip: Option<&'a dyn Fn(usize) -> String>,
}

impl std::fmt::Debug for TabRow<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TabRow")
            .field("tabs", &self.tabs)
            .field("selected", &self.selected)
            .field("more", &self.more)
            .field("trailing", &self.trailing.as_ref().map(|t| t.0))
            .field("draggable", &self.draggable)
            .field("max_w", &self.max_w)
            .finish_non_exhaustive()
    }
}

/// What happened on the row.
#[derive(Debug, Default)]
pub struct TabRowResponse {
    /// One per tab, in order.
    pub tabs: Vec<TabResp>,
    /// A tab was clicked.
    pub select: Option<usize>,
    /// A tab's × was clicked, or the tab middle-clicked.
    pub close: Option<usize>,
    /// A dragged tab was dropped on another: `(from, to)`.
    pub moved: Option<(usize, usize)>,
    /// The chevron's response, when it shows.
    pub more: Option<Response>,
}

/// One tab's responses: the tab (tooltip, context menu) and its close button.
#[derive(Debug)]
pub struct TabResp {
    pub row: Response,
    pub close: Response,
}

impl<'a> TabRow<'a> {
    /// `selected` is the tab whose underline shows (in a split, the focused pane's; it may be none).
    pub fn new(tabs: &'a [Tab<'a>], selected: Option<usize>) -> Self {
        Self {
            tabs,
            selected,
            more: More::Always,
            trailing: None,
            draggable: false,
            max_w: TAB_MAX,
            tip: None,
        }
    }

    #[must_use]
    pub fn more(mut self, more: More) -> Self {
        self.more = more;
        self
    }

    /// A `width` px slot between the tabs and the chevron, painted by `paint`.
    #[must_use]
    pub fn trailing(mut self, width: f32, paint: impl FnOnce(&mut Ui, Rect) + 'a) -> Self {
        self.trailing = Some((width, Box::new(paint)));
        self
    }

    /// Tabs can be dragged; one dropped on another reports [`TabRowResponse::moved`].
    #[must_use]
    pub fn draggable(mut self, yes: bool) -> Self {
        self.draggable = yes;
        self
    }

    /// A tab's widest (300 px, Rabbit's; the mockup's CSS has 280).
    #[must_use]
    pub fn max_width(mut self, w: f32) -> Self {
        self.max_w = w;
        self
    }

    /// The hover tooltip of tab `i`, built only while it is hovered. Default: `name\nwhere`.
    #[must_use]
    pub fn tip(mut self, tip: &'a dyn Fn(usize) -> String) -> Self {
        self.tip = Some(tip);
        self
    }

    pub fn show(self, ui: &mut Ui) -> TabRowResponse {
        let t = theme::theme(ui);
        let mut out = TabRowResponse::default();
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), theme::TAB_ROW),
            Sense::hover(),
        );
        widgets::hline(ui.painter(), rect.x_range(), rect.bottom() - 0.5, t.line);
        let over_id = ui.id().with("tabs-overflow");
        let overflowing = ui.data(|d| d.get_temp::<bool>(over_id)).unwrap_or(false);
        let more_w = match self.more {
            More::Always => MORE_W,
            More::OnOverflow if overflowing => MORE_W,
            More::OnOverflow => 0.0,
        };
        let trailing_w = self.trailing.as_ref().map_or(0.0, |s| s.0);
        let (tabs_rect, trailing_rect, more_rect) = row_layout(rect, trailing_w, more_w);
        // A newly selected tab is scrolled into view once (Rabbit `EditorTab`'s effect on `active`).
        let sel_id = ui.id().with("tabs-selected");
        let sel_key = self
            .selected
            .and_then(|i| self.tabs.get(i))
            .map(|tab| tab.key);
        let reveal = ui.data(|d| d.get_temp::<Option<Id>>(sel_id)) != Some(sel_key);
        if reveal {
            ui.data_mut(|d| d.insert_temp(sel_id, sel_key));
        }
        let drag = if self.draggable {
            dragged(ui.ctx())
        } else {
            None
        };
        let mut tui = ui.new_child(egui::UiBuilder::new().max_rect(tabs_rect));
        tui.set_clip_rect(tabs_rect);
        let scrolled = egui::ScrollArea::horizontal()
            .id_salt("tab-row")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            // A drag on a tab moves the tab, not the row.
            .scroll_source(egui::scroll_area::ScrollSource::MOUSE_WHEEL)
            .auto_shrink([false, false])
            .show(&mut tui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    for (i, tab) in self.tabs.iter().enumerate() {
                        let ctx = One {
                            selected: self.selected == Some(i),
                            reveal,
                            draggable: self.draggable,
                            drag: drag.as_deref(),
                            max_w: self.max_w,
                            tip: self.tip,
                        };
                        one_tab(ui, &ctx, i, tab, &mut out);
                    }
                });
            });
        if self.more == More::OnOverflow {
            let now_over = overflows(scrolled.content_size.x, scrolled.inner_rect.width());
            if now_over != overflowing {
                ui.data_mut(|d| d.insert_temp(over_id, now_over));
                ui.ctx().request_discard("tab row overflow changed");
            }
        }
        if let Some((_, paint)) = self.trailing {
            paint(ui, trailing_rect);
        }
        if more_w > 0.0 {
            let (resp, pick) = all_tabs(ui, more_rect, self.tabs, self.selected, t);
            if pick.is_some() {
                out.select = pick;
            }
            out.more = Some(resp);
        }
        out
    }
}

/// What every tab of the row needs to know.
struct One<'a> {
    selected: bool,
    /// The selected tab changed: scroll it into view.
    reveal: bool,
    draggable: bool,
    drag: Option<&'a TabDrag>,
    max_w: f32,
    tip: Option<&'a dyn Fn(usize) -> String>,
}

fn one_tab(ui: &mut Ui, c: &One<'_>, i: usize, tab: &Tab<'_>, out: &mut TabRowResponse) {
    let t = theme::theme(ui);
    let active = c.selected;
    let sg = widgets::galley(ui, tab.sub, theme::ui_font(theme::FS_SM), t.text_3);
    let pane_w = tab
        .pane
        .map_or(0.0, |p| pane_number_width(ui.painter(), p.index) + 7.0);
    let extra_w = pane_w + if tab.dirty.is_some() { DIRTY_W } else { 0.0 };
    let ng = widgets::galley(ui, tab.name, theme::body(), t.text);
    let room = name_room(ng.size().x, sg.size().x, extra_w, c.max_w);
    let ng = if ng.size().x > room {
        widgets::galley_truncated(ui, tab.name, theme::body(), t.text, room)
    } else {
        ng
    };
    let w = (tab_fixed(sg.size().x, extra_w) + room).min(c.max_w);
    let sense = if c.draggable {
        Sense::click_and_drag()
    } else {
        Sense::click()
    };
    let (r, resp) = ui.allocate_exact_size(Vec2::new(w, theme::TAB_ROW - 1.0), sense);
    if c.reveal && active {
        // just enough to show it, at once
        ui.scroll_to_rect_animation(r, None, egui::style::ScrollAnimation::none());
    }
    if c.draggable && resp.drag_started() {
        egui::DragAndDrop::set_payload(
            ui.ctx(),
            TabDrag {
                index: i,
                label: tab.name.to_owned(),
            },
        );
    }
    let hot = ui.rect_contains_pointer(r) && c.drag.is_none();
    if hot && !active {
        ui.painter().rect_filled(r, 0, t.row_hover);
    }
    // Another tab dragged over this one: it would take its place.
    if let Some(d) = c.drag.filter(|d| d.index != i)
        && ui.rect_contains_pointer(r)
    {
        ui.painter().rect_filled(
            Rect::from_min_size(r.min, vec2(2.0, r.height())),
            0,
            t.accent,
        );
        if ui.input(|i| i.pointer.any_released())
            && egui::DragAndDrop::take_payload::<TabDrag>(ui.ctx()).is_some()
        {
            out.moved = Some((d.index, i));
        }
    }
    if active {
        let u = Rect::from_min_max(
            pos2(r.left() + 6.0, r.bottom() - 2.0),
            pos2(r.right() - 6.0, r.bottom()),
        );
        ui.painter().rect_filled(
            u,
            egui::CornerRadius {
                nw: 2,
                ne: 2,
                sw: 0,
                se: 0,
            },
            tab.color,
        );
    }
    // Left to right like the flex row: a `where` too long for the tab runs past the tab's edge.
    let cy = r.center().y;
    let mut x = r.left() + 12.0;
    ui.painter().rect_filled(
        Rect::from_center_size(pos2(x + 4.0, cy), Vec2::splat(8.0)),
        2,
        tab.color,
    );
    x += 15.0;
    theme::paint_icon(
        ui.painter(),
        pos2(x + 7.0, cy),
        tab.icon,
        theme::ICON_SM,
        t.text_2,
    );
    x += 21.0;
    let fg = if active || hot { t.text } else { t.text_2 };
    ui.painter()
        .galley_with_override_text_color(pos2(x, cy - ng.size().y / 2.0), ng, fg);
    x += room + 7.0;
    let sw = sg.size().x;
    ui.painter()
        .galley(pos2(x, cy - sg.size().y / 2.0), sg, t.text_3);
    x += sw + 7.0;
    if let Some(p) = tab.pane {
        paint_pane_number(ui.painter(), pos2(x, cy), p.index, p.focused, t);
        x += pane_w;
    }
    if let Some(d) = tab.dirty {
        let c = match d {
            Dirty::Changed => t.text_2,
            Dirty::Pending => t.warn,
        };
        ui.painter().circle_filled(pos2(x + 3.5, cy), 3.5, c);
        x += DIRTY_W;
    }
    let xr = Rect::from_min_size(pos2(x, cy - 10.0), Vec2::splat(20.0));
    let xresp = ui.interact(xr, ui.id().with(("tab-x", i)), Sense::click());
    if active || hot || xresp.hovered() {
        if xresp.hovered() {
            ui.painter().rect_filled(xr, theme::R_SM, t.hover);
        }
        let col = if xresp.hovered() { t.text } else { t.text_3 };
        theme::paint_icon(ui.painter(), xr.center(), icons::X, theme::ICON_XS, col);
    }
    let xresp = widgets::tip(xresp, "Close (middle-click)");
    // `title`: built only while hovered
    let resp = if resp.hovered() {
        let tip = match c.tip {
            Some(f) => f(i),
            None => format!("{}\n{}", tab.name, tab.sub),
        };
        widgets::tip(resp, &tip)
    } else {
        resp
    };
    if xresp.clicked() || resp.middle_clicked() {
        out.close = Some(i);
    } else if resp.clicked() {
        out.select = Some(i);
    }
    out.tabs.push(TabResp {
        row: resp,
        close: xresp,
    });
}

/// The chevron and its menu of every tab (`name · where`, a check on the selected one), aligned to the
/// button's right edge. Returns its response and the tab picked.
fn all_tabs(
    ui: &mut Ui,
    more: Rect,
    tabs: &[Tab<'_>],
    selected: Option<usize>,
    t: &Theme,
) -> (Response, Option<usize>) {
    let mresp = ui.interact(more, ui.id().with("tabs-more"), Sense::click());
    widgets::vline(ui.painter(), more.left() + 0.5, more.y_range(), t.line_soft);
    let hot = mresp.hovered() || menu::is_open(&mresp);
    if hot {
        ui.painter()
            .rect_filled(more.shrink2(Vec2::new(0.5, 0.0)), 0, t.hover);
    }
    theme::paint_icon(
        ui.painter(),
        more.center(),
        icons::CHEVRON_DOWN,
        theme::ICON_SM,
        if hot { t.text } else { t.text_2 },
    );
    let mresp = widgets::tip(mresp, "All open tabs");
    let mut pick = None;
    egui::Popup::menu(&mresp)
        .gap(4.0)
        .align(egui::RectAlign::BOTTOM_END)
        .frame(menu::frame(t))
        .show(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            menu::head(ui, "Open tabs");
            for (i, tab) in tabs.iter().enumerate() {
                let icon = if selected == Some(i) {
                    icons::CHECK
                } else {
                    tab.icon
                };
                let label = format!("{} \u{b7} {}", tab.name, tab.sub);
                if menu::item(ui, Some(icon), &label, None).clicked() {
                    pick = Some(i);
                }
            }
        });
    (mresp, pick)
}

/// The width of a pane number's box.
pub fn pane_number_width(painter: &Painter, pane: usize) -> f32 {
    let g = painter.layout_no_wrap(
        (pane + 1).to_string(),
        theme::mono_medium(10.0),
        Color32::WHITE,
    );
    (g.size().x + 8.0).max(16.0)
}

/// Which pane shows a tab (Rabbit `PaneNumber`): its number, filled accent for the focused pane. Drawn with
/// its left edge at `left`, centered on `left.y`; returns its width.
pub fn paint_pane_number(
    painter: &Painter,
    left: Pos2,
    pane: usize,
    focused: bool,
    t: &Theme,
) -> f32 {
    let fg = if focused { t.on_accent } else { t.text_2 };
    let g = painter.layout_no_wrap((pane + 1).to_string(), theme::mono_medium(10.0), fg);
    let w = (g.size().x + 8.0).max(16.0);
    let r = Rect::from_min_size(pos2(left.x, (left.y - 8.0).round()), vec2(w, 16.0));
    if focused {
        painter.rect_filled(r, 3, t.accent);
    } else {
        painter.rect_stroke(
            r,
            3,
            Stroke::new(1.0, t.field_line),
            egui::StrokeKind::Inside,
        );
    }
    painter.galley(
        Align2::CENTER_CENTER.anchor_size(r.center(), g.size()).min,
        g,
        fg,
    );
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mb Studio's own tests, unchanged but for the explicit 300 px.
    #[test]
    fn the_chevron_waits_for_a_real_overflow() {
        assert!(!overflows(800.0, 800.0));
        // Rabbit's one pixel of slack
        assert!(!overflows(801.0, 800.0));
        assert!(overflows(801.5, 800.0));
    }

    #[test]
    fn the_name_gives_way_and_the_connection_does_not() {
        // short name and connection: the name keeps its width
        assert_eq!(name_room(60.0, 80.0, 0.0, TAB_MAX), 60.0);
        // a long name takes what is left of 300 px beside the connection name
        let room = name_room(400.0, 80.0, 0.0, TAB_MAX);
        assert_eq!(room, 300.0 - tab_fixed(80.0, 0.0));
        // a pane number takes its share too
        assert_eq!(name_room(400.0, 80.0, 23.0, TAB_MAX), room - 23.0);
        // a connection name too long for the tab leaves the name nothing (no floor)
        assert_eq!(name_room(100.0, 260.0, 0.0, TAB_MAX), 0.0);
    }

    #[test]
    fn tabs_are_as_wide_as_their_parts_up_to_the_cap() {
        // OPC Studio's fixed parts: no pane number
        assert_eq!(
            tab_fixed(80.0, 0.0),
            12.0 + 8.0 + 7.0 + 14.0 + 7.0 + 7.0 + 80.0 + 7.0 + 20.0 + 6.0
        );
        assert_eq!(
            tab_width(60.0, 80.0, 0.0, TAB_MAX),
            tab_fixed(80.0, 0.0) + 60.0
        );
        assert_eq!(tab_width(400.0, 80.0, 0.0, TAB_MAX), TAB_MAX);
        // a `where` wider than the cap: still capped (it runs past the edge, clipped)
        assert_eq!(tab_width(10.0, 400.0, 0.0, TAB_MAX), TAB_MAX);
        // the dirty dot is one more fixed part
        assert_eq!(tab_fixed(80.0, DIRTY_W) - tab_fixed(80.0, 0.0), 14.0);
    }

    #[test]
    fn the_row_splits_right_to_left_above_its_rule() {
        let rect = Rect::from_min_size(pos2(340.0, 40.0), Vec2::new(800.0, 36.0));
        // Mb: picker strip, chevron hidden
        let (tabs, picker, more) = row_layout(rect, 37.0, 0.0);
        assert_eq!(more.width(), 0.0);
        assert_eq!(picker.right(), rect.right());
        assert_eq!(picker.width(), 37.0);
        assert_eq!(tabs.right(), rect.right() - 37.0);
        assert_eq!(tabs.bottom(), rect.bottom() - 1.0);
        // OPC: chevron only
        let (tabs, slot, more) = row_layout(rect, 0.0, MORE_W);
        assert_eq!(more.left(), rect.right() - 32.0);
        assert_eq!(slot.width(), 0.0);
        assert_eq!(tabs.right(), more.left());
    }
}
