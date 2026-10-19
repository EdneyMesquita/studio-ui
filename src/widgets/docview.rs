//! A protocol-neutral document view: one file shown as a table, as its source and as a diff, a validation strip
//! under the toolbar, and Save / Revert on a working copy.
//!
//! The family's project files (a register map, a watch list, a topic map, the tags file) are versionable
//! documents that open in an editor tab. Each is drawn the same way: a toolbar with a segmented control of views,
//! the file's path, a dot while there are unsaved edits, the app's own buttons, Revert and Save; a strip of
//! checks (valid, resolves, warnings); then the current view. The application supplies the views, the path, the
//! checks, its table and the text and diff lines; the widget draws them and owns the seg, the strip and the
//! dirty state ([`WorkingCopy`]).
//!
//! ```ignore
//! let views = [
//!     SegItem::new(View::Table, "Table").icon(icons::TABLE),
//!     SegItem::new(View::Source, "JSON").icon(icons::FILE_BRACES),
//!     SegItem::new(View::Diff, "Diff").icon(icons::FILE_DIFF),
//! ];
//! let actions = [Action::new(icons::PLUS, "Add register").ghost()];
//! let out = Document::new(("map", &file), &mut st.view, &views)
//!     .path(&path).dirty(st.copy.dirty()).actions(&actions).checks(&checks)
//!     .show(ui);
//! if out.save { store.save(st.copy.work()); st.copy.saved(); }
//! if out.revert { st.copy.revert(); }
//! match st.view {
//!     View::Table => my_table(ui, st.copy.work()),
//!     View::Source => docview::source(ui, ("map-src", &file), &galley),
//!     View::Diff => { docview::diff(ui, ("map-diff", &file), &command, &lines); }
//! }
//! ```
//!
//! # Parts
//!
//! - [`Document`]: the toolbar and the strip. The toolbar fits its pane as Mb Studio's did ([`Fit`]): labels
//!   collapse to icons, then the seg collapses and the actions marked [`Action::overflow`] move into "⋯". The
//!   path ends with an ellipsis when it has to and shows in full on hover. Ctrl+S (⌘S) is Save while there
//!   are unsaved edits. [`BarStyle`] sets the strip's height, padding and the path's size.
//! - [`checks`] and [`check`]: the validation strip (`.map-val`): an icon and a few words each, wrapping.
//! - [`source`] (its own scroll area, both axes) and [`source_inline`] (a selectable label inside the page's
//!   scroll): a text coloured by [`Tok`] spans. [`source_job`] builds the job; [`yaml_spans`] colours YAML the
//!   way the watch list and the topic map do. JSON's spans stay the app's (Mb's `json_spans`).
//! - [`diff`]: the head (`git diff HEAD -- file`, `+N −M`) and 20 px lines on a green or red tint; context lines
//!   (new) on none. With no lines it draws nothing and returns `false`: the app's empty state goes there.
//! - [`WorkingCopy`]: the saved document and the edited one, a revision for caches, dirty, revert, saved and
//!   reload (the file changed on disk: taken when there are no edits).
//!
//! # How the family's documents map onto it
//!
//! **Mb Studio's register map** (`app/src/views/map.rs`). `State { work, base, rev, dirty }` is a
//! `WorkingCopy<Map>` (`edited()` → [`WorkingCopy::edit`], Revert → [`WorkingCopy::revert`], the store's new
//! `Arc<Map>` while clean → [`WorkingCopy::reload`]). `toolbar` is `Document::new(…, &mut st.view, &[Table,
//! JSON, Diff]).path(&tilde(path)).dirty(dirty).actions(&[Add register (ghost), Import CSV (ghost, overflow,
//! enabled while no file dialog is open), Export CSV (same)])` in the default [`BarStyle`] (`kit::tabbar2`):
//! the fit maths (`avail − 18`, the path's 80 px minimum, 22 px for the dot, 4 px between buttons, Save and
//! Revert counted with the actions) and the right-to-left order (⋯, Save, Revert, Export, Import, Add) are
//! the same; `out.action` gives the index of the action clicked, or of its menu entry. `validation` is
//! [`checks`] (`kit::check` is [`check`], `Check { status, info }` → [`Level`]). `json` is
//! [`source`] with `source_job(text, &spans, theme::data(), Some(20.0))` (the app keeps caching the galley by
//! revision and theme); `model::Tok` maps one to one onto [`Tok`] (`Space` → `Text`). `diff` is [`diff`] with
//! `DiffLine { sign, text }` → [`Line::add`] / [`Line::del`]; the empty pane stays `kit::pane_empty`.
//!
//! **OPC Studio's watch list** (`app/src/views/watch.rs`). Today it is one page: its 38 px strip is
//! `Document::new(…, &mut (), &[])` with no views and no Save / Revert (`.save_revert(false)`),
//! `BarStyle { height: 38.0, pad: (12.0, 8.0), path_size: FS_SM }`, and the actions `Open items` (default) and
//! `Subscribe on …` (primary). `checks` is [`checks`]; `yaml` is [`source_inline`] with
//! `source_job(text, &yaml_spans(text), theme::mono(FS_SM), None)`. The mockup's newer watch list (Items /
//! File / Resolve) is the same `Document` with three views; the Resolve table stays the app's.
//!
//! **IIoT Studio's MQTT topic map** (the mockup's `MQV.map`): views Table / YAML, the path with the branch
//! (`~/plant-floor/cold-storage.mqmap.yaml · git main`), `Import from births` (ghost), Save; the strip's ok,
//! warn, crit and info checks; the YAML view is [`source`] over [`yaml_spans`]. The table and the inspector are
//! the app's (rich grid and inspector).
//!
//! # Not matched exactly
//!
//! - OPC's checks strip had 14 px between checks, 12/8 px margins and drew every check's words in text 2; the
//!   kit's is Mb's (16 px, 12/7 px, a warning's or an error's words in the text colour), which is also the
//!   mockup's `.map-val`.
//! - Mb's diff had no context lines and its head counted `+` and `−` only; both still hold (a context line is
//!   new and opt-in).
//! - OPC's watch-list strip gave the path `avail − buttons − 16` px (at least 40) and no tooltip; the kit's
//!   [`path_room`] is Mb's (`avail − buttons − 30`, left out under 40) and the full path shows on hover.
//! - The toolbar's widget ids derive from `id_salt`; nothing persists them.

use std::hash::Hash;
use std::ops::Range;
use std::sync::Arc;

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, Galley, Id, Sense, Ui, Vec2};

use super::button::BtnKind;
use super::{Btn, Fit, SegItem};
use crate::icons;
use crate::theme::{self, Theme};

// ---------------------------------------------------------------------------------------------- working copy

/// The saved document and the one being edited.
#[derive(Clone, Debug, Default)]
pub struct WorkingCopy<T> {
    base: T,
    work: T,
    rev: u64,
    dirty: bool,
}

impl<T: Clone> WorkingCopy<T> {
    /// A copy of `saved`, with no edits.
    pub fn new(saved: T) -> Self {
        Self {
            work: saved.clone(),
            base: saved,
            rev: 1,
            dirty: false,
        }
    }
    /// The edited document.
    pub fn work(&self) -> &T {
        &self.work
    }
    /// The saved document.
    pub fn base(&self) -> &T {
        &self.base
    }
    /// The edited document, to change it: marks it dirty and bumps the revision.
    pub fn edit(&mut self) -> &mut T {
        self.rev += 1;
        self.dirty = true;
        &mut self.work
    }
    /// Back to the saved document.
    pub fn revert(&mut self) {
        self.work = self.base.clone();
        self.dirty = false;
        self.rev += 1;
    }
    /// The edited document was written: it is the saved one now.
    pub fn saved(&mut self) {
        self.base = self.work.clone();
        self.dirty = false;
    }
    /// The file changed on disk: the new version is taken when there are no edits (and `true` returned); with
    /// edits it waits, and the next Revert or Save decides.
    pub fn reload(&mut self, saved: T) -> bool {
        if self.dirty {
            return false;
        }
        self.work = saved.clone();
        self.base = saved;
        self.rev += 1;
        true
    }
    /// There are unsaved edits.
    pub fn dirty(&self) -> bool {
        self.dirty
    }
    /// Changes with every edit, revert and reload: the key of anything cached from the working copy.
    pub fn rev(&self) -> u64 {
        self.rev
    }
}

// ---------------------------------------------------------------------------------------------- checks

/// How a check reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Level {
    #[default]
    Ok,
    Info,
    Warn,
    Crit,
}

impl Level {
    pub fn icon(self) -> &'static str {
        match self {
            Level::Ok => icons::CIRCLE_CHECK,
            Level::Info => icons::INFO,
            Level::Warn => icons::TRIANGLE_ALERT,
            Level::Crit => icons::CIRCLE_X,
        }
    }
    pub fn color(self, t: &Theme) -> Color32 {
        match self {
            Level::Ok => t.ok,
            Level::Info => t.accent_text,
            Level::Warn => t.warn,
            Level::Crit => t.danger,
        }
    }
    /// The words' colour: a warning's and an error's in the text colour, the others in text 2.
    pub fn text_color(self, t: &Theme) -> Color32 {
        match self {
            Level::Warn | Level::Crit => t.text,
            Level::Ok | Level::Info => t.text_2,
        }
    }
}

/// One check of the validation strip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Check<'a> {
    pub level: Level,
    pub text: &'a str,
}

impl<'a> Check<'a> {
    pub fn new(level: Level, text: &'a str) -> Self {
        Self { level, text }
    }
}

/// One check: a 12 px icon and its words (12 px), never wider than the line it is on.
pub fn check(ui: &mut Ui, c: Check<'_>) -> egui::Response {
    let t = theme::theme(ui);
    let fg = c.level.text_color(t);
    let g = super::galley_truncated(
        ui,
        c.text,
        theme::ui_font(theme::FS_SM),
        fg,
        ui.max_rect().width() - 17.0,
    );
    let size = Vec2::new(17.0 + g.size().x, g.size().y.max(16.0));
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    theme::paint_icon(
        ui.painter(),
        egui::pos2(rect.left() + 6.0, rect.center().y),
        c.level.icon(),
        theme::ICON_XS,
        c.level.color(t),
    );
    ui.painter().galley(
        egui::pos2(rect.left() + 17.0, rect.center().y - g.size().y / 2.0),
        g,
        fg,
    );
    resp
}

/// The validation strip: the checks wrapping 16 px apart, 12/7 px margins, a rule under it. Nothing when
/// there are none.
pub fn checks(ui: &mut Ui, list: &[Check<'_>]) {
    if list.is_empty() {
        return;
    }
    let t = theme::theme(ui);
    let resp = egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(12, 7))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(16.0, 4.0);
                for c in list {
                    check(ui, *c);
                }
            });
        })
        .response;
    super::hline(
        ui.painter(),
        resp.rect.x_range(),
        resp.rect.bottom() - 0.5,
        t.line,
    );
}

// ---------------------------------------------------------------------------------------------- the toolbar

/// A button of the toolbar the application handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Action<'a> {
    pub icon: &'a str,
    pub label: &'a str,
    pub kind: BtnKind,
    pub enabled: bool,
    /// Moves into "⋯" when the toolbar is compact.
    pub overflow: bool,
}

impl<'a> Action<'a> {
    pub fn new(icon: &'a str, label: &'a str) -> Self {
        Self {
            icon,
            label,
            kind: BtnKind::Default,
            enabled: true,
            overflow: false,
        }
    }
    pub fn ghost(mut self) -> Self {
        self.kind = BtnKind::Ghost;
        self
    }
    pub fn primary(mut self) -> Self {
        self.kind = BtnKind::Primary;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn overflow(mut self) -> Self {
        self.overflow = true;
        self
    }
    fn btn(&self) -> Btn<'a> {
        Btn::new(self.label)
            .icon(self.icon)
            .kind(self.kind)
            .enabled(self.enabled)
    }
}

/// The toolbar strip's geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarStyle {
    pub height: f32,
    /// Left and right padding, gap between items.
    pub pad: (f32, f32),
    /// The path's mono size.
    pub path_size: f32,
}

impl BarStyle {
    /// `.tabbar2`: 36 px, 10 px in, 8 px apart, the path in mono 11.
    pub const DEFAULT: BarStyle = BarStyle {
        height: 36.0,
        pad: (10.0, 8.0),
        path_size: theme::FS_XS,
    };
}

impl Default for BarStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// What the toolbar's widths come to at each fit level, the room its right side takes, and the fit picked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarFit {
    pub fit: Fit,
    /// The width of the buttons at the right at this level.
    pub right: f32,
}

/// The toolbar's fit: `avail` is the pane's width less 18 px; `seg_w` the seg's full width; `widths` the full
/// widths of every button at the right (actions, then Revert and Save); `compact_n` how many buttons are left at
/// the compact level (the "⋯" counted).
pub fn bar_fit(avail: f32, seg_w: f32, widths: &[f32], compact_n: usize) -> BarFit {
    let fixed = seg_w + 12.0 + 80.0 + 22.0;
    let n = widths.len();
    let full = fixed + super::fit::row(widths, 4.0);
    let icons = fixed + icons_row(n);
    let fit = Fit::pick(avail, full, icons);
    let right = match fit {
        Fit::Full => super::fit::row(widths, 4.0),
        Fit::Icons => icons_row(n),
        Fit::Compact => icons_row(compact_n),
    };
    BarFit { fit, right }
}

fn icons_row(n: usize) -> f32 {
    if n == 0 {
        0.0
    } else {
        n as f32 * theme::CONTROL + 4.0 * (n - 1) as f32
    }
}

/// The room the path gets between the seg and the buttons: `None` under 40 px (it is left out).
pub fn path_room(avail: f32, right: f32) -> Option<f32> {
    let w = avail - right - 30.0;
    (w >= 40.0).then_some(w)
}

/// What the toolbar hands back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DocResponse {
    /// Save was clicked, or Ctrl+S pressed, while dirty.
    pub save: bool,
    pub revert: bool,
    /// The index of the action clicked (button or menu entry).
    pub action: Option<usize>,
    pub view_changed: bool,
}

/// The toolbar and the validation strip of a document.
pub struct Document<'a, V> {
    id: Id,
    view: &'a mut V,
    views: &'a [SegItem<'a, V>],
    path: &'a str,
    dirty: bool,
    actions: &'a [Action<'a>],
    save_revert: bool,
    checks: &'a [Check<'a>],
    bar: BarStyle,
}

impl<V> std::fmt::Debug for Document<'_, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Document")
            .field("id", &self.id)
            .field("path", &self.path)
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl<'a, V: PartialEq + Copy> Document<'a, V> {
    /// The document's toolbar: `view` is the current view, switched by the seg of `views` (none: no seg).
    pub fn new(
        id_salt: impl Hash + std::fmt::Debug,
        view: &'a mut V,
        views: &'a [SegItem<'a, V>],
    ) -> Self {
        Self {
            id: Id::new(id_salt),
            view,
            views,
            path: "",
            dirty: false,
            actions: &[],
            save_revert: true,
            checks: &[],
            bar: BarStyle::DEFAULT,
        }
    }
    /// The file's path, faint mono after the seg (and anything the app adds: `· git main`).
    pub fn path(mut self, path: &'a str) -> Self {
        self.path = path;
        self
    }
    /// There are unsaved edits: the dot after the path, Save and Revert enabled.
    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }
    /// The app's buttons, left to right, before Revert and Save.
    pub fn actions(mut self, actions: &'a [Action<'a>]) -> Self {
        self.actions = actions;
        self
    }
    /// Show Revert and Save (on by default; a read-only document has neither).
    pub fn save_revert(mut self, on: bool) -> Self {
        self.save_revert = on;
        self
    }
    /// The validation strip under the toolbar.
    pub fn checks(mut self, checks: &'a [Check<'a>]) -> Self {
        self.checks = checks;
        self
    }
    pub fn bar(mut self, bar: BarStyle) -> Self {
        self.bar = bar;
        self
    }

    pub fn show(self, ui: &mut Ui) -> DocResponse {
        let t = theme::theme(ui);
        let mut out = DocResponse::default();
        let save_btn = Btn::new("Save").icon(icons::SAVE).enabled(self.dirty);
        let revert_btn = Btn::new("Revert")
            .icon(icons::UNDO_2)
            .ghost()
            .enabled(self.dirty);
        let mut widths: Vec<f32> = self.actions.iter().map(|a| a.btn().width(ui)).collect();
        if self.save_revert {
            widths.push(revert_btn.width(ui));
            widths.push(save_btn.width(ui));
        }
        let any_overflow = self.actions.iter().any(|a| a.overflow);
        let compact_n = self.actions.iter().filter(|a| !a.overflow).count()
            + if self.save_revert { 2 } else { 0 }
            + usize::from(any_overflow);
        let seg_w = if self.views.is_empty() {
            0.0
        } else {
            super::seg_width(ui, self.views, false)
        };
        let fit = bar_fit(ui.available_width() - 18.0, seg_w, &widths, compact_n);
        let compact = fit.fit.compact();
        let icons_only = fit.fit.icons();
        let views = self.views;
        let actions = self.actions;
        let view = self.view;
        let (pad, gap) = self.bar.pad;
        super::strip(ui, self.bar.height, None, Some(t.line), (pad, gap), |ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if !views.is_empty() {
                out.view_changed = super::seg_fit(ui, view, views, compact).changed();
                ui.add_space(4.0);
            }
            if !self.path.is_empty()
                && let Some(w) = path_room(ui.available_width(), fit.right)
            {
                let g = super::galley_truncated(
                    ui,
                    self.path,
                    theme::mono(self.bar.path_size),
                    t.text_3,
                    w,
                );
                let (r, resp) = ui.allocate_exact_size(g.size(), Sense::hover());
                ui.painter().galley(r.min, g, t.text_3);
                super::tip(resp, self.path);
            }
            if self.dirty && self.save_revert {
                let (r, resp) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), Sense::hover());
                ui.painter().circle_filled(r.center(), 3.5, t.warn);
                super::tip(resp, "Unsaved edits");
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if compact && any_overflow {
                    let more = super::icon_button(ui, icons::ELLIPSIS, theme::CONTROL, "More");
                    super::menu::dropdown(&more, |ui| {
                        for (i, a) in actions.iter().enumerate().filter(|(_, a)| a.overflow) {
                            if a.enabled {
                                if super::menu::item(ui, Some(a.icon), a.label, None).clicked() {
                                    out.action = Some(i);
                                }
                            } else {
                                super::menu::item_disabled(ui, Some(a.icon), a.label);
                            }
                        }
                    });
                }
                if self.save_revert {
                    out.save = ui.add(save_btn.icon_only(icons_only)).clicked();
                    out.revert = ui.add(revert_btn.icon_only(icons_only)).clicked();
                }
                for (i, a) in actions.iter().enumerate().rev() {
                    if compact && a.overflow {
                        continue;
                    }
                    if ui.add(a.btn().icon_only(icons_only)).clicked() {
                        out.action = Some(i);
                    }
                }
            });
        });
        if self.save_revert {
            let key = ui.input_mut(|i| {
                i.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::S,
                ))
            });
            out.save = (out.save || key) && self.dirty;
            out.revert &= self.dirty;
        }
        checks(ui, self.checks);
        out
    }
}

// ---------------------------------------------------------------------------------------------- source

/// The colour of a run of source text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Tok {
    Key,
    Str,
    Num,
    Kw,
    Punct,
    Comment,
    #[default]
    Text,
}

impl Tok {
    pub fn color(self, t: &Theme) -> Color32 {
        match self {
            Tok::Key => t.syn_key,
            Tok::Str => t.syn_str,
            Tok::Num => t.syn_num,
            Tok::Kw => t.syn_kw,
            Tok::Punct => t.syn_punct,
            Tok::Comment => t.comment,
            Tok::Text => t.text,
        }
    }
}

/// `text` coloured by `spans` (byte ranges into it, in order; text between them is `Text`), in `font`, each line
/// `line_height` tall when given. Not wrapped.
pub fn source_job(
    text: &str,
    spans: &[(Range<usize>, Tok)],
    font: FontId,
    line_height: Option<f32>,
    t: &Theme,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let fmt = |tok: Tok| {
        let mut f = TextFormat::simple(font.clone(), tok.color(t));
        f.line_height = line_height;
        f
    };
    let mut at = 0;
    for (r, tok) in spans {
        let (start, end) = (r.start.min(text.len()), r.end.min(text.len()));
        if start < at || !text.is_char_boundary(start) || !text.is_char_boundary(end) {
            continue;
        }
        if start > at {
            job.append(&text[at..start], 0.0, fmt(Tok::Text));
        }
        job.append(&text[start..end], 0.0, fmt(*tok));
        at = end;
    }
    if at < text.len() {
        job.append(&text[at..], 0.0, fmt(Tok::Text));
    }
    job.wrap.max_width = f32::INFINITY;
    job
}

/// The source in a scroll area of its own (both axes), 12 px in and 8 px down (Mb's JSON view). Lay the job
/// out once per revision and theme and pass the galley.
pub fn source(ui: &mut Ui, id_salt: impl Hash + std::fmt::Debug, galley: &Arc<Galley>) {
    let t = theme::theme(ui);
    egui::ScrollArea::both()
        .id_salt(id_salt)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let size = galley.size() + egui::vec2(24.0, 24.0);
            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter()
                .galley(rect.min + egui::vec2(12.0, 8.0), galley.clone(), t.text);
        });
}

/// The source as a selectable label inside the page's own scroll, 12/8 px margins (OPC's watch list).
pub fn source_inline(ui: &mut Ui, job: LayoutJob) {
    egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.add(egui::Label::new(job).selectable(true).extend());
        });
}

/// YAML coloured line by line as the family's YAML files are: indentation plain, a `#` comment, the `- ` of a
/// list item in punctuation, a key and its `:` (key, punctuation), and the value after it: a number when it
/// starts with a digit, punctuation when it starts with `{`, a string otherwise; a line without a key is plain.
pub fn yaml_spans(text: &str) -> Vec<(Range<usize>, Tok)> {
    let mut out = Vec::new();
    let mut line_start = 0;
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        let indent = body.len() - body.trim_start().len();
        let rest = &body[indent..];
        let mut at = line_start + indent;
        let mut push = |len: usize, tok: Tok, at: &mut usize| {
            if len > 0 {
                out.push((*at..*at + len, tok));
                *at += len;
            }
        };
        if rest.starts_with('#') {
            push(rest.len(), Tok::Comment, &mut at);
        } else {
            let body = match rest.strip_prefix("- ") {
                Some(b) => {
                    push(2, Tok::Punct, &mut at);
                    b
                }
                None => rest,
            };
            let kv = body
                .split_once(": ")
                .or_else(|| body.strip_suffix(':').map(|k| (k, "")));
            match kv {
                Some((key, value)) => {
                    push(key.len(), Tok::Key, &mut at);
                    push(1, Tok::Punct, &mut at);
                    if !value.is_empty() {
                        let tok = if value.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                            Tok::Num
                        } else if value.starts_with('{') {
                            Tok::Punct
                        } else {
                            Tok::Str
                        };
                        // the space after the colon goes with the value, as OPC drew it
                        push(1 + value.len(), tok, &mut at);
                    }
                }
                None => push(body.len(), Tok::Text, &mut at),
            }
        }
        line_start += line.len();
    }
    out
}

// ---------------------------------------------------------------------------------------------- diff

/// A line of a diff.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sign {
    Add,
    Del,
    /// An unchanged line around the changes (new: Mb's diff has none).
    Context,
}

/// One line of a diff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line<'a> {
    pub sign: Sign,
    pub text: &'a str,
}

impl<'a> Line<'a> {
    pub fn add(text: &'a str) -> Self {
        Self {
            sign: Sign::Add,
            text,
        }
    }
    pub fn del(text: &'a str) -> Self {
        Self {
            sign: Sign::Del,
            text,
        }
    }
    pub fn context(text: &'a str) -> Self {
        Self {
            sign: Sign::Context,
            text,
        }
    }
}

/// Lines added and removed.
pub fn counts(lines: &[Line<'_>]) -> (usize, usize) {
    let adds = lines.iter().filter(|l| l.sign == Sign::Add).count();
    let dels = lines.iter().filter(|l| l.sign == Sign::Del).count();
    (adds, dels)
}

/// Height of a diff line.
pub const DIFF_ROW: f32 = 20.0;

/// The diff: a 34 px head with `command` (mono 12, text 2, ending with an ellipsis before the counts) and
/// `+N −M`, then the lines, 20 px each, on the ok or danger tint at 12 %, the sign at 14 px and the text at
/// 34 px. Only the lines in view are laid out. With no lines it draws nothing and returns `false`.
pub fn diff(
    ui: &mut Ui,
    id_salt: impl Hash + std::fmt::Debug,
    command: &str,
    lines: &[Line<'_>],
) -> bool {
    if lines.is_empty() {
        return false;
    }
    let t = theme::theme(ui);
    let (adds, dels) = counts(lines);
    super::strip(ui, 34.0, None, Some(t.line), (12.0, 10.0), |ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        // a narrow pane ends the command with an ellipsis, the counts stay
        let g = super::galley_truncated(
            ui,
            command,
            theme::mono(theme::FS_SM),
            t.text_2,
            (ui.available_width() - 80.0).max(0.0),
        );
        let (r, _) = ui.allocate_exact_size(g.size(), Sense::hover());
        ui.painter().galley(r.min, g, t.text_2);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            super::text(
                ui,
                format!("\u{2212}{dels}"),
                theme::mono(theme::FS_SM),
                t.danger,
            );
            super::text(ui, format!("+{adds}"), theme::mono(theme::FS_SM), t.ok);
        });
    });
    egui::ScrollArea::both()
        .id_salt(id_salt)
        .auto_shrink([false, false])
        .show_rows(ui, DIFF_ROW, lines.len(), |ui, range| {
            let w = ui.available_width();
            for i in range {
                let Some(l) = lines.get(i) else { continue };
                let (rect, _) = ui.allocate_exact_size(egui::vec2(w, DIFF_ROW), Sense::hover());
                let (sign, sc, fg) = match l.sign {
                    Sign::Add => ("+", t.ok, t.text),
                    Sign::Del => ("-", t.danger, t.text),
                    Sign::Context => (" ", t.text_3, t.text_2),
                };
                if l.sign != Sign::Context {
                    ui.painter().rect_filled(rect, 0, theme::alpha(sc, 0.12));
                }
                let cy = rect.center().y;
                let g = super::galley(ui, sign, theme::data(), sc);
                ui.painter()
                    .galley(egui::pos2(rect.left() + 14.0, cy - g.size().y / 2.0), g, sc);
                let g = super::galley(ui, l.text, theme::data(), fg);
                ui.painter()
                    .galley(egui::pos2(rect.left() + 34.0, cy - g.size().y / 2.0), g, fg);
            }
        });
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_working_copy_tracks_its_edits() {
        let mut w = WorkingCopy::new(vec![1, 2]);
        let r0 = w.rev();
        assert!(!w.dirty());
        w.edit().push(3);
        assert!(w.dirty());
        assert!(w.rev() > r0);
        assert_eq!(w.work(), &[1, 2, 3]);
        assert_eq!(w.base(), &[1, 2]);
        // a new version on disk waits while there are edits
        assert!(!w.reload(vec![9]));
        assert_eq!(w.work(), &[1, 2, 3]);
        let r1 = w.rev();
        w.revert();
        assert!(!w.dirty());
        assert_eq!(w.work(), &[1, 2]);
        assert!(w.rev() > r1);
        assert!(w.reload(vec![9]));
        assert_eq!((w.work(), w.base()), (&vec![9], &vec![9]));
        w.edit().push(4);
        w.saved();
        assert!(!w.dirty());
        assert_eq!(w.base(), &[9, 4]);
    }

    /// Mb's toolbar: Add, Import, Export, Revert, Save; Import and Export overflow, so four buttons stay
    /// (⋯, Save, Revert, Add) when compact.
    #[test]
    fn the_bar_fits_like_mbs() {
        let widths = [110.0, 100.0, 100.0, 80.0, 70.0];
        let full = 200.0 + 12.0 + 80.0 + 22.0 + 460.0 + 16.0;
        let f = bar_fit(full, 200.0, &widths, 4);
        assert_eq!(f.fit, Fit::Full);
        assert_eq!(f.right, 476.0);
        let f = bar_fit(full - 1.0, 200.0, &widths, 4);
        assert_eq!(f.fit, Fit::Icons);
        assert_eq!(f.right, 5.0 * theme::CONTROL + 16.0);
        let icons = 200.0 + 12.0 + 80.0 + 22.0 + 5.0 * theme::CONTROL + 16.0;
        let f = bar_fit(icons - 1.0, 200.0, &widths, 4);
        assert_eq!(f.fit, Fit::Compact);
        assert_eq!(f.right, 4.0 * theme::CONTROL + 12.0);
        // no buttons at all
        assert_eq!(bar_fit(500.0, 0.0, &[], 0).right, 0.0);
    }

    #[test]
    fn the_path_gives_way_first() {
        assert_eq!(path_room(300.0, 200.0), Some(70.0));
        assert_eq!(path_room(269.9, 200.0), None);
    }

    #[test]
    fn source_jobs_cover_the_text() {
        let t = &crate::theme::DARK;
        let text = "{ \"a\": 1 }";
        let job = source_job(
            text,
            &[(2..5, Tok::Key), (7..8, Tok::Num)],
            theme::data(),
            Some(20.0),
            t,
        );
        assert_eq!(job.text, text);
        let colors: Vec<Color32> = job.sections.iter().map(|s| s.format.color).collect();
        assert_eq!(colors, [t.text, t.syn_key, t.text, t.syn_num, t.text]);
        assert!(
            job.sections
                .iter()
                .all(|s| s.format.line_height == Some(20.0))
        );
        // overlapping or out-of-range spans are skipped, never a panic
        let job = source_job(
            "abc",
            &[(0..2, Tok::Key), (1..3, Tok::Num), (5..9, Tok::Str)],
            theme::data(),
            None,
            t,
        );
        assert_eq!(job.text, "abc");
    }

    /// OPC's `yaml`: comment, key and colon, a number, a string, a `{` value, a list item.
    #[test]
    fn yaml_like_the_watch_list() {
        let text = "# file\nversion: 1\nserver:\n  name: Plant\n  - node: plant:i=1001\nx: { a: 1 }\nplain\n";
        let spans = yaml_spans(text);
        let got: Vec<(&str, Tok)> = spans.iter().map(|(r, t)| (&text[r.clone()], *t)).collect();
        assert_eq!(
            got,
            [
                ("# file", Tok::Comment),
                ("version", Tok::Key),
                (":", Tok::Punct),
                (" 1", Tok::Num),
                ("server", Tok::Key),
                (":", Tok::Punct),
                ("name", Tok::Key),
                (":", Tok::Punct),
                (" Plant", Tok::Str),
                ("- ", Tok::Punct),
                ("node", Tok::Key),
                (":", Tok::Punct),
                (" plant:i=1001", Tok::Str),
                ("x", Tok::Key),
                (":", Tok::Punct),
                (" { a: 1 }", Tok::Punct),
                ("plain", Tok::Text),
            ]
        );
        // CRLF and no trailing newline
        let spans = yaml_spans("a: 1\r\nb: x");
        assert_eq!(spans.last().map(|s| s.0.clone()), Some(8..10));
    }

    #[test]
    fn diff_counts() {
        let lines = [
            Line::add("a"),
            Line::del("b"),
            Line::add("c"),
            Line::context("d"),
        ];
        assert_eq!(counts(&lines), (2, 1));
    }
}
