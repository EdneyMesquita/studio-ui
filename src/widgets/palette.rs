//! `.palette`: the Ctrl K command palette (Rabbit's `QuickOpenPalette` on cmdk): grouped items, every typed
//! word matched against the label and its context, the typed text highlighted in the label.

use egui::text::{LayoutJob, TextFormat};
use egui::{Align2, Color32, Frame, Key, Margin, Sense, Stroke, TextEdit, Vec2};

use crate::icons;
use crate::theme;

/// What changes from one app to the next: the input's hint, the groups listed before anything is typed,
/// and how many rows a group shows. Set it once on [`PaletteState::config`].
#[derive(Clone, Debug)]
pub struct PaletteConfig {
    /// Placeholder of the search field, naming what the app's palette reaches (`Nodes, servers, actions`).
    pub hint: String,
    /// Groups listed before anything is typed, in this order; every other group only answers a search.
    pub start_groups: Vec<&'static str>,
    /// Rows shown per group: the list stays short and cheap with thousands of items.
    pub shown_per_group: usize,
    /// Groups whose detail follows the label inline (commands) instead of sitting at the right (objects).
    pub inline_groups: Vec<&'static str>,
    /// Shown when nothing matches; `{}` is replaced by the query (`Nothing matches “{}” on any server.`).
    pub empty: String,
}

impl Default for PaletteConfig {
    fn default() -> Self {
        Self {
            hint: "Search actions and items".to_owned(),
            start_groups: vec!["Actions", "Connections"],
            shown_per_group: 40,
            inline_groups: vec!["Actions"],
            empty: "Nothing matches “{}”.".to_owned(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PaletteItem {
    pub group: &'static str,
    pub icon: &'static str,
    pub label: String,
    /// Faint context (`ns=2 · Line 2 · Bench`): after the label in Actions, at the right elsewhere.
    pub detail: String,
    pub kbd: Option<&'static str>,
    /// A connection chip in place of the icon (color, initials).
    pub chip: Option<(Color32, String)>,
    /// The connection's 8 px dot before the detail.
    pub dot: Option<Color32>,
    /// Label in JetBrains Mono (identifiers, file names).
    pub mono: bool,
    /// Listed only once something is typed, even in a group shown at start (the notation actions).
    pub typed_only: bool,
    /// Lowercased label, a space, lowercased detail: what a search matches.
    hay: String,
    /// Bytes of `hay` that are the label.
    label_len: usize,
}

impl PaletteItem {
    pub fn new(
        group: &'static str,
        icon: &'static str,
        label: String,
        detail: String,
        kbd: Option<&'static str>,
    ) -> Self {
        let mut hay = label.to_lowercase();
        let label_len = hay.len();
        hay.push(' ');
        hay.push_str(&detail.to_lowercase());
        Self {
            group,
            icon,
            label,
            detail,
            kbd,
            chip: None,
            dot: None,
            mono: false,
            typed_only: false,
            hay,
            label_len,
        }
    }
    pub fn chip(mut self, color: Color32, initials: &str) -> Self {
        self.chip = Some((color, initials.to_owned()));
        self
    }
    pub fn dot(mut self, color: Color32) -> Self {
        self.dot = Some(color);
        self
    }
    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }
    pub fn typed_only(mut self) -> Self {
        self.typed_only = true;
        self
    }
    fn label_low(&self) -> &str {
        &self.hay[..self.label_len]
    }
}

/// Palette state; keep one per app. Reuses its buffers, nothing grows per frame.
#[derive(Debug, Default)]
pub struct PaletteState {
    /// Per-app wording and limits; see [`PaletteConfig`].
    pub config: PaletteConfig,
    pub open: bool,
    pub query: String,
    pub selected: usize,
    /// The trimmed, lowercased query of this frame.
    q: String,
    /// (item index, score) of the current filter, in display order.
    hits: Vec<(usize, u8)>,
    /// Groups of the current filter with their best score, in display order.
    groups: Vec<(&'static str, u8)>,
    focus: bool,
}

impl PaletteState {
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.focus = true;
    }
    pub fn close(&mut self) {
        self.open = false;
    }
}

/// Rabbit's `matchScore`: 0 unless every typed word is in the haystack; then 3 when the label starts
/// with the query, 2 when it contains it, 1 when only the words match. `query` is trimmed and lowercased.
pub fn score(query: &str, label: &str, hay: &str) -> u8 {
    if query.is_empty() {
        return 3;
    }
    if !query.split_whitespace().all(|w| hay.contains(w)) {
        return 0;
    }
    if label.starts_with(query) {
        3
    } else if label.contains(query) {
        2
    } else {
        1
    }
}

/// Filter and order like cmdk: best score first inside a group, the group with the best item first;
/// ties keep the list's order. Before anything is typed only [`PaletteConfig::start_groups`] show.
fn filter(st: &mut PaletteState, items: &[PaletteItem]) {
    st.q.clear();
    st.q.push_str(&st.query.trim().to_lowercase());
    st.hits.clear();
    st.groups.clear();
    for (i, it) in items.iter().enumerate() {
        if st.q.is_empty() && (it.typed_only || !st.config.start_groups.contains(&it.group)) {
            continue;
        }
        let s = score(&st.q, it.label_low(), &it.hay);
        if s == 0 {
            continue;
        }
        st.hits.push((i, s));
        match st.groups.iter_mut().find(|g| g.0 == it.group) {
            Some(g) => g.1 = g.1.max(s),
            None => st.groups.push((it.group, s)),
        }
    }
    if !st.q.is_empty() {
        // stable: groups with the same best score keep their order
        st.groups.sort_by_key(|g| std::cmp::Reverse(g.1));
    }
    let groups = &st.groups;
    let rank = |i: usize| {
        items
            .get(i)
            .and_then(|it| groups.iter().position(|g| g.0 == it.group))
            .unwrap_or(usize::MAX)
    };
    st.hits
        .sort_by_key(|&(i, s)| (rank(i), std::cmp::Reverse(s), i));
    // at most `shown_per_group` per group
    let shown = st.config.shown_per_group;
    let mut kept = 0;
    let mut run = 0;
    let mut last = "";
    for k in 0..st.hits.len() {
        let g = items.get(st.hits[k].0).map_or("", |it| it.group);
        if g != last {
            last = g;
            run = 0;
        }
        run += 1;
        if run <= shown {
            st.hits[kept] = st.hits[k];
            kept += 1;
        }
    }
    st.hits.truncate(kept);
}

/// Show the palette if open. Returns the chosen item (Enter or click); the palette closes itself then,
/// and on Esc or a click outside.
pub fn palette(
    ctx: &egui::Context,
    state: &mut PaletteState,
    items: &[PaletteItem],
) -> Option<usize> {
    if !state.open && super::menu::dev_once(|| "palette".into()) {
        state.open();
    }
    if !state.open {
        return None;
    }
    let t = theme::Theme::of(ctx.theme());
    filter(state, items);
    let hint = state.config.hint.clone();
    // cmdk mounts afresh on each open: the list starts at its top (`focus` is set by `open`)
    let fresh = state.focus;
    let n = state.hits.len();
    // cmdk's keys (with `loop`): arrows wrap, Home and End jump to the ends. Of its vim bindings only
    // Ctrl+J gets here: Rabbit's global keys take Ctrl+K / P (toggle) and Ctrl+N (New connection) first.
    let (down, up, home, end, enter) = ctx.input_mut(|i| {
        let ctrl = egui::Modifiers::CTRL;
        let mut down = i.consume_key(egui::Modifiers::NONE, Key::ArrowDown);
        down |= i.consume_key(ctrl, Key::J);
        let up = i.consume_key(egui::Modifiers::NONE, Key::ArrowUp);
        (
            down,
            up,
            i.consume_key(egui::Modifiers::NONE, Key::Home),
            i.consume_key(egui::Modifiers::NONE, Key::End),
            i.consume_key(egui::Modifiers::NONE, Key::Enter),
        )
    });
    if n > 0 {
        if down {
            state.selected = (state.selected + 1) % n;
        }
        if up {
            state.selected = (state.selected + n - 1) % n;
        }
        if home {
            state.selected = 0;
        }
        if end {
            state.selected = n - 1;
        }
    }
    state.selected = state.selected.min(n.saturating_sub(1));
    let mut chosen = if enter {
        state.hits.get(state.selected).map(|h| h.0)
    } else {
        None
    };
    let scroll_sel = down || up || home || end;

    // Rabbit: `w-[min(640px,calc(100%-32px))]` (border included), 72 px from the top
    let width = 640.0f32.min(ctx.content_rect().width() - 32.0);
    let inner_w = width - 2.0;
    let area = egui::Modal::default_area(egui::Id::new("studio-ui-palette"))
        .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 72.0));
    let modal = egui::Modal::new(egui::Id::new("studio-ui-palette"))
        .area(area)
        // the palette is a Rabbit `Dialog`: same overlay as every dialog
        .backdrop_color(t.scrim)
        .frame(
            Frame::NONE
                .fill(t.raised)
                .stroke(Stroke::new(1.0, t.line))
                .corner_radius(theme::R_LG)
                .shadow(t.shadow),
        )
        .show(ctx, |ui| {
            ui.set_width(inner_w);
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            super::strip(ui, 46.0, None, Some(t.line), (14.0, 14.0), |ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                // `SearchIcon opacity-50`: the text color at half strength
                theme::icon(
                    ui,
                    icons::SEARCH,
                    theme::ICON,
                    theme::mix(t.text, 0.5, t.raised),
                );
                let resp = ui.add(
                    TextEdit::singleline(&mut state.query)
                        .frame(Frame::NONE)
                        .margin(Margin::ZERO)
                        .font(theme::ui_font(14.0))
                        .text_color(t.text)
                        .hint_text(
                            egui::RichText::new(hint.as_str())
                                .color(t.text_3)
                                .font(theme::ui_font(14.0)),
                        )
                        // `pr-10`: the Esc keycap sits in the input's right padding
                        .desired_width(ui.available_width() - 40.0)
                        .vertical_align(egui::Align::Center),
                );
                if resp.changed() {
                    state.selected = 0;
                }
                let eg = super::galley(ui, "Esc", theme::medium(10.5), t.text_2);
                let er =
                    super::kbd_rect_right(&eg, ui.max_rect().right(), ui.max_rect().center().y);
                super::paint_kbd(ui.painter(), er, eg, t.editor, t.line, t.text_2);
                if state.focus {
                    resp.request_focus();
                    state.focus = false;
                }
            });
            let mut list = egui::ScrollArea::vertical()
                .max_height(420.0)
                .auto_shrink([false, true]);
            if fresh {
                list = list.vertical_scroll_offset(0.0);
            }
            list.show(ui, |ui| {
                // `CommandList p-1.5`
                Frame::NONE.inner_margin(Margin::same(6)).show(ui, |ui| {
                    let w = inner_w - 12.0;
                    ui.set_width(w);
                    if state.hits.is_empty() {
                        // `CommandEmpty`: `py-6 text-center text-sm text-fg-3`
                        let font = theme::ui_font(theme::FS_SM);
                        let (r, _) = ui.allocate_exact_size(
                            Vec2::new(w, 48.0 + super::lh(theme::FS_SM)),
                            Sense::hover(),
                        );
                        ui.painter().text(
                            r.center(),
                            Align2::CENTER_CENTER,
                            state.config.empty.replace("{}", &state.query),
                            font,
                            t.text_3,
                        );
                    }
                    let mut last_group = "";
                    for (k, &(idx, _)) in state.hits.iter().enumerate() {
                        let Some(it) = items.get(idx) else {
                            continue;
                        };
                        if it.group != last_group {
                            // `CommandGroup p-1`: 4 px under the previous group, 4 over the head
                            ui.add_space(if last_group.is_empty() { 4.0 } else { 8.0 });
                            last_group = it.group;
                            // heading: `px-2 py-1.5 text-xs font-medium text-fg-3`
                            let (r, _) = ui.allocate_exact_size(
                                Vec2::new(w, 12.0 + super::lh(theme::FS_XS)),
                                Sense::hover(),
                            );
                            let g =
                                super::galley(ui, it.group, theme::medium(theme::FS_XS), t.text_3);
                            ui.painter().galley(
                                egui::pos2(
                                    r.left() + 12.0,
                                    (r.center().y - g.size().y / 2.0).round(),
                                ),
                                g,
                                t.text_3,
                            );
                        }
                        let (r, resp) = ui.allocate_exact_size(Vec2::new(w, 32.0), Sense::click());
                        if resp.hovered() && ui.input(|i| i.pointer.is_moving()) {
                            state.selected = k;
                        }
                        if resp.clicked() {
                            chosen = Some(idx);
                        }
                        let sel = k == state.selected;
                        if sel && scroll_sel {
                            // cmdk's `scrollIntoView({ block: "nearest" })`, at once, with the
                            // list's `scroll-py-1`
                            ui.scroll_to_rect_animation(
                                r.expand2(Vec2::new(0.0, 4.0)),
                                None,
                                egui::style::ScrollAnimation::none(),
                            );
                        }
                        if !ui.is_rect_visible(r) {
                            continue;
                        }
                        // the item: `mx-1` inside the group, `px-2 gap-2`
                        let r = r.shrink2(Vec2::new(4.0, 0.0));
                        if sel {
                            ui.painter().rect_filled(r, theme::R, t.sel);
                        }
                        row(
                            ui,
                            r,
                            it,
                            &state.q,
                            sel,
                            state.config.inline_groups.contains(&it.group),
                            t,
                        );
                    }
                    if !state.hits.is_empty() {
                        ui.add_space(4.0);
                    }
                });
            });
        });
    if modal.should_close() {
        state.close();
    }
    if chosen.is_some() {
        state.close();
    }
    chosen
}

/// One `CommandItem`: icon or chip, the label with the typed part marked, the context, the shortcut.
fn row(
    ui: &egui::Ui,
    r: egui::Rect,
    it: &PaletteItem,
    q: &str,
    sel: bool,
    inline: bool,
    t: &theme::Theme,
) {
    let p = ui.painter();
    let cy = r.center().y;
    let mut x = r.left() + 8.0;
    match &it.chip {
        Some((c, initials)) => super::chip::paint_chip(
            p,
            egui::Rect::from_center_size(egui::pos2(x + 7.0, cy), Vec2::splat(14.0)),
            *c,
            initials,
            super::chip::ChipSize::Sm,
        ),
        None => theme::paint_icon(p, egui::pos2(x + 8.0, cy), it.icon, theme::ICON, t.text_2),
    }
    x += if it.chip.is_some() { 14.0 } else { 16.0 } + 8.0;
    // the right end: shortcut (`ml-auto pl-4 text-xs`), else the context (`ml-auto pl-3 text-sm`)
    let mut right = r.right() - 8.0;
    if let Some(k) = it.kbd {
        let c = if sel { t.text_2 } else { t.text_3 };
        let kg = super::galley(ui, k, theme::ui_font(theme::FS_XS), c);
        right -= kg.size().x;
        p.galley(egui::pos2(right, cy - kg.size().y / 2.0), kg, c);
        right -= 16.0;
    }
    let detail_font = theme::ui_font(theme::FS_SM);
    let label_font = if it.mono {
        theme::data()
    } else {
        theme::body()
    };
    if !inline && !it.detail.is_empty() {
        let dg = super::galley(ui, it.detail.as_str(), detail_font.clone(), t.text_3);
        right -= dg.size().x;
        p.galley(egui::pos2(right, cy - dg.size().y / 2.0), dg, t.text_3);
        if let Some(c) = it.dot {
            right -= 6.0 + 8.0;
            p.rect_filled(
                egui::Rect::from_min_size(egui::pos2(right, cy - 4.0), Vec2::splat(8.0)),
                2,
                c,
            );
        }
        right -= 12.0;
    }
    // label (`min-w-0 truncate`), the first occurrence of the query in link blue, semibold
    let mut job = LayoutJob::default();
    let base = TextFormat::simple(label_font.clone(), t.text);
    let at = (!q.is_empty() && it.label_len == it.label.len())
        .then(|| it.label_low().find(q))
        .flatten();
    match at {
        Some(a) => {
            let b = a + q.len();
            let mark = TextFormat::simple(
                if it.mono {
                    theme::mono_medium(theme::FS_DATA)
                } else {
                    theme::semibold(theme::FS)
                },
                t.accent_text,
            );
            job.append(&it.label[..a], 0.0, base.clone());
            job.append(&it.label[a..b], 0.0, mark);
            job.append(&it.label[b..], 0.0, base);
        }
        None => job.append(&it.label, 0.0, base),
    }
    if inline && !it.detail.is_empty() {
        job.append(&it.detail, 8.0, TextFormat::simple(detail_font, t.text_3));
    }
    job.wrap = egui::text::TextWrapping::truncate_at_width((right - x).max(0.0));
    let g = p.layout_job(job);
    p.galley(egui::pos2(x, cy - g.size().y / 2.0), g, t.text);
}

#[cfg(test)]
mod tests {
    use super::{PaletteItem, PaletteState, filter, score};

    #[test]
    fn every_word_must_match_and_the_label_ranks_first() {
        assert_eq!(score("", "anything", "anything "), 3);
        assert_eq!(score("cold", "cold room", "cold room bench"), 3);
        assert_eq!(score("room", "cold room", "cold room bench"), 2);
        assert_eq!(score("cold bench", "cold room", "cold room bench"), 1);
        assert_eq!(
            score("crc", "cold room 1 controller", "cold room 1 controller "),
            0
        );
    }

    #[test]
    fn groups_follow_their_best_match_and_start_with_actions_and_connections() {
        let items = vec![
            PaletteItem::new(
                "Actions",
                "",
                "Record this connection".into(),
                String::new(),
                None,
            ),
            PaletteItem::new("Connections", "", "Bench".into(), "TCP".into(), None),
            PaletteItem::new(
                "Registers",
                "",
                "setpoint".into(),
                "unit 1 · Bench".into(),
                None,
            ),
        ];
        let mut st = PaletteState::default();
        filter(&mut st, &items);
        assert_eq!(st.hits.iter().map(|h| h.0).collect::<Vec<_>>(), [0, 1]);
        st.query = "Bench".into();
        filter(&mut st, &items);
        // the connection's label starts with it: its group comes first
        assert_eq!(st.hits.iter().map(|h| h.0).collect::<Vec<_>>(), [1, 2]);
    }
}
