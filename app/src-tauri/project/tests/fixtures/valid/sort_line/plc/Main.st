(* The 4.4c sort line: belt In, roller A with a stopper, a turntable, belt B with a pusher, and belt C off the
   turntable's 90 degrees. Belt In holds the next carton at its end while roller A carries one, so a single carton
   stands at the stopper and the blade never rises under another. The stopper lets it onto the turntable when the
   deck is home and empty, and rises again once the carton reaches the turntable's photocell. Every second carton is
   stopped there, turned to 90 degrees, run off onto belt C, and the deck turns home; the others run straight on to
   belt B, where every second one is pushed off to the reject sink as it passes the pusher's photocell. The sinks'
   counts are declared so the I/O panel shows them. *)
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
    belt_in_run AT %QX0.0 : BOOL;
    roller_run AT %QX0.1 : BOOL;
    lane_run AT %QX0.2 : BOOL;
    belt_b_run AT %QX0.3 : BOOL;
    belt_c_run AT %QX0.4 : BOOL;
    stop AT %QX0.5 : BOOL;
    extend AT %QX0.6 : BOOL;
    target AT %QD8 : REAL;
    count_b AT %ID4 : DINT;
    count_c AT %ID8 : DINT;
    count_reject AT %ID12 : DINT;
    step : INT;
    occupied : BOOL;
    cartons : DINT;
    straight : DINT;
    left_in : F_TRIG;
    arrived : R_TRIG;
    passing : R_TRIG;
    settle : TON;
    carry : TON;
END_VAR
left_in(CLK := at_in);
IF left_in.Q THEN
    occupied := TRUE;
END_IF;
arrived(CLK := at_turn);
settle(IN := step = 2, PT := T#300MS);
carry(IN := step = 4, PT := T#2000MS);
CASE step OF
    0:
        target := 0.0;
        lane_run := TRUE;
        IF at_stop AND in_position AND NOT at_turn THEN
            step := 1;
        END_IF;
    1:
        IF arrived.Q THEN
            occupied := FALSE;
            cartons := cartons + 1;
            IF cartons MOD 2 = 0 THEN
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
passing(CLK := at_push);
IF passing.Q THEN
    straight := straight + 1;
    IF straight MOD 2 = 0 THEN
        extend := TRUE;
    END_IF;
END_IF;
IF extended THEN
    extend := FALSE;
END_IF;
END_PROGRAM
