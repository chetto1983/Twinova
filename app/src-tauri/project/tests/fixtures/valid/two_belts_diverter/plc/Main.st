(* The 4.4a line: belt A, a roller conveyor with a chain transfer at its centre, belt B. Every third carton is raised
   and carried to the reject sink.
   - The photocell at belt A's end counts cartons. The roller conveyor runs faster than belt A, so cartons part there
     and each one breaks the beams.
   - The third carton waits at belt A's end until no carton is between the two photocells and the one at the transfer
     has been clear long enough for the carton before it to leave the deck; otherwise that carton would be raised too.
   - It is raised when it reaches the transfer. A raised carton rides above the beam, so the chains stay up for the
     carry time once they have topped out (0.6 m at 0.5 m/s, with margin), then go down. Belt A and the rollers hold
     the next carton while the chains are up, so it cannot run into the raised deck.
   The two sinks' counts are declared so the I/O panel shows them. *)
PROGRAM Main
VAR
    detected_a AT %IX0.0 : BOOL;
    detected_b AT %IX0.1 : BOOL;
    detected_in AT %IX0.6 : BOOL;
    raised_fb AT %IX0.4 : BOOL;
    belt_a_run AT %QX0.0 : BOOL;
    belt_b_run AT %QX0.1 : BOOL;
    popup AT %QX0.2 : BOOL;
    roller_run AT %QX0.3 : BOOL;
    ok_count AT %ID4 : DINT;
    reject_count AT %ID8 : DINT;
    entered : R_TRIG;
    released : F_TRIG;
    arrived : R_TRIG;
    deck_clear : TON;
    carry : TON;
    count : DINT;
    reject_next : BOOL;
    reject_go : BOOL;
    hold : BOOL;
END_VAR
VAR RETAIN
    (* The cartons between the two photocells: they stay on the line through a warm restart, so does their count. *)
    in_zone : DINT;
END_VAR
belt_b_run := TRUE;
entered(CLK := detected_in);
IF entered.Q THEN
    count := count + 1;
    in_zone := in_zone + 1;
    reject_next := count MOD 3 = 0;
END_IF;
released(CLK := detected_in);
IF released.Q AND reject_next THEN
    reject_next := FALSE;
    reject_go := TRUE;
END_IF;
arrived(CLK := detected_a);
IF arrived.Q THEN
    in_zone := in_zone - 1;
    IF reject_go THEN
        popup := TRUE;
        reject_go := FALSE;
    END_IF;
END_IF;
deck_clear(IN := NOT detected_a, PT := T#800MS);
hold := reject_next AND NOT (in_zone = 1 AND deck_clear.Q);
carry(IN := popup AND raised_fb, PT := T#1500MS);
IF carry.Q THEN
    popup := FALSE;
END_IF;
belt_a_run := NOT popup AND NOT hold;
roller_run := NOT popup;
END_PROGRAM
