extends SceneTree

## Checked-in scene generator.
## Run with:
##   godot --path <worktree> --headless --script res://kit/demo/tools/generate_demo_scene.gd
## This produces kit/demo/kit_demo.tscn with a root Control that has the demo script attached.
## UI construction and GamestrumentsPlayer wiring live in the .gd (never hand-edit the .tscn).

func _init() -> void:
	print("[generate_demo_scene] Building kit demo scene...")

	var root := Control.new()
	root.name = "KitDemo"
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	# layout_mode 3 + anchors_preset 0 is what the hand example uses; script will override in _ready anyway.

	var script_res := load("res://kit/demo/kit_demo.gd")
	if script_res == null:
		push_error("Failed to load res://kit/demo/kit_demo.gd — write it first.")
		quit(1)
		return

	root.set_script(script_res)

	var packed := PackedScene.new()
	var err := packed.pack(root)
	if err != OK:
		push_error("Failed to pack scene: %s" % err)
		quit(1)
		return

	var save_path := "res://kit/demo/kit_demo.tscn"
	var save_err := ResourceSaver.save(packed, save_path)
	if save_err != OK:
		push_error("Failed to save %s: %s" % [save_path, save_err])
		quit(1)
		return

	print("[generate_demo_scene] Saved %s" % save_path)
	quit(0)
