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
    in_zone : DINT;
END_VAR

(* Belt B always runs *)
(belt_b_run);
(* A carton entering the zone is counted; every third is marked for reject *)
|detected_in| entered(CLK);
|entered.Q| ADD(IN1 := count, IN2 := 1, OUT => count) ADD(IN1 := in_zone, IN2 := 1, OUT => in_zone) MOVE(IN := count MOD 3 = 0, OUT => reject_next);
|detected_in| released(CLK);
|released.Q| |reject_next| (R reject_next) (S reject_go);
(* At the transfer: the marked carton raises the chains *)
|detected_a| arrived(CLK);
|arrived.Q| SUB(IN1 := in_zone, IN2 := 1, OUT => in_zone) |reject_go| (S popup) (R reject_go);
|/detected_a| deck_clear(IN, PT := T#800MS);
[|/reject_next|, EQ(IN1 := in_zone, IN2 := 1, OUT) |deck_clear.Q|] (/hold);
|popup| |raised_fb| carry(IN, PT := T#1500MS);
|carry.Q| (R popup);
(* Belt A and the rollers hold while the chains are up *)
|/popup| |/hold| (belt_a_run);
|/popup| (roller_run);
