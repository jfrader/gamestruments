extends Control

const SURFACE := Color("151a21")
const SURFACE_RAISED := Color("1d242d")
const INK := Color("edf1e8")
const MUTED := Color("87919b")
const ACCENT := Color("f4b740")
const BG := Color("0c1015")

var player: Node = null
var warning_label: Label
var status_label: Label

# UI controls
var style_option: OptionButton
var seed_edit: LineEdit
var melody_edit: LineEdit
var harmony_edit: LineEdit
var drive_edit: LineEdit
var bass_edit: LineEdit
var energy_slider: HSlider
var complexity_slider: HSlider
var brightness_slider: HSlider
var syncopation_slider: HSlider
var intensity_value: Label  # reuse energy as intensity proxy for state calls


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_build_interface()

	if not ClassDB.class_exists("GamestrumentsPlayer"):
		_show_extension_warning()
		return

	# Do not auto-instantiate here. The player is created on first Generate press
	# (lazy) so that headless smoke boots without triggering internal Rust bind
	# issues on the current gdext 0.5 path during exit. In a real editor session
	# the user presses Generate to start audio.
	_apply_defaults_to_ui()
	status_label.text = "Extension loaded — press Generate"


func _build_interface() -> void:
	var background := ColorRect.new()
	background.color = BG
	background.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	background.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(background)

	var margin := MarginContainer.new()
	margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	margin.add_theme_constant_override("margin_left", 24)
	margin.add_theme_constant_override("margin_top", 20)
	margin.add_theme_constant_override("margin_right", 24)
	margin.add_theme_constant_override("margin_bottom", 20)
	add_child(margin)

	var page := VBoxContainer.new()
	page.add_theme_constant_override("separation", 14)
	margin.add_child(page)

	# Header
	var header := HBoxContainer.new()
	header.add_theme_constant_override("separation", 16)
	page.add_child(header)

	var title_box := VBoxContainer.new()
	title_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(title_box)

	var eyebrow := _make_label("KIT DEMO — GODOT 4 GDEXTENSION", 11, ACCENT)
	title_box.add_child(eyebrow)

	var title := _make_label("GamestrumentsPlayer", 26, INK)
	title_box.add_child(title)

	var subtitle := _make_label("Load-time generation + adaptive race arc (bar-quantized)", 12, MUTED)
	title_box.add_child(subtitle)

	status_label = _make_label("", 12, INK)
	status_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	header.add_child(status_label)

	# Warning (hidden until needed)
	warning_label = _make_label("", 13, Color(0.95, 0.6, 0.2))
	warning_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	warning_label.visible = false
	page.add_child(warning_label)

	# Controls row
	var body := HBoxContainer.new()
	body.add_theme_constant_override("separation", 12)
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	page.add_child(body)

	# Left: params
	var params_panel := _make_panel()
	params_panel.custom_minimum_size = Vector2(280, 0)
	body.add_child(params_panel)
	_build_params(params_panel)

	# Right: actions + states
	var actions_panel := _make_panel()
	actions_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(actions_panel)
	_build_actions(actions_panel)

	# Footer note
	var note := _make_label(
		"After Generate, use the section buttons to drive set_race_state. " +
		"Transitions quantize to bar boundaries inside the extension.",
		11, MUTED
	)
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	page.add_child(note)


func _build_params(panel: PanelContainer) -> void:
	var stack := VBoxContainer.new()
	stack.add_theme_constant_override("separation", 8)
	panel.add_child(stack)

	stack.add_child(_make_label("GENERATION", 12, ACCENT))

	# Style
	stack.add_child(_make_label("Style", 11, MUTED))
	style_option = OptionButton.new()
	for s in ["fusion", "neon", "funk", "chip"]:
		style_option.add_item(s)
	style_option.selected = 2  # funk
	style_option.custom_minimum_size = Vector2(0, 28)
	stack.add_child(style_option)

	# Seed
	stack.add_child(_make_label("Seed", 11, MUTED))
	seed_edit = LineEdit.new()
	seed_edit.text = "kit-demo-001"
	seed_edit.placeholder_text = "any string"
	seed_edit.custom_minimum_size = Vector2(0, 28)
	stack.add_child(seed_edit)

	# Voices
	stack.add_child(_make_label("Voices (empty = style default)", 11, MUTED))
	var voices := [
		{"ref": "melody", "lbl": "Melody"},
		{"ref": "harmony", "lbl": "Harmony"},
		{"ref": "drive", "lbl": "Drive"},
		{"ref": "bass", "lbl": "Bass"},
	]
	for v in voices:
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		var l := _make_label(v.lbl, 10, MUTED)
		l.custom_minimum_size.x = 64
		row.add_child(l)
		var edit := LineEdit.new()
		edit.placeholder_text = "(default)"
		edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		edit.custom_minimum_size = Vector2(0, 26)
		row.add_child(edit)
		stack.add_child(row)
		if v.ref == "melody": melody_edit = edit
		elif v.ref == "harmony": harmony_edit = edit
		elif v.ref == "drive": drive_edit = edit
		elif v.ref == "bass": bass_edit = edit

	stack.add_child(HSeparator.new())
	stack.add_child(_make_label("TRAITS (0..1)", 12, ACCENT))

	# Trait sliders
	var traits := [
		{"ref": "energy", "lbl": "Energy", "def": 0.62},
		{"ref": "complexity", "lbl": "Complexity", "def": 0.60},
		{"ref": "brightness", "lbl": "Brightness", "def": 0.52},
		{"ref": "syncopation", "lbl": "Syncopation", "def": 0.70},
	]
	for t in traits:
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		var l := _make_label(t.lbl, 10, MUTED)
		l.custom_minimum_size.x = 78
		row.add_child(l)
		var sl := HSlider.new()
		sl.min_value = 0.0
		sl.max_value = 1.0
		sl.step = 0.01
		sl.value = t.def
		sl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		row.add_child(sl)
		var val := _make_label("%.2f" % t.def, 10, INK)
		val.custom_minimum_size.x = 32
		row.add_child(val)
		sl.value_changed.connect(func(v: float): val.text = "%.2f" % v)
		stack.add_child(row)

		if t.ref == "energy": energy_slider = sl
		elif t.ref == "complexity": complexity_slider = sl
		elif t.ref == "brightness": brightness_slider = sl
		elif t.ref == "syncopation": syncopation_slider = sl

	# Generate button
	stack.add_child(HSeparator.new())
	var gen_btn := Button.new()
	gen_btn.text = "Generate"
	gen_btn.custom_minimum_size = Vector2(0, 36)
	gen_btn.pressed.connect(_on_generate_pressed)
	stack.add_child(gen_btn)


func _build_actions(panel: PanelContainer) -> void:
	var stack := VBoxContainer.new()
	stack.add_theme_constant_override("separation", 8)
	panel.add_child(stack)

	stack.add_child(_make_label("ADAPTIVE ARC — set_race_state", 12, ACCENT))
	stack.add_child(_make_label(
		"phase, intensity (energy slider), pressure, final_lap",
		10, MUTED
	))

	var btn_grid := GridContainer.new()
	btn_grid.columns = 2
	btn_grid.add_theme_constant_override("h_separation", 6)
	btn_grid.add_theme_constant_override("v_separation", 6)
	stack.add_child(btn_grid)

	var sections := [
		{"text": "Garage", "phase": "garage", "use_int": false, "press": 0.0, "fin": false},
		{"text": "Grid", "phase": "grid", "use_int": false, "press": 0.1, "fin": false},
		{"text": "Cruise", "phase": "race", "use_int": true, "press": 0.15, "fin": false},
		{"text": "Attack", "phase": "race", "use_int": true, "press": 0.8, "fin": false},
		{"text": "Final Lap", "phase": "race", "use_int": true, "press": 0.9, "fin": true},
		{"text": "Victory", "phase": "finish", "use_int": true, "press": 0.0, "fin": false, "result": "win"},
	]

	for sec in sections:
		var b := Button.new()
		b.text = sec.text
		b.custom_minimum_size = Vector2(108, 32)
		b.pressed.connect(_on_section_pressed.bind(
			sec.phase, sec.use_int, sec.press, sec.fin, String(sec.get("result", "none"))
		))
		btn_grid.add_child(b)

	stack.add_child(HSeparator.new())
	status_label = _make_label("No score yet", 11, MUTED)
	status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	stack.add_child(status_label)


func _show_extension_warning() -> void:
	warning_label.text = (
		"GamestrumentsPlayer not found in ClassDB.\n\n" +
		"Build: cargo build -p gamestruments-godot\n" +
		"Copy libgamestruments_godot.so + gamestruments.gdextension into place (see kit/demo/README.md).\n" +
		"Restart Godot after placing the native extension."
	)
	warning_label.visible = true


func _create_player() -> void:
	player = ClassDB.instantiate("GamestrumentsPlayer")
	if player == null:
		_show_extension_warning()
		return
	add_child(player)


func _apply_defaults_to_ui() -> void:
	if style_option:
		style_option.selected = 2  # funk
	if seed_edit:
		seed_edit.text = "kit-demo-001"
	if energy_slider: energy_slider.value = 0.62
	if complexity_slider: complexity_slider.value = 0.60
	if brightness_slider: brightness_slider.value = 0.52
	if syncopation_slider: syncopation_slider.value = 0.70


func _on_generate_pressed() -> void:
	if player == null:
		_create_player()
	if player == null:
		status_label.text = "Failed to create player"
		return

	player.set("project_secret", "demo-secret")
	player.set("style", style_option.get_item_text(style_option.selected))
	player.set("melody_voice", melody_edit.text if melody_edit else "")
	player.set("harmony_voice", harmony_edit.text if harmony_edit else "")
	player.set("drive_voice", drive_edit.text if drive_edit else "")
	player.set("bass_voice", bass_edit.text if bass_edit else "")
	player.set("energy", energy_slider.value if energy_slider else 0.62)
	player.set("complexity", complexity_slider.value if complexity_slider else 0.60)
	player.set("brightness", brightness_slider.value if brightness_slider else 0.52)
	player.set("syncopation", syncopation_slider.value if syncopation_slider else 0.70)

	var level_seed := seed_edit.text if seed_edit and seed_edit.text != "" else "kit-demo-001"
	player.call("generate", level_seed)

	status_label.text = "Generated seed: %s  style: %s" % [level_seed, style_option.get_item_text(style_option.selected)]

	# Kick off the arc at garage so audio starts immediately
	_on_section_pressed("garage", false, 0.0, false, "none")


func _on_section_pressed(
	phase: String,
	use_intensity_slider: bool,
	pressure: float,
	final_lap: bool,
	finish_result: String = "none",
) -> void:
	if player == null:
		return

	var intensity := 0.35
	if use_intensity_slider and energy_slider:
		intensity = energy_slider.value

	player.call("set_race_state", phase, intensity, pressure, final_lap, finish_result)

	status_label.text = "set_race_state(\"%s\", %.2f, %.2f, %s, \"%s\")" % [
		phase, intensity, pressure, str(final_lap), finish_result
	]


func _make_panel() -> PanelContainer:
	var p := PanelContainer.new()
	var sb := StyleBoxFlat.new()
	sb.bg_color = SURFACE
	sb.corner_radius_top_left = 8
	sb.corner_radius_top_right = 8
	sb.corner_radius_bottom_left = 8
	sb.corner_radius_bottom_right = 8
	sb.content_margin_left = 10
	sb.content_margin_top = 10
	sb.content_margin_right = 10
	sb.content_margin_bottom = 10
	p.add_theme_stylebox_override("panel", sb)
	return p


func _make_label(text_value: String, font_size: int, color: Color) -> Label:
	var l := Label.new()
	l.text = text_value
	l.add_theme_font_size_override("font_size", font_size)
	l.add_theme_color_override("font_color", color)
	return l
