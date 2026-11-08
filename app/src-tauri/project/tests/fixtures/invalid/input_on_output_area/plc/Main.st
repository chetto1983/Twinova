PROGRAM Main
VAR
  detected_a AT %IX0.0 : BOOL;
  detected_b AT %IX0.1 : BOOL;
  belt_a_run AT %QX0.0 : BOOL;
  belt_b_run AT %QX0.1 : BOOL;
  divert AT %QX0.2 : BOOL;
END_VAR
belt_a_run := TRUE;
belt_b_run := TRUE;
divert := detected_a AND NOT detected_b;
END_PROGRAM
