(* Twinova chain transfer, re-expressed from Open Industry Project's chain transfer
   (examples/parts/chain-transfer/behavior.ts at 1e7488c, MIT). popup raises the chains and carries sideways at once;
   the lift slews linearly over raise_time, as OIP's approach(). raised_fb reports the top of the stroke, which OIP
   does not have. The field cycles every 1 ms (spec 2026-11-09-component-behaviour-design.md, decision 3), so travel
   counts the milliseconds of stroke done; TIME_TO_DWORD gives whole milliseconds. *)
FUNCTION_BLOCK ChainTransferBehaviour
VAR_INPUT
    raise_time : TIME;
END_VAR
VAR_OUTPUT
    lift : REAL;
    raised : BOOL;
END_VAR
VAR
    popup AT %I* : BOOL;
    raised_fb AT %Q* : BOOL;
    stroke : DINT;
    travel : DINT;
END_VAR
raised := popup;
stroke := MAX(DWORD_TO_DINT(TIME_TO_DWORD(raise_time)), 1);
IF raised THEN
    travel := MIN(travel + 1, stroke);
ELSE
    travel := MAX(travel - 1, 0);
END_IF;
lift := DINT_TO_REAL(travel) / DINT_TO_REAL(stroke);
raised_fb := raised AND travel = stroke;
END_FUNCTION_BLOCK
