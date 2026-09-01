extends Control

const SCORE_PATH := "res://data/pocket-circuit.score.json"
const SURFACE := Color("151a21")
const SURFACE_RAISED := Color("1d242d")
const INK := Color("edf1e8")
const MUTED := Color("87919b")
const ACCENT := Color("f4b740")

var score: Dictionary = {}
var game_state := {
	"numeric": {
		"intensity": 0.35,
		"positionPressure": 0.2,
		"finalLap": 0.0,
	},
	"categorical": {
		"racePhase": "garage",
		"finishResult": "none",
	},
}
var current_section := ""
var pending_section := ""
var transition: Dictionary = {}
var current_tick := 0
var elapsed_seconds := 0.0
var bar_ticks := 3840
var ticks_per_second := 1920.0
var section_bars: Dictionary = {}
var section_values: Dictionary = {}
var phase_buttons: Dictionary = {}
var section_label: Label
var transport_label: Label
var state_label: Label
var transition_label: Label


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	score = _load_score()
	if score.is_empty():
		return
	current_section = str(score.get("defaultSection", ""))
	bar_ticks = int(score.get("beatsPerBar", 4)) * int(score.get("ticksPerBeat", 960))
	ticks_per_second = float(score.get("bpm", 120)) * float(score.get("ticksPerBeat", 960)) / 60.0
	_build_interface()
	_refresh_interface()


func _process(delta: float) -> void:
	if score.is_empty():
		return
	elapsed_seconds += delta
	current_tick = int(floor(elapsed_seconds * ticks_per_second))
	_advance_transport()
	_refresh_interface()


func _load_score() -> Dictionary:
	if not FileAccess.file_exists(SCORE_PATH):
		push_error("Portable score not found: %s" % SCORE_PATH)
		return {}
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(SCORE_PATH))
	if typeof(parsed) != TYPE_DICTIONARY:
		push_error("Portable score is not a JSON object")
		return {}
	var loaded := parsed as Dictionary
	if int(loaded.get("schemaVersion", 0)) != 1:
		push_error("Unsupported portable score schema")
		return {}
	if (loaded.get("sections", []) as Array).is_empty():
		push_error("Portable score has no sections")
		return {}
	return loaded


func _build_interface() -> void:
	var background := ColorRect.new()
	background.color = Color("0c1015")
	background.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	background.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(background)

	var margin := MarginContainer.new()
	margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	margin.add_theme_constant_override("margin_left", 28)
	margin.add_theme_constant_override("margin_top", 24)
	margin.add_theme_constant_override("margin_right", 28)
	margin.add_theme_constant_override("margin_bottom", 24)
	add_child(margin)

	var page := VBoxContainer.new()
	page.add_theme_constant_override("separation", 18)
	margin.add_child(page)

	var header := HBoxContainer.new()
	header.add_theme_constant_override("separation", 20)
	page.add_child(header)

	var title_stack := VBoxContainer.new()
	title_stack.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(title_stack)
	var eyebrow := _make_label("ENGINE-AGNOSTIC ADAPTIVE MUSIC", 12, ACCENT)
	title_stack.add_child(eyebrow)
	var title := _make_label(str(score.get("title", "Portable score")), 30, INK)
	title_stack.add_child(title)
	var subtitle := _make_label("Loaded from JSON  /  schema v%s  /  no Strudel runtime" % score.get("schemaVersion", 0), 13, MUTED)
	title_stack.add_child(subtitle)

	transport_label = _make_label("", 15, INK)
	transport_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	header.add_child(transport_label)

	var body := HBoxContainer.new()
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", 16)
	page.add_child(body)

	var controls := _make_panel()
	controls.custom_minimum_size = Vector2(250, 0)
	body.add_child(controls)
	_build_controls(controls)

	var mixer := _make_panel()
	mixer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(mixer)
	_build_mixer(mixer)

	var inspector := _make_panel()
	inspector.custom_minimum_size = Vector2(265, 0)
	body.add_child(inspector)
	_build_inspector(inspector)


func _build_controls(panel: PanelContainer) -> void:
	var stack := VBoxContainer.new()
	stack.add_theme_constant_override("separation", 12)
	panel.add_child(stack)
	stack.add_child(_make_label("GAME STATE", 13, ACCENT))
	stack.add_child(_make_label("Race phase", 12, MUTED))

	var phase_grid := GridContainer.new()
	phase_grid.columns = 2
	phase_grid.add_theme_constant_override("h_separation", 8)
	phase_grid.add_theme_constant_override("v_separation", 8)
	stack.add_child(phase_grid)
	for phase in ["garage", "grid", "race", "finish"]:
		var button := Button.new()
		button.text = str(phase).capitalize()
		button.toggle_mode = true
		button.custom_minimum_size = Vector2(105, 38)
		button.pressed.connect(_set_phase.bind(str(phase)))
		phase_grid.add_child(button)
		phase_buttons[str(phase)] = button

	stack.add_child(HSeparator.new())
	var intensity := HSlider.new()
	intensity.min_value = 0.0
	intensity.max_value = 1.0
	intensity.step = 0.01
	intensity.value = 0.35
	intensity.value_changed.connect(_set_numeric.bind("intensity"))
	stack.add_child(_make_label("Speed intensity", 12, MUTED))
	stack.add_child(intensity)

	var pressure := HSlider.new()
	pressure.min_value = 0.0
	pressure.max_value = 1.0
	pressure.step = 0.01
	pressure.value = 0.2
	pressure.value_changed.connect(_set_numeric.bind("positionPressure"))
	stack.add_child(_make_label("Position pressure", 12, MUTED))
	stack.add_child(pressure)

	var final_lap := CheckButton.new()
	final_lap.text = "Final lap"
	final_lap.toggled.connect(_set_final_lap)
	stack.add_child(final_lap)

	stack.add_child(HSeparator.new())
	state_label = _make_label("", 12, MUTED)
	state_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	stack.add_child(state_label)


func _build_mixer(panel: PanelContainer) -> void:
	var stack := VBoxContainer.new()
	stack.add_theme_constant_override("separation", 10)
	panel.add_child(stack)
	stack.add_child(_make_label("SECTION MIX", 13, ACCENT))
	section_label = _make_label("", 24, INK)
	stack.add_child(section_label)
	stack.add_child(_make_label("Equal-power gains across a bar-quantized two-bar transition", 12, MUTED))
	stack.add_child(HSeparator.new())

	for section_value in score.get("sections", []):
		var section := section_value as Dictionary
		var section_id := str(section.get("id", ""))
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 10)
		stack.add_child(row)
		var section_name_label := _make_label(str(section.get("label", section_id)), 13, INK)
		section_name_label.custom_minimum_size.x = 112
		row.add_child(section_name_label)
		var bar := ProgressBar.new()
		bar.min_value = 0.0
		bar.max_value = 1.0
		bar.show_percentage = false
		bar.custom_minimum_size = Vector2(0, 18)
		bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		bar.add_theme_stylebox_override("background", _style(Color("0f141a"), 5))
		var section_color := Color.from_string(str(section.get("color", "#f4b740")), ACCENT)
		bar.add_theme_stylebox_override("fill", _style(section_color, 5))
		row.add_child(bar)
		var value := _make_label("0%", 12, MUTED)
		value.custom_minimum_size.x = 38
		value.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		row.add_child(value)
		section_bars[section_id] = bar
		section_values[section_id] = value


func _build_inspector(panel: PanelContainer) -> void:
	var stack := VBoxContainer.new()
	stack.add_theme_constant_override("separation", 12)
	panel.add_child(stack)
	stack.add_child(_make_label("PORTABLE SCORE", 13, ACCENT))
	stack.add_child(_make_stat("Score", str(score.get("id", ""))))
	stack.add_child(_make_stat("Tempo", "%s BPM" % score.get("bpm", 0)))
	stack.add_child(_make_stat("Clock", "%s ticks / beat" % score.get("ticksPerBeat", 0)))
	stack.add_child(_make_stat("Sections", str((score.get("sections", []) as Array).size())))
	stack.add_child(_make_stat("Rules", str((score.get("rules", []) as Array).size())))
	stack.add_child(_make_stat("Crossfade", "%s bars" % score.get("crossfadeBars", 0)))
	stack.add_child(HSeparator.new())
	transition_label = _make_label("", 12, MUTED)
	transition_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	stack.add_child(transition_label)
	var note := _make_label("This project reads the exported event data directly. The JavaScript generator and Strudel authoring layer are not present.", 12, MUTED)
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	note.size_flags_vertical = Control.SIZE_EXPAND_FILL
	stack.add_child(note)


func _set_phase(phase: String) -> void:
	game_state.categorical.racePhase = phase
	game_state.categorical.finishResult = "win" if phase == "finish" else "none"
	_request_state()


func _set_numeric(value: float, key: String) -> void:
	game_state.numeric[key] = value
	_request_state()


func _set_final_lap(enabled: bool) -> void:
	game_state.numeric.finalLap = 1.0 if enabled else 0.0
	_request_state()


func _request_state() -> void:
	_request_section(_select_section())


func _select_section() -> String:
	var selected := str(score.get("defaultSection", ""))
	var selected_priority := -2147483648
	for rule_value in score.get("rules", []):
		var rule := rule_value as Dictionary
		var priority := int(rule.get("priority", 0))
		if priority > selected_priority and _condition_matches(rule.get("when", {}) as Dictionary):
			selected = str(rule.get("target", selected))
			selected_priority = priority
	return selected


func _condition_matches(condition: Dictionary) -> bool:
	for key_value in (condition.get("numeric", {}) as Dictionary).keys():
		var key := str(key_value)
		var limits := (condition.get("numeric", {}) as Dictionary).get(key, {}) as Dictionary
		var value := float(game_state.numeric.get(key, 0.0))
		if limits.has("min") and value < float(limits.min):
			return false
		if limits.has("max") and value > float(limits.max):
			return false
	for key_value in (condition.get("categorical", {}) as Dictionary).keys():
		var key := str(key_value)
		var expected: Variant = (condition.get("categorical", {}) as Dictionary).get(key)
		var actual := str(game_state.categorical.get(key, ""))
		if typeof(expected) == TYPE_ARRAY:
			if not (expected as Array).has(actual):
				return false
		elif actual != str(expected):
			return false
	return true


func _request_section(target: String) -> void:
	if target == current_section and transition.is_empty():
		pending_section = ""
		return
	if not transition.is_empty():
		if target == str(transition.get("to", "")):
			pending_section = ""
			return
		if current_tick < int(transition.get("startTick", 0)):
			if target == current_section:
				transition.clear()
			else:
				transition = _make_transition(current_section, target)
			return
		pending_section = target
		return
	transition = _make_transition(current_section, target)


func _make_transition(from: String, to: String) -> Dictionary:
	var start_tick := int(ceil(float(current_tick) / float(bar_ticks))) * bar_ticks
	return {
		"from": from,
		"to": to,
		"requestedAtTick": current_tick,
		"startTick": start_tick,
		"endTick": start_tick + int(score.get("crossfadeBars", 2)) * bar_ticks,
	}


func _advance_transport() -> void:
	if transition.is_empty() or current_tick < int(transition.get("endTick", 0)):
		return
	current_section = str(transition.get("to", current_section))
	transition.clear()
	if pending_section != "" and pending_section != current_section:
		var queued := pending_section
		pending_section = ""
		transition = _make_transition(current_section, queued)
	else:
		pending_section = ""


func _mix_gains() -> Dictionary:
	var gains := {current_section: 1.0}
	if transition.is_empty() or current_tick < int(transition.get("startTick", 0)):
		return gains
	var start_tick := int(transition.startTick)
	var end_tick := int(transition.endTick)
	if current_tick >= end_tick:
		return {str(transition.to): 1.0}
	var progress := float(current_tick - start_tick) / float(end_tick - start_tick)
	return {
		str(transition.from): cos(progress * PI * 0.5),
		str(transition.to): sin(progress * PI * 0.5),
	}


func _refresh_interface() -> void:
	var bar_number := floori(float(current_tick) / float(bar_ticks)) + 1
	var beat_ticks := int(score.get("ticksPerBeat", 960))
	var beat_number := floori(float(current_tick % bar_ticks) / float(beat_ticks)) + 1
	transport_label.text = "BAR %02d  /  BEAT %02d\n%s BPM" % [bar_number, beat_number, score.get("bpm", 0)]

	var gains := _mix_gains()
	var dominant := current_section
	var dominant_gain := 0.0
	for section_id_value in section_bars.keys():
		var section_id := str(section_id_value)
		var gain := float(gains.get(section_id, 0.0))
		(section_bars[section_id] as ProgressBar).value = gain
		(section_values[section_id] as Label).text = "%d%%" % roundi(gain * 100.0)
		if gain > dominant_gain:
			dominant = section_id
			dominant_gain = gain
	section_label.text = _section_name(dominant)

	for phase_value in phase_buttons.keys():
		var phase := str(phase_value)
		(phase_buttons[phase] as Button).button_pressed = str(game_state.categorical.racePhase) == phase
	state_label.text = "intensity %.2f\npressure %.2f\nfinalLap %d" % [game_state.numeric.intensity, game_state.numeric.positionPressure, int(game_state.numeric.finalLap)]
	if transition.is_empty():
		transition_label.text = "Stable on %s\nPending: %s" % [_section_name(current_section), pending_section if pending_section != "" else "none"]
	else:
		transition_label.text = "%s -> %s\nStarts bar %d / ends bar %d\nPending: %s" % [
			_section_name(str(transition.from)),
			_section_name(str(transition.to)),
			floori(float(transition.startTick) / float(bar_ticks)) + 1,
			floori(float(transition.endTick) / float(bar_ticks)) + 1,
			pending_section if pending_section != "" else "none",
		]


func _section_name(section_id: String) -> String:
	for section_value in score.get("sections", []):
		var section := section_value as Dictionary
		if str(section.get("id", "")) == section_id:
			return str(section.get("label", section_id))
	return section_id


func _make_panel() -> PanelContainer:
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", _style(SURFACE, 12, 18))
	return panel


func _make_label(text_value: String, font_size: int, color: Color) -> Label:
	var label := Label.new()
	label.text = text_value
	label.add_theme_font_size_override("font_size", font_size)
	label.add_theme_color_override("font_color", color)
	return label


func _make_stat(stat_name: String, value: String) -> Control:
	var row := VBoxContainer.new()
	row.add_child(_make_label(stat_name.to_upper(), 10, MUTED))
	var value_label := _make_label(value, 12, INK)
	value_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	row.add_child(value_label)
	return row


func _style(color: Color, radius: int, margin: int = 0) -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = color
	style.corner_radius_top_left = radius
	style.corner_radius_top_right = radius
	style.corner_radius_bottom_left = radius
	style.corner_radius_bottom_right = radius
	style.content_margin_left = margin
	style.content_margin_top = margin
	style.content_margin_right = margin
	style.content_margin_bottom = margin
	return style
