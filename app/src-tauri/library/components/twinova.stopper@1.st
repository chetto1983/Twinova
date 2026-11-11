(* Twinova stopper, re-expressed from Open Industry Project's blade stop (examples/parts/blade-stop/part.js step()
   at 1e7488c, MIT). stop raises the blade; the lift slews linearly over raise_time, as OIP's approach() over its
   fixed 0.15 s. raised_fb reports the top of the stroke, which OIP does not have. The field cycles every 1 ms, so
   travel counts the milliseconds of stroke done; TIME_TO_DWORD gives whole milliseconds. *)
FUNCTION_BLOCK StopperBehaviour
VAR_INPUT
    raise_time : TIME;
END_VAR
VAR_OUTPUT
    lift : REAL;
END_VAR
VAR
    stop AT %I* : BOOL;
    raised_fb AT %Q* : BOOL;
    stroke : DINT;
    travel : DINT;
END_VAR
stroke := MAX(DWORD_TO_DINT(TIME_TO_DWORD(raise_time)), 1);
IF stop THEN
    travel := MIN(travel + 1, stroke);
ELSE
    travel := MAX(travel - 1, 0);
END_IF;
lift := DINT_TO_REAL(travel) / DINT_TO_REAL(stroke);
raised_fb := stop AND travel = stroke;
END_FUNCTION_BLOCK
