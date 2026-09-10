extends Node

var level_seed := "night-circuit-001"
var style := "neon"

var player: Node
var error_message := ""
var requested_section := "garage"
var last_state: Dictionary = {}


func _ready() -> void:
	if not ClassDB.class_exists("GamestrumentsPlayer"):
		error_message = "Music extension missing. Open this project in Godot 4.7.x and let the initial import finish."
		return
	player = ClassDB.instantiate("GamestrumentsPlayer")
	player.name = "GamestrumentsPlayer"
	add_child(player)
	player.set("project_secret", "gamestruments-night-circuit")


# Regenerates for a different circuit style/seed. Safe to call from the garage:
# the player resets transport and starts at the garage section.
func configure(new_style: String, new_seed: String) -> bool:
	style = new_style
	level_seed = new_seed
	requested_section = "garage"
	last_state = {}
	if player == null:
		error_message = "Music extension missing. Open this project in Godot 4.7.x and let the initial import finish."
		return false
	error_message = ""
	player.set("style", style)
	if not bool(player.call("generate", level_seed)):
		error_message = "Music generation failed. See Godot Output; the race can still be played."
		return false
	return true


# This is the integration seam: game telemetry in, public library calls out.
# The readout is the requested section, not a claim about the current audio bar.
func sync_race(state: Dictionary) -> void:
	if player == null or not error_message.is_empty() or state == last_state:
		return
	if not bool(player.call("set_race_state", state.phase, state.intensity, state.pressure, state.final_lap, state.result)):
		error_message = "Music state rejected. See Godot Output."
		return
	last_state = state.duplicate()
	if state.phase == "finish":
		requested_section = "victory"
	elif state.final_lap:
		requested_section = "final-lap"
	elif state.pressure >= 0.68 or state.intensity >= 0.72:
		requested_section = "attack"
	else:
		requested_section = "cruise" if state.phase == "race" else state.phase
