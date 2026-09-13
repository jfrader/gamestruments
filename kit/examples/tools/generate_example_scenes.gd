extends SceneTree

## Checked-in scene generator for the three focused examples.
## Run ONLY as:
##   godot --headless --path kit/examples --script res://tools/generate_example_scenes.gd
##
## Builds plain Control + widget trees (no custom draw), assigns recursive
## owners, attaches the checked-in .gd scripts, packs, saves, then frees every
## node so the run exits clean (no ObjectDB/RID leaks). The .tscn files are
## never hand-edited; re-run this script after changing node structure.
##
## Determinism: Godot 4.7 `PackedScene.pack()` stamps each node with a
## `unique_id` derived from runtime instance IDs, which changes on every run.
## The examples reference nodes by path; normalize generated IDs for stable files.

var _failed := false


func _init() -> void:
	print("[generate_example_scenes] Building focused example scenes...")
	_generate("01-playback/playback.gd", "01-playback/playback.tscn", _build_playback)
	_generate("02-game-signals/game_signals.gd", "02-game-signals/game_signals.tscn", _build_game_signals)
	_generate("03-song-form/song_form.gd", "03-song-form/song_form.tscn", _build_song_form)
	if _failed:
		quit(1)
		return
	print("[generate_example_scenes] All scenes generated.")
	quit(0)


func _generate(script_rel: String, save_rel: String, build: Callable) -> void:
	var root: Control = build.call()
	if root == null:
		_failed = true
		return
	var ok := _attach_pack_save(root, script_rel, save_rel)
	root.free()
	if not ok:
		_failed = true


func _attach_pack_save(root: Control, script_rel: String, save_rel: String) -> bool:
	var script_res := load("res://" + script_rel)
	if script_res == null:
		push_error("Failed to load script res://%s" % script_rel)
		return false
	root.set_script(script_res)
	_set_owners_recursive(root, root)

	var packed := PackedScene.new()
	var err := packed.pack(root)
	if err != OK:
		push_error("Failed to pack res://%s: %s" % [save_rel, err])
		return false

	var save_err := ResourceSaver.save(packed, "res://" + save_rel)
	if save_err != OK:
		push_error("Failed to save res://%s: %s" % [save_rel, save_err])
		return false

	if not _strip_unique_ids(save_rel):
		return false
	print("[generate_example_scenes] Saved res://%s" % save_rel)
	return true


func _strip_unique_ids(save_rel: String) -> bool:
	var path := "res://" + save_rel
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		push_error("Failed to reopen res://%s to normalize" % save_rel)
		return false
	var text := file.get_as_text()
	file.close()
	var regex := RegEx.new()
	if regex.compile(" unique_id=\\d+") != OK:
		return false
	var stripped := regex.sub(text, "", true)
	var out := FileAccess.open(path, FileAccess.WRITE)
	if out == null:
		push_error("Failed to write normalized res://%s" % save_rel)
		return false
	out.store_string(stripped)
	out.close()
	return true


func _set_owners_recursive(node: Node, owner: Node) -> void:
	if node != owner:
		node.owner = owner
	for child in node.get_children():
		_set_owners_recursive(child, owner)


func _make_scene_root(scene_name: String) -> Control:
	var root := Control.new()
	root.name = scene_name
	root.set_anchors_preset(Control.PRESET_FULL_RECT)

	var scroll := ScrollContainer.new()
	scroll.name = "ScrollContainer"
	scroll.set_anchors_preset(Control.PRESET_FULL_RECT)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root.add_child(scroll)

	var margin := MarginContainer.new()
	margin.name = "Margin"
	margin.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	margin.add_theme_constant_override("margin_left", 12)
	margin.add_theme_constant_override("margin_right", 12)
	margin.add_theme_constant_override("margin_top", 12)
	margin.add_theme_constant_override("margin_bottom", 12)
	scroll.add_child(margin)

	var main := VBoxContainer.new()
	main.name = "Main"
	main.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	main.add_theme_constant_override("separation", 6)
	margin.add_child(main)
	return root


func _add_label(parent: Control, text: String, size: int = 14, node_name := "") -> Label:
	var l := Label.new()
	l.text = text
	l.add_theme_font_size_override("font_size", size)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if not node_name.is_empty():
		l.name = node_name
	parent.add_child(l)
	return l


func _add_button(parent: Control, text: String, node_name: String) -> Button:
	var b := Button.new()
	b.text = text
	b.name = node_name
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(b)
	return b


func _add_link(parent: Control, text: String, node_name: String) -> LinkButton:
	var l := LinkButton.new()
	l.text = text
	l.name = node_name
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(l)
	return l


func _add_error_label(main: Control) -> Label:
	var err := Label.new()
	err.name = "ErrorLabel"
	err.text = ""
	err.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	err.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	err.add_theme_color_override("font_color", Color(1, 0.4, 0.4))
	err.visible = false
	main.add_child(err)
	return err


func _add_slider_row(vbox: Control, label_text: String, slider_name: String) -> HSlider:
	var h := HBoxContainer.new()
	h.name = slider_name.trim_suffix("Slider") + "Row"
	var lab := Label.new()
	lab.text = label_text
	lab.custom_minimum_size = Vector2(90, 0)
	h.add_child(lab)
	var sl := HSlider.new()
	sl.name = slider_name
	sl.min_value = 0.0
	sl.max_value = 1.0
	sl.step = 0.05
	sl.value = 0.5
	sl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	h.add_child(sl)
	vbox.add_child(h)
	return sl


func _build_playback() -> Control:
	var root := _make_scene_root("Playback")
	var main := root.get_node("ScrollContainer/Margin/Main")

	_add_label(main, "01 — Playback (Suspense Theme)", 18, "TitleLabel")
	_add_label(main, "Title music: generate once at load, not per frame. Set project_secret + recipe + style + arrangement before generate().", 11, "HintLabel")
	_add_label(main, "Status: (pending)", 12, "StatusLabel")
	_add_label(main, "Current section: (none)", 16, "SectionLabel")

	var restart := _add_button(main, "Restart / Regenerate", "RestartButton")
	restart.custom_minimum_size = Vector2(0, 32)

	_add_link(main, "Open web Audio Lab", "WebLabLink")
	_add_error_label(main)
	return root


func _build_game_signals() -> Control:
	var root := _make_scene_root("GameSignals")
	var main := root.get_node("ScrollContainer/Margin/Main")

	_add_label(main, "02 — Game Signals (Racing / neon)", 18, "TitleLabel")
	_add_label(main, "Map game events to set_race_state. Requested and current sections differ: changes commit on bar boundaries.", 11, "HintLabel")
	_add_label(main, "Status: (pending)", 12, "StatusLabel")

	var phases := HBoxContainer.new()
	phases.name = "PhaseButtons"
	for ph in ["garage", "grid", "race", "finish"]:
		var b := Button.new()
		b.text = ph.capitalize()
		b.name = ph.capitalize() + "Button"
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		phases.add_child(b)
	main.add_child(phases)

	_add_slider_row(main, "Intensity:", "IntensitySlider")
	_add_slider_row(main, "Pressure:", "PressureSlider")

	var final_h := HBoxContainer.new()
	final_h.name = "FinalLapRow"
	var final_lab := Label.new()
	final_lab.text = "Final lap:"
	final_h.add_child(final_lab)
	var final_cb := CheckBox.new()
	final_cb.name = "FinalLapCheck"
	final_h.add_child(final_cb)
	main.add_child(final_h)

	_add_button(main, "Apply set_race_state (check result)", "ApplyButton")

	var finish_h := HBoxContainer.new()
	finish_h.name = "FinishButtons"
	for res in ["win", "loss"]:
		var b := Button.new()
		b.text = "Finish " + res.capitalize()
		b.name = "Finish" + res.capitalize() + "Button"
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		finish_h.add_child(b)
	main.add_child(finish_h)

	_add_label(main, "Both finish outcomes use the same closing section.", 11, "FinishNoteLabel")

	_add_label(main, "Requested: (none)", 12, "RequestedLabel")
	_add_label(main, "Current section: (none)", 14, "CurrentLabel")
	_add_label(main, "set_race_state returned: (n/a)", 11, "ResultLabel")

	_add_link(main, "Open web Audio Lab", "WebLabLink")
	_add_error_label(main)
	return root


func _build_song_form() -> Control:
	var root := _make_scene_root("SongForm")
	var main := root.get_node("ScrollContainer/Margin/Main")

	_add_label(main, "03 — Song Form (suspense / terminal / extended)", 18, "TitleLabel")
	_add_label(main, "set_trace_state + cue_section + set_form_hold + advance_form. Check returns + is_form_held + get_current_section.", 11, "HintLabel")
	_add_label(main, "Status: (pending)", 12, "StatusLabel")

	var form_h := HBoxContainer.new()
	form_h.name = "FormButtons"
	for act in ["Hold", "Resume", "Advance", "CueChorus"]:
		var b := Button.new()
		b.text = act.replace("CueChorus", "Cue chorus")
		b.name = act + "Button"
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		form_h.add_child(b)
	main.add_child(form_h)

	var trace_h := HBoxContainer.new()
	trace_h.name = "TraceButtons"
	for ph in ["scan", "alert", "complete", "extract"]:
		var b := Button.new()
		b.text = ph
		b.name = ph.capitalize() + "Button"
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		trace_h.add_child(b)
	main.add_child(trace_h)

	_add_slider_row(main, "Heat:", "HeatSlider")
	_add_slider_row(main, "Focus:", "FocusSlider")
	_add_slider_row(main, "Progress:", "ProgressSlider")

	_add_button(main, "Apply set_trace_state (check result)", "ApplyTraceButton")

	_add_label(main, "Form held: false", 12, "HeldLabel")
	_add_label(main, "Current section: (none)", 14, "CurrentLabel")
	_add_label(main, "Last op returned: (n/a)", 11, "ResultLabel")

	_add_link(main, "Open web Audio Lab", "WebLabLink")
	_add_error_label(main)
	return root
