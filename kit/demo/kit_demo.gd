extends Control

const RaceModel = preload("res://race_model.gd")
const RaceMusic = preload("res://race_music.gd")
const BG := Color("0b1519")
const INK := Color("edf4e8")
const MUTED := Color("8aaca9")
const RIVAL := Color("ff936c")
const CENTER := Vector2(480, 321)
const CIRCUITS := [
	{"name": "NIGHT CIRCUIT", "style": "neon", "style_label": "NEON", "seed": "night-circuit-001", "accent": Color("c9f36b"), "radius": Vector2(333, 155)},
	{"name": "HARBOR SPRINT", "style": "funk", "style_label": "POCKET FUNK", "seed": "harbor-sprint-002", "accent": Color("ffb347"), "radius": Vector2(300, 178)},
	{"name": "CANYON RUN", "style": "fusion", "style_label": "FUSION", "seed": "canyon-run-003", "accent": Color("6be3ff"), "radius": Vector2(350, 138)},
	{"name": "MICRO MILE", "style": "chip", "style_label": "MICRO MOTOR", "seed": "micro-mile-004", "accent": Color("ff8ad8"), "radius": Vector2(318, 162)},
]

var race = RaceModel.new()
var music: Node
var paused := false
var music_clock := 0.0
var circuit_index := 0
var circuit: Dictionary = CIRCUITS[0]
var accent: Color = CIRCUITS[0]["accent"]
var radius: Vector2 = CIRCUITS[0]["radius"]
var start_button: Button
var prev_button: Button
var next_button: Button
var pause_button: Button
var intro_button: Button
var intro_visible := true
var hud: Control
var font: Font


func _ready() -> void:
	font = ThemeDB.fallback_font
	music = RaceMusic.new()
	music.name = "RaceMusic"
	add_child(music)
	hud = Control.new()
	hud.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(hud)
	start_button = _button("RACE  /  ENTER", Vector2(385, 367), Vector2(190, 42))
	start_button.pressed.connect(start_race)
	prev_button = _button("PREV", Vector2(285, 367), Vector2(90, 42))
	prev_button.pressed.connect(_on_prev_pressed)
	next_button = _button("NEXT", Vector2(585, 367), Vector2(90, 42))
	next_button.pressed.connect(_on_next_pressed)
	pause_button = _button("PAUSE  /  ESC", Vector2(792, 24), Vector2(140, 34))
	pause_button.pressed.connect(toggle_pause)
	pause_button.visible = false
	intro_button = _button("GOT IT  /  ENTER", Vector2(385, 376), Vector2(190, 42))
	intro_button.pressed.connect(dismiss_intro)
	_set_circuit(0)
	intro_button.grab_focus()
	get_window().focus_exited.connect(_on_focus_exited)
	resized.connect(_layout)
	_layout()


func _button(caption: String, at: Vector2, dimensions: Vector2) -> Button:
	var button := Button.new()
	button.text = caption
	button.position = at
	button.size = dimensions
	button.add_theme_font_size_override("font_size", 14)
	button.add_theme_color_override("font_color", BG)
	button.add_theme_color_override("font_focus_color", BG)
	button.add_theme_color_override("font_hover_color", BG)
	button.add_theme_color_override("font_pressed_color", BG)
	var style := StyleBoxFlat.new()
	style.bg_color = accent
	style.set_corner_radius_all(4)
	button.add_theme_stylebox_override("normal", style)
	var hover := style.duplicate() as StyleBoxFlat
	hover.bg_color = INK
	button.add_theme_stylebox_override("hover", hover)
	button.add_theme_stylebox_override("pressed", hover)
	hud.add_child(button)
	return button


func _set_circuit(index: int) -> void:
	circuit_index = wrapi(index, 0, CIRCUITS.size())
	circuit = CIRCUITS[circuit_index]
	accent = circuit["accent"]
	radius = circuit["radius"]
	race = RaceModel.new()
	paused = false
	music.process_mode = Node.PROCESS_MODE_INHERIT
	music_clock = 0.0
	if music.configure(circuit["style"], circuit["seed"]):
		music.sync_race(race.music_state())
	_tint_buttons()
	queue_redraw()


func _tint_buttons() -> void:
	for button in [start_button, prev_button, next_button, pause_button, intro_button]:
		var normal := button.get_theme_stylebox("normal") as StyleBoxFlat
		if normal != null:
			normal.bg_color = accent


func dismiss_intro() -> void:
	if not intro_visible:
		return
	intro_visible = false
	intro_button.release_focus()
	start_button.grab_focus()


func _on_prev_pressed() -> void:
	_set_circuit(circuit_index - 1)


func _on_next_pressed() -> void:
	_set_circuit(circuit_index + 1)


func _layout() -> void:
	var factor := minf(size.x / 960.0, size.y / 620.0)
	hud.scale = Vector2.ONE * factor
	hud.position = (size - Vector2(960, 620) * factor) * 0.5
	queue_redraw()


func start_race() -> void:
	race.start()
	paused = false
	music.process_mode = Node.PROCESS_MODE_INHERIT
	music_clock = 0.0
	music.sync_race(race.music_state())
	start_button.release_focus()


func toggle_pause() -> void:
	if race.phase not in ["race", "grid"]:
		return
	paused = not paused
	pause_button.release_focus()
	# Disabling this subtree freezes native synthesis/transport as well as playback.
	music.process_mode = Node.PROCESS_MODE_DISABLED if paused else Node.PROCESS_MODE_INHERIT


func _on_focus_exited() -> void:
	if not paused and race.phase in ["race", "grid"]:
		toggle_pause()


func _unhandled_key_input(event: InputEvent) -> void:
	if not event.is_pressed() or event.is_echo():
		return
	if intro_visible:
		if event.keycode in [KEY_ENTER, KEY_SPACE, KEY_ESCAPE, KEY_R]:
			dismiss_intro()
		return
	if race.phase in ["garage", "finish"] and event.keycode in [KEY_A, KEY_LEFT]:
		_set_circuit(circuit_index - 1)
	elif race.phase in ["garage", "finish"] and event.keycode in [KEY_D, KEY_RIGHT]:
		_set_circuit(circuit_index + 1)
	elif event.keycode == KEY_ENTER and race.phase in ["garage", "finish"]:
		start_race()
	elif event.keycode == KEY_R:
		start_race()
	elif event.keycode == KEY_ESCAPE:
		toggle_pause()


func _physics_process(delta: float) -> void:
	if intro_visible:
		start_button.visible = false
		prev_button.visible = false
		next_button.visible = false
		pause_button.visible = false
		intro_button.visible = true
		queue_redraw()
		return
	intro_button.visible = false
	if not paused:
		var steering := float(Input.is_physical_key_pressed(KEY_D) or Input.is_physical_key_pressed(KEY_RIGHT)) - float(Input.is_physical_key_pressed(KEY_A) or Input.is_physical_key_pressed(KEY_LEFT))
		race.step(delta, Input.is_physical_key_pressed(KEY_W) or Input.is_physical_key_pressed(KEY_UP), steering, Input.is_physical_key_pressed(KEY_SPACE), Input.is_physical_key_pressed(KEY_S) or Input.is_physical_key_pressed(KEY_DOWN))
		music_clock -= delta
		if music_clock <= 0.0:
			music.sync_race(race.music_state())
			music_clock = 0.2
	start_button.visible = race.phase in ["garage", "finish"]
	start_button.text = "RACE AGAIN  /  ENTER" if race.phase == "finish" else "RACE  /  ENTER"
	prev_button.visible = start_button.visible
	next_button.visible = start_button.visible
	pause_button.visible = race.phase in ["grid", "race"]
	pause_button.text = "RESUME  /  ESC" if paused else "PAUSE  /  ESC"
	queue_redraw()


func _track_point(progress: float, lane: float = 0.0) -> Vector2:
	var angle := progress * TAU - PI * 0.5
	var base := Vector2(cos(angle) * radius.x, sin(angle) * radius.y)
	var normal := Vector2(cos(angle) / radius.x, sin(angle) / radius.y).normalized()
	return CENTER + base + normal * lane


func _ring(lane: float) -> PackedVector2Array:
	var points := PackedVector2Array()
	for index in range(161):
		points.append(_track_point(index / 160.0, lane))
	return points


func _text(at: Vector2, text: String, font_size: int, color: Color = INK) -> void:
	draw_string(font, at, text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, color)


func _centered(y: float, text: String, font_size: int, color: Color = INK) -> void:
	_text(Vector2(480 - font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x * 0.5, y), text, font_size, color)


func _draw() -> void:
	if font == null:
		return
	draw_rect(Rect2(Vector2.ZERO, size), BG)
	var factor := minf(size.x / 960.0, size.y / 620.0)
	var origin := (size - Vector2(960, 620) * factor) * 0.5
	draw_set_transform(origin, 0.0, Vector2.ONE * factor)
	for x in range(0, 960, 40):
		draw_line(Vector2(x, 90), Vector2(x, 540), Color("112226"))
	for y in range(100, 541, 40):
		draw_line(Vector2(20, y), Vector2(940, y), Color("112226"))
	_draw_track()
	_draw_car(race.rival_progress, race.rival_lane, race.rival_heading, RIVAL, false)
	_draw_car(race.progress, race.lane, race.heading, accent, race.boosting)
	_draw_hud()
	_draw_island()
	_draw_intro()
	draw_set_transform(Vector2.ZERO)


func _draw_track() -> void:
	draw_polyline(_ring(0), Color("060d10"), 94.0, true)
	draw_polyline(_ring(0), Color("33494a"), 80.0, true)
	draw_polyline(_ring(0), Color("202f34"), 69.0, true)
	for side in [-1, 1]:
		for index in range(80):
			var p := _track_point(index / 80.0, side * 38.0)
			var q := _track_point((index + 0.72) / 80.0, side * 38.0)
			draw_line(p, q, accent.darkened(0.42) if index % 2 == 0 else Color("1a3032"), 5.0, true)
	for index in range(48):
		draw_line(_track_point(index / 48.0), _track_point((index + 0.4) / 48.0), Color("55716e"), 1.5, true)
	var line := _track_point(0.0)
	for row in range(8):
		for col in range(2):
			draw_rect(Rect2(line.x - 6 + col * 6, line.y - 38 + row * 9, 6, 9), INK if (row + col) % 2 == 0 else BG)
	_text(Vector2(line.x + 34, line.y - 49), "START / FINISH", 11, MUTED)
	_text(Vector2(82, 547), "%02d   %s" % [circuit_index + 1, circuit["name"]], 12, MUTED)
	_text(Vector2(710, 547), "3 LAPS   /   1 RIVAL", 12, MUTED)


func _car_direction(progress: float, lane: float, heading: float) -> Vector2:
	var epsilon := 0.002
	var tangent := _track_point(progress + epsilon, lane) - _track_point(progress - epsilon, lane)
	var normal := _track_point(progress, lane + 1.0) - _track_point(progress, lane)
	var motion := tangent * cos(heading) - normal * sin(heading) * RaceModel.LANE_PER_LAP * 2.0 * epsilon
	return motion.normalized() if motion.length_squared() > 0.000001 else tangent.normalized()


func _draw_car(progress: float, lane: float, heading: float, color: Color, boosting: bool) -> void:
	var pos := _track_point(progress, lane)
	var direction := _car_direction(progress, lane, heading)
	var side := direction.orthogonal()
	var body := PackedVector2Array()
	for point in [Vector2(12, -5), Vector2(12, 5), Vector2(-11, 7), Vector2(-11, -7)]:
		body.append(pos + direction * point.x + side * point.y)
	if boosting:
		draw_line(pos - direction * 10, pos - direction * 33, Color(0.78, 0.95, 0.42, 0.24), 10.0, true)
		draw_line(pos - direction * 10, pos - direction * 27, INK, 3.0, true)
	draw_circle(pos + Vector2(2, 4), 12, Color(0, 0, 0, 0.3))
	draw_colored_polygon(body, color)
	draw_line(pos - side * 4, pos + side * 4, BG, 6.0, true)
	draw_line(pos - direction * 8 - side * 8, pos - direction * 8 + side * 8, INK, 2.0, true)
	draw_circle(pos + direction * 10 - side * 4, 2, INK)
	draw_circle(pos + direction * 10 + side * 4, 2, INK)


func _draw_hud() -> void:
	_text(Vector2(28, 27), "GAMESTRUMENTS  /  PLAYABLE INTEGRATION", 11, MUTED)
	_text(Vector2(26, 65), circuit["name"], 30)
	_text(Vector2(404, 29), "LAP", 11, MUTED)
	_text(Vector2(401, 64), "%02d / 03" % race.lap(), 26)
	_text(Vector2(548, 29), "TIME", 11, MUTED)
	_text(Vector2(545, 64), "%05.1f" % race.elapsed, 26)
	_text(Vector2(683, 29), "POSITION", 11, MUTED)
	_text(Vector2(682, 64), "1 / 2" if race.progress >= race.rival_progress else "2 / 2", 26, accent)
	draw_line(Vector2(28, 83), Vector2(932, 83), Color("294044"))
	draw_line(Vector2(28, 560), Vector2(932, 560), Color("294044"))
	_text(Vector2(28, 583), "MUSIC  /  %s  /  %s" % [circuit["style_label"], music.requested_section.to_upper()], 13, accent)
	_text(Vector2(28, 604), "Changes land on the next musical bar", 11, MUTED)
	_text(Vector2(390, 583), "W / UP  THROTTLE     S / DOWN  BRAKE", 11)
	_text(Vector2(390, 604), "A D / ARROWS  STEER     SPACE  BOOST     R  RESTART", 11, MUTED)
	_text(Vector2(810, 582), "BOOST", 11, MUTED)
	draw_rect(Rect2(810, 594, 122, 7), Color("294044"))
	draw_rect(Rect2(810, 594, 122 * race.charge, 7), accent)
	if not music.error_message.is_empty():
		draw_rect(Rect2(20, 90, 920, 31), BG)
		_text(Vector2(28, 111), music.error_message, 12, RIVAL)


func _draw_island() -> void:
	if paused:
		_centered(292, "PAUSED", 36)
		_centered(325, "Take a breath. ESC to resume.", 15, MUTED)
	elif race.phase == "garage":
		_centered(255, "CIRCUIT  %02d / %02d" % [circuit_index + 1, CIRCUITS.size()], 13, MUTED)
		_centered(292, circuit["name"], 32)
		_centered(325, "MUSIC  /  %s" % circuit["style_label"], 15, accent)
		_centered(350, "A / D choose circuit   /   ENTER race   /   SPACE boost", 11, MUTED)
	elif race.phase == "grid":
		_centered(305, str(maxi(1, int(ceil(race.countdown)))), 70, accent)
		_centered(344, "HOLD W  /  LIGHTS OUT", 14, MUTED)
	elif race.phase == "finish":
		_centered(278, "YOU WIN." if race.won else "RIVAL WINS.", 36, accent if race.won else RIVAL)
		_centered(315, "3 laps  /  %.1f seconds  /  %d contacts" % [race.elapsed, race.impacts], 14, MUTED)
		_centered(341, "ENTER race again   /   A / D next circuit", 13, MUTED)
	else:
		var callout := "FINAL LAP" if race.lap() == 3 else "KEEP YOUR LINE"
		if race.off_road():
			callout = "BACK ON TRACK"
		elif race.collision_cooldown > 0.6:
			callout = "CONTACT"
		elif race.boosting:
			callout = "FULL SEND"
		elif race.pressure() >= 0.68:
			callout = "RIVAL CLOSE"
		_centered(294, callout, 29, RIVAL if race.off_road() else accent)
		_centered(327, "%03d" % int(race.speed / RaceModel.CRUISE_SPEED * 160), 36)
		_centered(350, "KM/H  /  GRIP-ASSISTED STEERING", 11, MUTED)


func _draw_intro() -> void:
	if not intro_visible:
		return
	draw_rect(Rect2(0, 0, 960, 620), Color(0, 0, 0, 0.62))
	draw_rect(Rect2(200, 190, 560, 250), BG)
	draw_rect(Rect2(200, 190, 560, 250), accent, false, 2.0)
	_centered(236, "PLAY TO HEAR THE MUSIC ADAPT", 24, accent)
	_centered(278, "Every circuit generates its soundtrack live in Godot.", 14, INK)
	_centered(302, "Drive, and the music follows the race.", 14, INK)
	_centered(334, "4 circuits   ·   4 styles   ·   6 adaptive sections", 12, MUTED)
	_centered(356, "No music buttons. No pre-baked tracks.", 12, MUTED)
