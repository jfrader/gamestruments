extends RefCounted

const LAPS := 3
const CRUISE_SPEED := 0.064
const BOOST_SPEED := 0.092
const ROAD_EDGE := 32.0

var phase := "garage"
var countdown := 3.0
var elapsed := 0.0
var progress := 0.0
var rival_progress := 0.015
var lane := 0.0
var rival_lane := -14.0
var speed := 0.0
var charge := 1.0
var boosting := false
var boost_exhausted := false
var collision_cooldown := 0.0
var impacts := 0
var won := false


func start() -> void:
	phase = "grid"
	countdown = 3.0
	elapsed = 0.0
	progress = 0.0
	rival_progress = 0.015
	lane = 0.0
	rival_lane = -14.0
	speed = 0.0
	charge = 1.0
	boosting = false
	boost_exhausted = false
	collision_cooldown = 0.0
	impacts = 0
	won = false


func step(delta: float, throttle: bool, steering: float, boost: bool, brake: bool) -> void:
	if phase == "grid":
		countdown = maxf(0.0, countdown - delta)
		if countdown == 0.0:
			phase = "race"
		return
	if phase != "race":
		return
	elapsed += delta
	collision_cooldown = maxf(0.0, collision_cooldown - delta)
	lane = clampf(lane + steering * 65.0 * delta, -58.0, 58.0)
	if not boost:
		boost_exhausted = false
	boosting = boost and throttle and not brake and charge > 0.0 and not boost_exhausted and not off_road()
	charge = clampf(charge + (-0.32 if boosting else 0.13) * delta, 0.0, 1.0)
	if charge == 0.0:
		boost_exhausted = true
	var target_speed := (BOOST_SPEED if boosting else CRUISE_SPEED) if throttle else 0.0
	if brake:
		target_speed = 0.0
	if off_road():
		target_speed = minf(target_speed, 0.025)
	speed = move_toward(speed, target_speed, delta * (0.12 if brake or off_road() else 0.035))
	rival_lane = sin(elapsed * 0.55) * 19.0
	rival_progress = minf(float(LAPS), rival_progress + (0.061 + 0.004 * sin(elapsed * 0.3)) * delta)
	progress += speed * delta
	if rival_progress < LAPS and absf(rival_progress - progress) < 0.008 and absf(lane - rival_lane) < 12.0 and collision_cooldown == 0.0:
		speed *= 0.45
		collision_cooldown = 1.2
		impacts += 1
	if progress >= LAPS:
		progress = float(LAPS)
		won = rival_progress < LAPS
		phase = "finish"
		boosting = false
		speed = 0.0


func lap() -> int:
	return mini(LAPS, int(progress) + 1)


func off_road() -> bool:
	return absf(lane) > ROAD_EDGE


func pressure() -> float:
	return clampf(1.0 - absf(rival_progress - progress) / 0.14, 0.0, 1.0) if rival_progress < LAPS else 0.0


func music_state() -> Dictionary:
	return {
		"phase": phase,
		"intensity": clampf(0.30 + speed / CRUISE_SPEED * 0.35 + (0.25 if boosting else 0.0), 0.0, 1.0) if phase == "race" else 0.0,
		"pressure": pressure() if phase == "race" else 0.0,
		"final_lap": phase == "race" and lap() == LAPS,
		"result": ("win" if won else "loss") if phase == "finish" else "none",
	}
