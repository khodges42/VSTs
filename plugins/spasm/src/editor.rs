// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    library,
    params::SpasmParams,
    shared::{Shared, WAVE_SIZE},
};
use nih_plug::prelude::*;
use nih_plug_egui::{egui::*, resizable_window::ResizableWindow, widgets::ParamSlider};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
};
pub const GREEN: Color32 = Color32::from_rgb(202, 246, 54);
pub const MAGENTA: Color32 = Color32::from_rgb(250, 57, 184);
pub const MUTED: Color32 = Color32::from_rgb(169, 161, 183);
const BG: Color32 = Color32::from_rgb(20, 16, 28);
const LCD: Color32 = Color32::from_rgb(9, 13, 14);
const PANEL: Color32 = Color32::from_rgb(48, 39, 60);
const SILVER: Color32 = Color32::from_rgb(191, 192, 204);
pub struct UiState {
    pressed: [bool; 8],
    pulse: [bool; 8],
    pub name: String,
    pub notes: String,
    pub last_name: String,
    pub status: String,
    pub path: String,
    pub users: Vec<PathBuf>,
}
impl Default for UiState {
    fn default() -> Self {
        Self {
            pressed: [false; 8],
            pulse: [false; 8],
            name: String::new(),
            notes: String::new(),
            last_name: String::new(),
            status: String::new(),
            path: String::new(),
            users: library::scan(&library::root()),
        }
    }
}
pub fn setup(ctx: &Context, _: &mut UiState) {
    let mut s = (*ctx.style()).clone();
    s.visuals = Visuals::dark();
    s.visuals.panel_fill = BG;
    s.visuals.window_fill = PANEL;
    s.visuals.extreme_bg_color = LCD;
    s.visuals.override_text_color = Some(Color32::from_rgb(231, 233, 217));
    s.visuals.selection.bg_fill = Color32::from_rgb(88, 37, 100);
    s.visuals.selection.stroke = Stroke::new(1., GREEN);
    for v in [
        &mut s.visuals.widgets.inactive,
        &mut s.visuals.widgets.hovered,
        &mut s.visuals.widgets.active,
        &mut s.visuals.widgets.open,
    ] {
        v.corner_radius = CornerRadius::ZERO;
        v.bg_stroke = Stroke::new(1., Color32::from_rgb(109, 100, 123));
    }
    s.visuals.widgets.inactive.bg_fill = PANEL;
    s.visuals.widgets.inactive.weak_bg_fill = PANEL;
    s.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(73, 55, 85);
    s.spacing.item_spacing = vec2(8., 7.);
    s.spacing.button_padding = vec2(10., 6.);
    s.text_styles
        .insert(TextStyle::Body, FontId::proportional(13.));
    s.text_styles
        .insert(TextStyle::Button, FontId::monospace(12.));
    ctx.set_style(s);
}
#[allow(clippy::too_many_arguments)]
pub fn draw(
    ctx: &Context,
    s: &ParamSetter,
    state: &mut UiState,
    p: &SpasmParams,
    shared: &Shared,
    held: &[AtomicBool; 8],
    wake: &AtomicU8,
) {
    if state.last_name != p.name.get() {
        state.last_name = p.name.get();
        state.name = state.last_name.clone();
        state.notes = p.notes.get();
    }
    ResizableWindow::new("spasm")
        .min_size(vec2(920., 760.))
        .show(ctx, p.editor.as_ref(), |ui| {
            ui.spacing_mut().item_spacing = vec2(8., 7.);
            header(ui, p, s);
            ui.horizontal(|ui| {
                if ui
                    .button("◀")
                    .on_hover_text("Previous factory performance")
                    .clicked()
                {
                    let i = (p.preset.value() + 9) % 10;
                    library::apply(spasm_core::factory(i as usize), i, state, p, s);
                }
                ComboBox::from_id_salt("preset")
                    .width(310.)
                    .selected_text(
                        RichText::new(p.name.get().to_uppercase())
                            .size(19.)
                            .color(GREEN),
                    )
                    .show_ui(ui, |ui| {
                        for (i, name) in spasm_core::patch::NAMES.iter().enumerate() {
                            if ui
                                .selectable_label(p.preset.value() == i as i32, *name)
                                .clicked()
                            {
                                library::apply(spasm_core::factory(i), i as i32, state, p, s);
                            }
                        }
                        ui.separator();
                        let mut selected = None;
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
                        if let Some(path) = selected {
                            match library::read(&path) {
                                Ok(d) => library::apply(d, 10, state, p, s),
                                Err(e) => state.status = e,
                            }
                        }
                    });
                if ui
                    .button("▶")
                    .on_hover_text("Next factory performance")
                    .clicked()
                {
                    let i = (p.preset.value() + 1) % 10;
                    library::apply(spasm_core::factory(i as usize), i, state, p, s);
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .selectable_label(p.edit.load(Ordering::Relaxed), "EDIT")
                        .clicked()
                    {
                        p.edit.fetch_xor(true, Ordering::Relaxed);
                        p.metadata_changed(s);
                    }
                    toggle(ui, &p.bypass, s, "BYPASS");
                });
            });
            ui.add_sized(
                [ui.available_width(), 32.],
                Label::new(RichText::new(p.notes.get()).color(MUTED)).wrap(),
            );
            if p.edit.load(Ordering::Relaxed) {
                crate::edit::draw(ui, state, p, s);
            } else {
                let status = shared.status();
                waveform(ui, shared, p, &status);
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [(ui.available_width() - 365.).max(260.), 24.],
                        Label::new(
                            RichText::new(format!(
                                "{}  {}   {:05.1} BPM",
                                if status.host_sync { "HOST" } else { "FREE" },
                                status.effect.name(),
                                status.bpm
                            ))
                            .monospace()
                            .color(GREEN),
                        ),
                    );
                    ui.label("QUANTIZE");
                    ui.add(ParamSlider::for_param(&p.quantize, s).with_width(100.));
                    toggle(ui, &p.latch, s, "LATCH");
                });
                let height = (ui.available_height() - 228.).clamp(235., 650.);
                xy(ui, p, s, height);
                ui.horizontal_top(|ui| {
                    let width = (ui.available_width() - 32.) / 5.;
                    control(
                        ui,
                        &p.repeat_div,
                        s,
                        width,
                        "REPEAT",
                        "Stored repeat division. X adds faster subdivisions.",
                    );
                    control(
                        ui,
                        &p.stutter_ms,
                        s,
                        width,
                        "FRAGMENT",
                        "Stutter fragment size before macro offsets.",
                    );
                    control(
                        ui,
                        &p.drive,
                        s,
                        width,
                        "DRIVE",
                        "Active wet-path saturation.",
                    );
                    control(
                        ui,
                        &p.mix,
                        s,
                        width,
                        "MIX",
                        "Linear clean/effect mix. Idle remains clean.",
                    );
                    control(
                        ui,
                        &p.fx_output_db,
                        s,
                        width,
                        "FX OUTPUT",
                        "Wet-path trim only. Clean live signal remains unchanged.",
                    );
                });
                for row in 0..2 {
                    ui.horizontal(|ui| {
                        let width = (ui.available_width() - 24.) / 4.;
                        for col in 0..4 {
                            let i = row * 4 + col;
                            let active = if i == 7 {
                                p.pads()[i].value()
                            } else {
                                status.engaged & (1 << i) != 0
                            };
                            let pending = i < 7 && status.pending & (1 << i) != 0;
                            let response = ui.add(
                                Button::new(
                                    RichText::new(format!(
                                        "{}{}\n{}",
                                        spasm_core::patch::PAD_NAMES[i],
                                        if i < 7 && status.latched & (1 << i) != 0 {
                                            " / LATCH"
                                        } else {
                                            ""
                                        },
                                        if pending {
                                            "WAIT / GRID"
                                        } else if active {
                                            "ENGAGED"
                                        } else if i == 7 {
                                            "RETURN TO LIVE"
                                        } else {
                                            p.config().pad_assign[i].name()
                                        }
                                    ))
                                    .monospace()
                                    .size(15.),
                                )
                                .fill(if active {
                                    Color32::from_rgb(79, 107, 21)
                                } else if pending {
                                    Color32::from_rgb(105, 22, 89)
                                } else {
                                    PANEL
                                })
                                .min_size(vec2(width, 60.))
                                .sense(Sense::click_and_drag()),
                            );
                            if (response.is_pointer_button_down_on() || response.clicked())
                                && !state.pressed[i]
                            {
                                wake.fetch_or(1 << i, Ordering::Relaxed);
                                held[i].store(true, Ordering::Relaxed);
                                state.pressed[i] = true;
                                state.pulse[i] = response.clicked();
                                s.begin_set_parameter(p.pads()[i]);
                                s.set_parameter(p.pads()[i], true);
                            }
                        }
                    });
                }

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("HOLD A PAD → FRACTURE → RELEASE")
                            .monospace()
                            .color(MUTED),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(14.);
                        ui.label(
                            RichText::new(if status.short_history && status.effect.captured() {
                                "SHORT CAPTURE"
                            } else {
                                "RESET ALWAYS RETURNS LIVE"
                            })
                            .small()
                            .color(if status.short_history { MAGENTA } else { MUTED }),
                        );
                    });
                });
            }
            if !state.status.is_empty() {
                ui.label(RichText::new(&state.status).small().color(GREEN));
            }
        });
    if !ctx.input(|i| i.pointer.primary_down()) {
        for (i, param) in p.pads().iter().enumerate() {
            if state.pressed[i] && !state.pulse[i] {
                s.set_parameter(*param, false);
                s.end_set_parameter(*param);
                state.pressed[i] = false;
                held[i].store(false, Ordering::Relaxed);
            }
        }
    }
    state.pulse.fill(false);
    ctx.request_repaint_after(std::time::Duration::from_millis(33));
}

pub fn set<P: Param>(p: &P, v: P::Plain, s: &ParamSetter) {
    s.begin_set_parameter(p);
    s.set_parameter(p, v);
    s.end_set_parameter(p);
}
pub fn toggle(ui: &mut Ui, p: &BoolParam, s: &ParamSetter, label: &str) {
    if ui
        .selectable_label(p.value(), label)
        .on_hover_text(p.name())
        .clicked()
    {
        set(p, !p.value(), s);
    }
}
pub fn control<P: Param>(ui: &mut Ui, p: &P, s: &ParamSetter, width: f32, label: &str, tip: &str) {
    ui.vertical(|ui| {
        ui.set_width(width);
        ui.label(RichText::new(label).monospace().size(11.).color(MUTED));
        ui.add(ParamSlider::for_param(p, s).with_width((width - 90.).max(35.)))
            .on_hover_text(tip);
    });
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
fn header(ui: &mut Ui, p: &SpasmParams, _s: &ParamSetter) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 81.), Sense::hover());
    let paint = ui.painter();
    gradient(
        paint,
        r,
        Color32::from_rgb(78, 72, 91),
        Color32::from_rgb(28, 24, 39),
    );
    paint.line_segment([r.left_top(), r.right_top()], Stroke::new(2., SILVER));
    paint.line_segment(
        [r.left_bottom(), r.right_bottom()],
        Stroke::new(2., Color32::BLACK),
    );
    paint.text(
        r.left_top() + vec2(15., 1.),
        Align2::LEFT_TOP,
        "S P A S M",
        FontId::monospace(44.),
        Color32::from_rgb(10, 6, 16),
    );
    paint.text(
        r.left_top() + vec2(14., 0.),
        Align2::LEFT_TOP,
        "S P A S M",
        FontId::monospace(44.),
        GREEN,
    );
    paint.text(
        r.left_bottom() + vec2(16., -10.),
        Align2::LEFT_BOTTOM,
        "RHYTHMIC AUDIO DESTRUCTION SYSTEM",
        FontId::monospace(11.),
        SILVER,
    );
    let stripe = Rect::from_min_max(
        pos2(r.left() + 365., r.bottom() - 12.),
        pos2(r.right() - 110., r.bottom() - 5.),
    );
    paint.rect_filled(stripe, 0., Color32::BLACK);
    let clip = paint.with_clip_rect(stripe);
    for i in 0..100 {
        let x = stripe.left() + i as f32 * 16.;
        clip.line_segment(
            [pos2(x, stripe.top()), pos2(x - 7., stripe.bottom())],
            Stroke::new(5., GREEN),
        );
    }
    paint.text(
        r.right_top() + vec2(-10., 12.),
        Align2::RIGHT_TOP,
        "VOIDMUTT",
        FontId::monospace(11.),
        SILVER,
    );
    paint.text(
        r.right_bottom() + vec2(-10., -13.),
        Align2::RIGHT_BOTTOM,
        if p.bypass.value() {
            "BYPASS"
        } else {
            "DSP / 01"
        },
        FontId::monospace(10.),
        GREEN,
    );
}
fn xy(ui: &mut Ui, p: &SpasmParams, s: &ParamSetter, height: f32) {
    let (r, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click_and_drag());
    ui.painter().rect_filled(r, 0., LCD);
    ui.painter()
        .rect_stroke(r, 0., Stroke::new(2., GREEN), StrokeKind::Inside);
    for n in 1..8 {
        let x = r.left() + r.width() * n as f32 / 8.;
        let y = r.top() + r.height() * n as f32 / 8.;
        ui.painter().vline(
            x,
            r.y_range(),
            Stroke::new(1., Color32::from_rgb(39, 43, 40)),
        );
        ui.painter().hline(
            r.x_range(),
            y,
            Stroke::new(1., Color32::from_rgb(39, 43, 40)),
        );
    }
    for y in (0..height as usize).step_by(4) {
        ui.painter().hline(
            r.x_range(),
            r.top() + y as f32,
            Stroke::new(0.5, Color32::from_black_alpha(55)),
        );
    }
    let c = p.config();
    let point = pos2(r.left() + c.x * r.width(), r.bottom() - c.y * r.height());
    ui.painter().hline(
        r.x_range(),
        point.y,
        Stroke::new(1., MAGENTA.gamma_multiply(0.45)),
    );
    ui.painter().vline(
        point.x,
        r.y_range(),
        Stroke::new(1., MAGENTA.gamma_multiply(0.45)),
    );
    ui.painter()
        .circle_stroke(point, 15., Stroke::new(1., MAGENTA));
    ui.painter().circle_filled(point, 5., MAGENTA);
    ui.painter().text(
        r.right_top() + vec2(-14., 12.),
        Align2::RIGHT_TOP,
        format!("X {:03}  Y {:03}", (c.x * 100.) as i32, (c.y * 100.) as i32),
        FontId::monospace(14.),
        GREEN,
    );
    ui.painter().text(
        r.left_top() + vec2(12., 12.),
        Align2::LEFT_TOP,
        "Y / DAMAGE",
        FontId::monospace(13.),
        Color32::WHITE,
    );
    ui.painter().text(
        r.center_bottom() - vec2(0., 12.),
        Align2::CENTER_BOTTOM,
        "X / TIME FRACTURE",
        FontId::monospace(13.),
        Color32::WHITE,
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
            p.midi.clear();
            s.set_parameter(&p.x, ((pos.x - r.left()) / r.width()).clamp(0., 1.));
            s.set_parameter(&p.y, ((r.bottom() - pos.y) / r.height()).clamp(0., 1.));
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
}
fn waveform(ui: &mut Ui, shared: &Shared, p: &SpasmParams, status: &spasm_core::Status) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 89.), Sense::hover());
    let paint = ui.painter();
    paint.rect_filled(r, 0., LCD);
    paint.rect_stroke(
        r,
        0.,
        Stroke::new(1., Color32::from_rgb(91, 81, 106)),
        StrokeKind::Inside,
    );
    let plot = r.shrink2(vec2(12., 20.));
    let cursor = shared.cursor.load(Ordering::Relaxed) as usize % WAVE_SIZE;
    let valid = (shared.valid.load(Ordering::Acquire) as usize).min(WAVE_SIZE);
    let history = p.history_seconds.value();
    for i in 0..WAVE_SIZE {
        let j = (cursor + i) % WAVE_SIZE;
        let v = if i >= WAVE_SIZE - valid {
            f32::from_bits(shared.wave[j].load(Ordering::Relaxed)).clamp(0., 1.)
        } else {
            0.
        };
        let x = plot.left() + i as f32 / WAVE_SIZE as f32 * plot.width();
        let h = v.sqrt() * plot.height() * 0.48;
        paint.line_segment(
            [pos2(x, plot.center().y - h), pos2(x, plot.center().y + h)],
            Stroke::new(1., GREEN.gamma_multiply(0.7)),
        );
    }
    if status.capture_seconds > 0. {
        let end = 1. - status.capture_age_seconds / history;
        let start = end - status.repeat_seconds / history;
        if end > 0. {
            let region = Rect::from_min_max(
                pos2(plot.left() + start.clamp(0., 1.) * plot.width(), plot.top()),
                pos2(
                    plot.left() + end.clamp(0., 1.) * plot.width(),
                    plot.bottom(),
                ),
            );
            paint.rect_filled(region, 0., MAGENTA.gamma_multiply(0.13));
            paint.rect_stroke(region, 0., Stroke::new(1., MAGENTA), StrokeKind::Inside);
            let phase = if status.reverse {
                1. - status.phase
            } else {
                status.phase
            };
            let x = plot.left() + (start + (end - start) * phase).clamp(0., 1.) * plot.width();
            paint.vline(x, plot.y_range(), Stroke::new(2., MAGENTA));
        }
    }
    paint.vline(plot.right(), plot.y_range(), Stroke::new(2., GREEN));
    paint.text(
        r.left_top() + vec2(10., 4.),
        Align2::LEFT_TOP,
        format!(
            "ROLLING INPUT  {:.2}/{:.1}s",
            status.history_seconds, history
        ),
        FontId::monospace(10.),
        MUTED,
    );
    paint.text(
        r.right_top() + vec2(-10., 4.),
        Align2::RIGHT_TOP,
        format!(
            "{}  {:.1}ms  {:.2}x  {}",
            if status.reverse { "◀ REV" } else { "▶ FWD" },
            status.repeat_seconds * 1000.,
            status.speed,
            if status.capture_age_seconds > history {
                "FROZEN / OFFSCREEN"
            } else {
                "LIVE →"
            }
        ),
        FontId::monospace(10.),
        MAGENTA,
    );
    paint.text(
        r.left_bottom() + vec2(10., -3.),
        Align2::LEFT_BOTTOM,
        format!(
            "IN {:5.1}dB   OUT {:5.1}dB",
            20. * status.input_peak.max(1e-6).log10(),
            20. * status.output_peak.max(1e-6).log10()
        ),
        FontId::monospace(10.),
        GREEN,
    );
}
