(* Twinova starter program: one belt conveyor.
   The scene writes sensor (the photocell near the end of the belt) and the buttons write start and stop.
   The scene reads motor. *)
PROGRAM Main
VAR
    sensor : BOOL;
    start : BOOL;
    stop : BOOL;
    motor : BOOL;
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
