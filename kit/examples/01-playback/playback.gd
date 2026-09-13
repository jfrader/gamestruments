extends Control

## 01 — Playback: generate a Suspense Theme title bed once at load, play it,
## restart with a fixed seed, and read the current section. Plain reference:
## copy the generate/restart flow, not the widget layout.

@onready var status_label: Label = $ScrollContainer/Margin/Main/StatusLabel
@onready var section_label: Label = $ScrollContainer/Margin/Main/SectionLabel
@onready var error_label: Label = $ScrollContainer/Margin/Main/ErrorLabel
@onready var restart_button: Button = $ScrollContainer/Margin/Main/RestartButton
@onready var web_link: LinkButton = $ScrollContainer/Margin/Main/WebLabLink

const PROJECT_SECRET := "gamestruments-kit-example-01"
const SEED := "title-001"

var player: Node = null
var _ready_to_act := false


func _ready() -> void:
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
	player.set("arrangement", "theme")

	status_label.text = "Generating (at load)..."
	if not bool(player.call("generate", SEED)):
		status_label.text = "generate() failed"
		section_label.text = "Current section: (failed)"
		restart_button.disabled = true
		return

	_ready_to_act = true
	status_label.text = "generate() OK"
	restart_button.pressed.connect(_on_restart)
	_update_current_section()

	var timer := Timer.new()
	timer.wait_time = 0.25
	timer.timeout.connect(_update_current_section)
	add_child(timer)
	timer.start()


func _show_missing_addon() -> void:
	error_label.text = "Music extension missing. Copy addons/ next to your project.godot, reopen Godot and let import finish."
	error_label.visible = true
	restart_button.disabled = true
	web_link.disabled = false
	status_label.text = "Addon required for audio."
	section_label.text = "Current section: (disabled)"


func _on_restart() -> void:
	if not _ready_to_act:
		return
	status_label.text = "Restarting..."
	if not bool(player.call("generate", SEED)):
		status_label.text = "Regenerate failed"
		return
	status_label.text = "Restarted (fixed seed)"
	_update_current_section()


func _update_current_section() -> void:
	if not _ready_to_act:
		return
	var sec: String = player.call("get_current_section")
	section_label.text = "Current section: %s" % (sec if sec.length() > 0 else "(starting)")


func _on_open_lab() -> void:
	var err := OS.shell_open("https://gamestruments.gurisitos.games")
	if err != OK:
		error_label.text = "shell_open failed. Visit https://gamestruments.gurisitos.games manually."
		error_label.visible = true
