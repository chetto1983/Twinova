(* The merge line: a spur belt merges square onto the main belt's right flank, through the opening Build cuts in its
   guard. Start runs both belts, Stop stops them; the sink's count is declared so the I/O panel shows it. *)
PROGRAM Main
VAR
    start AT %IX1.0 : BOOL;
    stop AT %IX1.1 : BOOL;
    main_running AT %IX0.1 : BOOL;
    spur_running AT %IX0.2 : BOOL;
    main AT %QX0.0 : BOOL;
    spur AT %QX0.1 : BOOL;
    delivered AT %ID4 : DINT;
END_VAR

IF start THEN
    main := TRUE;
    spur := TRUE;
END_IF;
IF stop THEN
    main := FALSE;
    spur := FALSE;
END_IF;
END_PROGRAM
