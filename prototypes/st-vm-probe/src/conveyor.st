FUNCTION_BLOCK FB_Conveyor
VAR_INPUT
    Start : BOOL;
    Stop : BOOL;
    PartAtEnd : BOOL;
END_VAR
VAR_OUTPUT
    Motor : BOOL;
    Count : DINT;
    Jam : BOOL;
END_VAR
VAR
    edge : R_TRIG;
    jamTimer : TON;
END_VAR
IF Start THEN
    Motor := TRUE;
END_IF;
IF Stop OR Jam THEN
    Motor := FALSE;
END_IF;
edge(CLK := PartAtEnd);
IF edge.Q THEN
    Count := Count + DINT#1;
END_IF;
(* The jam alarm latches: it stops the motor, which drops the timer's IN, so Q alone would clear it. *)
jamTimer(IN := PartAtEnd AND Motor, PT := T#2S);
IF jamTimer.Q THEN
    Jam := TRUE;
END_IF;
IF Start AND NOT PartAtEnd THEN
    Jam := FALSE;
END_IF;
END_FUNCTION_BLOCK

PROGRAM Main
VAR
    start : BOOL;
    stop : BOOL;
    sensor : BOOL;
    belt : FB_Conveyor;
    motor : BOOL;
    count : DINT;
    jam : BOOL;
END_VAR
belt(Start := start, Stop := stop, PartAtEnd := sensor);
motor := belt.Motor;
count := belt.Count;
jam := belt.Jam;
END_PROGRAM
