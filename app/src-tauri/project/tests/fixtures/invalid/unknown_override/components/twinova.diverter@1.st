(* Twinova diverter, its flap. The flap starts at default_output; each change of the PLC's divert moves it for
   switch_time, while position is -1, then it rests at output B (1) for divert TRUE or A (0) for FALSE. in_position
   reports that the flap rests where divert asks. *)
FUNCTION_BLOCK DiverterBehaviour
VAR_INPUT
    switch_time : TIME;
    default_output : DINT;
END_VAR
VAR_OUTPUT
    position : DINT := -1;
END_VAR
VAR
    divert AT %I* : BOOL;
    in_position AT %Q* : BOOL;
    started : BOOL;
    target : DINT;
    heading : DINT;
    moving : TON;
END_VAR
IF NOT started THEN
    started := TRUE;
    heading := default_output;
    position := default_output;
END_IF;
IF divert THEN
    target := 1;
ELSE
    target := 0;
END_IF;
IF target <> heading THEN
    heading := target;
    position := -1;
END_IF;
moving(IN := position = -1, PT := switch_time);
IF moving.Q THEN
    position := heading;
END_IF;
in_position := position = target;
END_FUNCTION_BLOCK
