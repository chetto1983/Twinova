(* Twinova pusher, re-expressed from Open Industry Project's diverter (examples/parts/diverter/behavior.ts step() at
   1e7488c, MIT). extend sweeps the paddle out across its host; the reach slews linearly over push_time, out and back,
   as OIP's approach(). extended_fb and retracted_fb report the two ends of the stroke, which OIP shows only as its
   red and green lamps. The field cycles every 1 ms, so travel counts the milliseconds of stroke done, out of span. *)
FUNCTION_BLOCK PusherBehaviour
VAR_INPUT
    push_time : TIME;
END_VAR
VAR_OUTPUT
    reach : REAL;
END_VAR
VAR
    extend AT %I* : BOOL;
    extended_fb AT %Q* : BOOL;
    retracted_fb AT %Q* : BOOL;
    span : DINT;
    travel : DINT;
END_VAR
span := MAX(DWORD_TO_DINT(TIME_TO_DWORD(push_time)), 1);
IF extend THEN
    travel := MIN(travel + 1, span);
ELSE
    travel := MAX(travel - 1, 0);
END_IF;
reach := DINT_TO_REAL(travel) / DINT_TO_REAL(span);
extended_fb := travel = span;
retracted_fb := travel = 0;
END_FUNCTION_BLOCK
