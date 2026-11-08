// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    editor::{GREEN, MUTED, UiState, set},
    params::SpasmParams,
};
use nih_plug::prelude::*;
use nih_plug_egui::egui::*;
use spasm_core::{Preset, patch::MAX_BYTES};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub fn root() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| ".".into())
        .join("VOIDMUTT/SPASM/Presets")
}
pub fn scan(root: &Path) -> Vec<PathBuf> {
    let mut v: Vec<_> = fs::read_dir(root)
        .into_iter()
        .flatten()
        .take(1024)
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.to_string_lossy().ends_with(".spasm.json"))
        .collect();
    v.sort();
    v
}
pub fn read(path: &Path) -> Result<Preset, String> {
    let mut s = String::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take((MAX_BYTES + 1) as u64)
        .read_to_string(&mut s)
        .map_err(|e| e.to_string())?;
    Preset::from_json(&s)
}
pub fn write(path: &Path, p: &Preset) -> Result<(), String> {
    let s = p.to_json()?;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?
        .write_all(s.as_bytes())
        .map_err(|e| e.to_string())
}
pub fn slug(s: &str) -> String {
    let s: String = s
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .collect();
    if s.is_empty() { "User".into() } else { s }
}
pub fn apply(preset: Preset, index: i32, state: &mut UiState, p: &SpasmParams, s: &ParamSetter) {
    match preset.validate() {
        Ok(()) => {
            p.apply(&preset, index, s);
            state.name = preset.name;
            state.notes = preset.description;
            state.status.clear();
            state.last_name = p.name.get();
        }
        Err(e) => state.status = e,
    }
}
pub fn draw(ui: &mut Ui, state: &mut UiState, p: &SpasmParams, s: &ParamSetter) {
    ui.label(RichText::new("PORTABLE USER PRESETS").color(GREEN));
    ui.label(
        RichText::new(root().display().to_string())
            .small()
            .color(MUTED),
    );
    ui.horizontal(|ui| {
        ui.label("NAME");
        ui.add(TextEdit::singleline(&mut state.name).desired_width(320.));
        if ui.button("RENAME IN SESSION").clicked() {
            let mut d = p.document();
            d.name = state.name.clone();
            if let Err(e) = d.validate() {
                state.status = e;
            } else {
                p.set_metadata(&d.name, &d.description, s);
                set(&p.preset, 10, s);
                state.last_name = d.name;
            }
        }
        if ui.button("DUPLICATE").clicked() {
            while state.name.len() > 26 {
                state.name.pop();
            }
            state.name.push_str(" COPY");
        }
    });
    ui.label("DESCRIPTION / PERFORMANCE NOTES");
    ui.add(
        TextEdit::multiline(&mut state.notes)
            .desired_rows(2)
            .desired_width(ui.available_width()),
    );
    ui.horizontal(|ui| {
        if ui.button("SAVE AS NEW").clicked() {
            let mut d = p.document();
            d.name = state.name.clone();
            d.description = state.notes.clone();
            let path = root().join(format!("{}.spasm.json", slug(&d.name)));
            state.status = match fs::create_dir_all(root())
                .map_err(|e| e.to_string())
                .and_then(|_| write(&path, &d))
            {
                Ok(()) => {
                    p.set_metadata(&d.name, &d.description, s);
                    set(&p.preset, 10, s);
                    state.last_name = d.name;
                    format!("Saved {}", path.display())
                }
                Err(e) => {
                    format!("Save failed: {e}. Use a new name; existing files are protected.")
                }
            };
            state.users = scan(&root());
        }
        if ui.button("REFRESH LIBRARY").clicked() {
            state.users = scan(&root());
        }
        if ui.button("RESTORE FACTORY").clicked() {
            let index = if p.preset.value() < 10 {
                p.preset.value()
            } else {
                0
            };
            apply(spasm_core::factory(index as usize), index, state, p, s);
        }
    });
    ui.horizontal(|ui| {
        ui.add(
            TextEdit::singleline(&mut state.path)
                .desired_width((ui.available_width() - 180.).max(200.))
                .hint_text("Full path to .spasm.json"),
        );
        if ui.button("IMPORT").clicked() {
            match read(Path::new(&state.path)) {
                Ok(d) => apply(d, 10, state, p, s),
                Err(e) => state.status = e,
            }
        }
        if ui.button("EXPORT").clicked() {
            let mut d = p.document();
            d.name = state.name.clone();
            d.description = state.notes.clone();
            state.status = match write(Path::new(&state.path), &d) {
                Ok(()) => "Exported portable preset".into(),
                Err(e) => e,
            };
        }
    });
    ui.label("Live edits use the same automated parameters. Saving takes their current values; it does not apply a hidden editor snapshot.");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_file_roundtrip_protects_existing_and_rejects_large() {
        let root = std::env::temp_dir().join(format!("spasm-presets-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("unit.spasm.json");
        let _ = fs::remove_file(&path);
        let p = spasm_core::factory(8);
        write(&path, &p).unwrap();
        assert_eq!(read(&path).unwrap(), p);
        assert!(write(&path, &p).is_err());
        fs::write(&path, "x".repeat(MAX_BYTES + 1)).unwrap();
        assert!(read(&path).is_err());
        fs::remove_file(path).unwrap();
        fs::remove_dir(root).unwrap();
    }
}
