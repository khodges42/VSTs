// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    analysis::{FFT_SIZE, Shared, spectrum},
    params::EisParams,
};
use eis_core::{Preset, factory, preset::NAMES};
use nih_plug::prelude::*;
use nih_plug_egui::{egui::*, resizable_window::ResizableWindow, widgets::ParamSlider};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};
const BG: Color32 = Color32::from_rgb(28, 28, 29);
const PANEL: Color32 = Color32::from_rgb(53, 53, 55);
const LCD: Color32 = Color32::from_rgb(21, 10, 12);
const RED: Color32 = Color32::from_rgb(160, 31, 42);
const HOT: Color32 = Color32::from_rgb(255, 105, 100);
const SILVER: Color32 = Color32::from_rgb(205, 207, 207);
const TEXT: Color32 = Color32::from_rgb(233, 233, 222);
const YELLOW: Color32 = Color32::from_rgb(245, 211, 98);
pub struct UiState {
    pub status: String,
    pub selected: usize,
    pub spectrum: [f32; FFT_SIZE / 2],
    pub spectrum_id: u64,
    library: PathBuf,
    users: Vec<PathBuf>,
    name_draft: String,
    notes_draft: String,
    metadata_source: String,
    notes_source: String,
    path: String,
}
impl Default for UiState {
    fn default() -> Self {
        let library = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| ".".into())
            .join("SweatyFerguson/EIS/Presets");
        let users = scan(&library);
        Self {
            status: String::new(),
            selected: 0,
            spectrum: [-120.; FFT_SIZE / 2],
            spectrum_id: u64::MAX,
            library,
            users,
            name_draft: String::new(),
            notes_draft: String::new(),
            metadata_source: String::new(),
            notes_source: String::new(),
            path: String::new(),
        }
    }
}
pub fn setup(ctx: &Context, _: &mut UiState) {
    let mut s = (*ctx.style()).clone();
    s.visuals = Visuals::dark();
    s.visuals.panel_fill = BG;
    s.visuals.extreme_bg_color = LCD;
    s.visuals.override_text_color = Some(TEXT);
    s.visuals.selection.bg_fill = RED;
    s.visuals.widgets.inactive.bg_fill = PANEL;
    s.visuals.widgets.inactive.weak_bg_fill = PANEL;
    s.visuals.widgets.active.bg_fill = RED;
    for v in [
        &mut s.visuals.widgets.inactive,
        &mut s.visuals.widgets.hovered,
        &mut s.visuals.widgets.active,
        &mut s.visuals.widgets.open,
    ] {
        v.corner_radius = CornerRadius::ZERO;
        v.bg_stroke = Stroke::new(1., SILVER.gamma_multiply(0.5));
    }
    s.spacing.item_spacing = vec2(8., 7.);
    s.spacing.button_padding = vec2(12., 7.);
    s.text_styles
        .insert(TextStyle::Button, FontId::monospace(12.));
    ctx.set_style(s);
}
pub fn set<P: Param>(p: &P, v: P::Plain, s: &ParamSetter) {
    s.begin_set_parameter(p);
    s.set_parameter(p, v);
    s.end_set_parameter(p);
}
fn gradient(p: &Painter, r: Rect, a: Color32, b: Color32) {
    let mut m = Mesh::default();
    for (pos, c) in [
        (r.left_top(), a),
        (r.right_top(), a),
        (r.right_bottom(), b),
        (r.left_bottom(), b),
    ] {
        m.colored_vertex(pos, c);
    }
    m.add_triangle(0, 1, 2);
    m.add_triangle(0, 2, 3);
    p.add(Shape::mesh(m));
}
fn steak(p: &Painter, origin: Pos2) {
    // Original code-native pixel steak: no bitmap, borrowed icon or skin.
    let rows = [
        "00003333300000",
        "00331111133000",
        "03111111111300",
        "31122111111130",
        "31112211111130",
        "31111122121130",
        "31111111221130",
        "03111111121130",
        "00311111111300",
        "00033333333000",
    ];
    for (y, row) in rows.iter().enumerate() {
        for (x, c) in row.bytes().enumerate() {
            let color = match c {
                b'1' => Color32::from_rgb(157, 42, 53),
                b'2' => Color32::from_rgb(233, 191, 168),
                b'3' => Color32::from_rgb(63, 30, 35),
                _ => continue,
            };
            p.rect_filled(
                Rect::from_min_size(origin + vec2(x as f32 * 5., y as f32 * 5.), vec2(5., 5.)),
                0.,
                color,
            );
        }
    }
}
pub fn draw(ctx: &Context, s: &ParamSetter, state: &mut UiState, p: &EisParams, shared: &Shared) {
    let name = p.name.get();
    let notes = p.notes.get();
    if state.metadata_source != name || state.notes_source != notes {
        state.name_draft = name.clone();
        state.notes_draft = notes.clone();
        state.metadata_source = name;
        state.notes_source = notes;
    }
    ResizableWindow::new("eis")
        .min_size(vec2(900., 760.))
        .show(ctx, p.editor.as_ref(), |ui| {
            let width = ui.available_width();
            let (r, _) = ui.allocate_exact_size(vec2(width, 82.), Sense::hover());
            gradient(ui.painter(), r, Color32::from_rgb(227,228,222), Color32::from_rgb(105,107,111));
            ui.painter().text(r.min + vec2(16.,11.), Align2::LEFT_TOP,
                "SWEATY FERGUSON AUDIO SYSTEMS", FontId::monospace(10.), Color32::from_rgb(63,22,27));
            ui.painter().text(r.min + vec2(14.,24.), Align2::LEFT_TOP,
                "8 INCH STEAK", FontId::monospace(37.), LCD);
            ui.painter().text(r.min + vec2(18.,68.), Align2::LEFT_CENTER,
                "MULTIBAND AUDIO TENDERIZER", FontId::monospace(11.), Color32::from_rgb(85,17,28));
            steak(ui.painter(), r.right_top() + vec2(-90.,15.));
            ui.painter().line_segment([r.left_bottom(),r.right_bottom()], Stroke::new(3.,RED));
            ui.horizontal(|ui| {
                if ui.button("<").clicked() {
                    let i = (p.preset.value() + 7) % 8;
                    p.apply(&factory(i as usize), i, s);
                }
                ComboBox::from_id_salt("presets").width(210.)
                    .selected_text(RichText::new(p.name.get()).color(YELLOW))
                    .show_ui(ui, |ui| {
                        for (i, name) in NAMES.iter().enumerate() {
                            if ui.selectable_label(p.preset.value() == i as i32, *name).clicked() {
                                p.apply(&factory(i), i as i32, s);
                            }
                        }
                        ui.separator();
                        let mut selected = None;
                        for path in &state.users {
                            if ui.selectable_label(false, path.file_name().unwrap_or_default()
                                .to_string_lossy().trim_end_matches(".eis.json")).clicked()
                            {
                                selected = Some(path.clone());
                            }
                        }
                        if let Some(path) = selected {
                            match read(&path) {
                                Ok(preset) => p.apply(&preset, 8, s),
                                Err(e) => state.status = e,
                            }
                        }
                        if ui.button("REFRESH USER BANK").clicked() {
                            state.users = scan(&state.library);
                        }
                    });
                if ui.button(">").clicked() {
                    let i = if p.preset.value() == 8 { 0 } else { (p.preset.value() + 1) % 8 };
                    p.apply(&factory(i as usize), i, s);
                }
                for (name, value) in [("RARE",0.25),("MEDIUM",0.65),("WELL DONE",1.15)] {
                    if ui.button(name).on_hover_text("Set TENDERIZE intensity; retain all base band settings.").clicked() {
                        set(&p.amount, value, s);
                    }
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let butcher=p.butcher.load(Ordering::Relaxed);
                    if ui.selectable_label(butcher, if butcher { "EASY" } else { "BUTCHER" })
                        .on_hover_text("Switch the view; the sound and all band settings stay intact.").clicked()
                    {
                        p.butcher.store(!butcher,Ordering::Relaxed);
                        set(&p.revision,!p.revision.value(),s);
                    }
                });
            });
            ScrollArea::vertical().id_salt("preset-notes").max_height(28.).show(ui, |ui| {
                ui.label(RichText::new(p.notes.get()).size(11.).color(SILVER));
            });
            let m = shared.meter();
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("IN {:5.1} / OUT {:5.1} dBFS", m.input, m.output)).monospace().color(YELLOW));
                io_meter(ui, m.input, m.output);
                toggle(ui, "LEVEL MATCH", &p.auto_gain, s, "Slow wet-only energy matching, bounded to -12/+6 dB; not perceptual loudness matching.");
                toggle(ui, "BYPASS", &p.bypass, s, "24-sample delayed raw bypass removes crossover phase.");
                if p.auto_gain.value() {
                    ui.label(RichText::new(format!("MATCH {:+.1} dB",m.compensation_db)).monospace().color(SILVER));
                }
            });
            let butcher = p.butcher.load(Ordering::Relaxed);
            meters(ui, width, if butcher { 120. } else { (ui.available_height() - 278.).max(190.) }, m);
            if butcher {
                if let Some((id, x)) = shared.snapshot()
                    && id != state.spectrum_id {
                        state.spectrum = spectrum(x);
                        state.spectrum_id = id;
                }
                ScrollArea::vertical().id_salt("butcher").max_height((ui.available_height()-32.).max(250.)).show(ui, |ui| {
                    advanced(ui, width-10., p, s, state, shared);
                });
            } else {
                easy(ui, width, p, s);
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new("A SWEATY FERGUSON PRODUCTION").monospace().size(10.).color(SILVER));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(if m.safety_active { "SAFETY CLIPPING" } else { "24 SAMPLE FILTER LATENCY" })
                        .monospace().size(10.).color(if m.safety_active { HOT } else { SILVER }));
                });
            });
            if !state.status.is_empty() {
                ui.label(RichText::new(&state.status).small().color(YELLOW));
            }
        });
    ctx.request_repaint_after(std::time::Duration::from_millis(33));
}
fn toggle(ui: &mut Ui, label: &str, p: &BoolParam, s: &ParamSetter, tip: &str) {
    if ui
        .selectable_label(p.value(), label)
        .on_hover_text(tip)
        .clicked()
    {
        set(p, !p.value(), s);
    }
}
fn meters(ui: &mut Ui, width: f32, height: f32, m: eis_core::Meter) {
    let (r, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let painter = ui.painter();
    let w = (width - 16.) / 3.;
    for i in 0..3 {
        let panel = Rect::from_min_size(r.min + vec2(i as f32 * (w + 8.), 0.), vec2(w, height));
        painter.rect_filled(panel, 0., LCD);
        painter.rect_stroke(
            panel,
            0.,
            Stroke::new(2., SILVER.gamma_multiply(0.6)),
            StrokeKind::Inside,
        );
        painter.text(
            panel.left_top() + vec2(12., 10.),
            Align2::LEFT_TOP,
            ["01 / LOW", "02 / MID", "03 / HIGH"][i],
            FontId::monospace(16.),
            SILVER,
        );
        let top = panel.top() + 40.;
        let bottom = panel.bottom() - 44.;
        let h = bottom - top;
        for j in 0..=4 {
            let y = top + h * j as f32 / 4.;
            painter.line_segment(
                [pos2(panel.left() + 12., y), pos2(panel.right() - 12., y)],
                Stroke::new(1., Color32::from_rgb(54, 34, 37)),
            );
        }
        let up = m.up[i].clamp(0., 24.) / 24.;
        let down = (-m.down[i]).clamp(0., 48.) / 48.;
        let bw = (w - 48.) / 2.;
        painter.rect_filled(
            Rect::from_min_max(
                pos2(panel.left() + 16., bottom - h * up),
                pos2(panel.left() + 16. + bw, bottom),
            ),
            0.,
            YELLOW,
        );
        painter.rect_filled(
            Rect::from_min_max(
                pos2(panel.left() + 32. + bw, bottom - h * down),
                pos2(panel.right() - 16., bottom),
            ),
            0.,
            RED,
        );
        painter.text(
            pos2(panel.left() + 16., bottom + 9.),
            Align2::LEFT_TOP,
            format!("UP +{:04.1}", m.up[i].max(0.)),
            FontId::monospace(16.),
            YELLOW,
        );
        painter.text(
            pos2(panel.right() - 16., bottom + 9.),
            Align2::RIGHT_TOP,
            format!("DN {:05.1}", m.down[i].min(0.)),
            FontId::monospace(16.),
            HOT,
        );
    }
}
fn easy(ui: &mut Ui, width: f32, p: &EisParams, s: &ParamSetter) {
    Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1., SILVER))
        .inner_margin(8.)
        .show(ui, |ui| {
            let w = (width - 55.) / 5.;
            for row in [
                [
                    (
                        "TENDERIZE",
                        &p.amount,
                        "Scales upward/downward gain in dB. 0% removes compression movement.",
                    ),
                    ("TIME", &p.time, "Scale all stored attack/release times."),
                    (
                        "JUICE",
                        &p.juice,
                        "Upward-compression strength, independent of makeup gain.",
                    ),
                    (
                        "PUNCH",
                        &p.punch,
                        "Lengthen downward gain attack to preserve transients.",
                    ),
                    (
                        "MIX",
                        &p.mix,
                        "Blend with crossover-phase-aligned dry audio.",
                    ),
                ],
                [
                    (
                        "FAT / LOW",
                        &p.low,
                        "Low-band output trim; thresholds stay unchanged.",
                    ),
                    (
                        "MID",
                        &p.mid,
                        "Mid-band output trim; thresholds stay unchanged.",
                    ),
                    (
                        "HIGH",
                        &p.high,
                        "High-band output trim; thresholds stay unchanged.",
                    ),
                    (
                        "SIZZLE",
                        &p.drive,
                        "Global per-band saturation drive; 4x oversampled.",
                    ),
                    (
                        "OUTPUT",
                        &p.output_db,
                        "Final output gain after MIX and level compensation.",
                    ),
                ],
            ] {
                ui.horizontal_top(|ui| {
                    for (label, param, tip) in row {
                        knob(ui, label, param, s, w, tip);
                    }
                });
            }
        });
}
fn knob(ui: &mut Ui, label: &str, p: &FloatParam, s: &ParamSetter, width: f32, tip: &str) {
    ui.vertical(|ui| {
        ui.set_width(width);
        let (r, response) = ui.allocate_exact_size(vec2(width, 65.), Sense::click_and_drag());
        let center = r.center();
        let radius = 26.;
        let v = p.unmodulated_normalized_value();
        let angle = std::f32::consts::PI * (0.75 + v * 1.5);
        let painter = ui.painter();
        painter.circle_filled(center, radius + 3., Color32::from_rgb(114, 115, 116));
        painter.circle_filled(center, radius, PANEL);
        painter.circle_stroke(center, radius - 3., Stroke::new(1., SILVER));
        painter.circle_filled(center, radius - 6., Color32::from_rgb(28, 27, 28));
        painter.line_segment(
            [
                center + vec2(angle.cos(), angle.sin()) * 13.,
                center + vec2(angle.cos(), angle.sin()) * 23.,
            ],
            Stroke::new(3., if label == "TENDERIZE" { HOT } else { YELLOW }),
        );
        if response.drag_started() {
            s.begin_set_parameter(p);
        }
        if response.dragged() {
            let delta = ui.input(|i| i.pointer.delta().y);
            s.set_parameter_normalized(p, (v - delta * 0.004).clamp(0., 1.));
        }
        if response.drag_stopped() {
            s.end_set_parameter(p);
        }
        if response.double_clicked() {
            set(p, p.default_plain_value(), s);
        }
        response.on_hover_text(tip);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(label).monospace().size(11.).color(SILVER));
            ui.label(
                RichText::new(p.normalized_value_to_string(v, true))
                    .monospace()
                    .size(12.)
                    .color(YELLOW),
            );
        });
    });
}

fn io_meter(ui: &mut Ui, input: f32, output: f32) {
    let (r, _) = ui.allocate_exact_size(vec2(100., 20.), Sense::hover());
    for (i, value) in [input, output].iter().enumerate() {
        let strip = Rect::from_min_size(r.min + vec2(0., i as f32 * 11.), vec2(100., 8.));
        ui.painter().rect_filled(strip, 0., LCD);
        ui.painter().rect_filled(
            Rect::from_min_size(
                strip.min,
                vec2(100. * ((*value + 60.) / 60.).clamp(0., 1.), 8.),
            ),
            0.,
            if *value > -0.1 { HOT } else { YELLOW },
        );
    }
}
fn parameter(ui: &mut Ui, label: &str, p: &FloatParam, s: &ParamSetter, width: f32, tip: &str) {
    ui.set_width(width);
    ui.label(RichText::new(label).size(11.).color(SILVER));
    ui.add(ParamSlider::for_param(p, s).with_width((width - 90.).max(60.)))
        .on_hover_text(tip);
}
fn advanced(
    ui: &mut Ui,
    width: f32,
    p: &EisParams,
    s: &ParamSetter,
    state: &mut UiState,
    shared: &Shared,
) {
    ui.horizontal_top(|ui| {
        let w = (width - 24.) / 4.;
        for (label, param, tip) in [
            (
                "TENDERIZE / AMOUNT",
                &p.amount,
                "Scale gain movement relative to stored band settings.",
            ),
            ("TIME", &p.time, "Multiply base attack/release times."),
            (
                "JUICE / UPWARD",
                &p.juice,
                "Scale upward movement before the boost cap.",
            ),
            ("PUNCH", &p.punch, "Lengthen only downward gain attack."),
        ] {
            ui.vertical(|ui| parameter(ui, label, param, s, w, tip));
        }
    });
    ui.horizontal_top(|ui| {
        let w = (width - 16.)/3.;
        ui.vertical(|ui| parameter(ui,"LOW CROSSOVER",&p.cross_low,s,w,"40–1000 Hz. Later splits receive phase compensation."));
        ui.vertical(|ui| parameter(ui,"HIGH CROSSOVER",&p.cross_high,s,w,"600–16000 Hz; effective value is bounded above 1.5x LOW and below 0.4x sample rate."));
        ui.vertical(|ui| parameter(ui,"UPWARD NOISE FLOOR",&p.floor_db,s,w,"Upward boost fades out over 18 dB above this detector floor."));
    });
    ui.horizontal_top(|ui| {
        let spectrum_width = width * 0.63;
        input_spectrum(ui, spectrum_width, 165., state, shared);
        curve(ui, width - spectrum_width - 8., 165., p, state.selected);
    });
    ui.horizontal(|ui| {
        for (i, name) in ["LOW", "MID", "HIGH"].iter().enumerate() {
            ui.selectable_value(&mut state.selected, i, *name);
        }
        let band = p.bands()[state.selected];
        toggle(
            ui,
            "BAND BYPASS",
            band.bypass,
            s,
            "Bypass band dynamics, makeup, saturation and tone trim; preserve the crossover.",
        );
        toggle(
            ui,
            "SOLO",
            band.solo,
            s,
            "Solo selected bands; all other bands fade out.",
        );
        toggle(
            ui,
            "SATURATION",
            band.sat_enabled,
            s,
            "Enable this band's independent 4x saturation path.",
        );
    });
    let b = p.bands()[state.selected];
    let w = (width - 24.) / 3.;
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            parameter(ui,"UPWARD THRESHOLD",b.up_threshold,s,w,"Below this detector level, quieter signals are lifted. Limited to the downward threshold if host controls overlap.");
            parameter(ui,"UPWARD RATIO",b.up_ratio,s,w,"Conventional ratio >=1. Slope below threshold is 1/ratio before macros and floor fade.");
            parameter(ui,"UPWARD GAIN LIMIT",b.up_limit_db,s,w,"Maximum upward boost in dB, after TENDERIZE and JUICE.");
            parameter(ui,"SOFT KNEE",b.knee_db,s,w,"Knee width in dB, shared by upper/lower gain curves.");
        });
        ui.vertical(|ui| {
            parameter(ui,"DOWNWARD THRESHOLD",b.down_threshold,s,w,"Compress loud input above this dBFS detector level.");
            parameter(ui,"DOWNWARD RATIO",b.down_ratio,s,w,"Conventional downward compression ratio.");
            parameter(ui,"BASE ATTACK",b.attack_ms,s,w,"Base gain attack; TIME multiplies it and PUNCH lengthens downward attack.");
            parameter(ui,"BASE RELEASE",b.release_ms,s,w,"Base downward release; TIME multiplies it. Upward gain returns quickly on loud transients.");
        });
        ui.vertical(|ui| {
            parameter(ui,"MAKEUP",b.makeup_db,s,w,"Band output makeup, separate from upward compression and tone trim.");
            parameter(ui,"BAND DRIVE",b.drive,s,w,"Added to SIZZLE. The nonlinear path is 4x oversampled and aligned with band dry.");
            parameter(ui,"STEREO LINK",b.link,s,w,"0% independent detection; 100% shared maximum envelope preserves image.");
            parameter(ui,"BAND MIX",b.mix,s,w,"Blend processed and equally delayed unprocessed band audio.");
        });
    });
    if b.up_threshold.value() > b.down_threshold.value() {
        ui.label(
            RichText::new("Effective upward threshold is limited to the downward threshold.")
                .color(YELLOW),
        );
    }
    CollapsingHeader::new("TONE / MIX / OUTPUT").show(ui, |ui| {
        ui.horizontal_top(|ui| {
            for (label, param) in [("LOW",&p.low),("MID",&p.mid),("HIGH",&p.high),("SIZZLE",&p.drive),("MIX",&p.mix),("OUTPUT",&p.output_db)] {
                ui.vertical(|ui| parameter(ui,label,param,s,(width-40.)/6.,"Same stored control as Easy mode."));
            }
        });
        toggle(ui,"OUTPUT SAFETY",&p.safety,s,"Transparent below 0.98, soft ceiling approaching 1. Disable for unclipped headroom and use OUTPUT responsibly.");
    });
    CollapsingHeader::new("USER PRESETS / IMPORT / EXPORT").show(ui, |ui| library(ui, state, p, s));
}
fn input_spectrum(ui: &mut Ui, width: f32, height: f32, state: &UiState, shared: &Shared) {
    let (r, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let inner = r.shrink2(vec2(12., 25.));
    let painter = ui.painter();
    painter.rect_filled(r, 0., LCD);
    painter.rect_stroke(r, 0., Stroke::new(1., SILVER), StrokeKind::Inside);
    painter.text(
        r.left_top() + vec2(10., 6.),
        Align2::LEFT_TOP,
        "INPUT SPECTRUM / 1024",
        FontId::monospace(11.),
        SILVER,
    );
    let sr = f32::from_bits(shared.rate.load(Ordering::Relaxed));
    let top = (sr * 0.49).min(20000.);
    let xpos =
        |f: f32| inner.left() + ((f / 20.).ln() / (top / 20.).ln()).clamp(0., 1.) * inner.width();
    for f in [100., 1000., 10000.] {
        let x = xpos(f);
        painter.line_segment(
            [pos2(x, inner.top()), pos2(x, inner.bottom())],
            Stroke::new(1., PANEL),
        );
        painter.text(
            pos2(x, r.bottom() - 9.),
            Align2::CENTER_CENTER,
            format!("{f:.0}"),
            FontId::monospace(9.),
            SILVER,
        );
    }
    let mut points = Vec::with_capacity(240);
    for i in 0..240 {
        let f = 20. * (top / 20.).powf(i as f32 / 239.);
        let bin = (f / sr * FFT_SIZE as f32).clamp(1., (FFT_SIZE / 2 - 2) as f32);
        let j = bin as usize;
        let db = state.spectrum[j] * (1. - bin.fract()) + state.spectrum[j + 1] * bin.fract();
        points.push(pos2(
            xpos(f),
            inner.bottom() - (db + 90.).clamp(0., 90.) / 90. * inner.height(),
        ));
    }
    painter.add(Shape::line(points, Stroke::new(1.5, YELLOW)));
    let cross = shared.meter().crossovers;
    for (i, f) in cross.iter().enumerate() {
        let x = xpos(f.max(20.));
        painter.line_segment(
            [pos2(x, inner.top()), pos2(x, inner.bottom())],
            Stroke::new(2., HOT),
        );
        painter.text(
            pos2(x, inner.top() + 3.),
            Align2::CENTER_TOP,
            format!("{:.0} Hz", cross[i]),
            FontId::monospace(10.),
            HOT,
        );
    }
}
fn curve(ui: &mut Ui, width: f32, height: f32, p: &EisParams, index: usize) {
    let (r, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(r, 0., LCD);
    painter.rect_stroke(r, 0., Stroke::new(1., SILVER), StrokeKind::Inside);
    painter.text(
        r.left_top() + vec2(10., 6.),
        Align2::LEFT_TOP,
        "STATIC CURVE / dB",
        FontId::monospace(11.),
        SILVER,
    );
    let inner = r.shrink2(vec2(12., 25.));
    let pos = |x: f32, y: f32| {
        pos2(
            inner.left() + (x + 90.) / 96. * inner.width(),
            inner.bottom() - (y + 90.).clamp(0., 96.) / 96. * inner.height(),
        )
    };
    painter.line_segment([pos(-90., -90.), pos(6., 6.)], Stroke::new(1., PANEL));
    let c = p.config();
    let b = c.bands[index];
    let points = (0..193)
        .map(|i| {
            let x = -90. + i as f32 * 0.5;
            let g = eis_core::dynamics::transfer(x, b, c.amount, c.juice, c.floor_db);
            pos(x, x + g[0] + g[1] + b.makeup_db + c.tone_db[index])
        })
        .collect();
    painter.add(Shape::line(points, Stroke::new(2., HOT)));
    painter.text(
        r.left_bottom() + vec2(10., -8.),
        Align2::LEFT_BOTTOM,
        "-90 INPUT > +6",
        FontId::monospace(9.),
        SILVER,
    );
}
fn scan(root: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(root)
        .into_iter()
        .flatten()
        .take(1024)
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.to_string_lossy().ends_with(".eis.json"))
        .collect();
    paths.sort();
    paths
}
fn read(path: &Path) -> Result<Preset, String> {
    let mut text = String::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take((eis_core::preset::MAX_BYTES + 1) as u64)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    Preset::from_json(&text)
}
fn write(path: &Path, p: &Preset) -> Result<(), String> {
    let text = p.to_json()?;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?
        .write_all(text.as_bytes())
        .map_err(|e| e.to_string())
}
fn slug(text: &str) -> String {
    let s: String = text
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .collect();
    if s.is_empty() { "User".into() } else { s }
}
fn library(ui: &mut Ui, state: &mut UiState, p: &EisParams, s: &ParamSetter) {
    ui.label(
        RichText::new(state.library.display().to_string())
            .small()
            .color(SILVER),
    );
    ui.horizontal(|ui| {
        ui.label("NAME");
        ui.text_edit_singleline(&mut state.name_draft);
        if ui.button("DUPLICATE").clicked() {
            while state.name_draft.len() > 27 {
                state.name_draft.pop();
            }
            state.name_draft = format!("{} COPY", state.name_draft);
        }
        if ui.button("APPLY NAME").clicked() {
            let mut preset = p.document();
            preset.name = state.name_draft.clone();
            preset.description = state.notes_draft.clone();
            match preset.validate() {
                Ok(()) => p.apply(&preset, 8, s),
                Err(e) => state.status = e,
            }
        }
    });
    ui.text_edit_multiline(&mut state.notes_draft);
    ui.horizontal(|ui| {
        if ui.button("SAVE AS NEW").clicked() {
            let mut preset = p.document();
            preset.name = state.name_draft.clone();
            preset.description = state.notes_draft.clone();
            let path = state
                .library
                .join(format!("{}.eis.json", slug(&preset.name)));
            state.status = match preset
                .validate()
                .and_then(|_| fs::create_dir_all(&state.library).map_err(|e| e.to_string()))
                .and_then(|_| write(&path, &preset))
            {
                Ok(()) => {
                    p.apply(&preset, 8, s);
                    format!("Saved {}", path.display())
                }
                Err(e) => format!("Save failed: {e}. Rename to save a new version."),
            };
            state.users = scan(&state.library);
        }
        if ui.button("REFRESH BANK").clicked() {
            state.users = scan(&state.library);
        }
        if ui.button("RESET DEFAULT").clicked() {
            p.apply(&factory(0), 0, s);
        }
    });
    ui.horizontal(|ui| {
        ui.add(
            TextEdit::singleline(&mut state.path)
                .desired_width((ui.available_width() - 180.).max(200.))
                .hint_text("Full path to .eis.json"),
        );
        if ui.button("IMPORT").clicked() {
            match read(Path::new(&state.path)) {
                Ok(preset) => p.apply(&preset, 8, s),
                Err(e) => state.status = e,
            }
        }
        if ui.button("EXPORT").clicked() {
            let mut preset = p.document();
            preset.name = state.name_draft.clone();
            preset.description = state.notes_draft.clone();
            state.status = match write(Path::new(&state.path), &preset) {
                Ok(()) => "Exported".into(),
                Err(e) => e,
            };
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn portable_user_files_roundtrip_and_never_overwrite() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/eis-validation")
            .join(format!("preset-io-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("User.eis.json");
        let mut p = eis_core::factory(2);
        p.name = "USER REESE".into();
        p.config.bands[1].up_ratio = 5.1;
        super::write(&path, &p).unwrap();
        assert_eq!(super::read(&path).unwrap(), p);
        assert!(super::write(&path, &eis_core::factory(0)).is_err());
        assert_eq!(super::read(&path).unwrap(), p);
        assert_eq!(super::scan(&root), vec![path.clone()]);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
