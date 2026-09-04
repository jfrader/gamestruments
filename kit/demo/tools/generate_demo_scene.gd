extends SceneTree

## Checked-in scene generator.
## Run with:
##   godot --path <worktree> --headless --script res://kit/demo/tools/generate_demo_scene.gd
## This produces kit/demo/kit_demo.tscn with a root Control that has the demo script attached.
## UI construction and GamestrumentsPlayer wiring live in the .gd (never hand-edit the .tscn).
##
## When the demo is opened as its own project (kit/demo/ as the Godot project root),
## res:// resolves to the demo folder contents, so the script must be referenced as
## res://kit_demo.gd (not res://kit/demo/kit_demo.gd). We derive the demo-relative
## path from the tool's own location and rewrite the script resource_path before
## packing so the saved .tscn records the correct path for the demo-as-project case.
## The load() and save_path still use repo layout because the generator runs from
## repo root.

func _init() -> void:
	print("[generate_demo_scene] Building kit demo scene...")

	var root := Control.new()
	root.name = "KitDemo"
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	# layout_mode 3 + anchors_preset 0 is what the hand example uses; script will override in _ready anyway.

	# Derive paths from the tool script location (res://kit/demo/tools/...) so the
	# generator stays runnable from repo root while targeting the demo-project view.
	var tool_path: String = get_script().resource_path
	var demo_repo_dir: String = tool_path.get_base_dir().get_base_dir()  # "res://kit/demo"
	var repo_script_path: String = demo_repo_dir.path_join("kit_demo.gd")  # load from repo layout
	var demo_project_script_path: String = "res://kit_demo.gd"  # what the .tscn must record when demo/ is res://

	var script_res := load(repo_script_path)
	if script_res == null:
		push_error("Failed to load %s — write it first." % repo_script_path)
		quit(1)
		return

	root.set_script(script_res)

	# Rewrite the resource path on the script so PackedScene + ResourceSaver emit
	# the demo-project path in the ext_resource line. (This is the supported way
	# for generators that must produce scenes consumable under a different res root.)
	script_res.resource_path = demo_project_script_path

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
