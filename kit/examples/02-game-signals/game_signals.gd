extends Control

## 02 — Game signals: translate simulated race events into set_race_state
## requests on the Racing recipe. Requested and currently playing sections are
## intentionally different — changes commit on bar boundaries.

@onready var status_label: Label = $ScrollContainer/Margin/Main/StatusLabel
@onready var requested_label: Label = $ScrollContainer/Margin/Main/RequestedLabel
@onready var current_label: Label = $ScrollContainer/Margin/Main/CurrentLabel
@onready var result_label: Label = $ScrollContainer/Margin/Main/ResultLabel
@onready var error_label: Label = $ScrollContainer/Margin/Main/ErrorLabel
@onready var intensity: HSlider = $ScrollContainer/Margin/Main/IntensityRow/IntensitySlider
@onready var pressure: HSlider = $ScrollContainer/Margin/Main/PressureRow/PressureSlider
@onready var final_lap: CheckBox = $ScrollContainer/Margin/Main/FinalLapRow/FinalLapCheck
@onready var web_link: LinkButton = $ScrollContainer/Margin/Main/WebLabLink
@onready var apply_button: Button = $ScrollContainer/Margin/Main/ApplyButton

const PROJECT_SECRET := "gamestruments-kit-example-02"
const SEED := "signals-garage-001"
const PHASES: Array[String] = ["garage", "grid", "race", "finish"]
const RESULTS: Array[String] = ["win", "loss"]

var player: Node = null
var _ready_to_act := false
var current_phase := "garage"
var finish_result := "none"


func _ready() -> void:
	for ph in PHASES:
		_get_phase_button(ph).pressed.connect(_on_phase.bind(ph))
	apply_button.pressed.connect(_on_apply)
	for res in RESULTS:
		_get_finish_button(res).pressed.connect(_on_finish.bind(res))
	web_link.pressed.connect(_on_open_lab)

	if not ClassDB.class_exists("GamestrumentsPlayer"):
		_show_missing_addon()
		return

	player = ClassDB.instantiate("GamestrumentsPlayer")
	player.name = "GamestrumentsPlayer"
	add_child(player)

	player.set("project_secret", PROJECT_SECRET)
	player.set("recipe", "racing")
	player.set("arrangement", "original")
	player.set("autoplay", false)
	player.set("style", "neon")

	status_label.text = "Generating..."
	if not bool(player.call("generate", SEED)):
		status_label.text = "generate() failed"
		_disable_action_controls()
		return

	_ready_to_act = true
	status_label.text = "Generated. Use the buttons."
	_request_state("garage", 0.0, 0.0, false)

	var timer := Timer.new()
	timer.wait_time = 0.2
	timer.timeout.connect(_update_current)
	add_child(timer)
	timer.start()


func _get_phase_button(ph: String) -> Button:
	return get_node("ScrollContainer/Margin/Main/PhaseButtons/%sButton" % ph.capitalize())


func _get_finish_button(res: String) -> Button:
	return get_node("ScrollContainer/Margin/Main/FinishButtons/Finish%sButton" % res.capitalize())


func _disable_action_controls() -> void:
	apply_button.disabled = true
	final_lap.disabled = true
	intensity.editable = false
	pressure.editable = false
	for ph in PHASES:
		_get_phase_button(ph).disabled = true
	for res in RESULTS:
		_get_finish_button(res).disabled = true


func _show_missing_addon() -> void:
	error_label.text = "Music extension missing. Copy addons/ next to your project.godot, reopen Godot and let import finish."
	error_label.visible = true
	_disable_action_controls()
	web_link.disabled = false
	status_label.text = "Addon required — controls disabled."


func _on_phase(ph: String) -> void:
	# Finish clears inputs that would otherwise select attack or final-lap.
	if ph == "finish":
		_finish("none")
		return
	current_phase = ph
	_request_state(ph, intensity.value, pressure.value, final_lap.button_pressed)


func _on_apply() -> void:
	# After a finish, Apply re-sends the same finish request (normalized UI +
	# the last result) so it can never jump back to the previous race phase.
	if current_phase == "finish":
		_finish(finish_result)
		return
	_request_state(current_phase, intensity.value, pressure.value, final_lap.button_pressed)


func _on_finish(res: String) -> void:
	_finish(res)


func _finish(result: String) -> void:
	current_phase = "finish"
	finish_result = result
	intensity.value = 0.0
	pressure.value = 0.0
	final_lap.button_pressed = false
	_request_state("finish", 0.0, 0.0, false, result)


func _request_state(phase: String, inten: float, pres: float, fin: bool, res: String = "none") -> void:
	if not _ready_to_act:
		return
	var ok: bool = player.call("set_race_state", phase, inten, pres, fin, res)
	requested_label.text = "Requested: %s i=%.2f p=%.2f final=%s res=%s" % [phase, inten, pres, fin, res]
	result_label.text = "set_race_state returned: %s" % ("true" if ok else "false")
	if not ok:
		status_label.text = "set_race_state rejected"
	_update_current()


func _update_current() -> void:
	if not _ready_to_act:
		return
	var sec: String = player.call("get_current_section")
	current_label.text = "Current section: %s" % (sec if sec.length() > 0 else "(crossfading)")


func _on_open_lab() -> void:
	var err := OS.shell_open("https://gamestruments.gurisitos.games")
	if err != OK:
		error_label.text = "shell_open failed."
		error_label.visible = true
