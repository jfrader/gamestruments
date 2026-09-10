extends Node

@export var level_seed := "night-circuit-001"
@export var music_style := "funk"

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
	player.set("style", music_style)
	if not bool(player.call("generate", level_seed)):
		error_message = "Music generation failed. See Godot Output; the race can still be played."


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
