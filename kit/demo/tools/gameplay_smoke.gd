extends SceneTree

const RaceModel = preload("res://race_model.gd")
var failures: Array[String] = []
var capture_dir := ""
var without_music := false


func _initialize() -> void:
	var args := OS.get_cmdline_user_args()
	without_music = "--without-music" in args
	if args.size() == 2 and args[0] == "--screenshots":
		capture_dir = args[1]
	call_deferred("_run")


func _check(condition: bool, message: String) -> void:
	if not condition:
		failures.append(message)


func _key(code: Key, pressed: bool) -> void:
	var event := InputEventKey.new()
	event.keycode = code
	event.physical_keycode = code
	event.pressed = pressed
	Input.parse_input_event(event)


func _capture(filename: String) -> void:
	if capture_dir.is_empty():
		return
	await process_frame
	await RenderingServer.frame_post_draw
	_check(root.get_texture().get_image().save_png(capture_dir.path_join(filename)) == OK, "Screenshot failed: " + filename)


func _click(button: Button) -> void:
	var position := button.get_global_rect().get_center()
	var motion := InputEventMouseMotion.new()
	motion.position = position
	root.push_input(motion, true)
	for pressed in [true, false]:
		var event := InputEventMouseButton.new()
		event.position = position
		event.button_index = MOUSE_BUTTON_LEFT
		event.pressed = pressed
		root.push_input(event, true)
		await process_frame


func _run() -> void:
	_check_model()
	var demo := (load("res://kit_demo.tscn") as PackedScene).instantiate()
	root.add_child(demo)
	_check(root.focus_exited.is_connected(demo._on_focus_exited), "Focus-loss pause must be connected")
	# OS focus can change while a CI screenshot window is being resized.
	root.focus_exited.disconnect(demo._on_focus_exited)
	demo.set_physics_process(false)
	demo._physics_process(0.0)
	await process_frame
	if without_music:
		_check(not demo.music.error_message.is_empty(), "Missing extension must produce a visible warning")
		_check(demo.music.player == null, "Missing extension must not create a player")
	else:
		_check(demo.music.error_message.is_empty(), "Demo music failed to initialize")
		_check(demo.find_children("*", "GamestrumentsPlayer", true, false).size() == 1, "Demo must own exactly one music player")
	_check(demo.find_children("*", "LineEdit", true, false).is_empty(), "Gameplay must not be a generation form")
	_check(demo.CIRCUITS.size() == 4, "Demo must expose four circuits")
	var circuit_styles := {}
	for entry in demo.CIRCUITS:
		circuit_styles[entry["style"]] = true
	_check(circuit_styles.keys().size() == 4, "Four circuits must cover four music styles")
	for index in range(demo.CIRCUITS.size()):
		demo._set_circuit(index)
		demo._physics_process(0.0)
		await process_frame
		if without_music:
			_check(not demo.music.error_message.is_empty(), "Missing extension must warn on every circuit")
		else:
			_check(demo.music.error_message.is_empty(), "Circuit music failed: " + str(demo.music.error_message))
			_check(demo.music.style == demo.CIRCUITS[index]["style"], "Circuit must configure its own style")
		await _capture("01-garage-%d-%s.png" % [index + 1, demo.CIRCUITS[index]["style"]])
	demo._set_circuit(0)
	demo._physics_process(0.0)
	await process_frame
	await _click(demo.start_button)
	_check(demo.race.phase == "grid", "Start action must start the countdown")
	if not without_music:
		_check(demo.music.last_state.phase == "grid", "Start action must request grid music")
	await _capture("02-grid.png")
	_key(KEY_W, true)
	await process_frame
	for _tick in range(200):
		demo._physics_process(1.0 / 60.0)
	_check(demo.race.phase == "race" and demo.race.speed > 0, "Throttle must move the car after countdown")
	demo.toggle_pause()
	var paused_progress: float = demo.race.progress
	for _tick in range(100):
		demo._physics_process(1.0 / 60.0)
	_check(demo.race.progress == paused_progress, "Pause must freeze the race")
	_check(demo.music.process_mode == Node.PROCESS_MODE_DISABLED, "Pause must suspend music processing")
	await _capture("03-paused.png")
	demo.toggle_pause()
	_check(demo.music.process_mode == Node.PROCESS_MODE_INHERIT, "Resume must restore music processing")
	_key(KEY_D, true)
	await process_frame
	for _tick in range(18):
		demo._physics_process(1.0 / 60.0)
	_key(KEY_D, false)
	_key(KEY_SPACE, true)
	await process_frame
	var sections: Array[String] = []
	var captured_race := false
	var captured_final := false
	for tick in range(6000):
		# Releasing boost lets it recharge; steering stays inside the road.
		if tick % 240 == 100:
			_key(KEY_SPACE, false)
		elif tick % 240 == 0:
			_key(KEY_SPACE, true)
		demo._physics_process(1.0 / 60.0)
		var section: String = demo.music.requested_section
		if section not in sections:
			sections.append(section)
		if not captured_race and demo.race.progress > 0.4:
			await _capture("04-racing.png")
			captured_race = true
		if not captured_final and demo.race.lap() == 3:
			await _capture("05-final-lap.png")
			captured_final = true
		if tick % 120 == 0:
			await process_frame
		if demo.race.phase == "finish":
			break
	demo._physics_process(0.21)
	_check(demo.race.phase == "finish", "Driving must complete three laps")
	if not without_music:
		_check("attack" in sections and "cruise" in sections and "final-lap" in sections, "Gameplay must request cruise, attack, and final-lap music")
		_check(demo.music.last_state.phase == "finish", "Finish must be sent to music")
		_check(demo.music.last_state.result == ("win" if demo.race.won else "loss"), "Music result must match the race result")
		_check(demo.music.error_message.is_empty(), "Music rejected gameplay telemetry")
	await _capture("06-finish.png")
	_key(KEY_W, false)
	_key(KEY_SPACE, false)
	_key(KEY_R, true)
	await process_frame
	_key(KEY_R, false)
	_check(demo.race.phase == "grid" and demo.race.progress == 0.0, "R must restart the race")
	if not capture_dir.is_empty():
		root.size = Vector2i(1280, 800)
		await process_frame
		demo._physics_process(0.0)
		await _capture("07-resized.png")
		_key(KEY_W, true)
		await process_frame
		demo.set_physics_process(true)
		await create_timer(7.0).timeout
		_check(demo.race.progress > 0.0 and demo.race.elapsed > 2.0, "Real-time physics must advance gameplay")
		print("GAMEPLAY_RENDER_METRICS fps=%d draw_calls=%d" % [Performance.get_monitor(Performance.TIME_FPS), Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)])
		demo.toggle_pause()
		await process_frame
		var stream := demo.find_children("*", "AudioStreamPlayer", true, false)[0] as AudioStreamPlayer
		_check(stream.stream_paused, "Real-time pause must pause audio playback")
		demo.toggle_pause()
		await process_frame
		_check(not stream.stream_paused, "Real-time resume must restore audio playback")
		demo._on_focus_exited()
		_check(demo.paused, "Focus loss must pause a live race")
		_key(KEY_W, false)
	demo.queue_free()
	for _frame in range(30):
		await process_frame
	if not failures.is_empty():
		for failure in failures:
			push_error(failure)
		quit(1)
		return
	print("GAMESTRUMENTS_GAMEPLAY_SMOKE_PASS")
	quit(0)


func _check_model() -> void:
	var race = RaceModel.new()
	race.step(5.0, true, 1.0, true, false)
	_check(race.progress == 0.0, "Garage must not advance the race")
	race.start()
	race.step(1.0, true, 1.0, true, false)
	_check(race.progress == 0.0 and race.lane == 0.0, "Grid must lock cars")
	race.step(2.0, true, 0.0, false, false)
	_check(race.phase == "race", "Countdown must begin racing")
	for _tick in range(300):
		race.step(1.0 / 60.0, false, 0.0, false, false)
	_check(race.progress == 0.0 and race.rival_progress > 0.0, "Player must need throttle while rival races")
	race.lane = 24.0
	for _tick in range(120):
		race.step(1.0 / 60.0, true, 0.0, false, false)
	var cruise_speed: float = race.speed
	for _tick in range(60):
		race.step(1.0 / 60.0, true, 0.0, true, false)
	_check(race.speed > cruise_speed and race.charge < 1.0, "Boost must spend charge for speed")
	for _tick in range(300):
		race.step(1.0 / 60.0, true, 0.0, true, false)
	_check(not race.boosting and race.boost_exhausted, "Holding exhausted boost must not pulse on recharge")
	race.step(0.1, true, 0.0, false, false)
	_check(not race.boost_exhausted, "Releasing boost must rearm it")
	for _tick in range(30):
		race.step(1.0 / 60.0, true, 1.0, false, false)
	_check(race.heading > 0.2, "Steering must turn the car instead of strafing it")
	var left_road := race.off_road()
	for _tick in range(120):
		race.step(1.0 / 60.0, true, 1.0, false, false)
		left_road = left_road or race.off_road()
	_check(left_road and race.speed < cruise_speed, "Leaving the road must slow the car")
	_check(absf(race.lane) <= RaceModel.WALL_EDGE, "Steering must not escape the circuit")
	for _tick in range(90):
		race.step(1.0 / 60.0, true, 0.0, false, false)
	_check(absf(race.heading) < 0.05, "Releasing the steering must straighten the car")
	for _tick in range(120):
		race.step(1.0 / 60.0, true, 0.0, false, true)
	_check(race.speed == 0.0, "Brake must override throttle")
	race.start()
	race.step(3.0, false, 0.0, false, false)
	race.rival_progress = race.progress
	race.step(1.0 / 60.0, true, 0.0, false, false)
	_check(race.impacts == 1, "Overlapping rival must cause contact")
	race.progress = 2.0
	_check(race.lap() == 3 and race.music_state().final_lap, "Final lap boundary must drive music")
	for should_win in [true, false]:
		race.phase = "race"
		race.progress = 2.9999
		race.rival_progress = 2.9 if should_win else 3.0
		race.speed = RaceModel.CRUISE_SPEED
		race.step(0.1, true, 0.0, false, false)
		_check(race.phase == "finish" and race.won == should_win, "Finish order must determine win/loss")
		var finish_time: float = race.elapsed
		race.step(1.0, true, 1.0, true, false)
		_check(race.elapsed == finish_time and race.progress == 3.0, "Finish must freeze race results")
	race.start()
	_check(race.elapsed == 0 and race.impacts == 0 and race.charge == 1.0, "Restart must clear race state")
