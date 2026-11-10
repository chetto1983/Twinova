(* The 4.4b curve line: belt A, a left belt curve, belt B, a right roller curve, belt C. Every conveyor runs; the
   photocells and the sink's count are declared so the I/O panel shows them. *)
PROGRAM Main
VAR
    detected_a AT %IX0.0 : BOOL;
    detected_c AT %IX0.4 : BOOL;
    belt_a_run AT %QX0.0 : BOOL;
    curve_a_run AT %QX0.1 : BOOL;
    belt_b_run AT %QX0.2 : BOOL;
    curve_b_run AT %QX0.3 : BOOL;
    belt_c_run AT %QX0.4 : BOOL;
    delivered AT %ID4 : DINT;
END_VAR
belt_a_run := TRUE;
curve_a_run := TRUE;
belt_b_run := TRUE;
curve_b_run := TRUE;
belt_c_run := TRUE;
END_PROGRAM
