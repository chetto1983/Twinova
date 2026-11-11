(* The 4.4d line: 4.4c's sort line, its products sorted by what they carry. The source makes red, green and blue
   cartons with codes 1, 2 and 3, in turn. Belt In's through-beam holds the next carton at its end while roller A
   carries one, so a single carton stands at the stopper, where the colour sensor reads it. The stopper lets it onto
   the turntable when the deck is home and empty; a green one is turned to 90 degrees and run off onto belt C, the
   others run straight on to belt B. The barcode scanner on belt B reads each code once, and the pusher rejects code
   3 as it passes its photocell. Red reaches sink B, green sink C and blue the reject sink. The laser on belt C
   measures the distance to what passes it. A carton leaving belt In is the fall of its through-beam, taken by hand:
   an F_TRIG fires on its first scan, its beam being clear. *)
PROGRAM Main
VAR
    at_in AT %IX0.0 : BOOL;
    at_stop AT %IX0.1 : BOOL;
    at_turn AT %IX0.2 : BOOL;
    at_push AT %IX0.3 : BOOL;
    raised AT %IX0.4 : BOOL;
    in_position AT %IX0.5 : BOOL;
    extended AT %IX0.6 : BOOL;
    retracted AT %IX0.7 : BOOL;
    read AT %IX2.0 : BOOL;
    belt_in_run AT %QX0.0 : BOOL;
    roller_run AT %QX0.1 : BOOL;
    lane_run AT %QX0.2 : BOOL;
    belt_b_run AT %QX0.3 : BOOL;
    belt_c_run AT %QX0.4 : BOOL;
    stop AT %QX0.5 : BOOL;
    extend AT %QX0.6 : BOOL;
    target AT %QD8 : REAL;
    belt_in_running AT %IX1.0 : BOOL;
    roller_running AT %IX1.1 : BOOL;
    lane_running AT %IX1.2 : BOOL;
    belt_b_running AT %IX1.3 : BOOL;
    belt_c_running AT %IX1.4 : BOOL;
    count_b AT %ID4 : DINT;
    count_c AT %ID8 : DINT;
    count_reject AT %ID12 : DINT;
    color AT %ID16 : DINT;
    reads AT %ID20 : DINT;
    code AT %ID24 : DINT;
    distance AT %ID28 : REAL;
    step : INT;
    occupied : BOOL;
    was_in : BOOL;
    colour : DINT;
    reject : BOOL;
    arrived : R_TRIG;
    passing : R_TRIG;
    scanned : R_TRIG;
    settle : TON;
    carry : TON;
END_VAR
IF was_in AND NOT at_in THEN
    occupied := TRUE;
END_IF;
was_in := at_in;
arrived(CLK := at_turn);
settle(IN := step = 2, PT := T#300MS);
carry(IN := step = 4, PT := T#2000MS);
CASE step OF
    0:
        target := 0.0;
        lane_run := TRUE;
        IF at_stop AND in_position AND NOT at_turn THEN
            colour := color;
            step := 1;
        END_IF;
    1:
        IF arrived.Q THEN
            occupied := FALSE;
            IF colour = 2 THEN
                lane_run := FALSE;
                step := 2;
            ELSE
                step := 5;
            END_IF;
        END_IF;
    2:
        IF settle.Q THEN
            target := 90.0;
            step := 3;
        END_IF;
    3:
        IF in_position THEN
            lane_run := TRUE;
            step := 4;
        END_IF;
    4:
        IF carry.Q THEN
            target := 0.0;
            step := 6;
        END_IF;
    5:
        IF NOT at_turn THEN
            step := 0;
        END_IF;
    6:
        IF in_position THEN
            step := 0;
        END_IF;
END_CASE;
stop := step <> 1;
belt_in_run := NOT (at_in AND occupied);
roller_run := TRUE;
belt_b_run := TRUE;
belt_c_run := TRUE;
scanned(CLK := read);
IF scanned.Q THEN
    reject := code = 3;
END_IF;
passing(CLK := at_push);
IF passing.Q AND reject THEN
    extend := TRUE;
    reject := FALSE;
END_IF;
IF extended THEN
    extend := FALSE;
END_IF;
END_PROGRAM
