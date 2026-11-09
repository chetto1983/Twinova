(* Twinova starter program: one belt conveyor, M3's program at addresses.
   The photocell near the end of the belt is sensor; the panel's Start and Stop buttons are start and stop.
   motor is the belt's run command; running is its feedback. *)
PROGRAM Main
VAR
    sensor AT %IX0.0 : BOOL;
    running AT %IX0.1 : BOOL;
    start AT %IX1.0 : BOOL;
    stop AT %IX1.1 : BOOL;
    motor AT %QX0.0 : BOOL;
    jam : BOOL;
    jamTimer : TON;
END_VAR

IF start THEN
    motor := TRUE;
    jam := FALSE;
END_IF;

(* A part held under the photocell for 2 s while the belt runs is a jam. The alarm latches: it stops the
   motor, which drops the timer's input, so jamTimer.Q alone would clear it again. *)
jamTimer(IN := sensor AND motor, PT := T#2S);
IF jamTimer.Q THEN
    jam := TRUE;
END_IF;

IF stop OR jam THEN
    motor := FALSE;
END_IF;
END_PROGRAM
