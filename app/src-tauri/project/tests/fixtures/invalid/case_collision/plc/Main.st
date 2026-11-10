(* The 4.4a line: belt A, a roller conveyor with a chain transfer at its centre, belt B. The photocell at the transfer
   counts cartons; every third one is raised and carried to the reject sink. A raised carton rides above the beam, so
   the chains stay up for the carry time once they have topped out (0.6 m at 0.5 m/s, with margin), then go down.
   Belt A and the rollers hold the next carton while the chains are up, so it cannot run into the raised deck. The
   two sinks' counts are declared so the I/O panel shows them. *)
PROGRAM Main
VAR
    detected_a AT %IX0.0 : BOOL;
    detected_b AT %IX0.1 : BOOL;
    raised_fb AT %IX0.4 : BOOL;
    belt_a_run AT %QX0.0 : BOOL;
    belt_b_run AT %QX0.1 : BOOL;
    popup AT %QX0.2 : BOOL;
    roller_run AT %QX0.3 : BOOL;
    ok_count AT %ID4 : DINT;
    reject_count AT %ID8 : DINT;
    arrived : R_TRIG;
    carry : TON;
    count : DINT;
END_VAR
belt_b_run := TRUE;
arrived(CLK := detected_a);
IF arrived.Q THEN
    count := count + 1;
    IF count MOD 3 = 0 THEN
        popup := TRUE;
    END_IF;
END_IF;
carry(IN := popup AND raised_fb, PT := T#1500MS);
IF carry.Q THEN
    popup := FALSE;
END_IF;
belt_a_run := NOT popup;
roller_run := NOT popup;
END_PROGRAM
