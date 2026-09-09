extends SceneTree

func _initialize() -> void:
	call_deferred("_run")


func _fail(message: String) -> void:
	push_error(message)
	quit(1)


func _run() -> void:
	var demo_scene := load("res://kit_demo.tscn") as PackedScene
	if demo_scene == null:
		_fail("kit_demo.tscn could not be loaded")
		return

	if not ClassDB.class_exists("GamestrumentsPlayer"):
		_fail("GamestrumentsPlayer is unavailable")
		return
	var music_bus := AudioServer.get_bus_index("Music")
	var added_music_bus := false
	if music_bus < 0:
		AudioServer.add_bus()
		music_bus = AudioServer.bus_count - 1
		AudioServer.set_bus_name(music_bus, "Music")
		added_music_bus = true

	var demo := demo_scene.instantiate()
	root.add_child(demo)
	for _frame in range(30):
		await process_frame
	var demo_players := demo.find_children("*", "GamestrumentsPlayer", true, false)
	if demo_players.size() != 1:
		_fail("kit demo did not create exactly one GamestrumentsPlayer")
		return
	if not bool(demo_players[0].call("set_race_state", "race", 0.7, 0.3, false, "none")):
		_fail("kit demo did not auto-generate a playable score")
		return
	demo.queue_free()
	for _frame in range(30):
		await process_frame

	for cycle in range(2):
		var player: Node = ClassDB.instantiate("GamestrumentsPlayer")
		if player == null:
			_fail("GamestrumentsPlayer could not be instantiated")
			return
		root.add_child(player)
		player.set("project_secret", "release-smoke")
		player.set("style", "neon" if cycle == 0 else "chip")
		player.set("energy", 0.9)
		player.set("complexity", 0.9)
		player.set("brightness", 0.9)
		player.set("syncopation", 0.1)
		await process_frame
		if not bool(player.call("generate", "runtime-smoke-%d" % cycle)):
			_fail("GamestrumentsPlayer generation failed")
			return
		if not bool(player.call("set_race_state", "attack", 0.95, 0.8, false, "none")):
			_fail("GamestrumentsPlayer state transition failed")
			return
		for _frame in range(30):
			await process_frame
		player.queue_free()
		for _frame in range(30):
			await process_frame

	if added_music_bus:
		AudioServer.remove_bus(music_bus)
	print("GAMESTRUMENTS_RUNTIME_SMOKE_PASS")
	quit(0)
