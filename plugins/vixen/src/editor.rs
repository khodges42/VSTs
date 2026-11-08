// SPDX-License-Identifier: GPL-3.0-or-later
use crate::VixenParams;
use crossbeam_queue::ArrayQueue;
use nih_plug::prelude::*;
use nih_plug_egui::{
    egui::{self, *},
    resizable_window::ResizableWindow,
};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};
use vixen_core::{Event, Patch, factory, patch::*};

const BG: Color32 = Color32::from_rgb(45, 48, 45);
const PANEL: Color32 = Color32::from_rgb(61, 65, 61);
const PAD: Color32 = Color32::from_rgb(20, 24, 20);
const GREEN: Color32 = Color32::from_rgb(165, 230, 80);
const TEXT: Color32 = Color32::from_rgb(232, 236, 230);
const MUTED: Color32 = Color32::from_rgb(177, 187, 175);
const BORDER: Color32 = Color32::from_rgb(109, 119, 107);

pub struct UiState {
    edit: bool,
    draft: Patch,
    revision: u64,
    factory: usize,
    status: String,
    path: String,
    library: PathBuf,
    users: Vec<PathBuf>,
    note: Option<u8>,
    dirty: bool,
}
impl UiState {
    pub fn new() -> Self {
        let library = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("VOIDMUTT/VIXEN/Patches");
        Self {
            edit: false,
            draft: factory::patch(0),
            revision: 0,
            factory: 0,
            status: String::new(),
            path: String::new(),
            library,
            users: Vec::new(),
            note: None,
            dirty: false,
        }
    }
}
pub fn setup(ctx: &Context, _: &mut UiState) {
    let mut style = (*ctx.style()).clone();
    style.visuals = Visuals::dark();
    style.visuals.panel_fill = BG;
    style.visuals.window_fill = PANEL;
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.extreme_bg_color = PAD;
    style.visuals.faint_bg_color = PANEL;
    style.visuals.selection.bg_fill = Color32::from_rgb(56, 78, 37);
    style.visuals.selection.stroke = Stroke::new(1., GREEN);
    style.visuals.hyperlink_color = GREEN;
    style.visuals.widgets.inactive.bg_fill = PANEL;
    style.visuals.widgets.inactive.weak_bg_fill = PANEL;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1., BORDER);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(79, 87, 77);
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(79, 87, 77);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1., GREEN);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(56, 78, 37);
    style.visuals.widgets.active.weak_bg_fill = Color32::from_rgb(56, 78, 37);
    style.visuals.widgets.active.bg_stroke = Stroke::new(1., GREEN);
    style.visuals.widgets.active.fg_stroke = Stroke::new(2., GREEN);
    style.visuals.widgets.open.bg_fill = PANEL;
    style.visuals.widgets.open.weak_bg_fill = PANEL;
    style.visuals.widgets.open.bg_stroke = Stroke::new(1., GREEN);
    for widget in [
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        widget.corner_radius = CornerRadius::ZERO;
    }
    style.visuals.window_corner_radius = CornerRadius::ZERO;
    style.spacing.item_spacing = vec2(10., 8.);
    style.spacing.button_padding = vec2(12., 7.);
    style
        .text_styles
        .insert(TextStyle::Body, FontId::monospace(13.));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::monospace(12.));
    ctx.set_style(style);
}
fn set(param: &FloatParam, value: f32, setter: &ParamSetter) {
    setter.begin_set_parameter(param);
    setter.set_parameter(param, value);
    setter.end_set_parameter(param);
}
fn notify(p: &VixenParams, setter: &ParamSetter) {
    setter.begin_set_parameter(&p.change);
    setter.set_parameter(&p.change, !p.change.value());
    setter.end_set_parameter(&p.change);
}
fn load(patch: Patch, state: &mut UiState, p: &VixenParams, setter: &ParamSetter) {
    match p.patch.publish(patch.clone()) {
        Ok(()) => {
            let c = patch.controls;
            for (param, v) in [
                (&p.x, c.x),
                (&p.y, c.y),
                (&p.cutoff, c.cutoff),
                (&p.resonance, c.resonance),
                (&p.drive, c.drive),
                (&p.decay, c.decay),
                (&p.glide, c.glide),
            ] {
                set(param, v, setter);
            }
            // Output is deliberately global when browsing; session state still persists it.
            p.midi_x.store(-1., Ordering::Relaxed);
            p.midi_y.store(-1., Ordering::Relaxed);
            notify(p, setter);
            state.draft = patch;
            state.revision = p.patch.revision();
            state.dirty = false;
            state.status.clear();
        }
        Err(e) => state.status = e,
    }
}
pub fn draw(
    ctx: &Context,
    setter: &ParamSetter,
    state: &mut UiState,
    p: &Arc<VixenParams>,
    preview: &ArrayQueue<Event>,
    meter: &AtomicU32,
) {
    if p.patch.revision() != state.revision {
        state.draft = p.patch.read();
        if let Some(index) = factory::NAMES
            .iter()
            .position(|name| *name == state.draft.name)
        {
            state.factory = index;
        }
        state.revision = p.patch.revision();
        state.dirty = false;
    }
    ResizableWindow::new("vixen-window")
        .min_size(vec2(840., 660.))
        .show(ctx, p.editor.as_ref(), |ui| {
            ui.spacing_mut().item_spacing = vec2(10., 8.);
            header(ui);
            ui.add_space(4.);
            egui::Frame::new()
                .fill(PANEL)
                .stroke(Stroke::new(1., BORDER))
                .inner_margin(12.)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button("<")
                            .on_hover_text("Previous factory patch")
                            .clicked()
                        {
                            state.factory = (state.factory + 9) % 10;
                            load(factory::patch(state.factory), state, p, setter);
                        }
                        let mut selected = None;
                        ComboBox::from_id_salt("factory")
                            .width(210.)
                            .selected_text(RichText::new(&state.draft.name).color(GREEN).strong())
                            .show_ui(ui, |ui| {
                                for (i, name) in factory::NAMES.iter().enumerate() {
                                    if ui
                                        .selectable_label(
                                            state.factory == i,
                                            format!("{:02}  {name}", i + 1),
                                        )
                                        .clicked()
                                    {
                                        selected = Some(i);
                                    }
                                }
                            });
                        if let Some(selected) = selected {
                            state.factory = selected;
                            load(factory::patch(selected), state, p, setter);
                        }
                        if ui.button(">").on_hover_text("Next factory patch").clicked() {
                            state.factory = (state.factory + 1) % 10;
                            load(factory::patch(state.factory), state, p, setter);
                        }
                        ui.label(RichText::new(&state.draft.category).color(MUTED).size(11.));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.selectable_label(state.edit, "EDIT").clicked() {
                                state.edit = !state.edit;
                            }
                            if ui.selectable_label(!state.edit, "PERFORM").clicked() {
                                state.edit = false;
                            }
                        });
                    });
                });
            ui.add_space(4.);
            if state.edit {
                edit(ui, state, p, setter);
            } else {
                perform(ui, state, p, setter, preview, meter);
            }
            ui.add_space(4.);
            if !state.status.is_empty() {
                ui.label(RichText::new(&state.status).size(11.).color(GREEN));
            }
            let error = p.patch.error();
            if !error.is_empty() {
                ui.colored_label(GREEN, format!("State rejected: {error}"));
            }
        });
    if (!ctx.input(|i| i.pointer.primary_down()) || state.edit)
        && let Some(note) = state.note.take()
    {
        let _ = preview.push(Event::NoteOff {
            note,
            channel: 15,
            id: Some(-(note as i32) - 1),
        });
    }
    ctx.request_repaint_after(std::time::Duration::from_millis(33));
}
fn header(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 64.), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 0., PANEL);
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(2., GREEN),
    );
    painter.text(
        rect.min + vec2(12., 30.),
        Align2::LEFT_CENTER,
        "VIXEN",
        FontId::monospace(42.),
        TEXT,
    );
    painter.text(
        rect.right_top() + vec2(-14., 32.),
        Align2::RIGHT_CENTER,
        "BY VOIDMUTT",
        FontId::monospace(13.),
        MUTED,
    );
}
fn perform(
    ui: &mut Ui,
    state: &mut UiState,
    p: &VixenParams,
    setter: &ParamSetter,
    preview: &ArrayQueue<Event>,
    meter: &AtomicU32,
) {
    let width = ui.available_width();
    ui.add(
        Label::new(
            RichText::new(format!(
                "{} / {}",
                factory::NAMES
                    .iter()
                    .position(|name| *name == state.draft.name)
                    .map(|i| format!("{:02}", i + 1))
                    .unwrap_or_else(|| "USR".into()),
                state.draft.name
            ))
            .monospace()
            .size(25.)
            .strong()
            .color(TEXT),
        )
        .truncate(),
    )
    .on_hover_text(&state.draft.name);
    ScrollArea::vertical()
        .id_salt("description")
        .max_height(34.)
        .show(ui, |ui| {
            ui.label(
                RichText::new(&state.draft.description)
                    .size(12.)
                    .color(MUTED),
            );
        });
    let padheight = (ui.available_height() - 208.).max(208.);
    xy_pad(ui, width, padheight, &state.draft, p, setter);
    ScrollArea::vertical()
        .id_salt("performance-notes")
        .max_height(32.)
        .show(ui, |ui| {
            ui.label(
                RichText::new(&state.draft.performance)
                    .size(12.)
                    .color(MUTED),
            );
        });
    ui.add_space(5.);
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1., BORDER))
        .inner_margin(8.)
        .show(ui, |ui| {
            ui.set_min_width(width - 16.);
            ui.horizontal_top(|ui| {
                let kw = ((width - 98.) / 6.).max(90.);
                for (label, param) in [
                    ("BRIGHTNESS", &p.cutoff),
                    ("RESONANCE", &p.resonance),
                    ("DRIVE", &p.drive),
                    ("CHARACTER", &p.decay),
                    ("GLIDE", &p.glide),
                    ("OUTPUT", &p.output),
                ] {
                    knob(ui, label, param, setter, kw);
                }
                ui.vertical(|ui| {
                    let peak = f32::from_bits(meter.load(Ordering::Relaxed));
                    ui.label(RichText::new("OUT").monospace().size(10.).color(MUTED));
                    let (r, _) = ui.allocate_exact_size(vec2(10., 63.), Sense::hover());
                    ui.painter().rect_filled(r, 0., PAD);
                    let h = r.height() * peak.clamp(0., 1.);
                    ui.painter().rect_filled(
                        Rect::from_min_max(pos2(r.left(), r.bottom() - h), r.max),
                        0.,
                        GREEN,
                    );
                });
            });
        });
    ui.horizontal(|ui| {
        ui.label(RichText::new("AUDITION").size(10.).color(MUTED));
        for (note, name) in [
            (48, "C3"),
            (50, "D"),
            (51, "Eb"),
            (53, "F"),
            (55, "G"),
            (58, "Bb"),
            (60, "C4"),
            (63, "Eb"),
            (67, "G"),
        ] {
            let response = ui.add(
                Button::new(
                    RichText::new(name)
                        .monospace()
                        .color(if state.note == Some(note) { BG } else { TEXT }),
                )
                .fill(if state.note == Some(note) {
                    GREEN
                } else {
                    PANEL
                })
                .min_size(vec2(37., 28.))
                .sense(Sense::click_and_drag()),
            );
            if response.is_pointer_button_down_on() && state.note != Some(note) {
                if let Some(old) = state.note {
                    let _ = preview.push(Event::NoteOff {
                        note: old,
                        channel: 15,
                        id: Some(-(old as i32) - 1),
                    });
                }
                if preview
                    .push(Event::NoteOn {
                        note,
                        channel: 15,
                        velocity: 0.9,
                        id: Some(-(note as i32) - 1),
                    })
                    .is_ok()
                {
                    state.note = Some(note);
                }
            }
        }
        if ui.button("PANIC").clicked() {
            let _ = preview.push(Event::AllOff);
            state.note = None;
        }
        ui.label(
            RichText::new(format!(
                "CC {} / {}",
                state.draft.synth.x_cc, state.draft.synth.y_cc
            ))
            .size(10.)
            .color(MUTED),
        );
    });
}
fn xy_pad(
    ui: &mut Ui,
    width: f32,
    height: f32,
    patch: &Patch,
    p: &VixenParams,
    setter: &ParamSetter,
) {
    let (r, response) = ui.allocate_exact_size(vec2(width, height), Sense::click_and_drag());
    let pad = r.shrink2(vec2(27., 24.));
    let painter = ui.painter();
    painter.rect_filled(r, 0., PAD);
    painter.rect_stroke(r, 0., Stroke::new(2., BORDER), StrokeKind::Inside);
    for i in 0..=8 {
        let t = i as f32 / 8.;
        let color = if i == 4 {
            Color32::from_rgb(74, 83, 70)
        } else {
            Color32::from_rgb(44, 51, 42)
        };
        painter.line_segment(
            [
                pos2(pad.left() + pad.width() * t, pad.top()),
                pos2(pad.left() + pad.width() * t, pad.bottom()),
            ],
            Stroke::new(1., color),
        );
        painter.line_segment(
            [
                pos2(pad.left(), pad.top() + pad.height() * t),
                pos2(pad.right(), pad.top() + pad.height() * t),
            ],
            Stroke::new(1., color),
        );
    }
    let [x, y] = p.xy();
    let point = pos2(
        pad.left() + pad.width() * x,
        pad.bottom() - pad.height() * y,
    );
    painter.line_segment(
        [pos2(point.x, pad.top()), pos2(point.x, pad.bottom())],
        Stroke::new(1., GREEN),
    );
    painter.line_segment(
        [pos2(pad.left(), point.y), pos2(pad.right(), point.y)],
        Stroke::new(1., GREEN),
    );
    painter.rect_filled(Rect::from_center_size(point, vec2(20., 20.)), 0., PAD);
    painter.rect_stroke(
        Rect::from_center_size(point, vec2(20., 20.)),
        0.,
        Stroke::new(1., GREEN),
        StrokeKind::Inside,
    );
    painter.rect_filled(Rect::from_center_size(point, vec2(8., 8.)), 0., GREEN);
    painter.text(
        pos2(r.center().x, r.bottom() - 11.),
        Align2::CENTER_CENTER,
        format!("-    {}    +", patch.x_label),
        FontId::monospace(11.),
        GREEN,
    );
    painter.text(
        pos2(r.left() + 10., r.top() + 10.),
        Align2::LEFT_TOP,
        format!("+ {}", patch.y_label),
        FontId::monospace(11.),
        GREEN,
    );
    painter.text(
        r.right_top() + vec2(-12., 10.),
        Align2::RIGHT_TOP,
        format!("X {:03} / Y {:03}", (x * 100.) as u32, (y * 100.) as u32),
        FontId::monospace(10.),
        MUTED,
    );
    if response.drag_started() {
        setter.begin_set_parameter(&p.x);
        setter.begin_set_parameter(&p.y);
    }
    if response.dragged() || response.clicked() {
        if response.clicked() {
            setter.begin_set_parameter(&p.x);
            setter.begin_set_parameter(&p.y);
        }
        if let Some(pos) = response.interact_pointer_pos() {
            p.midi_x.store(-1., Ordering::Relaxed);
            p.midi_y.store(-1., Ordering::Relaxed);
            setter.set_parameter(&p.x, ((pos.x - pad.left()) / pad.width()).clamp(0., 1.));
            setter.set_parameter(&p.y, ((pad.bottom() - pos.y) / pad.height()).clamp(0., 1.));
        }
        if response.clicked() {
            setter.end_set_parameter(&p.x);
            setter.end_set_parameter(&p.y);
        }
    }
    if response.drag_stopped() {
        setter.end_set_parameter(&p.x);
        setter.end_set_parameter(&p.y);
    }
    if response.double_clicked() {
        set(&p.x, 0.5, setter);
        set(&p.y, 0.5, setter);
        p.midi_x.store(-1., Ordering::Relaxed);
        p.midi_y.store(-1., Ordering::Relaxed);
    }
    response.on_hover_text(format!("X: {} / Y: {}. Drag to perform; double-click to center. MIDI CC {}/{}. Edit all routes in EDIT mode.",patch.x_label,patch.y_label,patch.synth.x_cc,patch.synth.y_cc));
}
fn knob(ui: &mut Ui, label: &str, param: &FloatParam, setter: &ParamSetter, width: f32) {
    ui.allocate_ui(vec2(width, 91.), |ui| {
        ui.vertical_centered(|ui| {
            let (r, response) = ui.allocate_exact_size(vec2(52., 58.), Sense::click_and_drag());
            let center = r.center() - vec2(0., 3.);
            let painter = ui.painter();
            let value = param.unmodulated_normalized_value();
            let face = Rect::from_center_size(center, vec2(50., 50.));
            painter.rect_filled(face, 0., PAD);
            painter.rect_stroke(face, 0., Stroke::new(1., BORDER), StrokeKind::Inside);
            painter.circle_stroke(center, 18., Stroke::new(2., BORDER));
            let a = std::f32::consts::PI * 0.75 + value * std::f32::consts::PI * 1.5;
            let dir = vec2(a.cos(), a.sin());
            painter.line_segment(
                [center + dir * 9., center + dir * 18.],
                Stroke::new(3., GREEN),
            );
            ui.label(RichText::new(label).monospace().size(10.).color(MUTED));
            ui.label(
                RichText::new(format!("{:.2}", param.value()))
                    .monospace()
                    .size(11.)
                    .color(GREEN),
            );
            if response.drag_started() {
                setter.begin_set_parameter(param);
            }
            if response.dragged() {
                let delta = ui.input(|i| i.pointer.delta());
                setter.set_parameter_normalized(
                    param,
                    (value + (delta.x - delta.y) * 0.005).clamp(0., 1.),
                );
            }
            if response.drag_stopped() {
                setter.end_set_parameter(param);
            }
            if response.double_clicked() {
                set(param, param.default_plain_value(), setter);
            }
            response.on_hover_text(format!(
                "{}: drag vertically or horizontally. Double-click to reset.",
                param.name()
            ));
        });
    });
}
fn slider(ui: &mut Ui, value: &mut f32, min: f32, max: f32, name: &str) {
    ui.add(Slider::new(value, min..=max).text(name));
}
fn choice<T: PartialEq + Copy + std::fmt::Debug>(ui: &mut Ui, id: &str, v: &mut T, options: &[T]) {
    ComboBox::from_id_salt(id)
        .selected_text(format!("{v:?}"))
        .show_ui(ui, |ui| {
            for option in options {
                ui.selectable_value(v, *option, format!("{option:?}"));
            }
        });
}
fn envelope(ui: &mut Ui, e: &mut Envelope) {
    slider(ui, &mut e.attack, 0.001, 5., "Attack / s");
    slider(ui, &mut e.decay, 0.001, 5., "Decay / s");
    slider(ui, &mut e.sustain, 0., 1., "Sustain");
    slider(ui, &mut e.release, 0.002, 5., "Release / s");
}
fn edit(ui: &mut Ui, state: &mut UiState, p: &VixenParams, setter: &ParamSetter) {
    ui.horizontal(|ui|{
        if ui.button(RichText::new(if state.dirty{"APPLY PATCH *"}else{"APPLY PATCH"}).color(GREEN)).on_hover_text("Apply the draft with a short de-click; ends old notes. Factory source stays protected.").clicked(){state.draft.controls=p.controls();load(state.draft.clone(),state,p,setter);}
        if ui.button("DUPLICATE").clicked(){state.draft.name=format!("{} USER",state.draft.name.chars().take(70).collect::<String>());state.draft.author="USER".into();state.dirty=true;}
        if ui.button("RESTORE FACTORY").clicked(){load(factory::patch(state.factory),state,p,setter);}
    });
    let before = state.draft.clone();
    ScrollArea::vertical().id_salt("patch-lab").max_height((ui.available_height()-48.).max(250.)).show(ui,|ui|{
        ui.horizontal(|ui|{ui.label("NAME");ui.text_edit_singleline(&mut state.draft.name);ui.label("BY");ui.add(TextEdit::singleline(&mut state.draft.author).desired_width(100.));});
        ui.horizontal(|ui|{ui.label("CATEGORY");ui.text_edit_singleline(&mut state.draft.category);});
        CollapsingHeader::new("01 / OSCILLATORS").default_open(true).show(ui,|ui|{
            ui.columns(3,|cols|{for (i,col) in cols.iter_mut().enumerate(){let o=&mut state.draft.synth.oscillators[i];col.label(RichText::new(format!("OSC {}",i+1)).color(GREEN));choice(col,&format!("wave-{i}"),&mut o.wave,&[Wave::Saw,Wave::Pulse,Wave::Triangle,Wave::Sine,Wave::Noise]);slider(col,&mut o.level,0.,1.,"Level");slider(col,&mut o.semitones,-36.,36.,"Semitones");col.add(Slider::new(&mut o.unison,1..=7).text("Stack"));slider(col,&mut o.detune,0.,1.,"Detune");slider(col,&mut o.width,0.,1.,"Spread");slider(col,&mut o.pulse_width,0.05,0.95,"Pulse width");slider(col,&mut o.pwm,0.,0.45,"LFO 1 PWM");}});
            slider(ui,&mut state.draft.synth.phase_mod,0.,3.,"Serial phase modulation");
        });
        CollapsingHeader::new("02 / ENVELOPES").show(ui,|ui|{ui.columns(3,|cols|{cols[0].label("AMPLITUDE");envelope(&mut cols[0],&mut state.draft.synth.amp);cols[1].label("FILTER");envelope(&mut cols[1],&mut state.draft.synth.filter_env);cols[2].label("PITCH");envelope(&mut cols[2],&mut state.draft.synth.pitch_env);});slider(ui,&mut state.draft.synth.pitch_amount,-36.,36.,"Pitch amount / semitones");});
        CollapsingHeader::new("03 / FILTER + VOICES").show(ui,|ui|{
            let s=&mut state.draft.synth;choice(ui,"filter-mode",&mut s.filter_mode,&[FilterMode::Low12,FilterMode::Low24,FilterMode::High12,FilterMode::Band12]);ui.add(Slider::new(&mut s.cutoff,25. ..=20000.).logarithmic(true).text("Cutoff / Hz"));slider(ui,&mut s.resonance,0.,0.95,"Resonance");slider(ui,&mut s.filter_amount,-8.,8.,"Envelope / octaves");slider(ui,&mut s.keytrack,0.,1.,"Key tracking");slider(ui,&mut s.drive,0.,1.,"Saturation");slider(ui,&mut s.velocity,0.,1.,"Velocity -> amplitude");slider(ui,&mut s.accent,0.,1.,"High velocity -> filter accent");choice(ui,"voice-mode",&mut s.voice_mode,&[VoiceMode::Poly,VoiceMode::Mono,VoiceMode::Legato]);ui.add(Slider::new(&mut s.voices,1..=32).text("Polyphony"));slider(ui,&mut s.glide,0.,2.,"Glide / s");slider(ui,&mut s.bend_range,0.,24.,"Bend / semitones");slider(ui,&mut s.gain,0.,1.,"Patch trim");
        });
        CollapsingHeader::new("04 / LFOs").show(ui,|ui|{ui.columns(2,|cols|{for (i,col) in cols.iter_mut().enumerate(){let l=&mut state.draft.synth.lfos[i];col.label(format!("LFO {}",i+1));choice(col,&format!("lfo-{i}"),&mut l.shape,&[LfoShape::Sine,LfoShape::Triangle,LfoShape::Saw,LfoShape::Square]);col.checkbox(&mut l.sync,"Host tempo sync");slider(col,&mut l.rate,0.01,30.,"Rate / Hz");slider(col,&mut l.beats,0.0625,16.,"Quarter notes / cycle");slider(col,&mut l.pitch,0.,2.,"Pitch / semitones");slider(col,&mut l.filter,0.,4.,"Filter / octaves");slider(col,&mut l.pwm,0.,0.45,"PWM");}});});
        CollapsingHeader::new("05 / EFFECTS").show(ui,|ui|{let e=&mut state.draft.synth.effects;slider(ui,&mut e.chorus,0.,1.,"Chorus");slider(ui,&mut e.chorus_rate,0.05,4.,"Chorus rate / Hz");slider(ui,&mut e.delay,0.,0.7,"Delay mix");slider(ui,&mut e.delay_beats,0.0625,2.,"Delay / quarter notes");slider(ui,&mut e.feedback,0.,0.75,"Feedback");slider(ui,&mut e.reverb,0.,0.6,"Room");});
        CollapsingHeader::new("06 / X/Y + WHEEL ROUTING").show(ui,|ui|{
            ui.label("X/Y routes are centered at 0.5. Wheel starts at zero. Signed amount sets polarity; curve 1 is linear.");
            ui.horizontal(|ui|{ui.label("X LABEL");ui.text_edit_singleline(&mut state.draft.x_label);ui.label("Y LABEL");ui.text_edit_singleline(&mut state.draft.y_label);});
            ui.horizontal(|ui|{ui.add(DragValue::new(&mut state.draft.synth.x_cc).range(0..=119).prefix("X CC "));ui.add(DragValue::new(&mut state.draft.synth.y_cc).range(0..=119).prefix("Y CC "));ui.label("Distinct CCs; 1 and 64 reserved.");});
            for (i,m) in state.draft.synth.mappings.iter_mut().enumerate(){ui.push_id(i,|ui|{ui.horizontal(|ui|{ui.checkbox(&mut m.enabled,format!("{:02}",i+1));choice(ui,"source",&mut m.source,&[Source::X,Source::Y,Source::Wheel]);choice(ui,"target",&mut m.target,&Target::ALL);ui.add(DragValue::new(&mut m.amount).range(-4. ..=4.).speed(0.01).prefix("DEPTH "));ui.add(DragValue::new(&mut m.curve).range(0.2..=5.).speed(0.01).prefix("CURVE "));});});}
        });
        CollapsingHeader::new("07 / PATCH NOTES").show(ui,|ui|{ui.label("Sound description");ui.text_edit_multiline(&mut state.draft.description);ui.label("Performance / X/Y behavior");ui.text_edit_multiline(&mut state.draft.performance);});
        ui.separator();library(ui,state,p,setter);
    });
    if state.draft != before {
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
fn write_new(path: &Path, patch: &Patch) -> Result<(), String> {
    let text = patch.to_json()?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(text.as_bytes()).map_err(|e| e.to_string())
}
fn read_patch(path: &Path) -> Result<Patch, String> {
    let mut text = String::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take((MAX_PATCH_BYTES + 1) as u64)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    Patch::from_json(&text)
}
fn scan(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.take(1024).flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(children) = fs::read_dir(path) {
                    result.extend(
                        children
                            .take(1024 - result.len())
                            .flatten()
                            .map(|e| e.path())
                            .filter(|p| p.to_string_lossy().ends_with(".vixen.json")),
                    );
                }
            } else if path.to_string_lossy().ends_with(".vixen.json") {
                result.push(path);
            }
            if result.len() >= 1024 {
                break;
            }
        }
    }
    result.sort();
    result
}
fn library(ui: &mut Ui, state: &mut UiState, p: &VixenParams, setter: &ParamSetter) {
    ui.label(
        RichText::new("USER BANK / PORTABLE PATCHES")
            .color(GREEN)
            .monospace(),
    );
    ui.label(
        RichText::new(state.library.display().to_string())
            .small()
            .color(MUTED),
    );
    ui.horizontal(|ui| {
        if ui.button("SAVE AS NEW USER PATCH").clicked() {
            let mut patch = state.draft.clone();
            patch.controls = p.controls();
            let dir = state.library.join(slug(&patch.category));
            let path = dir.join(format!("{}.vixen.json", slug(&patch.name)));
            state.status = match fs::create_dir_all(&dir)
                .map_err(|e| e.to_string())
                .and_then(|_| write_new(&path, &patch))
            {
                Ok(()) => format!("SAVED {}", path.display()),
                Err(e) => format!("Save failed: {e}. Rename to save a new version."),
            };
            state.users = scan(&state.library);
        }
        if ui.button("REFRESH BANK").clicked() {
            state.users = scan(&state.library);
        }
        let mut chosen = None;
        ComboBox::from_id_salt("users")
            .selected_text("LOAD USER PATCH")
            .show_ui(ui, |ui| {
                for path in &state.users {
                    if ui
                        .selectable_label(
                            false,
                            path.file_stem().unwrap_or_default().to_string_lossy(),
                        )
                        .clicked()
                    {
                        chosen = Some(path.clone());
                    }
                }
            });
        if let Some(path) = chosen {
            match read_patch(&path) {
                Ok(patch) => load(patch, state, p, setter),
                Err(e) => state.status = e,
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label("FILE");
        ui.add(
            TextEdit::singleline(&mut state.path)
                .desired_width((ui.available_width() - 190.).max(180.))
                .hint_text("Full path to a .vixen.json file"),
        );
        if ui.button("IMPORT").clicked() {
            match read_patch(Path::new(&state.path)) {
                Ok(patch) => load(patch, state, p, setter),
                Err(e) => state.status = format!("Import failed: {e}"),
            }
        }
        if ui.button("EXPORT").clicked() {
            let mut patch = state.draft.clone();
            patch.controls = p.controls();
            state.status = match write_new(Path::new(&state.path), &patch) {
                Ok(()) => "EXPORTED".into(),
                Err(e) => format!("Export failed: {e}"),
            };
        }
    });
}
