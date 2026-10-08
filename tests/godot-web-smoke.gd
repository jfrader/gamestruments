extends Node

## Web runtime smoke, run inside the exported page by tests/godot-web-smoke.mjs.
## Proves the side module loaded (GamestrumentsPlayer is registered), that
## generate() and set_adventure_state() work, that the live stream is audible
## on the Music bus, and that a bar-quantized state change commits within
## STATE_TIMEOUT_MS. Prints one
## GAMESTRUMENTS_WEB_SMOKE line with the measurements, then PASS or FAIL.

const SEED := "web-smoke"
const STATE_TIMEOUT_MS := 3000
const AUDIO_TIMEOUT_MS := 20000

var _failures: Array[String] = []


func _check(cond: bool, msg: String) -> void:
	if not cond:
		_failures.append(msg)


func _ready() -> void:
	_run.call_deferred()


func _run() -> void:
	var report := {
		"threads": OS.has_feature("threads"),
		"mix_rate": AudioServer.get_mix_rate(),
	}
	var exists := ClassDB.class_exists("GamestrumentsPlayer")
	report["class_exists"] = exists
	_check(exists, "GamestrumentsPlayer is not registered (web side module did not load)")
	if not exists:
		_finish(report)
		return
	if AudioServer.get_bus_index("Music") < 0:
		AudioServer.add_bus()
		AudioServer.set_bus_name(AudioServer.bus_count - 1, "Music")
	var music := AudioServer.get_bus_index("Music")
	var player: Node = ClassDB.instantiate("GamestrumentsPlayer")
	player.set("project_secret", "gamestruments-web-smoke")
	player.set("recipe", "adventure")
	player.set("style", "folk")
	add_child(player)
	var stream := player.get_node_or_null("LiveStream") as AudioStreamPlayer
	_check(stream != null and stream.bus == "Music", "live stream is not routed to the Music bus")
	var t0 := Time.get_ticks_usec()
	var generated: bool = player.generate(SEED, "explore")
	report["generate_ms"] = (Time.get_ticks_usec() - t0) / 1000.0
	_check(generated, "generate() failed")
	_check(player.get_current_section() == "explore", "generate() did not open on explore")

	var audible_at := -1
	var started := Time.get_ticks_msec()
	while Time.get_ticks_msec() - started < AUDIO_TIMEOUT_MS:
		await get_tree().process_frame
		if AudioServer.get_bus_peak_volume_left_db(music, 0) > -60.0:
			audible_at = Time.get_ticks_msec() - started
			break
	report["audible_after_ms"] = audible_at
	_check(audible_at >= 0, "Music bus stayed silent")

	# Let the opening settle, as a game would before its first state change.
	await get_tree().create_timer(2.0).timeout
	var requested := Time.get_ticks_msec()
	var accepted: bool = player.set_adventure_state("combat", 0.2, 0.9, false)
	_check(accepted, "set_adventure_state() was rejected")
	var reached := ""
	while Time.get_ticks_msec() - requested < STATE_TIMEOUT_MS:
		await get_tree().process_frame
		var section: String = player.get_current_section()
		if section != "explore":
			reached = section
			break
	report["state_section"] = reached
	report["state_latency_ms"] = Time.get_ticks_msec() - requested
	_check(reached == "boss", "state change did not reach boss (got '%s')" % reached)
	_check(player.cue_section("sanctuary"), "cue_section() was rejected")
	player.queue_free()
	_finish(report)


func _finish(report: Dictionary) -> void:
	report["failures"] = _failures
	print("GAMESTRUMENTS_WEB_SMOKE ", JSON.stringify(report))
	print("GAMESTRUMENTS_WEB_SMOKE_PASS" if _failures.is_empty() else "GAMESTRUMENTS_WEB_SMOKE_FAIL")
