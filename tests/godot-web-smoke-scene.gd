extends SceneTree

## Packs res://web_smoke.tscn (a Node running web_smoke.gd) for the web export.


func _initialize() -> void:
	var root := Node.new()
	root.name = "WebSmoke"
	root.set_script(load("res://web_smoke.gd"))
	var scene := PackedScene.new()
	var packed := scene.pack(root)
	var saved := ResourceSaver.save(scene, "res://web_smoke.tscn") if packed == OK else packed
	root.free()
	if saved != OK:
		printerr("could not save web_smoke.tscn: %s" % error_string(saved))
		quit(1)
		return
	print("WEB_SMOKE_SCENE_SAVED")
	quit(0)
