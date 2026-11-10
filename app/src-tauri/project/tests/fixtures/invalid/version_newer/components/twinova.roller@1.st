(* Twinova roller conveyor, its drive, re-expressed from Open Industry Project's conveyor drive
   (examples/parts/_shared/conveyorDrive.js at 1e7488c, MIT). While the PLC's run is on, the surface ramps to
   speed_ref, which reads the speed parameter unless the PLC writes it: up at speed_at_30 per accel_time, down at
   speed_at_30 per decel_time, through 0 before it reverses; a ramp time of 0 is instant. running reports run after
   the start delay. velocity reads 30 at speed_at_30; encoder is the surface's travel in metres; pulse is TRUE for one
   cycle each pulse_pitch of travel. The field cycles every 1 ms, and TIME_TO_DWORD gives whole milliseconds, made a
   REAL through DINT: truST's DWORD_TO_REAL keeps the bits, not the number. *)
FUNCTION_BLOCK RollerBehaviour
VAR_INPUT
    start_delay : TIME;
    accel_time : TIME;
    decel_time : TIME;
    speed_at_30 : REAL;
    pulse_pitch : REAL;
END_VAR
VAR_OUTPUT
    surface_speed : REAL;
END_VAR
VAR
    run AT %I* : BOOL;
    speed_ref AT %I* : REAL;
    running AT %Q* : BOOL;
    velocity AT %Q* : REAL;
    encoder AT %Q* : REAL;
    pulse AT %Q* : BOOL;
    started : TON;
    target : REAL;
    ramp : TIME;
    step : REAL;
    odometer : LREAL;
    travel : LREAL;
END_VAR
IF run THEN
    target := speed_ref;
ELSE
    target := 0.0;
END_IF;
IF target * surface_speed < 0.0 THEN
    IF decel_time > T#0MS THEN
        target := 0.0;
    ELSE
        surface_speed := 0.0;
    END_IF;
END_IF;
IF ABS(target) > ABS(surface_speed) THEN
    ramp := accel_time;
ELSE
    ramp := decel_time;
END_IF;
step := 0.0;
IF ramp > T#0MS THEN
    step := speed_at_30 / DINT_TO_REAL(DWORD_TO_DINT(TIME_TO_DWORD(ramp)));
END_IF;
(* n steps of speed_at_30 / n add up to speed_at_30 only within a rounding of REAL, so the ramp lands on its target
   within a hundredth of a step, on its millisecond. *)
IF step <= 0.0 OR ABS(target - surface_speed) <= step * 1.01 THEN
    surface_speed := target;
ELSIF target > surface_speed THEN
    surface_speed := surface_speed + step;
ELSE
    surface_speed := surface_speed - step;
END_IF;
started(IN := run, PT := start_delay);
running := started.Q;
IF speed_at_30 > 0.0 THEN
    velocity := surface_speed / speed_at_30 * 30.0;
ELSE
    velocity := 0.0;
END_IF;
odometer := odometer + REAL_TO_LREAL(surface_speed) * 0.001;
encoder := LREAL_TO_REAL(odometer);
travel := travel + REAL_TO_LREAL(ABS(surface_speed)) * 0.001;
pulse := pulse_pitch > 0.0 AND travel >= REAL_TO_LREAL(pulse_pitch) * 0.999999;
IF pulse THEN
    travel := MAX(travel - REAL_TO_LREAL(pulse_pitch), 0.0);
END_IF;
END_FUNCTION_BLOCK
