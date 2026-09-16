extends Control

## 03 — Song form: drive the Suspense all-phases pool with trace-state
## requests, section cues, and hold/advance form controls.

@onready var status_label: Label = $ScrollContainer/Margin/Main/StatusLabel
@onready var held_label: Label = $ScrollContainer/Margin/Main/HeldLabel
@onready var current_label: Label = $ScrollContainer/Margin/Main/CurrentLabel
@onready var result_label: Label = $ScrollContainer/Margin/Main/ResultLabel
@onready var error_label: Label = $ScrollContainer/Margin/Main/ErrorLabel
@onready var heat: HSlider = $ScrollContainer/Margin/Main/HeatRow/HeatSlider
@onready var focus_s: HSlider = $ScrollContainer/Margin/Main/FocusRow/FocusSlider
@onready var progress: HSlider = $ScrollContainer/Margin/Main/ProgressRow/ProgressSlider
@onready var web_link: LinkButton = $ScrollContainer/Margin/Main/WebLabLink
@onready var apply_trace_button: Button = $ScrollContainer/Margin/Main/ApplyTraceButton

const PROJECT_SECRET := "gamestruments-kit-example-03"
const SEED := "form-demo-001"
const FORM_ACTIONS: Array[String] = ["Hold", "Resume", "Advance", "CueChorus"]
const TRACE_PHASES: Array[String] = ["scan", "alert", "complete", "extract"]

var player: Node = null
var _ready_to_act := false
var current_trace_phase := "scan"


func _ready() -> void:
	var form_handlers := {
		"Hold": _on_hold.bind(true),
		"Resume": _on_hold.bind(false),
		"Advance": _on_advance,
		"CueChorus": _on_cue,
	}
	for act in FORM_ACTIONS:
		var btn := _get_form_button(act)
		btn.pressed.connect(form_handlers[act])

	for ph in TRACE_PHASES:
		_get_trace_button(ph).pressed.connect(_on_trace_phase.bind(ph))

	apply_trace_button.pressed.connect(_on_apply_trace)
	web_link.pressed.connect(_on_open_lab)

	if not ClassDB.class_exists("GamestrumentsPlayer"):
		_show_missing_addon()
		return

	player = ClassDB.instantiate("GamestrumentsPlayer")
	player.name = "GamestrumentsPlayer"
	add_child(player)

	player.set("project_secret", PROJECT_SECRET)
	player.set("recipe", "suspense")
	player.set("style", "terminal")
	player.set("arrangement", "all-phases")

	status_label.text = "Generating..."
	if not bool(player.call("generate", SEED)):
		status_label.text = "generate() failed"
		_disable_action_controls()
		return

	_ready_to_act = true
	status_label.text = "Generated."
	_update_state()

	var timer := Timer.new()
	timer.wait_time = 0.2
	timer.timeout.connect(_update_state)
	add_child(timer)
	timer.start()


func _get_form_button(act: String) -> Button:
	return get_node("ScrollContainer/Margin/Main/FormButtons/%sButton" % act)


func _get_trace_button(ph: String) -> Button:
	return get_node("ScrollContainer/Margin/Main/TraceButtons/%sButton" % ph.capitalize())


func _disable_action_controls() -> void:
	apply_trace_button.disabled = true
	heat.editable = false
	focus_s.editable = false
	progress.editable = false
	for act in FORM_ACTIONS:
		_get_form_button(act).disabled = true
	for ph in TRACE_PHASES:
		_get_trace_button(ph).disabled = true


func _show_missing_addon() -> void:
	error_label.text = "Music extension missing. Copy addons/ next to your project.godot, reopen Godot and let import finish."
	error_label.visible = true
	_disable_action_controls()
	web_link.disabled = false
	status_label.text = "Addon required — controls disabled."


func _on_hold(held: bool) -> void:
	if not _ready_to_act:
		return
	var ok: bool = player.call("set_form_hold", held)
	result_label.text = "set_form_hold(%s) returned: %s" % [held, ok]
	_update_state()


func _on_advance() -> void:
	if not _ready_to_act:
		return
	var ok: bool = player.call("advance_form")
	result_label.text = "advance_form returned: %s" % ok
	_update_state()


func _on_cue() -> void:
	if not _ready_to_act:
		return
	var ok: bool = player.call("cue_section", "chorus")
	result_label.text = "cue_section returned: %s" % ok
	_update_state()


func _on_trace_phase(ph: String) -> void:
	current_trace_phase = ph
	_apply_trace()


func _on_apply_trace() -> void:
	_apply_trace()


func _apply_trace() -> void:
	if not _ready_to_act:
		return
	var ok: bool = player.call("set_trace_state", current_trace_phase, heat.value, focus_s.value, progress.value)
	result_label.text = "set_trace_state(%s) returned: %s" % [current_trace_phase, ok]
	_update_state()


func _update_state() -> void:
	if not _ready_to_act:
		return
	var sec: String = player.call("get_current_section")
	var held: bool = player.call("is_form_held")
	held_label.text = "Form held: %s" % held
	current_label.text = "Current section: %s" % (sec if sec.length() > 0 else "(progressing)")


func _on_open_lab() -> void:
	var err := OS.shell_open("https://gamestruments.gurisitos.games")
	if err != OK:
		error_label.text = "shell_open failed."
		error_label.visible = true
