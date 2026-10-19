//! The hex field view on three made-up messages, in both themes.
//!
//! `cargo run --example hexview`. Left, an OPC UA–like ReadRequest as the trace inspector draws it: the grid
//! table, the legend and the 16-byte lines, a field selected. Middle, a Modbus RTU request and reply as the
//! frame inspector draws them: the wrapped bytes over the mini table, register names from a map, a CRC
//! check. Right, an MQTT 5 PUBLISH: a two-byte Remaining Length and the Property Length as varints, a Topic
//! Alias nested in the properties block with its parent tinted, and the alias resolved to its topic as a
//! derived row. Hover a row to see its bytes, hover or click a byte to find its field.
//!
//! Environment, for captures: `STUDIO_UI_HEXVIEW_THEME=light` starts in the light theme.

use egui::{Frame, Ui, Vec2};

use studio_ui::icons;
use studio_ui::theme::{self, Theme};
use studio_ui::widgets::hexview::{
    self, Class, Field, FieldTable, HexBlock, HexStyle, TableStyle, Tone,
};
use studio_ui::widgets::sec::{sec_h, sec_h_with};
use studio_ui::widgets::{self, SegItem, seg};

/// A field the writer logged, owned.
#[derive(Clone, Debug, Default)]
struct Logged {
    name: String,
    pre: String,
    ty: String,
    text: String,
    note: String,
    depth: usize,
    range: Option<std::ops::Range<usize>>,
    group: bool,
    class: Class,
    tone: Tone,
    check: Option<bool>,
    varint: bool,
}

/// Bytes with a field log, like the decoders of the apps keep.
#[derive(Debug, Default)]
struct Writer {
    bytes: Vec<u8>,
    fields: Vec<Logged>,
}

impl Writer {
    fn fld(
        &mut self,
        depth: usize,
        name: &str,
        ty: &str,
        text: &str,
        class: Class,
        b: &[u8],
    ) -> usize {
        let s = self.bytes.len();
        self.bytes.extend_from_slice(b);
        self.fields.push(Logged {
            name: name.into(),
            ty: ty.into(),
            text: text.into(),
            depth,
            range: Some(s..self.bytes.len()),
            class,
            ..Logged::default()
        });
        self.fields.len() - 1
    }
    fn open(&mut self, depth: usize, name: &str, text: &str) -> usize {
        let s = self.bytes.len();
        self.fields.push(Logged {
            name: name.into(),
            text: text.into(),
            depth,
            range: Some(s..s),
            group: true,
            ..Logged::default()
        });
        self.fields.len() - 1
    }
    fn close(&mut self, i: usize) {
        if let Some(r) = &mut self.fields[i].range {
            r.end = self.bytes.len();
        }
    }
    fn derived(&mut self, depth: usize, name: &str, text: &str) {
        self.fields.push(Logged {
            name: name.into(),
            text: text.into(),
            depth,
            ..Logged::default()
        });
    }
}

fn view(l: &Logged) -> Field<'_> {
    let mut f = Field::new(&l.name, &l.text)
        .pre(&l.pre)
        .ty(&l.ty)
        .note(&l.note)
        .depth(l.depth)
        .group_if(l.group)
        .class(l.class)
        .tone(l.tone);
    f.range = l.range.clone();
    f.check = l.check;
    f.varint = l.varint;
    f
}

fn opc_read_request() -> Writer {
    let mut w = Writer::default();
    let g = w.open(0, "Message header", "");
    w.fld(1, "MessageType", "Byte[3]", "MSG", Class::Header, b"MSG");
    w.fld(1, "ChunkType", "Byte", "F final", Class::Header, b"F");
    w.fld(
        1,
        "MessageSize",
        "UInt32",
        "93",
        Class::Header,
        &93u32.to_le_bytes(),
    );
    w.close(g);
    let g = w.open(0, "Security header", "");
    w.fld(
        1,
        "SecureChannelId",
        "UInt32",
        "7",
        Class::Header,
        &7u32.to_le_bytes(),
    );
    w.fld(
        1,
        "TokenId",
        "UInt32",
        "1",
        Class::Header,
        &1u32.to_le_bytes(),
    );
    w.close(g);
    let g = w.open(0, "Sequence header", "");
    w.fld(
        1,
        "SequenceNumber",
        "UInt32",
        "52",
        Class::Id,
        &52u32.to_le_bytes(),
    );
    w.fld(
        1,
        "RequestId",
        "UInt32",
        "41",
        Class::Id,
        &41u32.to_le_bytes(),
    );
    w.close(g);
    w.fld(
        0,
        "TypeId",
        "NodeId",
        "i=631 ReadRequest",
        Class::Kind,
        &[1, 0, 0x77, 0x02],
    );
    let g = w.open(0, "RequestHeader", "");
    w.fld(
        1,
        "AuthenticationToken",
        "NodeId",
        "b=4f1c…",
        Class::Plain,
        &[5, 0, 0, 4, 0, 0, 0, 0x4F, 0x1C, 0x22, 0x9E],
    );
    w.fld(
        1,
        "Timestamp",
        "DateTime",
        "2026-10-19 14:09:30.120Z",
        Class::Value,
        &0x01DD_41A2_7C3E_0F00u64.to_le_bytes(),
    );
    w.fld(
        1,
        "RequestHandle",
        "UInt32",
        "41",
        Class::Plain,
        &41u32.to_le_bytes(),
    );
    w.fld(
        1,
        "ReturnDiagnostics",
        "UInt32",
        "0",
        Class::Plain,
        &0u32.to_le_bytes(),
    );
    w.fld(
        1,
        "AuditEntryId",
        "String",
        "null",
        Class::Plain,
        &[0xFF; 4],
    );
    w.fld(
        1,
        "TimeoutHint",
        "UInt32",
        "10000",
        Class::Plain,
        &10_000u32.to_le_bytes(),
    );
    w.fld(
        1,
        "AdditionalHeader",
        "ExtensionObject",
        "null",
        Class::Plain,
        &[0, 0, 0],
    );
    w.close(g);
    w.fld(
        0,
        "MaxAge",
        "Double",
        "0",
        Class::Plain,
        &0f64.to_le_bytes(),
    );
    w.fld(
        0,
        "TimestampsToReturn",
        "Enumeration",
        "2 Both",
        Class::Plain,
        &2u32.to_le_bytes(),
    );
    let g = w.open(0, "NodesToRead", "ReadValueId[1]");
    w.fld(1, "length", "Int32", "1", Class::Plain, &1u32.to_le_bytes());
    let e = w.open(1, "[0]", "ReadValueId");
    w.fld(
        2,
        "NodeId",
        "NodeId",
        "ns=2;i=1001",
        Class::Plain,
        &[1, 2, 0xE9, 0x03],
    );
    w.fld(
        2,
        "AttributeId",
        "UInt32",
        "13 Value",
        Class::Plain,
        &13u32.to_le_bytes(),
    );
    w.fld(2, "IndexRange", "String", "null", Class::Plain, &[0xFF; 4]);
    w.fld(
        2,
        "DataEncoding",
        "QualifiedName",
        "null",
        Class::Plain,
        &[0, 0, 0xFF, 0xFF, 0xFF, 0xFF],
    );
    w.close(e);
    w.close(g);
    w
}

/// CRC-16/MODBUS, low byte first on the wire.
fn crc16(b: &[u8]) -> u16 {
    let mut crc = 0xFFFFu16;
    for &x in b {
        crc ^= u16::from(x);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xA001
            } else {
                crc >> 1
            };
        }
    }
    crc
}

fn rtu(pdu: &[u8]) -> Vec<u8> {
    let mut v = pdu.to_vec();
    v.extend_from_slice(&crc16(pdu).to_le_bytes());
    v
}

/// What Mb's frame inspector shows of an RTU frame: `bytes · field · value`, and the block's classes.
fn modbus(frame: &[u8], reply: bool) -> (Vec<Logged>, Vec<Class>) {
    let len = frame.len();
    let row = |name: &str, text: String, r: std::ops::Range<usize>, class: Class| Logged {
        name: name.into(),
        text,
        range: Some(r),
        class,
        ..Logged::default()
    };
    let mut rows = vec![
        row("slave address", frame[0].to_string(), 0..1, Class::Id),
        // the table showed a function code's byte in text 2, the block in syn kw
        row(
            "function",
            "03 · Read holding registers".into(),
            1..2,
            Class::Plain,
        ),
    ];
    if reply {
        rows.push(row(
            "byte count",
            format!("{} = {} registers", frame[2], frame[2] / 2),
            2..3,
            Class::Plain,
        ));
        let names = [
            ("40001", "voltage_l1", " V", 0.1),
            ("40002", "voltage_l2", " V", 0.1),
            ("40003", "current_l1", " A", 0.01),
        ];
        for k in 0..(usize::from(frame[2]) / 2) {
            let at = 3 + 2 * k;
            let w = u16::from_be_bytes([frame[at], frame[at + 1]]);
            match names.get(k) {
                Some((addr, name, unit, scale)) => {
                    let mut r = row(
                        name,
                        format!("{:.1}{unit}", f64::from(w) * scale),
                        at..at + 2,
                        Class::Plain,
                    );
                    r.pre = (*addr).into();
                    rows.push(r);
                }
                None => {
                    let mut r = row(
                        "not in map",
                        format!("{w}  0x{w:04X}"),
                        at..at + 2,
                        Class::Plain,
                    );
                    r.pre = format!("4000{}", k + 1);
                    r.tone = Tone::Faint;
                    rows.push(r);
                }
            }
        }
    } else {
        let a = u16::from_be_bytes([frame[2], frame[3]]);
        let q = u16::from_be_bytes([frame[4], frame[5]]);
        rows.push(row("start address", format!("{a}"), 2..4, Class::Plain));
        rows.push(row(
            "quantity",
            format!("{q} registers"),
            4..6,
            Class::Plain,
        ));
    }
    let got = u16::from_le_bytes([frame[len - 2], frame[len - 1]]);
    let computed = crc16(&frame[..len - 2]);
    let mut crc = row("CRC", format!("0x{got:04X}"), len - 2..len, Class::Check);
    if got == computed {
        crc.check = Some(true);
        crc.tone = Tone::Ok;
        crc.note = "matches computed, low byte first".into();
    } else {
        crc.text = format!("received 0x{got:04X} \u{2260} computed 0x{computed:04X}");
        crc.check = Some(false);
        crc.tone = Tone::Bad;
    }
    rows.push(crc);
    let classes = (0..len)
        .map(|i| match i {
            0 => Class::Id,
            1 => Class::Kind,
            _ if i + 2 >= len => Class::Check,
            _ => Class::Plain,
        })
        .collect();
    (rows, classes)
}

fn mqtt_publish() -> Writer {
    let payload = br#"{"temp_c":4.2,"setpoint_c":4.0,"door":"closed","compressor":{"running":true,"suction_bar":1.43},"ts":"2026-10-19T14:09:30.120Z","quality":"good"}"#;
    // variable header: topic (2) + packet id (2) + property length (1) + properties (3 + 5)
    let props = 3 + 5;
    let remaining = 2 + 2 + 1 + props + payload.len();
    let mut w = Writer::default();
    let g = w.open(0, "Fixed header", "");
    w.fld(
        1,
        "Packet type",
        "Byte",
        "0x32 · PUBLISH · QoS 1 · retain 0",
        Class::Header,
        &[0x32],
    );
    let rl = [(remaining & 0x7F) as u8 | 0x80, (remaining >> 7) as u8];
    let i = w.fld(
        1,
        "Remaining Length",
        "Variable Byte Integer",
        &remaining.to_string(),
        Class::Header,
        &rl,
    );
    w.fields[i].varint = true;
    w.close(g);
    let g = w.open(0, "Variable header", "");
    w.fld(
        1,
        "Topic Name",
        "UTF-8 String",
        "\"\" · the Topic Alias names it",
        Class::Value,
        &[0, 0],
    );
    w.fld(
        1,
        "Packet Identifier",
        "Two Byte Integer",
        "18",
        Class::Id,
        &[0, 18],
    );
    let p = w.open(1, "Properties", "2 · 8 bytes");
    let i = w.fld(
        2,
        "Property Length",
        "Variable Byte Integer",
        "8",
        Class::Kind,
        &[props as u8],
    );
    w.fields[i].varint = true;
    w.fld(
        2,
        "Topic Alias",
        "0x23 Two Byte Integer",
        "3",
        Class::Kind,
        &[0x23, 0, 3],
    );
    w.derived(2, "Topic (alias 3)", "plant/line-a/cold-room-2/temp");
    w.fld(
        2,
        "Message Expiry Interval",
        "0x02 Four Byte Integer",
        "60 s",
        Class::Kind,
        &[0x02, 0, 0, 0, 60],
    );
    w.close(p);
    w.close(g);
    w.fld(
        0,
        "Payload",
        "Binary Data",
        &format!("{} bytes · JSON", payload.len()),
        Class::Plain,
        payload,
    );
    w
}

#[derive(Debug)]
struct Demo {
    opc: Writer,
    opc_sel: Option<usize>,
    opc_scroll: bool,
    mb_req: Vec<u8>,
    mb_resp: Vec<u8>,
    mb: [(Vec<Logged>, Vec<Class>); 2],
    mq: Writer,
    mq_sel: Option<usize>,
}

impl Demo {
    fn new() -> Self {
        let opc = opc_read_request();
        let mq = mqtt_publish();
        let mb_req = rtu(&[30, 3, 0, 0, 0, 3]);
        let mut mb_resp = rtu(&[30, 3, 8, 0x08, 0xDA, 0x08, 0xE1, 0x02, 0x1C, 0x00, 0x2A]);
        // a reply with the map's three registers and one past it, its CRC damaged on the line
        let n = mb_resp.len();
        mb_resp[n - 1] ^= 0x5A;
        let mb = [modbus(&mb_req, false), modbus(&mb_resp, true)];
        let opc_sel = opc.fields.iter().position(|f| f.name == "Timestamp");
        let mq_sel = mq.fields.iter().position(|f| f.name == "Topic Alias");
        Self {
            opc,
            opc_sel,
            opc_scroll: true,
            mb_req,
            mb_resp,
            mb,
            mq,
            mq_sel,
        }
    }

    /// Column `k` of three, a panel-coloured inspector-like pane.
    fn column(ui: &mut Ui, k: usize, t: &Theme, add: impl FnOnce(&mut Ui)) {
        let all = ui.max_rect();
        let w = ((all.width() - 2.0) / 3.0).floor();
        let rect = egui::Rect::from_min_size(
            egui::pos2(all.left() + k as f32 * (w + 1.0), all.top()),
            Vec2::new(w, all.height()),
        );
        ui.painter().rect_filled(rect, 0, t.panel);
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        child.set_clip_rect(rect);
        child.spacing_mut().item_spacing = Vec2::ZERO;
        add(&mut child);
    }

    fn opc(&mut self, ui: &mut Ui) {
        let w = &self.opc;
        let fields = hexview::from_fn(w.fields.len(), |i| view(&w.fields[i]));
        sec_h(ui, "Fields", Some("click a row to find its bytes"), false);
        let table = FieldTable::new("opc", &w.bytes, &fields)
            .selected(self.opc_sel)
            .max_height(Some(252.0))
            .show(ui);
        let count = w.bytes.len().to_string();
        sec_h(ui, "Bytes", Some(&count), false);
        hexview::legend(
            ui,
            &[
                ("header", Class::Header),
                ("sequence", Class::Id),
                ("TypeId", Class::Kind),
                ("time", Class::Value),
                ("status", Class::Status),
            ],
        );
        let hex = HexBlock::new("opc", &w.bytes)
            .fields(&fields)
            .selected_field(self.opc_sel)
            .scroll_to_selected(std::mem::take(&mut self.opc_scroll))
            .style(HexStyle::Lines { max_height: 212.0 })
            .show(ui);
        if let Some(i) = table.clicked {
            self.opc_sel = Some(i);
            self.opc_scroll = true;
        }
        if let Some(i) = hex.clicked_field {
            self.opc_sel = Some(i);
        }
    }

    fn modbus(&self, ui: &mut Ui, t: &Theme) {
        for (k, (frame, (rows, classes))) in [&self.mb_req, &self.mb_resp]
            .into_iter()
            .zip(&self.mb)
            .enumerate()
        {
            let fields: Vec<Field<'_>> = rows.iter().map(view).collect();
            let n = format!("{} bytes", frame.len());
            sec_h_with(
                ui,
                if k == 0 { "Request" } else { "Response" },
                None,
                false,
                |ui| {
                    widgets::text(ui, n, theme::ui_font(theme::FS_XS), t.text_3);
                },
            );
            HexBlock::new(("mb", k), frame)
                .classes(classes)
                .fields(&fields)
                .style(HexStyle::Wrap)
                .show(ui);
            FieldTable::new(("mb", k), frame, &fields)
                .style(TableStyle::Mini)
                .show(ui);
        }
    }

    fn mqtt(&mut self, ui: &mut Ui) {
        let w = &self.mq;
        let fields = hexview::from_fn(w.fields.len(), |i| view(&w.fields[i]));
        sec_h(ui, "Fields", Some("click a row to find its bytes"), false);
        let table = FieldTable::new("mq", &w.bytes, &fields)
            .selected(self.mq_sel)
            .max_height(Some(300.0))
            .show(ui);
        let count = format!("{} B", w.bytes.len());
        sec_h(ui, "Bytes", Some(&count), false);
        hexview::legend(
            ui,
            &[
                ("fixed header", Class::Header),
                ("packet id · alias", Class::Id),
                ("topic", Class::Value),
                ("properties", Class::Kind),
                ("reason", Class::Status),
            ],
        );
        let hex = HexBlock::new("mq", &w.bytes)
            .fields(&fields)
            .selected_field(self.mq_sel)
            .context(true)
            .show(ui);
        if let Some(i) = table.clicked.or(hex.clicked_field) {
            self.mq_sel = Some(i);
        }
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
                    widgets::text(ui, "studio-ui hexview", theme::semibold(theme::FS), t.text);
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
                Self::column(ui, 0, t, |ui| self.opc(ui));
                Self::column(ui, 1, t, |ui| self.modbus(ui, t));
                Self::column(ui, 2, t, |ui| self.mqtt(ui));
            });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("studio-ui hexview")
            .with_inner_size([1400.0, 820.0]),
        ..Default::default()
    };
    eframe::run_native(
        "studio-ui hexview",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            if std::env::var("STUDIO_UI_HEXVIEW_THEME").as_deref() == Ok("light") {
                cc.egui_ctx.set_theme(egui::Theme::Light);
            }
            Ok(Box::new(Demo::new()))
        }),
    )
}
