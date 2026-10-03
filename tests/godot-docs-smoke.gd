extends SceneTree

## Runtime smoke for the shipped documentation snippets (kit/README.md,
## kit/docs/quickstart.md and kit/docs/limitations.md). The harness extracts
## each fenced GDScript block into res://snippets/*.gd in a fresh, minimal
## project and then runs this script.
## Each snippet is attached to a bare Node with a real GamestrumentsPlayer
## child, its documented _ready is verified to have generated a score, and every
## documented callback is invoked by name and checked against the real section
## it produces after bounded bar-boundary waits.
##
## The fresh project has no "Music" audio bus, so this also proves the player
## falls back to Master. Each snippet is freed after its probes, and the run
## quits plainly with music still playing, so a leak warning at exit fails the smoke.

var failures: Array[String] = []

const SECTION_TIMEOUT_MS := 180000
const SETTLE_FRAMES := 10


func _initialize() -> void:
	call_deferred("_run")


func _check(cond: bool, msg: String) -> void:
	if not cond:
		failures.append(msg)


func _run() -> void:
	# README ships exactly one standalone Racing _ready with no callbacks.
	await _run_snippet("res://snippets/readme_racing.gd", "garage", [])
	# quickstart.md ships a full standalone Racing script with three callbacks.
	await _run_snippet("res://snippets/quickstart_racing.gd", "garage", [
		{"method": "countdown_started", "args": [], "section": "grid"},
		{"method": "race_updated", "args": [0.9, 0.2, 1, 3], "section": "attack"},
		{"method": "race_finished", "args": [true], "section": "victory"},
		{"method": "race_finished", "args": [false], "section": "victory"},
	])
	# quickstart.md ships a full standalone Suspense script with six callbacks.
	await _run_snippet("res://snippets/quickstart_suspense.gd", "intro", [
		{"method": "set_hold", "args": [true], "held": true},
		{"method": "scan_started", "args": [], "section": "intro"},
		{"method": "advance", "args": [], "section": "verse"},
		{"method": "alarm_raised", "args": [], "section": "bridge"},
		{"method": "cue", "args": ["chorus"], "section": "chorus"},
		{"method": "chapter_finished", "args": [], "section": "coda"},
		{"method": "set_hold", "args": [false], "held": false},
	])
	if not failures.is_empty():
		for f in failures:
			push_error(f)
		quit(1)
		return
	print("DOCS_SMOKE_PASS")
	await _quit_with_music_playing()


## The addon must release its playback while the scene tree is torn down, so the
## run quits plainly with music still playing; a leak warning at exit fails the smoke.
func _quit_with_music_playing() -> void:
	var holder := Node.new()
	var player: Node = ClassDB.instantiate("GamestrumentsPlayer")
	player.name = "GamestrumentsPlayer"
	player.set("project_secret", "docs-smoke")
	holder.add_child(player)
	root.add_child(holder)
	player.call("generate", "quit")
	for _f in range(SETTLE_FRAMES):
		await process_frame
	quit(0)


func _run_snippet(path: String, initial: String, probes: Array) -> void:
	var script_res := load(path)
	_check(script_res != null, "snippet failed to load: " + path)
	if script_res == null:
		return
	var holder := Node.new()
	var player: Node = ClassDB.instantiate("GamestrumentsPlayer")
	player.name = "GamestrumentsPlayer"
	holder.add_child(player)
	holder.set_script(script_res)
	root.add_child(holder)
	for _f in range(SETTLE_FRAMES):
		await process_frame
	_check(bool(holder.get("music_ready")), "snippet generated a score: " + path)
	var sec := String(player.call("get_current_section"))
	_check(sec == initial, "%s: initial section %s (got %s)" % [path, initial, sec])
	var lives := player.find_children("LiveStream", "AudioStreamPlayer", false, false)
	_check(lives.size() == 1 and (lives[0] as AudioStreamPlayer).bus == "Master",
		"%s: falls back to Master bus (no Music bus)" % path)
	for probe in probes:
		await _run_probe(holder, player, probe)
	holder.queue_free()
	for _f in range(SETTLE_FRAMES):
		await process_frame


func _run_probe(holder: Node, player: Node, probe: Dictionary) -> void:
	var method := String(probe.get("method"))
	var args: Array = probe.get("args", [])
	var t0 := Time.get_ticks_msec()
	holder.callv(method, args)
	if probe.has("held"):
		for _f in range(4):
			await process_frame
		_check(bool(player.call("is_form_held")) == bool(probe["held"]), "probe %s held" % method)
	if probe.has("section"):
		var want := String(probe["section"])
		var got := await _wait_section(player, want)
		print("[DIAG] probe %s -> %s : %s (%d ms, sec=%s)" % [method, want, "OK" if got else "FAIL", Time.get_ticks_msec() - t0, String(player.call("get_current_section"))])
		_check(got, "probe %s -> %s" % [method, want])


func _wait_section(player: Node, want: String) -> bool:
	var deadline := Time.get_ticks_msec() + SECTION_TIMEOUT_MS
	while Time.get_ticks_msec() < deadline:
		await process_frame
		if String(player.call("get_current_section")) == want:
			return true
	return false
