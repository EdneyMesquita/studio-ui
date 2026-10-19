//! A protocol-neutral hex field view: a table of decoded fields and a block of hex bytes, linked both ways.
//!
//! The widget takes plain data and knows no protocol. The bytes are a `&[u8]`; the fields are [`Field`]s, each
//! with a name, a value text, a nesting depth and the byte range it was decoded from (or none, for a *derived*
//! row such as "alias 3 → metric name"). [`FieldTable`] draws the fields, [`HexBlock`] draws the bytes coloured
//! by [`Class`], and the two are separate widgets so an application can place them where it likes (the table
//! above the bytes, the bytes in an inspector and the table in the editor, two blocks for a request and its
//! reply). Given the same `id_salt`, they share their hover: hovering a row tints its bytes, hovering a byte
//! marks the row of the innermost field that covers it, and a click on a byte hands that field back.
//!
//! ```ignore
//! let fields = [
//!     Field::new("Fixed header", "").bytes(0..2).group(),
//!     Field::new("Packet type", "3 PUBLISH").bytes(0..1).depth(1).class(Class::Header),
//!     Field::new("Remaining Length", "47").bytes(1..2).depth(1).class(Class::Header).varint(),
//!     Field::new("Topic Alias", "3 → plant/line-a/temp").derived().depth(1),
//! ];
//! let classes = hexview::classes_of(bytes.len(), &fields);
//! let out = FieldTable::new("pkt", bytes, &fields).selected(sel).max_height(Some(252.0)).show(ui);
//! hexview::legend(ui, &[("fixed header", Class::Header), ("packet id", Class::Id)]);
//! let hex = HexBlock::new("pkt", bytes).classes(&classes).fields(&fields)
//!     .selected(sel.and_then(|i| hexview::range_of(&fields, i, bytes.len())))
//!     .show(ui);
//! sel = out.clicked.or(hex.clicked_field).or(sel);
//! ```
//!
//! Fields come through the [`Fields`] trait, so a large decode need not be copied each frame: a slice of
//! `Vec`, an array or a slice (`&&[Field]`) of [`Field`] is one, and [`from_fn`] adapts an application's own list (`|i| Field::new(&f[i].name, &f[i].text)…`).
//! The table in [`TableStyle::Grid`] is virtualised (only the rows in view are laid out) and the block in
//! [`HexStyle::Lines`] lays out the lines in view only, so a message of 25 KB costs what one of 400 bytes does.
//! The widgets ask for a repaint only when the shared hover moves from one of them to the other. The geometry
//! ([`byte_in_line`], [`bytes_per_row`], [`line_of`], [`fit_height`], [`mini_columns`]) and the field maths
//! ([`classes_of`], [`innermost`], [`parent_of`], [`range_of`], [`bytes_cell`]) are pure and tested.
//!
//! # Byte classes and colours
//!
//! A class is a colour slot; the words of the legend are the application's ([`legend`] takes them).
//!
//! | [`Class`]  | colour                        | OPC UA (trace, decoder) | Modbus (frame)  | MQTT (packet)            |
//! |------------|-------------------------------|-------------------------|-----------------|--------------------------|
//! | `Plain`    | text 2                        | body                    | data words      | payload                  |
//! | `Header`   | syn special                   | header, security        | MBAP            | fixed header             |
//! | `Length`   | syn str                       | —                       | —               | (opt-in) length prefixes |
//! | `Id`       | syn key                       | sequence                | unit / slave    | packet id, topic alias   |
//! | `Kind`     | syn kw                        | TypeId                  | function code   | properties               |
//! | `Value`    | syn num                       | DateTime                | —               | topic                    |
//! | `Status`   | ok                            | StatusCode              | —               | reason code              |
//! | `Check`    | text 3, underlined per byte   | —                       | CRC             | —                        |
//!
//! A selected field's bytes, and the gaps between them, sit on the accent at 28 % in the text colour (OPC's
//! highlight). The hovered field's bytes sit on the accent at 14 % and keep their class colour. With
//! [`HexBlock::context`], the selected field's parent group is tinted at 8 % around it, so a property shows
//! inside its properties block. A [`Field::varint`] field's bytes are joined by a 1 px rule under them in their
//! class colour, so a variable-length integer reads as one span.
//!
//! # Styles
//!
//! | Style                                  | look                                                                 |
//! |----------------------------------------|----------------------------------------------------------------------|
//! | [`TableStyle::Grid`] (default)         | `off │ field │ value │ bytes` on [`RichGrid`]: 24 px rows, the offset right-aligned, depth × 12 px indent, a group's name semibold, the type faint after the name, the first 10 bytes then `…` |
//! | [`TableStyle::Mini`]                   | `bytes · field · value`, hand-painted, rows grow to fit the value, which wraps; the bytes in the field's class colour; a check glyph and a faint note after the value |
//! | [`HexStyle::Lines`] (default)          | 16 bytes a line, 20 px lines, mono 12, in a scroll area of at most `max_height`; only the lines in view are laid out |
//! | [`HexStyle::Wrap`]                     | the bytes wrap at the width, data font (mono 12.5), as tall as they need |
//!
//! # How the family's inspectors map onto it
//!
//! **OPC Studio's trace inspector** (`app/src/trace/inspector.rs`). `fields_table` is
//! `FieldTable::new(salt, &m.bytes, &fields).selected(field).max_height(Some(FIELDS_MAX_H)).show(ui)` with the
//! default [`TableStyle::Grid`]: `field_grid`'s columns (`off` 42 right, `field` 160 flex, `value` 98 flex,
//! `bytes` 78), fonts, indent and `bytes` cell are the grid's, and its height rule `(HEADER_H + n·ROW_H)` clamped
//! to `HEADER_H + 2·ROW_H ..= 252` is [`fit_height`]. `TableResponse::clicked` is `Out::field`. The codec's
//! `Field { name, depth, range, type_name, text, group, section }` maps one to one through [`from_fn`]:
//! `Field::new(&f.name, &f.text).bytes(f.range.clone()).depth(f.depth).ty(&f.type_name).group_if(f.group)`, with
//! `.class(…)` from `digest::classes`'s rule (Header and Security → `Header`, Sequence → `Id`, Type → `Kind`,
//! a body `DateTime` → `Value`, a body `StatusCode` → `Status`). The block keeps `Detail::classes`, mapped the
//! same way, through [`HexBlock::classes`] (or [`classes_of`] computes the same vector from the fields).
//! `hex_block` is `HexBlock::new(salt, &m.bytes).classes(&classes).fields(&fields).selected(range)
//! .scroll_to_selected(input.scroll_hex).style(HexStyle::Lines { max_height: HEX_MAX_H })`; the selected range is
//! [`range_of`] (`Detail::range`). `legend` is [`legend`] with `[("header", Header), ("sequence", Id),
//! ("TypeId", Kind), ("time", Value), ("status", Status)]`; `hex_text` is [`hex_text`]; `line_of` is
//! [`line_of`]` (r.start, PER_LINE)`. **The decoder** (`app/src/decoder/view.rs`) draws the same pieces: the
//! block with `max_height` = `(content height · 0.5).clamp(160, 520)`, the table with `max_height(None)` (it
//! fills what the inspector leaves). Its arrow keys stay in the app. Pass the same `id_salt` to the table and the
//! block so their hover is shared; give each a different one, or [`FieldTable::link`]`(false)`, to keep today's
//! behaviour exactly.
//!
//! **Mb Studio's frame inspector** (`app/src/views/traffic.rs`, `traffic/fields.rs`). `field_table` is
//! `FieldTable::new(salt, frame, &rows).style(TableStyle::Mini).show(ui)`; `hex_block` is
//! `HexBlock::new(salt, frame).classes(&classes).style(HexStyle::Wrap).show(ui)` with `classes[i]` from
//! `hex_class(i, len, tcp)` (Unit → `Id`, Function → `Kind`, Mbap → `Header`, Crc → `Check`). A `FieldRow`
//! becomes `Field::new(&r.key, &r.value).pre(&r.key_pre).bytes(range).class(…).tone(…)` plus
//! `.check(ok).note("matches computed, low byte first")` for a good CRC and `.check(false)` for a bad one; the
//! row's class colours its bytes as `field_table` did (Unit → `Id`, Mbap → `Header`, Crc → `Check`, and
//! Function and Plain → `Plain`, since the table showed a function code's byte in text 2). `FieldRow` keeps
//! the byte range instead of its `hex` string: `rows()` has it (`explain`'s `f.range`, `word_rows`' `span`).
//! `Tone::{Normal, Faint, Ok, Bad}` are [`Tone`]'s; a `Faint` row's name is drawn faint, as "not in map" was.
//!
//! **IIoT Studio's MQTT packet inspector** (the mockup's `fieldTable`, `hexBlock`, `pkInspector`). The field log
//! `{name, depth, s, e, type, text, sec, group}` is OPC's shape: [`TableStyle::Grid`] and [`HexStyle::Lines`],
//! `sec` → class (`hdr` → `Header`, `id` → `Id`, `prop` → `Kind`, `topic` → `Value`, `rc` → `Status`, the rest
//! `Plain`). Remaining Length and Property Length are `.varint()`; a property is nested in its properties
//! group (`.depth(d)`, with [`HexBlock::context`]); a resolved Topic Alias or a Sparkplug alias is a
//! `.derived()` row. The legend is `[("fixed header", Header), ("packet id · alias", Id), ("topic", Value),
//! ("properties", Kind), ("reason", Status)]`.
//!
//! # Not matched exactly
//!
//! - OPC's hex lines were `Sense::hover`; with a `fields` source they sense clicks (no visible change). The
//!   scroll area and grid ids derive from `id_salt` (`.with("hex")`, `.with("table")`), not the app's salts.
//! - Mb's hex block was one wrapped galley; the block lays out its rows itself at [`bytes_per_row`] bytes a
//!   row, which is where egui broke the galley (a test checks it against egui's own wrapping).
//! - Mb's rows sensed hover only; they sense clicks now, and a selected row is drawn on the selection tint.
//! - New, in none of the three today: the shared hover between table and block, a click on a byte, derived rows
//!   (no offset, `derived` in the bytes column), [`Field::varint`]'s rule, [`HexBlock::context`], the `Length`
//!   class (syn str), [`Tone`] colours in the grid style (OPC drew every value in the text colour, a group's in
//!   text 3; [`Tone::Normal`] keeps that).

use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Range;

use egui::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Color32, FontId, Id, Rect, ScrollArea, Sense, Stroke, Ui, Vec2};

use super::grid::{Column, HEADER_H, ROW_H};
use super::rich_grid::RichGrid;
use crate::icons;
use crate::theme::{self, Theme};

/// Bytes per line of [`HexStyle::Lines`].
pub const PER_LINE: usize = 16;
/// Line height of [`HexStyle::Lines`].
pub const LINE_H: f32 = 20.0;
/// Bytes shown in the grid table's `bytes` cell before `…`.
pub const CELL_BYTES: usize = 10;
/// The `bytes` cell of a derived row.
pub const DERIVED: &str = "derived";

const PAD_X: f32 = 12.0;
const SEL_TINT: f32 = 0.28;
const HOT_TINT: f32 = 0.14;
const CONTEXT_TINT: f32 = 0.08;

/// The colour slot of a byte. See the module docs for the colours and what each protocol puts in them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Class {
    #[default]
    Plain,
    Header,
    Length,
    Id,
    Kind,
    Value,
    Status,
    Check,
}

impl Class {
    /// The text colour of a byte of this class.
    pub fn color(self, t: &Theme) -> Color32 {
        match self {
            Class::Plain => t.text_2,
            Class::Header => t.syn_special,
            Class::Length => t.syn_str,
            Class::Id => t.syn_key,
            Class::Kind => t.syn_kw,
            Class::Value => t.syn_num,
            Class::Status => t.ok,
            Class::Check => t.text_3,
        }
    }
}

/// How a field's value reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Tone {
    /// Grid: text (a group's text 3). Mini: syn num.
    #[default]
    Normal,
    /// Grid: text 3. Mini: text 2, and the name in text 3.
    Faint,
    Ok,
    Bad,
}

/// One decoded field. Borrowed text: build these from the application's own rows each frame, or adapt the rows
/// with [`from_fn`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Field<'a> {
    pub name: &'a str,
    /// Faint text before the name (a register address).
    pub pre: &'a str,
    /// Faint type after the name (`UInt32`, `Two Byte Integer`).
    pub ty: &'a str,
    pub value: &'a str,
    /// Faint text after the value (and after the check glyph).
    pub note: &'a str,
    /// Nesting level, 0 at the top.
    pub depth: usize,
    /// The bytes the field was decoded from, `[start, end)`; `None` for a derived row.
    pub range: Option<Range<usize>>,
    /// A container: listed before its children, its range spans theirs.
    pub group: bool,
    /// The class of its bytes: their colour in [`TableStyle::Mini`]'s bytes column, and in the block when the
    /// classes come from [`classes_of`].
    pub class: Class,
    pub tone: Tone,
    /// A checksum row: `Some(true)` draws ✓, `Some(false)` ✗, after the value.
    pub check: Option<bool>,
    /// A variable-length integer: its bytes are joined by a rule in the block.
    pub varint: bool,
}

impl<'a> Field<'a> {
    pub fn new(name: &'a str, value: &'a str) -> Self {
        Self {
            name,
            value,
            ..Self::default()
        }
    }
    pub fn bytes(mut self, range: Range<usize>) -> Self {
        self.range = Some(range);
        self
    }
    /// No bytes of its own: a value the decoder worked out (an alias resolved to its name).
    pub fn derived(mut self) -> Self {
        self.range = None;
        self
    }
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }
    pub fn group(self) -> Self {
        self.group_if(true)
    }
    pub fn group_if(mut self, group: bool) -> Self {
        self.group = group;
        self
    }
    pub fn pre(mut self, pre: &'a str) -> Self {
        self.pre = pre;
        self
    }
    pub fn ty(mut self, ty: &'a str) -> Self {
        self.ty = ty;
        self
    }
    pub fn note(mut self, note: &'a str) -> Self {
        self.note = note;
        self
    }
    pub fn class(mut self, class: Class) -> Self {
        self.class = class;
        self
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
    pub fn check(mut self, ok: bool) -> Self {
        self.check = Some(ok);
        self
    }
    pub fn varint(mut self) -> Self {
        self.varint = true;
        self
    }
    /// Its range cut to a buffer of `len` bytes.
    pub fn clamped(&self, len: usize) -> Option<Range<usize>> {
        self.range
            .as_ref()
            .map(|r| r.start.min(len)..r.end.min(len).max(r.start.min(len)))
    }
}

/// A list of fields the widgets read by index.
pub trait Fields {
    fn len(&self) -> usize;
    fn field(&self, i: usize) -> Field<'_>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Fields for &[Field<'_>] {
    fn len(&self) -> usize {
        <[Field<'_>]>::len(self)
    }
    fn field(&self, i: usize) -> Field<'_> {
        self[i].clone()
    }
}

impl Fields for Vec<Field<'_>> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
    fn field(&self, i: usize) -> Field<'_> {
        self[i].clone()
    }
}

impl<const N: usize> Fields for [Field<'_>; N] {
    fn len(&self) -> usize {
        N
    }
    fn field(&self, i: usize) -> Field<'_> {
        self[i].clone()
    }
}

/// `len` fields made by `f`, so an application lends its own rows without copying them.
pub fn from_fn<'a, F>(len: usize, f: F) -> FromFn<'a, F>
where
    F: Fn(usize) -> Field<'a>,
{
    FromFn {
        len,
        f,
        _rows: PhantomData,
    }
}

/// See [`from_fn`].
pub struct FromFn<'a, F> {
    len: usize,
    f: F,
    _rows: PhantomData<&'a ()>,
}

impl<F> std::fmt::Debug for FromFn<'_, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FromFn").field("len", &self.len).finish()
    }
}

impl<'a, F: Fn(usize) -> Field<'a>> Fields for FromFn<'a, F> {
    fn len(&self) -> usize {
        self.len
    }
    fn field(&self, i: usize) -> Field<'_> {
        (self.f)(i)
    }
}

// ---------------------------------------------------------------------------------------------- pure maths

/// The bytes as `4D 53 47 46`.
pub fn hex_text(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        s.push_str(&format!("{b:02X}"));
    }
    s
}

/// The grid's `bytes` cell: the first `max` bytes of `range`, then ` …` when there are more.
pub fn bytes_cell(bytes: &[u8], range: &Range<usize>, max: usize) -> String {
    let end = range
        .end
        .min(range.start.saturating_add(max))
        .min(bytes.len());
    let mut s = bytes
        .get(range.start.min(end)..end)
        .map(hex_text)
        .unwrap_or_default();
    if range.len() > max {
        s.push_str(" \u{2026}");
    }
    s
}

/// The class of each of `len` bytes from the fields: each leaf (not a group, not derived) paints its range in
/// its class, later fields over earlier ones; bytes no leaf covers are `Plain`.
pub fn classes_of(len: usize, fields: &dyn Fields) -> Vec<Class> {
    let mut out = vec![Class::Plain; len];
    for i in 0..fields.len() {
        let f = fields.field(i);
        if f.group {
            continue;
        }
        if let Some(r) = f.clamped(len)
            && let Some(slice) = out.get_mut(r)
        {
            slice.fill(f.class);
        }
    }
    out
}

/// The innermost field whose bytes include byte `at`: the deepest, and of equal depth the last listed (a leaf
/// wins over its group, a property over its properties block). Derived rows have no bytes and never match.
pub fn innermost(fields: &dyn Fields, at: usize) -> Option<usize> {
    let mut best: Option<(usize, usize)> = None;
    for i in 0..fields.len() {
        let f = fields.field(i);
        if f.range.as_ref().is_some_and(|r| r.contains(&at))
            && best.is_none_or(|(_, d)| f.depth >= d)
        {
            best = Some((i, f.depth));
        }
    }
    best.map(|(i, _)| i)
}

/// The group a field is nested in: the nearest field above it that is shallower and a group.
pub fn parent_of(fields: &dyn Fields, i: usize) -> Option<usize> {
    if i >= fields.len() {
        return None;
    }
    let depth = fields.field(i).depth;
    (0..i).rev().find(|&j| {
        let f = fields.field(j);
        f.depth < depth && f.group
    })
}

/// Field `i`'s bytes cut to `len`, `None` for a derived row or an index past the end.
pub fn range_of(fields: &dyn Fields, i: usize, len: usize) -> Option<Range<usize>> {
    (i < fields.len())
        .then(|| fields.field(i).clamped(len))
        .flatten()
}

/// The line byte `at` is on, at `per_line` bytes a line.
pub fn line_of(at: usize, per_line: usize) -> usize {
    at / per_line.max(1)
}

/// Bytes that fit on a row `width` wide when every character is `char_w` wide: `XX` and a space each (egui
/// breaks a wrapped run so that a row's trailing space fits too). At least one.
pub fn bytes_per_row(width: f32, char_w: f32) -> usize {
    if char_w <= 0.0 {
        return 1;
    }
    ((width / (3.0 * char_w)).floor() as usize).max(1)
}

/// The byte under `x` (from the left of the first byte) on a line of `count` bytes: each byte owns its two
/// characters and half of the space on each side.
pub fn byte_in_line(x: f32, char_w: f32, count: usize) -> Option<usize> {
    if char_w <= 0.0 || x < -char_w / 2.0 {
        return None;
    }
    let k = ((x + char_w / 2.0) / (3.0 * char_w)).floor() as usize;
    (k < count).then_some(k)
}

/// Height of a grid table of `rows` rows that grows to `max`: never below the header and two rows, unless `max`
/// is smaller.
pub fn fit_height(rows: usize, max: f32) -> f32 {
    let want = HEADER_H + rows as f32 * ROW_H;
    let least = (HEADER_H + 2.0 * ROW_H).min(max);
    want.min(max).max(least)
}

/// Height of a [`HexStyle::Lines`] block of `len` bytes that grows to `max`.
pub fn lines_height(len: usize, max: f32) -> f32 {
    (len.div_ceil(PER_LINE) as f32 * LINE_H + 8.0).min(max)
}

/// The columns of [`TableStyle::Mini`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiniColumns {
    pub hex_w: f32,
    pub key_w: f32,
    pub x_key: f32,
    pub x_val: f32,
    pub val_w: f32,
}

/// The mini table's columns from the widest bytes text and the widest name (each capped: 118 and 160 px), in a
/// table `width` wide.
pub fn mini_columns(widest_hex: f32, widest_key: f32, width: f32) -> MiniColumns {
    let hex_w = widest_hex.min(118.0);
    let key_w = widest_key.min(160.0);
    let x_key = PAD_X + hex_w + 14.0;
    let x_val = x_key + key_w + 14.0;
    MiniColumns {
        hex_w,
        key_w,
        x_key,
        x_val,
        val_w: (width - x_val - 10.0).max(60.0),
    }
}

// ---------------------------------------------------------------------------------------------- shared hover

/// The hover the table and the block of one `id_salt` share, a frame late.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Link {
    table: Option<usize>,
    hex: Option<usize>,
}

fn link_id(id: Id) -> Id {
    id.with("hexview-link")
}

fn load_link(ui: &Ui, id: Id) -> Link {
    ui.ctx()
        .data(|d| d.get_temp::<Link>(link_id(id)))
        .unwrap_or_default()
}

fn store_link(ui: &Ui, id: Id, edit: impl FnOnce(&mut Link)) {
    let before = load_link(ui, id);
    let mut after = before;
    edit(&mut after);
    if after != before {
        ui.ctx().data_mut(|d| d.insert_temp(link_id(id), after));
        // the other half was painted with the old hover
        ui.ctx().request_repaint();
    }
}

// ---------------------------------------------------------------------------------------------- the table

/// How the field table looks. See the module docs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableStyle {
    #[default]
    Grid,
    Mini,
}

/// What the table hands back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableResponse {
    pub clicked: Option<usize>,
    pub hovered: Option<usize>,
}

/// The field table.
pub struct FieldTable<'a> {
    id: Id,
    bytes: &'a [u8],
    fields: &'a dyn Fields,
    style: TableStyle,
    selected: Option<usize>,
    max_height: Option<f32>,
    scroll_to: Option<usize>,
    link: bool,
}

impl std::fmt::Debug for FieldTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldTable")
            .field("id", &self.id)
            .field("fields", &self.fields.len())
            .field("style", &self.style)
            .finish()
    }
}

impl<'a> FieldTable<'a> {
    /// The table of `fields` decoded from `bytes`. Give a [`HexBlock`] the same `id_salt` to link the two.
    pub fn new(
        id_salt: impl Hash + std::fmt::Debug,
        bytes: &'a [u8],
        fields: &'a dyn Fields,
    ) -> Self {
        Self {
            id: Id::new(id_salt),
            bytes,
            fields,
            style: TableStyle::Grid,
            selected: None,
            max_height: None,
            scroll_to: None,
            link: true,
        }
    }
    pub fn style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }
    pub fn selected(mut self, row: Option<usize>) -> Self {
        self.selected = row;
        self
    }
    /// Grid style: `Some(max)` grows with its rows up to `max` ([`fit_height`]); `None` fills the room left.
    /// The mini style is always as tall as its rows.
    pub fn max_height(mut self, max: Option<f32>) -> Self {
        self.max_height = max;
        self
    }
    /// Grid style: scroll this row into view (a field picked in the block).
    pub fn scroll_to(mut self, row: Option<usize>) -> Self {
        self.scroll_to = row;
        self
    }
    /// Share the hover with the block of the same `id_salt` (on by default).
    pub fn link(mut self, on: bool) -> Self {
        self.link = on;
        self
    }

    pub fn show(self, ui: &mut Ui) -> TableResponse {
        let t = theme::theme(ui);
        let hot = if self.link {
            load_link(ui, self.id).hex
        } else {
            None
        };
        let out = match self.style {
            TableStyle::Grid => self.grid(ui, hot, t),
            TableStyle::Mini => self.mini(ui, hot, t),
        };
        if self.link {
            store_link(ui, self.id, |l| l.table = out.hovered);
        }
        out
    }

    fn grid(&self, ui: &mut Ui, hot: Option<usize>, t: &Theme) -> TableResponse {
        let Some(max) = self.max_height else {
            return self.grid_in(ui, hot, t);
        };
        let height = fit_height(self.fields.len(), max);
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        child.set_clip_rect(rect.intersect(ui.clip_rect()));
        self.grid_in(&mut child, hot, t)
    }

    fn grid_in(&self, ui: &mut Ui, hot: Option<usize>, t: &Theme) -> TableResponse {
        let fields = self.fields;
        let bytes = self.bytes;
        let cols = [
            Column::new("off", 42.0).right(),
            Column::new("field", 160.0).flex(),
            Column::new("value", 98.0).flex(),
            Column::new("bytes", 78.0),
        ];
        let data = theme::data();
        let hot_bg = |row: usize| (hot == Some(row)).then_some(t.row_hover);
        let resp = RichGrid::new(self.id.with("table"), &cols, fields.len())
            .selected(self.selected)
            .scroll_to(self.scroll_to)
            .row_bg(&hot_bg)
            .show(ui, |c| {
                if c.row >= fields.len() {
                    return;
                }
                let f = fields.field(c.row);
                match c.col {
                    0 => {
                        if let Some(r) = &f.range {
                            c.text(&r.start.to_string(), data.clone(), t.text_3);
                        }
                    }
                    1 => {
                        let indent = f.depth as f32 * 12.0;
                        let mut job = name_job(&f, theme::ui_font(theme::FS_SM), t);
                        let inner = c.inner();
                        job.wrap =
                            TextWrapping::truncate_at_width((inner.width() - indent).max(0.0));
                        let g = c.painter().layout_job(job);
                        c.painter().galley(
                            egui::pos2(inner.left() + indent, c.rect.center().y - g.size().y / 2.0),
                            g,
                            Color32::PLACEHOLDER,
                        );
                    }
                    2 => {
                        let color = match f.tone {
                            Tone::Normal if f.group => t.text_3,
                            Tone::Normal => t.text,
                            Tone::Faint => t.text_3,
                            Tone::Ok => t.ok,
                            Tone::Bad => t.danger,
                        };
                        c.text(f.value, data.clone(), color);
                    }
                    _ => match &f.range {
                        None => {
                            c.text(DERIVED, theme::mono(theme::FS_XS), t.text_3);
                        }
                        Some(r) if !f.group => {
                            let s = bytes_cell(bytes, r, CELL_BYTES);
                            c.text(&s, theme::mono(theme::FS_XS), t.text_3);
                        }
                        Some(_) => {}
                    },
                }
            });
        TableResponse {
            clicked: resp.clicked,
            hovered: resp.hovered,
        }
    }

    fn mini(&self, ui: &mut Ui, hot: Option<usize>, t: &Theme) -> TableResponse {
        let mut out = TableResponse::default();
        let width = ui.available_width();
        let mono = theme::mono(theme::FS_SM);
        let uif = theme::ui_font(theme::FS_SM);
        let n = self.fields.len();
        let hex_of = |f: &Field<'_>| match f.clamped(self.bytes.len()) {
            None => DERIVED.to_owned(),
            Some(_) if f.group => String::new(),
            Some(r) => hex_text(&self.bytes[r]),
        };
        let mut widest_hex = 0.0f32;
        let mut widest_key = 0.0f32;
        for i in 0..n {
            let f = self.fields.field(i);
            let hex = hex_of(&f);
            widest_hex = widest_hex.max(super::galley(ui, hex, mono.clone(), t.text_2).size().x);
            let g = ui.painter().layout_job(name_job(&f, uif.clone(), t));
            widest_key = widest_key.max(g.size().x + f.depth as f32 * 12.0);
        }
        let cols = mini_columns(widest_hex, widest_key, width);
        for i in 0..n {
            let f = self.fields.field(i);
            let hex_color = if f.range.is_none() {
                t.text_3
            } else {
                f.class.color(t)
            };
            let mut hf = TextFormat::simple(mono.clone(), hex_color);
            if f.class == Class::Check {
                hf.underline = Stroke::new(1.0, t.field_line);
            }
            let mut hj = LayoutJob::default();
            hj.append(&hex_of(&f), 0.0, hf);
            hj.wrap.max_width = cols.hex_w + 1.0;
            let hg = ui.painter().layout_job(hj);
            let indent = f.depth as f32 * 12.0;
            let mut kj = name_job(&f, uif.clone(), t);
            kj.wrap = TextWrapping::truncate_at_width((cols.key_w - indent).max(0.0) + 1.0);
            let kg = ui.painter().layout_job(kj);
            let vc = match f.tone {
                Tone::Normal if f.group => t.text_3,
                Tone::Normal => t.syn_num,
                Tone::Faint => t.text_2,
                Tone::Ok => t.ok,
                Tone::Bad => t.danger,
            };
            let mut vj = LayoutJob::default();
            vj.append(f.value, 0.0, TextFormat::simple(mono.clone(), vc));
            if let Some(ok) = f.check {
                let glyph = if ok { icons::CHECK } else { icons::X };
                vj.append(glyph, 5.0, TextFormat::simple(theme::icon_font(11.0), vc));
            }
            if !f.note.is_empty() {
                vj.append(
                    &format!("  {}", f.note),
                    0.0,
                    TextFormat::simple(mono.clone(), t.text_3),
                );
            }
            vj.wrap.max_width = cols.val_w;
            let vg = ui.painter().layout_job(vj);
            let h = hg.size().y.max(kg.size().y).max(vg.size().y) + 6.0;
            let (rect, resp) =
                ui.allocate_exact_size(Vec2::new(width, h.max(22.0)), Sense::click());
            if self.selected == Some(i) {
                ui.painter().rect_filled(rect, 0, t.sel_soft);
            } else if resp.hovered() || hot == Some(i) {
                ui.painter().rect_filled(rect, 0, t.row_hover);
            }
            if resp.hovered() {
                out.hovered = Some(i);
            }
            if resp.clicked() {
                out.clicked = Some(i);
            }
            super::hline(ui.painter(), rect.x_range(), rect.top() + 0.5, t.line_soft);
            let y = rect.top() + 3.0;
            let p = ui.painter();
            p.galley(egui::pos2(rect.left() + PAD_X, y), hg, hex_color);
            p.galley(
                egui::pos2(rect.left() + cols.x_key + indent, y + 1.0),
                kg,
                t.text_2,
            );
            p.galley(egui::pos2(rect.left() + cols.x_val, y), vg, vc);
        }
        out
    }
}

/// The name cell: the faint prefix, the name (a group's semibold in the text colour, a faint row's in text 3),
/// the faint type.
fn name_job(f: &Field<'_>, font: FontId, t: &Theme) -> LayoutJob {
    let mut job = LayoutJob::default();
    if !f.pre.is_empty() {
        job.append(f.pre, 0.0, TextFormat::simple(font.clone(), t.text_3));
    }
    let (name_font, color) = if f.group {
        (theme::semibold(font.size), t.text)
    } else if f.tone == Tone::Faint {
        (font.clone(), t.text_3)
    } else {
        (font.clone(), t.text_2)
    };
    job.append(
        f.name,
        if f.pre.is_empty() { 0.0 } else { 5.0 },
        TextFormat::simple(name_font, color),
    );
    if !f.group && !f.ty.is_empty() {
        job.append(
            f.ty,
            6.0,
            TextFormat::simple(theme::ui_font(10.5), t.text_3),
        );
    }
    job
}

// ---------------------------------------------------------------------------------------------- the block

/// How the hex block looks. See the module docs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HexStyle {
    /// [`PER_LINE`] bytes a line, [`LINE_H`] apart, scrolling inside at most `max_height` ([`lines_height`]).
    Lines { max_height: f32 },
    /// Wrapped at the width, as tall as it needs.
    Wrap,
}

impl Default for HexStyle {
    fn default() -> Self {
        HexStyle::Lines { max_height: 212.0 }
    }
}

/// What the block hands back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HexResponse {
    pub hovered: Option<usize>,
    pub clicked: Option<usize>,
    /// The innermost field under the pointer (needs [`HexBlock::fields`]).
    pub hovered_field: Option<usize>,
    /// The innermost field of the clicked byte.
    pub clicked_field: Option<usize>,
}

/// The hex block.
pub struct HexBlock<'a> {
    id: Id,
    bytes: &'a [u8],
    classes: Option<&'a [Class]>,
    fields: Option<&'a dyn Fields>,
    style: HexStyle,
    selected: Option<Range<usize>>,
    selected_field: Option<usize>,
    scroll_to_selected: bool,
    context: bool,
    link: bool,
}

impl std::fmt::Debug for HexBlock<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HexBlock")
            .field("id", &self.id)
            .field("bytes", &self.bytes.len())
            .field("style", &self.style)
            .finish()
    }
}

/// What each byte of the block looks like this frame.
struct Paint<'p> {
    classes: Option<&'p [Class]>,
    selected: Option<Range<usize>>,
    hot: Option<Range<usize>>,
    context: Option<Range<usize>>,
    varints: Vec<(Range<usize>, Class)>,
}

impl Paint<'_> {
    fn class(&self, i: usize) -> Class {
        self.classes
            .and_then(|c| c.get(i).copied())
            .unwrap_or_default()
    }
    /// The tint under byte `i`: selection over hover over context.
    fn tint(&self, i: usize) -> Option<f32> {
        let on = |r: &Option<Range<usize>>| r.as_ref().is_some_and(|r| r.contains(&i));
        if on(&self.selected) {
            Some(SEL_TINT)
        } else if on(&self.hot) {
            Some(HOT_TINT)
        } else if on(&self.context) {
            Some(CONTEXT_TINT)
        } else {
            None
        }
    }
    fn varint(&self, i: usize) -> Option<(usize, Class)> {
        self.varints
            .iter()
            .enumerate()
            .find(|(_, (r, _))| r.contains(&i))
            .map(|(k, (_, c))| (k, *c))
    }

    /// Bytes `range` as one row of text in `font`.
    fn job(&self, bytes: &[u8], range: Range<usize>, font: &FontId, t: &Theme) -> LayoutJob {
        let mut job = LayoutJob::default();
        let end = range.end;
        for i in range {
            let class = self.class(i);
            let tint = self.tint(i);
            let selected = tint == Some(SEL_TINT);
            let mut f =
                TextFormat::simple(font.clone(), if selected { t.text } else { class.color(t) });
            if let Some(a) = tint {
                f.background = theme::alpha(t.accent, a);
            }
            let varint = self.varint(i);
            if let Some((_, c)) = varint {
                f.underline = Stroke::new(1.0, theme::alpha(c.color(t), 0.7));
            } else if class == Class::Check {
                f.underline = Stroke::new(1.0, t.field_line);
            }
            job.append(&format!("{:02X}", bytes[i]), 0.0, f);
            if i + 1 < end {
                let mut gap = TextFormat::simple(font.clone(), t.text_2);
                // a gap takes the lighter tint of its two bytes
                if let (Some(a), Some(b)) = (tint, self.tint(i + 1)) {
                    gap.background = theme::alpha(t.accent, a.min(b));
                }
                if let (Some((k, c)), Some((k2, _))) = (varint, self.varint(i + 1))
                    && k == k2
                {
                    gap.underline = Stroke::new(1.0, theme::alpha(c.color(t), 0.7));
                }
                job.append(" ", 0.0, gap);
            }
        }
        job
    }
}

impl<'a> HexBlock<'a> {
    /// The block of `bytes`. Give a [`FieldTable`] the same `id_salt` to link the two.
    pub fn new(id_salt: impl Hash + std::fmt::Debug, bytes: &'a [u8]) -> Self {
        Self {
            id: Id::new(id_salt),
            bytes,
            classes: None,
            fields: None,
            style: HexStyle::default(),
            selected: None,
            selected_field: None,
            scroll_to_selected: false,
            context: false,
            link: true,
        }
    }
    /// The class of each byte (missing ones are `Plain`). Without it, the classes come from the fields
    /// ([`classes_of`]).
    pub fn classes(mut self, classes: &'a [Class]) -> Self {
        self.classes = Some(classes);
        self
    }
    /// The fields of the bytes: for the hover, a click on a byte, varints and the context tint.
    pub fn fields(mut self, fields: &'a dyn Fields) -> Self {
        self.fields = Some(fields);
        self
    }
    pub fn style(mut self, style: HexStyle) -> Self {
        self.style = style;
        self
    }
    /// The bytes on the selection tint.
    pub fn selected(mut self, range: Option<Range<usize>>) -> Self {
        self.selected = range;
        self
    }
    /// The selected field: its bytes go on the selection tint, and its parent group on the context tint when
    /// [`HexBlock::context`] is on. Needs [`HexBlock::fields`]; [`HexBlock::selected`] wins when both are given.
    pub fn selected_field(mut self, field: Option<usize>) -> Self {
        self.selected_field = field;
        self
    }
    /// Lines style: scroll the selection into view with a line of context above it, this frame.
    pub fn scroll_to_selected(mut self, scroll: bool) -> Self {
        self.scroll_to_selected = scroll;
        self
    }
    /// Tint the selected field's parent group around it (off by default).
    pub fn context(mut self, on: bool) -> Self {
        self.context = on;
        self
    }
    /// Share the hover with the table of the same `id_salt` (on by default).
    pub fn link(mut self, on: bool) -> Self {
        self.link = on;
        self
    }

    pub fn show(self, ui: &mut Ui) -> HexResponse {
        let t = theme::theme(ui);
        let len = self.bytes.len();
        let derived;
        let classes = match (self.classes, self.fields) {
            (Some(c), _) => Some(c),
            (None, Some(f)) => {
                derived = classes_of(len, f);
                Some(derived.as_slice())
            }
            (None, None) => None,
        };
        let link = if self.link {
            load_link(ui, self.id)
        } else {
            Link::default()
        };
        let fields = self.fields;
        let range = |i: Option<usize>| fields.and_then(|f| i.and_then(|i| range_of(f, i, len)));
        let selected = self.selected.clone().or_else(|| range(self.selected_field));
        let context = if self.context {
            let field = self.selected_field.or_else(|| {
                let s = selected.as_ref()?;
                fields.and_then(|f| innermost(f, s.start))
            });
            range(fields.zip(field).and_then(|(f, i)| parent_of(f, i)))
        } else {
            None
        };
        let mut varints = Vec::new();
        if let Some(f) = fields {
            for i in 0..f.len() {
                let fi = f.field(i);
                if fi.varint
                    && let Some(r) = fi.clamped(len)
                {
                    varints.push((r, fi.class));
                }
            }
        }
        let paint = Paint {
            classes,
            hot: range(link.table.or(link.hex)),
            selected,
            context,
            varints,
        };
        let sense = if fields.is_some() {
            Sense::click()
        } else {
            Sense::hover()
        };
        let mut out = match self.style {
            HexStyle::Lines { max_height } => self.lines(ui, &paint, sense, max_height, t),
            HexStyle::Wrap => self.wrap(ui, &paint, sense, t),
        };
        if let Some(f) = fields {
            out.hovered_field = out.hovered.and_then(|b| innermost(f, b));
            out.clicked_field = out.clicked.and_then(|b| innermost(f, b));
            if out.hovered_field.is_some() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        }
        if self.link {
            store_link(ui, self.id, |l| l.hex = out.hovered_field);
        }
        out
    }

    fn lines(
        &self,
        ui: &mut Ui,
        paint: &Paint<'_>,
        sense: Sense,
        max: f32,
        t: &Theme,
    ) -> HexResponse {
        let mut out = HexResponse::default();
        let bytes = self.bytes;
        let lines = bytes.len().div_ceil(PER_LINE);
        let height = lines_height(bytes.len(), max);
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        child.set_clip_rect(rect.intersect(ui.clip_rect()));
        let font = theme::mono(theme::FS_SM);
        let cw = super::galley(&child, "0", font.clone(), t.text).size().x;
        let mut area = ScrollArea::vertical()
            .id_salt(self.id.with("hex"))
            .auto_shrink([false, false]);
        if self.scroll_to_selected
            && let Some(r) = &paint.selected
        {
            // one line of context above the field
            area = area.vertical_scroll_offset(
                line_of(r.start, PER_LINE).saturating_sub(1) as f32 * LINE_H,
            );
        }
        area.show_rows(&mut child, LINE_H, lines, |ui, range| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            for line in range {
                let (r, resp) = ui.allocate_exact_size(Vec2::new(width, LINE_H), sense);
                if !ui.is_rect_visible(r) {
                    continue;
                }
                let start = line * PER_LINE;
                let end = (start + PER_LINE).min(bytes.len());
                let g = ui
                    .painter()
                    .layout_job(paint.job(bytes, start..end, &font, t));
                let pos = egui::pos2(r.left() + PAD_X, r.center().y - g.size().y / 2.0);
                ui.painter().galley(pos, g, Color32::PLACEHOLDER);
                if let Some(p) = resp.hover_pos()
                    && let Some(k) = byte_in_line(p.x - pos.x, cw, end - start)
                {
                    out.hovered = Some(start + k);
                    if resp.clicked() {
                        out.clicked = Some(start + k);
                    }
                }
            }
        });
        out
    }

    fn wrap(&self, ui: &mut Ui, paint: &Paint<'_>, sense: Sense, t: &Theme) -> HexResponse {
        let mut out = HexResponse::default();
        let bytes = self.bytes;
        let font = theme::data();
        let probe = super::galley(ui, "0", font.clone(), t.text_2);
        let (cw, row_h) = (probe.size().x, probe.size().y);
        let per = bytes_per_row(ui.available_width() - 2.0 * PAD_X, cw);
        let rows = bytes.len().div_ceil(per).max(1);
        let (rect, resp) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), rows as f32 * row_h + 8.0),
            sense,
        );
        let origin = rect.min + Vec2::new(PAD_X, 1.0);
        for row in 0..rows {
            let start = row * per;
            let end = (start + per).min(bytes.len());
            let top = origin.y + row as f32 * row_h;
            let line =
                Rect::from_min_size(egui::pos2(rect.left(), top), Vec2::new(rect.width(), row_h));
            if !ui.is_rect_visible(line) || start >= end {
                continue;
            }
            let g = ui
                .painter()
                .layout_job(paint.job(bytes, start..end, &font, t));
            ui.painter().galley(egui::pos2(origin.x, top), g, t.text_2);
        }
        if let Some(p) = resp.hover_pos() {
            let row = ((p.y - origin.y) / row_h).floor();
            if row >= 0.0 && (row as usize) < rows {
                let start = row as usize * per;
                let count = (start + per).min(bytes.len()).saturating_sub(start);
                if let Some(k) = byte_in_line(p.x - origin.x, cw, count) {
                    out.hovered = Some(start + k);
                    if resp.clicked() {
                        out.clicked = Some(start + k);
                    }
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------------------------- the legend

/// The legend as one line: each word in its class colour, mono 10.5, 10 px apart.
pub fn legend_job(entries: &[(&str, Class)], t: &Theme) -> LayoutJob {
    let mut job = LayoutJob::default();
    let font = theme::mono(10.5);
    for (i, (word, class)) in entries.iter().enumerate() {
        job.append(
            word,
            if i == 0 { 0.0 } else { 10.0 },
            TextFormat::simple(font.clone(), class.color(t)),
        );
    }
    job
}

/// The legend under a section head, 12 px in from each side (OPC's).
pub fn legend(ui: &mut Ui, entries: &[(&str, Class)]) {
    let t = theme::theme(ui);
    let job = legend_job(entries, t);
    egui::Frame::NONE
        .inner_margin(egui::Margin {
            left: 12,
            right: 12,
            top: 0,
            bottom: 2,
        })
        .show(ui, |ui| {
            ui.add(egui::Label::new(job).selectable(false));
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An MQTT 5 PUBLISH: fixed header (type, remaining length), topic, properties block with its length and a
    /// Topic Alias inside, the alias resolved (derived), the payload.
    fn publish() -> Vec<Field<'static>> {
        vec![
            Field::new("Fixed header", "").bytes(0..2).group(),
            Field::new("Packet type", "3 PUBLISH")
                .bytes(0..1)
                .depth(1)
                .class(Class::Header),
            Field::new("Remaining Length", "12")
                .bytes(1..2)
                .depth(1)
                .class(Class::Header)
                .varint(),
            Field::new("Variable header", "").bytes(2..10).group(),
            Field::new("Topic Name", "\"a/b\"")
                .bytes(2..7)
                .depth(1)
                .class(Class::Value),
            Field::new("Properties", "1 · 3 bytes")
                .bytes(7..10)
                .depth(1)
                .group(),
            Field::new("Property Length", "3")
                .bytes(7..8)
                .depth(2)
                .class(Class::Kind)
                .varint(),
            Field::new("Topic Alias", "3")
                .bytes(8..10)
                .depth(2)
                .class(Class::Id),
            Field::new("alias 3", "plant/a/temp").derived().depth(2),
            Field::new("Payload", "4 bytes").bytes(10..14),
        ]
    }

    #[test]
    fn hex_text_and_the_bytes_cell() {
        assert_eq!(hex_text(&[0x4D, 0x53, 0x47]), "4D 53 47");
        assert_eq!(hex_text(&[]), "");
        let b: Vec<u8> = (0..20).collect();
        assert_eq!(bytes_cell(&b, &(2..4), 10), "02 03");
        assert_eq!(
            bytes_cell(&b, &(0..12), 10),
            "00 01 02 03 04 05 06 07 08 09 \u{2026}"
        );
        // a range past the end is cut, not a panic
        assert_eq!(bytes_cell(&b, &(18..30), 10), "12 13 \u{2026}");
        assert_eq!(bytes_cell(&b, &(25..30), 10), "");
    }

    #[test]
    fn classes_come_from_the_leaves_later_over_earlier() {
        let f = publish();
        let c = classes_of(16, &f);
        assert_eq!(c[0], Class::Header);
        assert_eq!(c[1], Class::Header);
        assert_eq!(&c[2..7], &[Class::Value; 5]);
        // the properties group is skipped, its children paint
        assert_eq!(c[7], Class::Kind);
        assert_eq!(&c[8..10], &[Class::Id; 2]);
        assert_eq!(&c[10..16], &[Class::Plain; 6]);
        // OPC's rule: a later leaf over an earlier one, a range past the end cut
        let o = [
            Field::new("a", "").bytes(0..4).class(Class::Header),
            Field::new("b", "").bytes(2..9).class(Class::Status),
        ];
        assert_eq!(
            classes_of(6, &o),
            [
                Class::Header,
                Class::Header,
                Class::Status,
                Class::Status,
                Class::Status,
                Class::Status
            ]
        );
    }

    #[test]
    fn the_innermost_field_wins_and_derived_rows_never_match() {
        let f = publish();
        assert_eq!(innermost(&f, 0), Some(1));
        assert_eq!(innermost(&f, 1), Some(2));
        assert_eq!(innermost(&f, 7), Some(6));
        assert_eq!(innermost(&f, 9), Some(7));
        assert_eq!(innermost(&f, 12), Some(9));
        assert_eq!(innermost(&f, 99), None);
        // a byte only a group covers is the group's
        let g = [
            Field::new("g", "").bytes(0..4).group(),
            Field::new("x", "").bytes(0..2).depth(1),
        ];
        assert_eq!(innermost(&g, 3), Some(0));
        assert_eq!(innermost(&g, 1), Some(1));
    }

    #[test]
    fn parents_and_ranges() {
        let f = publish();
        assert_eq!(parent_of(&f, 7), Some(5));
        assert_eq!(parent_of(&f, 5), Some(3));
        assert_eq!(parent_of(&f, 3), None);
        assert_eq!(parent_of(&f, 8), Some(5));
        assert_eq!(parent_of(&f, 99), None);
        assert_eq!(range_of(&f, 7, 14), Some(8..10));
        assert_eq!(range_of(&f, 8, 14), None);
        assert_eq!(range_of(&f, 9, 12), Some(10..12));
        assert_eq!(range_of(&f, 42, 14), None);
    }

    #[test]
    fn from_fn_lends_rows() {
        struct Row {
            name: String,
            at: usize,
        }
        let rows = [
            Row {
                name: "x".into(),
                at: 3,
            },
            Row {
                name: "y".into(),
                at: 5,
            },
        ];
        let fields = from_fn(rows.len(), |i| {
            Field::new(&rows[i].name, "").bytes(rows[i].at..rows[i].at + 1)
        });
        assert_eq!(fields.len(), 2);
        assert_eq!(innermost(&fields, 5), Some(1));
        assert_eq!(fields.field(0).name, "x");
    }

    #[test]
    fn lines_and_rows() {
        assert_eq!(line_of(0, 16), 0);
        assert_eq!(line_of(15, 16), 0);
        assert_eq!(line_of(16, 16), 1);
        assert_eq!(line_of(5, 0), 5);
        // OPC: 400 bytes are 25 lines of 20 px plus 8, capped at 212
        assert_eq!(lines_height(400, 212.0), 212.0);
        assert_eq!(lines_height(20, 212.0), 48.0);
        // XX and a space each
        assert_eq!(bytes_per_row(24.0, 1.0), 8);
        assert_eq!(bytes_per_row(23.9, 1.0), 7);
        assert_eq!(bytes_per_row(1.0, 7.0), 1);
        assert_eq!(bytes_per_row(100.0, 0.0), 1);
    }

    #[test]
    fn a_byte_owns_its_characters_and_half_of_each_space() {
        let cw = 7.0;
        // byte 0 is x 0..14, the space 14..21: the first half of the space is still byte 0
        assert_eq!(byte_in_line(0.0, cw, 16), Some(0));
        assert_eq!(byte_in_line(-3.0, cw, 16), Some(0));
        assert_eq!(byte_in_line(-3.6, cw, 16), None);
        assert_eq!(byte_in_line(17.4, cw, 16), Some(0));
        assert_eq!(byte_in_line(17.6, cw, 16), Some(1));
        assert_eq!(byte_in_line(42.0, cw, 16), Some(2));
        assert_eq!(byte_in_line(15.0 * 21.0 + 13.0, cw, 16), Some(15));
        // past the last byte of a short line
        assert_eq!(byte_in_line(3.0 * 21.0 + 1.0, cw, 3), None);
        assert_eq!(byte_in_line(5.0, 0.0, 3), None);
    }

    #[test]
    fn grid_height_grows_to_its_max() {
        // OPC: header 30 + n · 24, at least two rows, at most 252
        assert_eq!(fit_height(0, 252.0), 78.0);
        assert_eq!(fit_height(5, 252.0), 150.0);
        assert_eq!(fit_height(50, 252.0), 252.0);
        // a max under two rows wins
        assert_eq!(fit_height(5, 60.0), 60.0);
    }

    #[test]
    fn mini_columns_are_capped() {
        let c = mini_columns(60.0, 80.0, 400.0);
        assert_eq!(c.x_key, 12.0 + 60.0 + 14.0);
        assert_eq!(c.x_val, c.x_key + 80.0 + 14.0);
        assert_eq!(c.val_w, 400.0 - c.x_val - 10.0);
        let c = mini_columns(500.0, 500.0, 300.0);
        assert_eq!((c.hex_w, c.key_w), (118.0, 160.0));
        assert_eq!(c.val_w, 60.0);
    }

    /// Mb's hex block was one galley wrapped by egui at the width; the block's own rows break where it did.
    #[test]
    fn rows_break_where_egui_wrapped() {
        let ctx = egui::Context::default();
        crate::theme::install(&ctx);
        let mut checked = 0;
        let mut frame = ctx.run_ui(egui::RawInput::default(), |ui| {
            let t = theme::theme(ui);
            let font = theme::data();
            let cw = crate::widgets::galley(ui, "0", font.clone(), t.text_2)
                .size()
                .x;
            let bytes: Vec<u8> = (0..=255).collect();
            for width in [120.0, 200.0, 263.0, 300.0, 333.3, 420.0, 517.0] {
                let mut job = LayoutJob::default();
                for (i, b) in bytes.iter().enumerate() {
                    if i > 0 {
                        job.append(" ", 0.0, TextFormat::simple(font.clone(), t.text_2));
                    }
                    job.append(
                        &format!("{b:02X}"),
                        0.0,
                        TextFormat::simple(font.clone(), t.text_2),
                    );
                }
                job.wrap.max_width = width;
                job.wrap.break_anywhere = false;
                let g = ui.painter().layout_job(job);
                let per = bytes_per_row(width, cw);
                assert_eq!(
                    g.rows.len(),
                    bytes.len().div_ceil(per),
                    "width {width}: egui made {} rows, {per} a row makes {}",
                    g.rows.len(),
                    bytes.len().div_ceil(per)
                );
                checked += 1;
            }
        });
        frame.textures_delta.clear();
        assert_eq!(checked, 7);
    }
}
