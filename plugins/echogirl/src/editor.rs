// SPDX-License-Identifier: GPL-3.0-or-later
use crate::params::EchoParams;
use echogirl_core::{
    Preset, factory,
    preset::{self, CHARACTERS, DIVISIONS, Source, Target},
};
use nih_plug::prelude::*;
use nih_plug_egui::{egui::*, resizable_window::ResizableWindow};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
};
const BG: Color32 = Color32::from_rgb(28, 25, 34);
const PANEL: Color32 = Color32::from_rgb(49, 42, 56);
const LCD: Color32 = Color32::from_rgb(16, 14, 21);
const AMBER: Color32 = Color32::from_rgb(255, 190, 84);
const VIOLET: Color32 = Color32::from_rgb(181, 143, 241);
const TEXT: Color32 = Color32::from_rgb(227, 223, 231);
const MUTED: Color32 = Color32::from_rgb(166, 155, 177);
const EDGE: Color32 = Color32::from_rgb(117, 104, 133);
pub struct UiState {
    edit: bool,
    draft: Preset,
    dirty: bool,
    status: String,
    path: String,
    library: PathBuf,
    users: Vec<PathBuf>,
    pressed: [bool; 3],
    pulse: [bool; 3],
}
impl Default for UiState {
    fn default() -> Self {
        let library = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| ".".into())
            .join("VOIDMUTT/ECHOGIRL/Presets");
        let users = scan(&library);
        Self {
            edit: false,
            draft: factory(0),
            dirty: false,
            status: String::new(),
            path: String::new(),
            library,
            users,
            pressed: [false; 3],
            pulse: [false; 3],
        }
    }
}
pub fn setup(ctx: &Context, _: &mut UiState) {
    let mut s = (*ctx.style()).clone();
    s.visuals = Visuals::dark();
    s.visuals.panel_fill = BG;
    s.visuals.window_fill = PANEL;
    s.visuals.extreme_bg_color = LCD;
    s.visuals.override_text_color = Some(TEXT);
    s.visuals.selection.bg_fill = Color32::from_rgb(88, 58, 111);
    s.visuals.selection.stroke = Stroke::new(1., VIOLET);
    s.visuals.hyperlink_color = AMBER;
    for v in [
        &mut s.visuals.widgets.inactive,
        &mut s.visuals.widgets.hovered,
        &mut s.visuals.widgets.active,
        &mut s.visuals.widgets.open,
    ] {
        v.corner_radius = CornerRadius::same(2);
        v.bg_stroke = Stroke::new(1., EDGE);
    }
    s.visuals.widgets.inactive.bg_fill = PANEL;
    s.visuals.widgets.inactive.weak_bg_fill = PANEL;
    s.visuals.widgets.hovered.bg_fill = Color32::from_rgb(78, 58, 88);
    s.visuals.widgets.active.bg_fill = Color32::from_rgb(87, 58, 44);
    s.spacing.item_spacing = vec2(10., 8.);
    s.spacing.button_padding = vec2(12., 7.);
    s.text_styles
        .insert(TextStyle::Body, FontId::proportional(13.));
    s.text_styles
        .insert(TextStyle::Button, FontId::monospace(12.));
    ctx.set_style(s);
}
fn set<P: Param>(p: &P, v: P::Plain, s: &ParamSetter) {
    s.begin_set_parameter(p);
    s.set_parameter(p, v);
    s.end_set_parameter(p);
}
fn apply(preset: Preset, index: i32, state: &mut UiState, p: &EchoParams, s: &ParamSetter) {
    match preset.validate() {
        Ok(()) => {
            p.apply(&preset, index, s);
            state.draft = preset;
            state.dirty = false;
            state.status.clear();
        }
        Err(e) => state.status = e,
    }
}
fn gradient(p: &Painter, r: Rect, a: Color32, b: Color32) {
    let mut m = Mesh::default();
    m.colored_vertex(r.left_top(), a);
    m.colored_vertex(r.right_top(), a);
    m.colored_vertex(r.right_bottom(), b);
    m.colored_vertex(r.left_bottom(), b);
    m.add_triangle(0, 1, 2);
    m.add_triangle(0, 2, 3);
    p.add(Shape::mesh(m));
}
pub fn draw(
    ctx: &Context,
    s: &ParamSetter,
    state: &mut UiState,
    p: &EchoParams,
    meter: &AtomicU32,
    gates: &[AtomicBool; 3],
) {
    ResizableWindow::new("echogirl")
        .min_size(vec2(820., 700.))
        .show(ctx, p.editor.as_ref(), |ui| {
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 70.), Sense::hover());
            gradient(ui.painter(), r, Color32::from_rgb(77, 63, 84), BG);
            ui.painter()
                .line_segment([r.left_bottom(), r.right_bottom()], Stroke::new(2., VIOLET));
            ui.painter().text(
                r.min + vec2(12., 26.),
                Align2::LEFT_CENTER,
                "ECHOGIRL",
                FontId::monospace(38.),
                TEXT,
            );
            ui.painter().text(
                r.min + vec2(15., 57.),
                Align2::LEFT_CENTER,
                "DUB / DELAY / FEEDBACK SYSTEM",
                FontId::monospace(10.),
                AMBER,
            );
            ui.painter().text(
                r.right_center() - vec2(16., 0.),
                Align2::RIGHT_CENTER,
                "BY VOIDMUTT",
                FontId::monospace(13.),
                VIOLET,
            );
            Frame::new()
                .fill(PANEL)
                .stroke(Stroke::new(1., EDGE))
                .inner_margin(8.)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("<").clicked() {
                            let i = (p.preset.value() + 9) % 10;
                            apply(factory(i as usize), i, state, p, s);
                        }
                        ComboBox::from_id_salt("presets")
                            .width(250.)
                            .selected_text(RichText::new(p.name.get()).color(AMBER))
                            .show_ui(ui, |ui| {
                                for (i, name) in preset::NAMES.iter().enumerate() {
                                    if ui
                                        .selectable_label(p.preset.value() == i as i32, *name)
                                        .clicked()
                                    {
                                        apply(factory(i), i as i32, state, p, s);
                                    }
                                }
                                ui.separator();
                                let mut selected = None;
                                for path in &state.users {
                                    if ui
                                        .selectable_label(
                                            false,
                                            path.file_name()
                                                .unwrap_or_default()
                                                .to_string_lossy()
                                                .trim_end_matches(".echogirl.json"),
                                        )
                                        .clicked()
                                    {
                                        selected = Some(path.clone());
                                    }
                                }
                                if let Some(path) = selected {
                                    match read(&path) {
                                        Ok(preset) => apply(preset, 10, state, p, s),
                                        Err(e) => state.status = e,
                                    }
                                }
                                if ui.button("REFRESH USER BANK").clicked() {
                                    state.users = scan(&state.library);
                                }
                            });
                        if ui.button(">").clicked() {
                            let i = if p.preset.value() == 10 {
                                0
                            } else {
                                (p.preset.value() + 1) % 10
                            };
                            apply(factory(i as usize), i, state, p, s);
                        }
                        ui.label(RichText::new("MODE").size(10.).color(MUTED));
                        ComboBox::from_id_salt("mode")
                            .width(75.)
                            .selected_text(CHARACTERS[p.character.value() as usize])
                            .show_ui(ui, |ui| {
                                for (i, name) in CHARACTERS.iter().enumerate() {
                                    if ui
                                        .selectable_label(p.character.value() == i as i32, *name)
                                        .clicked()
                                    {
                                        set(&p.character, i as i32, s);
                                    }
                                }
                            });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.selectable_label(state.edit, "EDIT").clicked() {
                                state.edit = !state.edit;
                                if state.edit {
                                    state.draft = p.preset_document();
                                    state.dirty = false;
                                }
                            }
                            if ui.selectable_label(!state.edit, "PERFORM").clicked() {
                                state.edit = false;
                            }
                        });
                    });
                });
            if state.edit {
                edit(ui, state, p, s);
            } else {
                performance(ui, state, p, s, meter, gates);
            }
            if !state.status.is_empty() {
                ui.label(RichText::new(&state.status).size(11.).color(AMBER));
            }
        });
    if !ctx.input(|i| i.pointer.primary_down()) || state.edit {
        for (i, param) in [&p.throw, &p.kill, &p.dub].into_iter().enumerate() {
            if state.pressed[i] && (!state.pulse[i] || state.edit) {
                s.set_parameter(param, false);
                s.end_set_parameter(param);
                state.pressed[i] = false;
                gates[i].store(false, Ordering::Relaxed);
            }
        }
    }
    state.pulse.fill(false);
    ctx.request_repaint_after(std::time::Duration::from_millis(33));
}
fn performance(
    ui: &mut Ui,
    state: &mut UiState,
    p: &EchoParams,
    s: &ParamSetter,
    meter: &AtomicU32,
    gates: &[AtomicBool; 3],
) {
    let c = p.config();
    let width = ui.available_width();
    ScrollArea::vertical()
        .id_salt("notes")
        .max_height(34.)
        .show(ui, |ui| {
            ui.label(RichText::new(p.notes.get()).size(12.).color(MUTED));
        });
    ui.horizontal(|ui| {
        toggle(ui, "SYNC", &p.sync, s, None, p);
        toggle(ui, "LINK L/R", &p.link, s, None, p);
        toggle(ui, "PING-PONG", &p.ping_pong, s, None, p);
        toggle(ui, "FEED", &p.feed, s, Some(6), p);
        ui.label(
            RichText::new(if c.sync {
                format!(
                    "L {} / R {}",
                    DIVISIONS[c.left_div as usize].0,
                    DIVISIONS[if c.link { c.left_div } else { c.right_div } as usize].0
                )
            } else {
                format!(
                    "L {:.0} ms / R {:.0} ms",
                    c.time_ms,
                    if c.link { c.time_ms } else { c.right_ms }
                )
            })
            .monospace()
            .color(AMBER),
        );
    });
    let height = (ui.available_height() - 223.).max(210.);
    pad(ui, width, height, p, s);
    Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1., EDGE))
        .inner_margin(8.)
        .show(ui, |ui| {
            ui.set_min_width(width - 16.);
            ui.horizontal_top(|ui| {
                let kw = (width - 105.) / 6.;
                if c.sync {
                    knob(ui, "TIME", &p.left_div, s, kw);
                } else {
                    knob(ui, "TIME", &p.time_ms, s, kw);
                }
                for (name, param) in [
                    ("FEEDBACK", &p.feedback),
                    ("FILTER", &p.filter),
                    ("DRIVE", &p.drive),
                    ("MIX", &p.mix),
                    ("OUTPUT", &p.output_db),
                ] {
                    knob(ui, name, param, s, kw);
                }
                let peak = f32::from_bits(meter.load(Ordering::Relaxed)).clamp(0., 1.);
                ui.vertical(|ui| {
                    ui.label(RichText::new("OUT").size(10.).color(MUTED));
                    let (r, _) = ui.allocate_exact_size(vec2(9., 60.), Sense::hover());
                    ui.painter().rect_filled(r, 0., LCD);
                    ui.painter().rect_filled(
                        Rect::from_min_max(pos2(r.left(), r.bottom() - r.height() * peak), r.max),
                        0.,
                        AMBER,
                    );
                });
            });
        });
    ui.horizontal(|ui| {
        let w = (width - 30.) / 4.;
        momentary(ui, "THROW", &p.throw, 0, 2, state, p, s, gates, w,
            "Select gated input and feed while pressed. Release stops new input; dry continues. FEED restores continuous input.");
        let response = ui.add(
            Button::new(RichText::new("HOLD").monospace().size(17.)
                .color(if c.hold { LCD } else { VIOLET }))
                .fill(if c.hold { VIOLET } else { PANEL })
                .min_size(vec2(w, 42.)),
        );
        if response.clicked() {
            p.midi.set_value(3, -1.);
            set(&p.hold, !c.hold, s);
        }
        response.on_hover_text("Sustain captured echo without new input. Tone and time remain playable.");
        momentary(ui, "KILL", &p.kill, 1, 4, state, p, s, gates, w,
            "Fade and clear the wet echo in 5 ms. Dry continues; hold to keep echoes silent.");
        momentary(ui, "DUB", &p.dub, 2, 5, state, p, s, gates, w,
            "Momentary preset-defined feedback, drive and filter boost.");
    });
}
fn toggle(
    ui: &mut Ui,
    label: &str,
    param: &BoolParam,
    s: &ParamSetter,
    index: Option<usize>,
    p: &EchoParams,
) {
    let c = p.config();
    let value = match index {
        Some(6) => c.feed,
        Some(7) => c.bypass,
        _ => param.value(),
    };
    if ui
        .selectable_label(
            value,
            RichText::new(label)
                .monospace()
                .color(if value { AMBER } else { MUTED }),
        )
        .clicked()
    {
        if let Some(i) = index {
            p.midi.set_value(i, -1.);
        }
        set(param, !value, s);
    }
}
#[allow(clippy::too_many_arguments)]
fn momentary(
    ui: &mut Ui,
    label: &str,
    param: &BoolParam,
    slot: usize,
    index: usize,
    state: &mut UiState,
    p: &EchoParams,
    s: &ParamSetter,
    gates: &[AtomicBool; 3],
    width: f32,
    tip: &str,
) {
    let c = p.config();
    let active = match index {
        2 => c.throw,
        4 => c.kill,
        _ => c.dub,
    };
    let response = ui.add(
        Button::new(RichText::new(label).monospace().size(17.).color(if active {
            LCD
        } else {
            AMBER
        }))
        .fill(if active { AMBER } else { PANEL })
        .min_size(vec2(width, 42.))
        .sense(Sense::click_and_drag()),
    );
    if (response.is_pointer_button_down_on() || response.clicked()) && !state.pressed[slot] {
        state.pulse[slot] = response.clicked();
        p.midi.set_value(index, -1.);
        if index == 2 {
            p.midi.set_value(6, -1.);
            set(&p.feed, false, s);
        }
        if index == 4 {
            p.midi.set_value(3, -1.);
            set(&p.hold, false, s);
        }
        s.begin_set_parameter(param);
        s.set_parameter(param, true);
        state.pressed[slot] = true;
        gates[slot].store(true, Ordering::Relaxed);
    }
    response.on_hover_text(tip);
}
fn pad(ui: &mut Ui, width: f32, height: f32, p: &EchoParams, s: &ParamSetter) {
    let (r, response) = ui.allocate_exact_size(vec2(width, height), Sense::click_and_drag());
    let inner = r.shrink2(vec2(25., 27.));
    gradient(ui.painter(), r, LCD, Color32::from_rgb(34, 19, 39));
    ui.painter()
        .rect_stroke(r, 2., Stroke::new(2., EDGE), StrokeKind::Inside);
    for i in 0..=8 {
        let t = i as f32 / 8.;
        let color = if i == 4 {
            Color32::from_rgb(91, 67, 100)
        } else {
            Color32::from_rgb(51, 38, 59)
        };
        ui.painter().line_segment(
            [
                pos2(inner.left() + inner.width() * t, inner.top()),
                pos2(inner.left() + inner.width() * t, inner.bottom()),
            ],
            Stroke::new(1., color),
        );
        ui.painter().line_segment(
            [
                pos2(inner.left(), inner.top() + inner.height() * t),
                pos2(inner.right(), inner.top() + inner.height() * t),
            ],
            Stroke::new(1., color),
        );
    }
    let c = p.config();
    let point = pos2(
        inner.left() + c.x * inner.width(),
        inner.bottom() - c.y * inner.height(),
    );
    ui.painter().line_segment(
        [pos2(inner.left(), point.y), pos2(inner.right(), point.y)],
        Stroke::new(1., AMBER),
    );
    ui.painter().line_segment(
        [pos2(point.x, inner.top()), pos2(point.x, inner.bottom())],
        Stroke::new(1., VIOLET),
    );
    for radius in [20., 13.] {
        ui.painter()
            .circle_stroke(point, radius, Stroke::new(2., VIOLET.gamma_multiply(0.25)));
    }
    ui.painter().circle_filled(point, 6., AMBER);
    ui.painter()
        .circle_stroke(point, 10., Stroke::new(1.5, VIOLET));
    ui.painter().text(
        r.left_top() + vec2(12., 10.),
        Align2::LEFT_TOP,
        "FEEDBACK / INTENSITY +",
        FontId::monospace(11.),
        AMBER,
    );
    ui.painter().text(
        pos2(r.center().x, r.bottom() - 12.),
        Align2::CENTER_CENTER,
        "DARK < FILTER / OPEN > THIN",
        FontId::monospace(11.),
        VIOLET,
    );
    ui.painter().text(
        r.right_top() + vec2(-12., 10.),
        Align2::RIGHT_TOP,
        format!(
            "X {:03} / Y {:03} / FB {:03}%",
            (c.x * 100.) as i32,
            (c.y * 100.) as i32,
            (echogirl_core::dsp::performance_feedback(c) * 100.) as i32
        ),
        FontId::monospace(10.),
        MUTED,
    );
    if response.drag_started() {
        s.begin_set_parameter(&p.x);
        s.begin_set_parameter(&p.y);
    }
    if response.dragged() || response.clicked() {
        if response.clicked() {
            s.begin_set_parameter(&p.x);
            s.begin_set_parameter(&p.y);
        }
        if let Some(pos) = response.interact_pointer_pos() {
            p.midi.set_value(0, -1.);
            p.midi.set_value(1, -1.);
            s.set_parameter(&p.x, ((pos.x - inner.left()) / inner.width()).clamp(0., 1.));
            s.set_parameter(
                &p.y,
                ((inner.bottom() - pos.y) / inner.height()).clamp(0., 1.),
            );
        }
        if response.clicked() {
            s.end_set_parameter(&p.x);
            s.end_set_parameter(&p.y);
        }
    }
    if response.drag_stopped() {
        s.end_set_parameter(&p.x);
        s.end_set_parameter(&p.y);
    }
    if response.double_clicked() {
        p.midi.set_value(0, -1.);
        p.midi.set_value(1, -1.);
        set(&p.x, 0.5, s);
        set(&p.y, 0., s);
    }
    response.on_hover_text("X offsets FILTER around open center. Curved Y routes add feedback, drive and resonance to the base knobs. Double-click: open filter / low intensity.");
}
fn knob<P: Param>(ui: &mut Ui, label: &str, param: &P, s: &ParamSetter, width: f32) {
    ui.allocate_ui_with_layout(vec2(width, 98.), Layout::top_down(Align::Center), |ui| {
        let (r, response) = ui.allocate_exact_size(vec2(54., 56.), Sense::click_and_drag());
        let c = r.center();
        let p = ui.painter();
        p.circle_filled(c + vec2(0., 2.), 25., Color32::BLACK);
        p.circle_filled(c, 24., EDGE);
        p.circle_filled(c, 21., BG);
        p.circle_stroke(c, 18., Stroke::new(1., Color32::from_gray(83)));
        let v = param.unmodulated_normalized_value();
        let a = std::f32::consts::PI * (0.75 + v * 1.5);
        let d = vec2(a.cos(), a.sin());
        p.line_segment([c + d * 8., c + d * 18.], Stroke::new(3., AMBER));
        ui.label(RichText::new(label).monospace().size(10.).color(MUTED));
        ui.label(
            RichText::new(param.normalized_value_to_string(v, true))
                .monospace()
                .size(11.)
                .color(AMBER),
        );
        if response.drag_started() {
            s.begin_set_parameter(param);
        }
        if response.dragged() {
            let d = ui.input(|i| i.pointer.delta());
            s.set_parameter_normalized(param, (v + (d.x - d.y) * 0.004).clamp(0., 1.));
        }
        if response.drag_stopped() {
            s.end_set_parameter(param);
        }
        if response.double_clicked() {
            s.begin_set_parameter(param);
            s.set_parameter_normalized(param, param.default_normalized_value());
            s.end_set_parameter(param);
        }
        response.on_hover_text(format!(
            "{}: base value; X/Y routes add to it. Drag to adjust; double-click to reset.",
            param.name()
        ));
    });
}
fn slider(ui: &mut Ui, v: &mut f32, min: f32, max: f32, name: &str) {
    ui.add(Slider::new(v, min..=max).text(name));
}
fn choose<T: Copy + PartialEq + std::fmt::Debug>(ui: &mut Ui, id: &str, v: &mut T, values: &[T]) {
    ComboBox::from_id_salt(id)
        .selected_text(format!("{v:?}"))
        .show_ui(ui, |ui| {
            for item in values {
                ui.selectable_value(v, *item, format!("{item:?}"));
            }
        });
}
fn division(ui: &mut Ui, id: &str, v: &mut u8) {
    ComboBox::from_id_salt(id)
        .selected_text(DIVISIONS[*v as usize].0)
        .show_ui(ui, |ui| {
            for (i, (name, _)) in DIVISIONS.iter().enumerate() {
                ui.selectable_value(v, i as u8, *name);
            }
        });
}
fn edit(ui: &mut Ui, state: &mut UiState, p: &EchoParams, s: &ParamSetter) {
    ui.horizontal(|ui| {
        if ui
            .button(RichText::new(if state.dirty { "APPLY *" } else { "APPLY" }).color(AMBER))
            .clicked()
        {
            apply(state.draft.clone(), 10, state, p, s);
        }
        if ui.button("DUPLICATE").clicked() {
            state.draft.name = format!(
                "{} COPY",
                state.draft.name.chars().take(25).collect::<String>()
            );
            state.dirty = true;
        }
        if ui.button("RESET DEFAULT").clicked() {
            apply(factory(0), 0, state, p, s);
        }
    });
    let before = state.draft.clone();
    ScrollArea::vertical()
        .id_salt("editor")
        .max_height((ui.available_height() - 40.).max(300.))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("NAME");
                ui.text_edit_singleline(&mut state.draft.name);
            });
            let c = &mut state.draft.config;
            CollapsingHeader::new("TIME / ROUTING").default_open(true).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut c.sync, "Tempo sync");
                    ui.checkbox(&mut c.link, "Link L/R");
                    ui.checkbox(&mut c.ping_pong, "Ping-pong");
                    ui.checkbox(&mut c.feed, "Continuous input / FEED");
                });
                ui.horizontal(|ui| {
                    ui.label("LEFT");
                    division(ui, "ld", &mut c.left_div);
                    ui.label("RIGHT");
                    division(ui, "rd", &mut c.right_div);
                });
                slider(ui, &mut c.time_ms, 10., 12000., "Left / ms");
                slider(ui, &mut c.right_ms, 10., 12000., "Right / ms");
                slider(ui, &mut c.input_db, -24., 12., "Wet input gain / dB");
            });
            CollapsingHeader::new("FEEDBACK / COLOR").show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (i, name) in CHARACTERS.iter().enumerate() {
                        ui.selectable_value(&mut c.character, i as u8, *name);
                    }
                });
                slider(ui, &mut c.feedback, 0., 1.08, "Base feedback");
                slider(ui, &mut c.filter, 0., 1., "Base filter position");
                slider(ui, &mut c.drive, 0., 1., "Base drive");
                slider(ui, &mut c.low_cut, 0., 3000., "Feedback high-pass / Hz");
                slider(ui, &mut c.high_cut, 500., 20000., "Feedback low-pass / Hz");
                slider(ui, &mut c.resonance, 0., 0.85, "Resonance");
                slider(ui, &mut c.spring, 0., 0.65, "Spring-inspired diffusion");
                slider(ui, &mut c.mix, 0., 1., "Mix");
                slider(ui, &mut c.output_db, -24., 6., "Output / dB");
            });
            CollapsingHeader::new("WOW / MODULATION").show(ui, |ui| {
                slider(ui, &mut c.wow, 0., 1., "Wow / flutter depth");
                ui.checkbox(&mut c.mod_sync, "Host tempo sync");
                slider(ui, &mut c.mod_rate, 0.05, 8., "Free rate / Hz");
                slider(ui, &mut c.mod_beats, 0.125, 16., "Quarter notes / cycle");
            });
            CollapsingHeader::new("X/Y MAPPINGS").show(ui, |ui| {
                ui.label("X is bipolar around center; Y starts at zero. Signed depth adds to the base knob. Curve preserves endpoints.");
                for (i, m) in c.mappings.iter_mut().enumerate() {
                    ui.push_id(i, |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut m.enabled, format!("{:02}", i + 1));
                            choose(ui, "src", &mut m.source, &[Source::X, Source::Y]);
                            choose(ui, "dst", &mut m.target, &Target::ALL);
                            ui.add(DragValue::new(&mut m.amount).range(-2.0..=2.0).speed(0.01).prefix("DEPTH "));
                            ui.add(DragValue::new(&mut m.curve).range(0.2..=5.0).speed(0.01).prefix("CURVE "));
                        });
                    });
                }
            });
            CollapsingHeader::new("DUB / MIDI").show(ui, |ui| {
                slider(ui, &mut c.dub_feedback, 0., 0.4, "Added feedback");
                slider(ui, &mut c.dub_drive, 0., 0.7, "Added drive");
                slider(ui, &mut c.dub_filter, -0.5, 0.5, "Filter shift");
                for (name, cc) in ["X", "Y", "THROW", "HOLD", "KILL", "DUB", "FEED", "BYPASS"]
                    .into_iter().zip(c.midi_cc.iter_mut())
                {
                    ui.horizontal(|ui| {
                        ui.label(name);
                        ui.add(DragValue::new(cc).range(0..=119));
                    });
                }
                ui.label("Unique assignments; host must route MIDI CC to the effect.");
            });
            CollapsingHeader::new("PRESET NOTES").show(ui, |ui| {
                ui.text_edit_multiline(&mut state.draft.description);
            });
            ui.separator();
            library(ui, state, p, s);
        });

    if before != state.draft {
        state.dirty = true;
    }
}
fn slug(s: &str) -> String {
    let v: String = s
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .collect();
    if v.is_empty() { "User".into() } else { v }
}
fn read(path: &Path) -> Result<Preset, String> {
    let mut text = String::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take((preset::MAX_BYTES + 1) as u64)
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
fn scan(root: &Path) -> Vec<PathBuf> {
    let mut v: Vec<_> = fs::read_dir(root)
        .into_iter()
        .flatten()
        .take(1024)
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.to_string_lossy().ends_with(".echogirl.json"))
        .collect();
    v.sort();
    v
}
fn library(ui: &mut Ui, state: &mut UiState, p: &EchoParams, s: &ParamSetter) {
    ui.label(RichText::new("USER PRESETS").color(AMBER));
    ui.label(
        RichText::new(state.library.display().to_string())
            .small()
            .color(MUTED),
    );
    ui.horizontal(|ui| {
        if ui.button("SAVE AS NEW").clicked() {
            let mut preset = state.draft.clone();
            preset.config.release_buttons();
            let path = state
                .library
                .join(format!("{}.echogirl.json", slug(&preset.name)));
            state.status = match fs::create_dir_all(&state.library)
                .map_err(|e| e.to_string())
                .and_then(|_| write(&path, &preset))
            {
                Ok(()) => format!("Saved {}", path.display()),
                Err(e) => format!("Save failed: {e}. Rename to save a new version."),
            };
            state.users = scan(&state.library);
        }
        if ui.button("REFRESH").clicked() {
            state.users = scan(&state.library);
        }
        let mut selected = None;
        ComboBox::from_id_salt("user")
            .selected_text("LOAD USER")
            .show_ui(ui, |ui| {
                for path in &state.users {
                    if ui
                        .selectable_label(
                            false,
                            path.file_stem().unwrap_or_default().to_string_lossy(),
                        )
                        .clicked()
                    {
                        selected = Some(path.clone());
                    }
                }
            });
        if let Some(path) = selected {
            match read(&path) {
                Ok(preset) => apply(preset, 10, state, p, s),
                Err(e) => state.status = e,
            }
        }
    });
    ui.horizontal(|ui| {
        ui.add(
            TextEdit::singleline(&mut state.path)
                .desired_width((ui.available_width() - 170.).max(200.))
                .hint_text("Full path to .echogirl.json"),
        );
        if ui.button("IMPORT").clicked() {
            match read(Path::new(&state.path)) {
                Ok(preset) => apply(preset, 10, state, p, s),
                Err(e) => state.status = e,
            }
        }
        if ui.button("EXPORT").clicked() {
            let mut preset = state.draft.clone();
            preset.config.release_buttons();
            state.status = match write(Path::new(&state.path), &preset) {
                Ok(()) => "Exported".into(),
                Err(e) => e,
            };
        }
    });
}
