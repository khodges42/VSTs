# FL Studio automation: current support and right-click menus

Assessment: 2026-11-07. Applies to VIXEN, ECHOGIRL, SPASM and 8 INCH STEAK (EIS).

## Decision

The plugins already publish host automation parameters. The missing feature is
opening FL Studio's host-owned parameter menu from a right-click inside the
custom plugin editor. These are separate capabilities.

Ordinary automation clips can be used now through FL Studio's parameter workflow.
Native right-click integration requires an extension to the pinned VST3 wrapper
and editor API, followed by host/UI lifecycle testing. It is not a parameter flag
or a small change to the synth/effect DSP. This delivery is the requested
assessment/documentation path; no plugin code, dependency pin or binary changed.

## Make an automation clip now

### For an ordinary performance knob

1. Open the plugin and move the desired performance knob slightly.
2. Immediately select **Tools > Last tweaked > Create automation clip** in FL Studio.
3. Edit the new clip in the Playlist and play the project to hear it.

Choose a real host parameter such as VIXEN Output or EIS TENDERIZE for the first
check. Changing another parameter or loading a preset before creating the clip
can change which control FL Studio considers last tweaked.

Image-Line documents this workflow for third-party plugins separately from native
FL controls' direct right-click menu. See the official
[Automation Clips manual](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/playlist_automationclip.htm).

### For X/Y axes or an unavailable Last tweaked target

1. Open the plugin wrapper's menu at its upper left and choose **Browse parameters**.
2. Find the desired parameter under that plugin instance in FL Studio's Browser.
3. Right-click the **Browser parameter**, then select **Create automation clip**.

The Browser can also be reached through Current project > Generators / Effects >
the plugin instance. The wrapper's separate generic MIDI CC entries are not the
same as the plugin's named X, Y, mix or output parameters. Prefer those named
parameters for this task. See Image-Line's
[Plugin Wrapper parameter controls](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/wrapper.htm).

Create **two clips** to automate the whole X/Y surface: one for X, one for Y.
Dragging the pad edits both axes, so Last tweaked is not a reliable way to choose
between them; the existing pad code sends X and then Y. Browse parameters lets
you select each explicitly. Preset-defined mappings still determine the sound
produced by the automated X/Y coordinates.

For momentary controls, draw explicit on/off changes. A flat automation value of
1 keeps a boolean control on; use 0 to release it. Latch, quantization, HOLD and
RESET semantics still follow the selected plugin/preset. Do not assume a rising
ramp is a sequence of repeated button presses.

## Which controls are available

| Plugin | Useful starting targets | Important boundary |
|---|---|---|
| VIXEN | X, Y, Brightness, Resonance, Drive, Envelope character, Glide, Output | Oscillators, ADSRs and other EDIT draft fields are applied patch data, not individual host parameters |
| ECHOGIRL | X, Y, TIME, FEEDBACK, FILTER, DRIVE, MIX, OUTPUT; THROW/HOLD/KILL/DUB/FEED | Ordinary EDIT sliders modify a draft; use the published parameter list for automation |
| SPASM | X, Y, repeat/subdivision, coloration, Mix, output and named performance pads | Preset name/notes and UI-only configuration are not automation targets |
| EIS / 8 INCH STEAK | TENDERIZE, TIME, LOW/MID/HIGH, DRIVE, JUICE, MIX, OUTPUT; per-band controls | There is no X/Y pad; Easy/Butcher views share underlying parameters |

The full Browser list is authoritative for the loaded build. These are examples,
not a promise that every piece of text or every draft editor control is a host
parameter. Hidden document-change/preset metadata intentionally remains excluded.

If a parameter is missing, check the loaded plugin version and file path. A DAW
may still be using an older copy. Replacing a file on disk does not update an
already loaded instance; reopen/rescan when appropriate and preserve the project.
If the named parameter exists and moves but sound does not change, check active
X/Y mappings, MIDI control sources and the patch itself before assuming an
unpublished parameter. Record the exact plugin/host versions and target name if
Browse parameters also fails.

## Findings in the current source

All four wrappers declare `SAMPLE_ACCURATE_AUTOMATION = true` and expose typed NIH-plug
parameters. Their performance widgets use `ParamSetter` begin/set/end gestures;
the framework forwards these as VST3 beginEdit/performEdit/endEdit calls. The DSP
reads the corresponding parameter values. This infrastructure already exists.

Relevant first-party files:

- [VIXEN parameters and wrapper](../plugins/vixen/src/lib.rs),
  [performance widgets](../plugins/vixen/src/editor.rs).
- [ECHOGIRL parameters](../plugins/echogirl/src/params.rs),
  [performance widgets](../plugins/echogirl/src/editor.rs).
- [SPASM parameters](../plugins/spasm/src/params.rs),
  [performance widgets](../plugins/spasm/src/editor.rs).
- [EIS parameters](../plugins/eis/src/params.rs),
  [performance widgets](../plugins/eis/src/editor.rs).

The pinned NIH-plug revision is
`de421011f41a6d10fc8c7a6084e4f4dee0143683`. Its
[GuiContext API](https://github.com/robbert-vdh/nih-plug/blob/de421011f41a6d10fc8c7a6084e4f4dee0143683/src/context/gui.rs)
exposes parameter editing but no host context-menu request. The
[VST3 GUI context](https://github.com/robbert-vdh/nih-plug/blob/de421011f41a6d10fc8c7a6084e4f4dee0143683/src/wrapper/vst3/context.rs)
forwards edit gestures; the inspected wrapper/editor has no IComponentHandler3 /
IContextMenu bridge. Our custom controls and the pinned egui parameter slider do
not request a host-owned automation menu on secondary click.

Adding an egui popup labelled Create automation clip would not itself ask FL
Studio to create a Playlist clip. Do not ship a misleading local imitation of
that host command or use unsupported FL-window/keyboard automation to trigger it.

## Scope of genuine right-click support

VST3 offers an optional host callback:
[IComponentHandler3::createContextMenu](https://steinbergmedia.github.io/vst3_doc/vstinterfaces/classSteinberg_1_1Vst_1_1IComponentHandler3.html).
It takes the plugin view and exported parameter ID. A host may return
parameter-specific actions through IContextMenu; the plugin can display the
host's menu. The host decides the available commands, so FL-specific wording
must not be assumed across all DAWs.

A small framework spike should precede a production change:

1. Add or adopt a pinned, license-reviewed NIH-plug implementation exposing a
   safe GUI-only request for the host menu. Keep ABI/COM ownership and native
   view access in the framework, not handwritten unsafe code in our plugin/DSP.
2. Map the actual clicked parameter to its existing exported ID. Test a single
   VIXEN Output knob first. An X/Y surface must offer an unambiguous axis choice;
   it must not arbitrarily automate whichever axis was edited last.
3. Handle unsupported host interfaces, null menus, cancellation, GUI scaling,
   native coordinates and editor lifetime. Menu operations must stay off audio.
4. Verify real FL Studio clip creation and playback, closing/reopening the editor,
   repeated menu requests and project restoration. Check another VST3 host and
   an unsupported-host fallback. Preserve plugin/parameter IDs and state formats.
5. Apply the tested GUI helper to all four editors, including ParamSlider widgets,
   then rerun independent-host, automation/state, realtime and pluginval gates.

This is a contained wrapper/UI feature, not a new synthesis engine or DAWJ
integration. It is nevertheless larger than the quick UI patch requested here:
framework changes, dependency/license inventory updates and real host testing are
needed. No exact effort estimate is claimed without the first host-menu spike.

## Validation performed for this assessment

The actual pinned dependency source and all four parameter/editor paths were
inspected. The official Image-Line workflow and VST3 interface documentation were
consulted. Existing independently testable bundled VST3s were loaded in the
project's native offline smoke hosts:

| Plugin | Result |
|---|---|
| VIXEN | PASS: parameter/state, sample-offset notes and reference audio; 2 passes |
| ECHOGIRL | PASS: published performance parameters and stereo/state reference audio; 6 scenario/pass results |
| EIS | PASS: stereo/state/automation reference comparisons; 8 scenario/pass results |
| SPASM | PASS: stereo/MIDI/CC/automation/state reference comparisons; 16 case/pass results |

Reports and renders from this run are under ignored
`target/automation-assessment/`. These checks used the existing binaries and are
not evidence that a new right-click menu exists or that FL Studio's own UI was
automated. No FL Studio project was modified. No new plugin build or native
right-click/Playlist integration test is claimed.
