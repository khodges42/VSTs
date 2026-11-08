// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    editor::{GREEN, UiState, control, set, toggle},
    params::SpasmParams,
};
use nih_plug::prelude::*;
use nih_plug_egui::egui::*;
use spasm_core::{
    Effect,
    patch::{PAD_NAMES, Source, Target},
};
pub fn draw(ui: &mut Ui, state: &mut UiState, p: &SpasmParams, s: &ParamSetter) {
    ScrollArea::vertical().id_salt("edit-scroll").max_height((ui.available_height()-24.).max(300.)).show(ui,|ui|{
        ui.label(RichText::new("EDIT / LIVE PARAMETERS").color(GREEN).monospace());
        ui.label("X/Y routes add signed contributions to these stored bases. Changing views preserves the sound.");
        capture(ui,p,s);repeat(ui,p,s);pitch(ui,p,s);color(ui,p,s);mappings(ui,p,s);routing(ui,p,s);
        CollapsingHeader::new("USER PRESETS / IMPORT / EXPORT").default_open(true).show(ui,|ui|crate::library::draw(ui,state,p,s));
    });
}
fn capture(ui: &mut Ui, p: &SpasmParams, s: &ParamSetter) {
    CollapsingHeader::new("CAPTURE / RELEASE / CLOCK").default_open(true).show(ui,|ui|{
        row(ui,s,&[&p.history_seconds,&p.crossfade_ms,&p.edge_ms]);
        ui.horizontal_wrapped(|ui|{
            toggle(ui,&p.capture_locked,s,"LOCK CAPTURE");
            toggle(ui,&p.reset_clear,s,"RESET CLEARS HISTORY");
            toggle(ui,&p.latch,s,"GLOBAL LATCH");
            toggle(ui,&p.auto_damage,s,"DAMAGE ON CAPTURE");
        }
        );
        row(ui,s,&[&p.quantize,&p.repeat_count]);
        ui.label("LOCK keeps a phrase across REPEAT > REVERSE > DIVE. Unlocked triggers capture fresh input; history may be shorter just after a capture.");
    }
    );
}
fn repeat(ui: &mut Ui, p: &SpasmParams, s: &ParamSetter) {
    CollapsingHeader::new("REPEAT / STUTTER / REVERSE")
        .default_open(true)
        .show(ui, |ui| {
            row(ui, s, &[&p.repeat_div, &p.stutter_div, &p.reverse_div]);
            row(ui, s, &[&p.stutter_ms, &p.acceleration, &p.pitch_semitones]);
            ui.horizontal(|ui| {
                toggle(ui, &p.stutter_reverse, s, "REVERSE STUTTER");
            });
        });
}
fn pitch(ui: &mut Ui, p: &SpasmParams, s: &ParamSetter) {
    CollapsingHeader::new("TAPE STOP / PITCH DIVE").show(ui, |ui| {
        ui.horizontal(|ui| {
            toggle(ui, &p.stop_sync, s, "STOP TEMPO SYNC");
        });
        row(ui, s, &[&p.stop_ms, &p.stop_beats, &p.stop_curve]);
        row(ui, s, &[&p.dive_ms, &p.dive_semitones, &p.wander]);
        ui.label("Variable-speed captured playback changes pitch and duration together.");
    });
}
fn color(ui: &mut Ui, p: &SpasmParams, s: &ParamSetter) {
    CollapsingHeader::new("GATE / DIGITAL DAMAGE / COLOR").show(ui,|ui|{
        row(ui,s,&[&p.gate_div]);
        row(ui,s,&[&p.gate_depth,&p.gate_attack_ms]);
        row(ui,s,&[&p.gate_release_ms,&p.bits,&p.reduction]);
        ui.horizontal(|ui|{
            ui.label("16-STEP GATE");
            let mask=p.gate_pattern.value();
            for i in 0..16 {
                if ui.selectable_label(mask&(1<<i)!=0,format!("{:X}",i+1)).clicked(){
                    set(&p.gate_pattern,mask^(1<<i),s);
                }
            }
        }
        );
        ui.horizontal(|ui|{
            toggle(ui,&p.filter_enabled,s,"LOW-PASS");
        }
        );
        row(ui,s,&[&p.filter_hz,&p.drive,&p.fx_output_db]);
        ui.label("Sample hold and waveshaping deliberately alias. Color/output trim affects active wet processing only.");
    }
    );
}
fn mappings(ui: &mut Ui, p: &SpasmParams, s: &ParamSetter) {
    CollapsingHeader::new("X/Y / VELOCITY PERFORMANCE MAPPINGS").default_open(true).show(ui,|ui|{
        ui.label("Value = START + (END - START) * source^CURVE. Routes sum, then clamp; X/Y run 0–1.");
        for (i,r) in p.routes().iter().enumerate(){
            ui.push_id(i,|ui|{
                ui.horizontal(|ui|{
                    toggle(ui,r.enabled,s,&format!("{:02}",i+1));
                    let mut source=r.source.value();
                    ComboBox::from_id_salt("source").width(82.).selected_text(format!("{:?}",Source::ALL[source as usize])).show_ui(ui,|ui|{
                        for (j,x) in Source::ALL.iter().enumerate(){
                            ui.selectable_value(&mut source,j as i32,format!("{x:?}"));
                        }
                    }
                    );
                    if source!=r.source.value(){
                        set(r.source,source,s);
                    }
                    let mut target=r.target.value();
                    ComboBox::from_id_salt("target").width(185.).selected_text(Target::ALL[target as usize].name()).show_ui(ui,|ui|{
                        for (j,x) in Target::ALL.iter().enumerate(){
                            ui.selectable_value(&mut target,j as i32,x.name());
                        }
                    }
                    );
                    if target!=r.target.value(){
                        set(r.target,target,s);
                        set(r.start,0.,s);
                        set(r.end,0.,s);
                    }
                    let (lo,hi)=Target::ALL[target as usize].range();
                    number(ui,r.start,s,lo,hi,"START ");
                    number(ui,r.end,s,lo,hi,"END ");
                    number(ui,r.curve,s,0.2,5.,"CURVE ");
                }
                );
            }
            );
        }
    }
    );
}
fn routing(ui: &mut Ui, p: &SpasmParams, s: &ParamSetter) {
    CollapsingHeader::new("PAD ASSIGNMENTS / MIDI / PRIORITY").show(ui,|ui|{
        ui.label("Notes 0–127; channel 0 = omni. Unique notes and CC assignments are required for portable presets.");
        for (i,label) in PAD_NAMES.iter().enumerate(){
            ui.push_id(i,|ui|{
                ui.horizontal(|ui|{
                    ui.add_sized([85.,20.],Label::new(*label));
                    if i<7 {
                        let a=p.assignments()[i];
                        let mut v=a.value();
                        ComboBox::from_id_salt("module").width(115.).selected_text(Effect::ALL[v as usize].name()).show_ui(ui,|ui|{
                            for e in Effect::ALL{
                                ui.selectable_value(&mut v,e.id() as i32,e.name());
                            }
                        }
                        );
                        if v!=a.value(){
                            set(a,v,s);
                        }
                    }
                    integer(ui,p.notes()[i],s,0,127,"NOTE ");
                    if i<7{
                        let mask=p.latch_mask.value();
                        if ui.selectable_label(mask&(1<<i)!=0,"LATCH").clicked(){
                            set(&p.latch_mask,mask^(1<<i),s);
                        }
                    }
                }
                );
            }
            );
        }
        row(ui,s,&[&p.midi_channel,&p.chaos_div,&p.seed]);
        ui.horizontal(|ui|{
            for (i,cc) in p.ccs().iter().enumerate(){
                integer(ui,cc,s,0,119,if i==0{
                    "X CC "}
                else{
                    "Y CC "}
                );
            }
        }
        );
        ui.label("PRIMARY PRIORITY / 0 wins. Changing a rank swaps the previous occupant.");
        ui.horizontal_wrapped(|ui|{
            let ranks=p.priorities();
            for (i,a) in ranks.iter().enumerate(){
                ui.label(Effect::PRIMARY[i].name());
                let old=a.value();
                let mut v=old;
                if ui.add(DragValue::new(&mut v).range(0..=4)).changed(){
                    if let Some(other)=ranks.iter().position(|b|b.value()==v){
                        set(ranks[other],old,s);
                    }
                    set(*a,v,s);
                }
            }
        }
        );
        ui.label("CHAOS MODULES");
        ui.horizontal_wrapped(|ui|{
            let mask=p.chaos_mask.value();
            for e in Effect::ALL.into_iter().skip(1).take(7){
                if ui.selectable_label(mask&e.bit() as i32!=0,e.name()).clicked(){
                    set(&p.chaos_mask,mask^e.bit() as i32,s);
                }
            }
        }
        );
    }
    );
}
fn row<P: Param>(ui: &mut Ui, s: &ParamSetter, params: &[&P]) {
    ui.horizontal_top(|ui| {
        let w = (ui.available_width() - 8. * (params.len() - 1) as f32) / params.len() as f32;
        for p in params {
            ui.allocate_ui(vec2(w, 42.), |ui| {
                control(
                    ui,
                    *p,
                    s,
                    w,
                    p.name(),
                    "Stored base value; host automatable.",
                )
            });
        }
    });
}
fn number(ui: &mut Ui, p: &FloatParam, s: &ParamSetter, lo: f32, hi: f32, label: &str) {
    let mut v = p.value();
    let response = ui.add(
        DragValue::new(&mut v)
            .range(lo..=hi)
            .speed(((hi - lo) / 500.).min(1.))
            .prefix(label),
    );
    if response.changed() {
        set(p, v, s);
    }
}
fn integer(ui: &mut Ui, p: &IntParam, s: &ParamSetter, lo: i32, hi: i32, label: &str) {
    let mut v = p.value();
    if ui
        .add(DragValue::new(&mut v).range(lo..=hi).prefix(label))
        .changed()
    {
        set(p, v, s);
    }
}
