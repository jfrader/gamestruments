extends SceneTree

## Runtime smoke for the three kit examples. Verifies wiring end-to-end:
##   - positive path: each action runs in a fresh scene instance so a queued
##     crossfade from a previous action can never mask a later one; waits use
##     real time (Time.get_ticks_msec + process_frame) with a bounded deadline.
##   - negative path (--missing-extension): no player, actionable error text,
##     every control disabled except the web lab link, sliders non-editable.
##
## Signal-wiring checks emit `pressed` after asserting the control is present
## and enabled (this is NOT a claim of full human-input QA). A representative
## action per scene also drives a real mouse press through push_input with
## ensure_control_visible so pointer/scroll geometry is exercised.

var failures: Array[String] = []
var capture_dir := ""
var missing_extension := false
var verified_scenes := 0
var _added_bus_index := -1

const SECTION_TIMEOUT_MS := 12000
const HELD_TIMEOUT_MS := 3000


func _initialize() -> void:
	var args := OS.get_cmdline_user_args()
	missing_extension = "--missing-extension" in args or "--without-addon" in args
	if args.size() >= 2 and args[0] == "--screenshots":
		capture_dir = args[1]
	call_deferred("_run")


func _check(cond: bool, msg: String) -> void:
	if not cond:
		failures.append(msg)


func _run() -> void:
	_setup_music_bus()
	if missing_extension:
		await _negative_scene("01-playback/playback.tscn", "playback")
		await _negative_scene("02-game-signals/game_signals.tscn", "game_signals")
		await _negative_scene("03-song-form/song_form.tscn", "song_form")
	else:
		await _check_playback()
		await _check_game_signals()
		await _check_song_form()
		await _check_lifecycle()
	_teardown_music_bus()

	if not missing_extension:
		await _direct_player_bus_fallback_smoke()
		await _native_player_smoke()
		# Let the audio thread retire the final 100 ms generator buffer before exit.
		await create_timer(0.25).timeout

	if not failures.is_empty():
		for f in failures:
			push_error(f)
		quit(1)
		return
	if verified_scenes != 3:
		push_error("not all 3 scenes verified (got %d)" % verified_scenes)
		quit(1)
		return
	print("GAMESTRUMENTS_EXAMPLES_SMOKE_PASS")
	quit(0)


func _setup_music_bus() -> void:
	if AudioServer.get_bus_index("Music") < 0:
		AudioServer.add_bus()
		_added_bus_index = AudioServer.bus_count - 1
		AudioServer.set_bus_name(_added_bus_index, "Music")


func _teardown_music_bus() -> void:
	if _added_bus_index >= 0:
		AudioServer.remove_bus(_added_bus_index)
		_added_bus_index = -1


func _capture(name: String) -> void:
	if capture_dir.is_empty():
		return
	await create_timer(0.3).timeout
	await process_frame
	await RenderingServer.frame_post_draw
	_check(root.get_texture().get_image().save_png(capture_dir.path_join(name)) == OK, "screenshot " + name)
	var original_size := root.size
	var original_content_size := root.content_scale_size
	root.size = Vector2i(480, 640)
	root.content_scale_size = root.size
	for _frame in range(4):
		await process_frame
	for control in root.find_children("*", "Control", true, false):
		if control.is_visible_in_tree() and (control is BaseButton or control is Label):
			_check(control.get_global_rect().end.x <= 480.5, "narrow layout overflow: " + str(control.name))
	await RenderingServer.frame_post_draw
	_check(root.get_texture().get_image().save_png(capture_dir.path_join("narrow-" + name)) == OK, "narrow screenshot " + name)
	root.size = original_size
	root.content_scale_size = original_content_size
	for _frame in range(4):
		await process_frame


func _instantiate(scene_path: String, tag: String) -> Node:
	var sc := load(scene_path) as PackedScene
	_check(sc != null, "%s: scene load failed" % tag)
	if sc == null:
		return null
	var inst: Node = sc.instantiate()
	root.add_child(inst)
	for _f in range(10):
		await process_frame
	return inst


func _free(inst: Node) -> void:
	if inst == null:
		return
	inst.queue_free()
	for _f in range(10):
		await process_frame


func _get_button(inst: Node, path: String, tag: String) -> Button:
	var btn := inst.get_node_or_null(path) as Button
	_check(btn != null, "%s: required control missing: %s" % [tag, path])
	return btn


func _press(btn: Button, tag: String) -> void:
	if btn == null:
		return
	_check(not btn.disabled, "%s: %s is disabled" % [tag, btn.name])
	btn.pressed.emit()
	await process_frame


func _pointer_click(inst: Node, btn: Button) -> void:
	if btn == null:
		return
	var scroll := inst.get_node_or_null("ScrollContainer") as ScrollContainer
	if scroll != null:
		scroll.ensure_control_visible(btn)
	for _f in range(3):
		await process_frame
	var pos := btn.get_global_rect().get_center()
	var motion := InputEventMouseMotion.new()
	motion.position = pos
	root.push_input(motion, true)
	for pressed in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.position = pos
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = pressed
		root.push_input(ev, true)
		await process_frame


func _wait_section(p: Node, want: String, timeout_ms: int = SECTION_TIMEOUT_MS) -> bool:
	if p == null:
		return false
	var deadline := Time.get_ticks_msec() + timeout_ms
	while Time.get_ticks_msec() < deadline:
		await process_frame
		if String(p.call("get_current_section")) == want:
			return true
	return false


func _wait_held(p: Node, want: bool, timeout_ms: int = HELD_TIMEOUT_MS) -> bool:
	if p == null:
		return false
	var deadline := Time.get_ticks_msec() + timeout_ms
	while Time.get_ticks_msec() < deadline:
		await process_frame
		if bool(p.call("is_form_held")) == want:
			return true
	return false


func _load_positive(scene_path: String, tag: String, initial: String) -> Node:
	var inst := await _instantiate(scene_path, tag)
	if inst == null:
		return null
	var players := inst.find_children("*", "GamestrumentsPlayer", true, false)
	_check(players.size() == 1, "%s: exactly one player" % tag)
	var err_label := inst.find_child("ErrorLabel", true, false) as Label
	_check(err_label == null or not err_label.visible, "%s: no error label" % tag)
	if players.size() == 1:
		var streams := players[0].find_children("LiveStream", "AudioStreamPlayer", false, false)
		_check(streams.size() == 1 and streams[0].bus == "Music", "%s: uses Music bus" % tag)
		var sec := String(players[0].call("get_current_section"))
		_check(sec == initial, "%s: initial section %s (got %s)" % [tag, initial, sec])
	return inst


func _player(inst: Node) -> Node:
	var players := inst.find_children("*", "GamestrumentsPlayer", true, false)
	return players[0] if players.size() == 1 else null


func _check_playback() -> void:
	var inst := await _load_positive("01-playback/playback.tscn", "playback", "intro")
	if inst != null:
		var player := _player(inst)
		var restart := _get_button(inst, "ScrollContainer/Margin/Main/RestartButton", "playback")
		if restart != null and player != null:
			await _pointer_click(inst, restart)
			for _f in range(10):
				await process_frame
			_check(String(player.call("get_current_section")) == "intro", "playback: restart returns to intro")
			var status := inst.find_child("StatusLabel", true, false) as Label
			_check(status != null and "Restarted" in status.text, "playback: restart status shown")
		if not capture_dir.is_empty():
			await _capture("playback-play.png")
		verified_scenes += 1
		await _free(inst)


func _check_game_signals() -> void:
	# garage -> grid (real pointer click)
	var inst := await _load_positive("02-game-signals/game_signals.tscn", "game_signals", "garage")
	if inst != null:
		var player := _player(inst)
		var grid := _get_button(inst, "ScrollContainer/Margin/Main/PhaseButtons/GridButton", "game_signals")
		if grid != null:
			await _pointer_click(inst, grid)
			_check(await _wait_section(player, "grid"), "game_signals: garage -> grid")
		await _free(inst)

	# high intensity -> attack (sliders + race button, signal wiring)
	inst = await _load_positive("02-game-signals/game_signals.tscn", "game_signals", "garage")
	if inst != null:
		var player := _player(inst)
		var int_sl := inst.find_child("IntensitySlider", true, false) as HSlider
		var pres_sl := inst.find_child("PressureSlider", true, false) as HSlider
		_check(int_sl != null and pres_sl != null, "game_signals: intensity/pressure sliders present")
		if int_sl != null and pres_sl != null:
			int_sl.value = 0.85
			pres_sl.value = 0.75
			var race := _get_button(inst, "ScrollContainer/Margin/Main/PhaseButtons/RaceButton", "game_signals")
			if race != null:
				await _press(race, "game_signals")
				_check(await _wait_section(player, "attack"), "game_signals: high intensity -> attack")
		await _free(inst)

	# finish win -> victory, and Apply stays in the finish context (the Win/Loss
	# buttons must set current_phase so Apply can't jump back to the race phase).
	inst = await _load_positive("02-game-signals/game_signals.tscn", "game_signals", "garage")
	if inst != null:
		var player := _player(inst)
		var win := _get_button(inst, "ScrollContainer/Margin/Main/FinishButtons/FinishWinButton", "game_signals")
		if win != null:
			await _press(win, "game_signals")
			_check(await _wait_section(player, "victory"), "game_signals: finish win -> victory")
			var apply := _get_button(inst, "ScrollContainer/Margin/Main/ApplyButton", "game_signals")
			if apply != null:
				await _press(apply, "game_signals")
				await _wait_section(player, "victory")
				_check(String(player.call("get_current_section")) == "victory", "game_signals: apply after win stays victory")
		await _free(inst)

	# finish loss -> same closing victory section as win.
	inst = await _load_positive("02-game-signals/game_signals.tscn", "game_signals", "garage")
	if inst != null:
		var player := _player(inst)
		var loss := _get_button(inst, "ScrollContainer/Margin/Main/FinishButtons/FinishLossButton", "game_signals")
		if loss != null:
			await _press(loss, "game_signals")
			_check(await _wait_section(player, "victory"), "game_signals: finish loss -> victory")
		await _free(inst)

	# durable finish path: stale high race state (intensity/pressure/final lap)
	# must normalize to finish and select victory, and Apply must stay there.
	inst = await _load_positive("02-game-signals/game_signals.tscn", "game_signals", "garage")
	if inst != null:
		var player := _player(inst)
		var int_sl := inst.find_child("IntensitySlider", true, false) as HSlider
		var pres_sl := inst.find_child("PressureSlider", true, false) as HSlider
		var final_cb := inst.find_child("FinalLapCheck", true, false) as CheckBox
		_check(int_sl != null and pres_sl != null and final_cb != null, "game_signals: finish controls present")
		if int_sl != null and pres_sl != null and final_cb != null:
			int_sl.value = 0.9
			pres_sl.value = 0.9
			final_cb.button_pressed = true
			var finish := _get_button(inst, "ScrollContainer/Margin/Main/PhaseButtons/FinishButton", "game_signals")
			if finish != null:
				await _press(finish, "game_signals")
				_check(await _wait_section(player, "victory"), "game_signals: generic finish -> victory")
			_check(int_sl.value == 0.0, "game_signals: finish normalizes intensity")
			_check(pres_sl.value == 0.0, "game_signals: finish normalizes pressure")
			_check(not final_cb.button_pressed, "game_signals: finish clears final lap")
			_check(String(inst.get("current_phase")) == "finish", "game_signals: finish tracks current_phase")
			var apply := _get_button(inst, "ScrollContainer/Margin/Main/ApplyButton", "game_signals")
			if apply != null:
				await _press(apply, "game_signals")
				await _wait_section(player, "victory")
				_check(String(player.call("get_current_section")) == "victory", "game_signals: apply after finish stays victory")
		if not capture_dir.is_empty():
			await _capture("game_signals-race.png")
		verified_scenes += 1
		await _free(inst)


func _check_song_form() -> void:
	# hold + resume (instant, same fresh scene)
	var inst := await _load_positive("03-song-form/song_form.tscn", "song_form", "intro")
	if inst != null:
		var player := _player(inst)
		var hold := _get_button(inst, "ScrollContainer/Margin/Main/FormButtons/HoldButton", "song_form")
		var resume := _get_button(inst, "ScrollContainer/Margin/Main/FormButtons/ResumeButton", "song_form")
		if hold != null:
			await _press(hold, "song_form")
			_check(await _wait_held(player, true), "song_form: hold -> is_form_held true")
		if resume != null:
			await _press(resume, "song_form")
			_check(await _wait_held(player, false), "song_form: resume -> held false")
		await _free(inst)

	# cue chorus (signal wiring)
	inst = await _load_positive("03-song-form/song_form.tscn", "song_form", "intro")
	if inst != null:
		var player := _player(inst)
		var cue := _get_button(inst, "ScrollContainer/Margin/Main/FormButtons/CueChorusButton", "song_form")
		if cue != null:
			await _press(cue, "song_form")
			_check(await _wait_section(player, "chorus"), "song_form: cue chorus -> chorus")
		await _free(inst)

	# advance form (signal wiring)
	inst = await _load_positive("03-song-form/song_form.tscn", "song_form", "intro")
	if inst != null:
		var player := _player(inst)
		var adv := _get_button(inst, "ScrollContainer/Margin/Main/FormButtons/AdvanceButton", "song_form")
		if adv != null:
			await _press(adv, "song_form")
			_check(await _wait_section(player, "verse"), "song_form: advance -> verse")
		await _free(inst)

	# trace alert -> bridge (real pointer click)
	inst = await _load_positive("03-song-form/song_form.tscn", "song_form", "intro")
	if inst != null:
		var player := _player(inst)
		var alert := _get_button(inst, "ScrollContainer/Margin/Main/TraceButtons/AlertButton", "song_form")
		if alert != null:
			await _pointer_click(inst, alert)
			_check(await _wait_section(player, "bridge"), "song_form: trace alert -> bridge")
		if not capture_dir.is_empty():
			await _capture("song_form-form.png")
		verified_scenes += 1
		await _free(inst)


func _check_lifecycle() -> void:
	# Tight create/generate/free loop mixing Control CanvasItems with the native
	# player in the same queue_free. This deterministically catches the teardown
	# deadlock where the addon's audio child is freed during the parent's
	# exit_tree (which raced with RenderingServer teardown under a separate
	# render thread) and repeated-lifecycle leaks. Hangs/timeouts here are the
	# regression signal, not an assertion failure. Uses the Racing recipe so the
	# loop stays fast and within the harness timeout.
	for i in range(15):
		var c := Control.new()
		root.add_child(c)
		var v := VBoxContainer.new()
		c.add_child(v)
		for j in range(3):
			var l := Label.new()
			l.text = "lifecycle %d" % j
			v.add_child(l)
		var p: Node = ClassDB.instantiate("GamestrumentsPlayer")
		c.add_child(p)
		p.set("project_secret", "lifecycle-smoke")
		p.set("recipe", "racing")
		p.set("style", "neon")
		_check(bool(p.call("generate", "lifecycle-%d" % i)), "lifecycle: generate cycle %d" % i)
		for _f in range(4):
			await process_frame
		c.queue_free()
		for _f in range(6):
			await process_frame


func _negative_scene(scene_path: String, tag: String) -> void:
	var inst := await _instantiate(scene_path, tag)
	if inst == null:
		return
	var players := inst.find_children("*", "GamestrumentsPlayer", true, false)
	_check(players.size() == 0, "%s: no player when addon missing" % tag)
	var err_label := inst.find_child("ErrorLabel", true, false) as Label
	_check(err_label != null and err_label.visible, "%s: actionable error visible" % tag)
	if err_label != null:
		_check("copy addons" in err_label.text.to_lower(), "%s: error mentions copying addons" % tag)
	var web_link := inst.find_child("WebLabLink", true, false) as LinkButton
	_check(web_link != null and not web_link.disabled, "%s: web lab link stays available" % tag)
	if web_link != null:
		_check(not web_link.pressed.get_connections().is_empty(), "%s: web lab link remains wired" % tag)
	var any_enabled := false
	for n in inst.find_children("*", "", true, false):
		if n.name == "WebLabLink":
			continue
		if n is BaseButton and not (n as BaseButton).disabled:
			any_enabled = true
		elif n is Slider and (n as Slider).editable:
			any_enabled = true
	_check(not any_enabled, "%s: controls disabled (recursive, except link)" % tag)
	if not capture_dir.is_empty():
		await _capture(tag + "-neg.png")
	verified_scenes += 1
	await _free(inst)


func _direct_player_bus_fallback_smoke() -> void:
	for cycle in range(2):
		var p: Node = ClassDB.instantiate("GamestrumentsPlayer")
		_check(p != null, "direct instantiate")
		if p == null:
			continue
		root.add_child(p)
		for _f in range(5):
			await process_frame
		var lives := p.find_children("LiveStream", "AudioStreamPlayer", false, false)
		_check(lives.size() == 1, "has LiveStream")
		if lives.size() == 1:
			_check((lives[0] as AudioStreamPlayer).bus == "Master", "direct falls back to Master after bus removed")
		p.set("project_secret", "examples-direct-smoke")
		p.set("recipe", "suspense")
		p.set("style", "terminal")
		p.set("arrangement", "seeded")
		var gok: bool = p.call("generate", "direct-seeded-%d" % cycle)
		_check(gok, "direct seeded generate")
		for _f in range(15):
			await process_frame
		var sec := String(p.call("get_current_section"))
		_check(sec.length() > 0, "direct seeded section")
		p.queue_free()
		for _f in range(8):
			await process_frame


# --- Native-player positive smoke (Node-only, no scene UI) -----------------
# Exercises the native extension directly with fresh players so a queued
# crossfade from a previous action can never mask a later one. The Suspense
# progress-parity checks run against the freshly built .so: the retired Extended
# preset's `progress >= 0.8 -> outro` rule is gone, so neither pool arrangement
# may land on outro for progress alone. A stale .so that still hardcodes that
# cue fails the "no outro" assertions below.

const NO_OUTRO_WAIT_SEC := 8.0


func _native_instantiate(tag: String, recipe: String, style: String, arrangement: String, autoplay: bool) -> Node:
	var p: Node = ClassDB.instantiate("GamestrumentsPlayer")
	_check(p != null, "%s: native player instantiate" % tag)
	if p == null:
		return null
	root.add_child(p)
	p.set("project_secret", "examples-native-smoke")
	p.set("recipe", recipe)
	p.set("style", style)
	p.set("arrangement", arrangement)
	p.set("autoplay", autoplay)
	for _f in range(5):
		await process_frame
	return p


func _native_free(p: Node) -> void:
	if p == null:
		return
	p.queue_free()
	for _f in range(8):
		await process_frame


func _native_player_smoke() -> void:
	await _native_racing_original()
	await _native_racing_extended()
	await _native_adventure_folk()
	await _native_suspense_progress_parity()


func _native_racing_original() -> void:
	var p := await _native_instantiate("native-racing-original", "racing", "neon", "original", true)
	if p == null:
		return
	_check(bool(p.call("generate", "native-original")), "native-racing-original: generate")
	_check(String(p.call("get_current_section")) == "garage", "native-racing-original: initial garage")
	_check(bool(p.call("set_form_hold", true)), "native-racing-original: set_form_hold(true)")
	_check(bool(p.call("is_form_held")), "native-racing-original: is_form_held true")
	_check(bool(p.call("advance_form")), "native-racing-original: advance_form")
	_check(await _wait_section(p, "grid"), "native-racing-original: advance -> grid")
	await _native_free(p)


func _native_racing_extended() -> void:
	var p := await _native_instantiate("native-racing-extended", "racing", "neon", "extended", true)
	if p == null:
		return
	_check(bool(p.call("generate", "native-extended")), "native-racing-extended: generate")
	_check(String(p.call("get_current_section")) == "garage", "native-racing-extended: initial garage")
	_check(bool(p.call("advance_form")), "native-racing-extended: advance_form")
	_check(await _wait_section(p, "ignition"), "native-racing-extended: advance -> ignition")
	_check(bool(p.call("cue_section", "slipstream")), "native-racing-extended: cue slipstream")
	_check(await _wait_section(p, "slipstream"), "native-racing-extended: cue -> slipstream")
	await _native_free(p)


func _native_adventure_folk() -> void:
	var p := await _native_instantiate("native-adventure-folk", "adventure", "folk", "original", true)
	if p == null:
		return
	_check(bool(p.call("generate", "native-adventure")), "native-adventure-folk: generate")
	_check(String(p.call("get_current_section")) == "camp", "native-adventure-folk: initial camp")
	_check(bool(p.call("advance_form")), "native-adventure-folk: advance_form")
	_check(await _wait_section(p, "explore"), "native-adventure-folk: advance -> explore")
	_check(bool(p.call("set_adventure_state", "explore", 0.4, 0.1, true)), "native-adventure-folk: set_adventure_state victory")
	_check(await _wait_section(p, "victory"), "native-adventure-folk: quest_complete -> victory")
	await _native_free(p)


func _native_suspense_progress_parity() -> void:
	for arrangement in ["all-phases", "seeded"]:
		await _native_suspense_case(arrangement)


func _native_suspense_case(arrangement: String) -> void:
	var tag := "native-suspense-" + arrangement
	var p := await _native_instantiate(tag, "suspense", "terminal", arrangement, false)
	if p == null:
		return
	_check(bool(p.call("generate", "native-suspense-" + arrangement)), "%s: generate" % tag)
	_check(bool(p.call("set_trace_state", "scan", 0.1, 0.2, 0.85)), "%s: set_trace_state" % tag)
	# Give the bar boundary + crossfade enough real time to commit a stray outro
	# cue before asserting it never landed; the form auto-advances, so we only
	# require that outro was not reached.
	await create_timer(NO_OUTRO_WAIT_SEC).timeout
	var sec := String(p.call("get_current_section"))
	_check(sec.length() > 0, "%s: section reporting after %.0fs" % [tag, NO_OUTRO_WAIT_SEC])
	_check(sec != "outro", "%s: no outro at progress 0.85 (got %s)" % [tag, sec])
	await _native_free(p)
