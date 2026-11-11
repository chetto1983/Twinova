(* Twinova turntable, re-expressed from Open Industry Project's turntable conveyor
   (examples/parts/turntable-conveyor/part.js step() at 1e7488c, MIT) and its conveyor drive
   (examples/parts/_shared/conveyorDrive.js). The deck slews the shortest way to target_angle, which reads the angle
   parameter unless the PLC writes it: up to rotation_speed over 0.5 s, braking to land on its target, and to a stop
   over 0.3 s while enable is off, resuming when it returns. Two changes from OIP: the deck starts at its angle
   parameter, not at 0, and a new target the other way ramps the turn through 0 instead of reversing it within a
   step. in_position reports the deck at rest on its target. The lane is a conveyor: while run is on and the deck is
   in position its surface ramps to speed_ref, as every conveyor's drive does (accel_time, decel_time, through 0
   before it reverses), and to 0 otherwise. The native model reads deck_angle (degrees, counter-clockwise seen from
   above), deck_rate (degrees per second) and surface_speed, and publishes the lane's velocity, encoder and pulse. The
   field cycles every 1 ms. *)
FUNCTION_BLOCK TurntableBehaviour
VAR_INPUT
    angle : REAL;
    rotation_speed : REAL;
    start_delay : TIME;
    accel_time : TIME;
    decel_time : TIME;
    speed_at_30 : REAL;
END_VAR
VAR_OUTPUT
    deck_angle : REAL;
    deck_rate : REAL;
    surface_speed : REAL;
END_VAR
VAR
    run AT %I* : BOOL;
    speed_ref AT %I* : REAL;
    target_angle AT %I* : REAL;
    enable AT %I* : BOOL;
    running AT %Q* : BOOL;
    in_position AT %Q* : BOOL;
    placed : BOOL;
    landed : BOOL;
    d : REAL;
    goal : REAL;
    change : REAL;
    started : TON;
    target : REAL;
    ramp : TIME;
    step : REAL;
END_VAR
IF NOT placed THEN
    deck_angle := angle;
    placed := TRUE;
END_IF;
d := target_angle - deck_angle;
IF d > 180.0 THEN
    d := d - 360.0;
ELSIF d <= -180.0 THEN
    d := d + 360.0;
END_IF;
change := rotation_speed / 0.5 * 0.001;
goal := 0.0;
IF NOT enable THEN
    change := rotation_speed / 0.3 * 0.001;
ELSIF deck_rate * d >= 0.0 AND ABS(d) > deck_rate * deck_rate / (rotation_speed / 0.5 * 2.0) THEN
    IF d > 0.0 THEN
        goal := rotation_speed;
    ELSE
        goal := -rotation_speed;
    END_IF;
END_IF;
IF deck_rate < goal THEN
    deck_rate := MIN(deck_rate + change, goal);
ELSE
    deck_rate := MAX(deck_rate - change, goal);
END_IF;
landed := deck_rate * d > 0.0 AND ABS(d) <= ABS(deck_rate) * 0.001;
IF landed THEN
    deck_angle := target_angle;
    deck_rate := 0.0;
ELSE
    deck_angle := deck_angle + deck_rate * 0.001;
END_IF;
IF deck_angle > 180.0 THEN
    deck_angle := deck_angle - 360.0;
ELSIF deck_angle <= -180.0 THEN
    deck_angle := deck_angle + 360.0;
END_IF;
in_position := deck_rate = 0.0 AND (landed OR d = 0.0);
IF run AND in_position THEN
    target := speed_ref;
ELSE
    target := 0.0;
END_IF;
IF target <> surface_speed THEN
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
    IF step <= 0.0 OR ABS(target - surface_speed) <= step * 1.01 THEN
        surface_speed := target;
    ELSIF target > surface_speed THEN
        surface_speed := surface_speed + step;
    ELSE
        surface_speed := surface_speed - step;
    END_IF;
END_IF;
started(IN := run, PT := start_delay);
running := started.Q;
END_FUNCTION_BLOCK
