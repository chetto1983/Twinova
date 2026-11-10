(* Twinova roller conveyor, its drive, as the belt's: the PLC's run starts the motor at once; running reports it
   after the start delay. *)
FUNCTION_BLOCK RollerBehaviour
VAR_INPUT
    start_delay : TIME;
END_VAR
VAR_OUTPUT
    motor : BOOL;
END_VAR
VAR
    run AT %I* : BOOL;
    running AT %Q* : BOOL;
    started : TON;
END_VAR
motor := run;
started(IN := run, PT := start_delay);
running := started.Q;
END_FUNCTION_BLOCK
