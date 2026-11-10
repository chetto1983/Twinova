(* The 4.4a line: belt A, a roller conveyor with a chain transfer at its centre, belt B. The photocell at the transfer
   counts cartons; every third one is raised and carried to the reject sink. The chains go down once they have topped
   out and the carton has left the beam. *)
PROGRAM Main
VAR
    detected_a AT %IX0.0 : BOOL;
    detected_b AT %IX0.1 : BOOL;
    raised_fb AT %IX0.4 : BOOL;
    belt_a_run AT %QX0.0 : BOOL;
    belt_b_run AT %QX0.1 : BOOL;
    popup AT %QX0.2 : BOOL;
    roller_run AT %QX0.3 : BOOL;
    arrived : R_TRIG;
    count : DINT;
END_VAR
belt_a_run := TRUE;
belt_b_run := TRUE;
roller_run := TRUE;
arrived(CLK := detected_a);
IF arrived.Q THEN
    count := count + 1;
    IF count MOD 3 = 0 THEN
        popup := TRUE;
    END_IF;
END_IF;
IF popup AND raised_fb AND NOT detected_a THEN
    popup := FALSE;
END_IF;
END_PROGRAM
