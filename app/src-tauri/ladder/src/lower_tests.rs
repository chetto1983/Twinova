use super::*;
use crate::parse::parse;
use crate::test_support::{blocks, lowered};

#[path = "lower_run_tests.rs"]
mod run;

#[path = "lower_oracle_tests.rs"]
mod oracle;

fn id(rung: usize, path: &[usize]) -> ElementId {
    ElementId {
        rung,
        path: path.to_vec(),
    }
}

fn errors(text: &str) -> Vec<(ElementId, String)> {
    let lowered = lower(&parse(text).unwrap(), "Ld", &blocks);
    lowered
        .errors
        .into_iter()
        .map(|e| (e.element, e.message))
        .collect()
}

#[test]
fn a_seal_in_lowers_to_a_program_with_a_power_bool_per_element() {
    let ld = "VAR Start, Stop, Motor : BOOL; END_VAR\n[|Start|, |Motor|] |/Stop| (Motor);";
    assert_eq!(
        lowered(ld).st,
        "PROGRAM Ld\nVAR Start, Stop, Motor : BOOL; END_VAR\nVAR\n    LD_P1 : BOOL;\n    LD_P2 : BOOL;\n    \
         LD_P3 : BOOL;\n    LD_P4 : BOOL;\nEND_VAR\nLD_P1 := Start;\nLD_P2 := Motor;\nLD_P3 := LD_P1 OR LD_P2;\n\
         LD_P4 := LD_P3 AND NOT Stop;\nMotor := LD_P4;\nEND_PROGRAM\n"
    );
}

#[test]
fn each_element_names_the_bool_holding_its_power_out() {
    let ld = "VAR Start, Stop, Motor : BOOL; END_VAR\n[|Start|, |Motor|] |/Stop| (Motor);";
    let power: Vec<(ElementId, String)> = lowered(ld)
        .power
        .into_iter()
        .map(|p| (p.element, p.variable))
        .collect();
    assert_eq!(
        power,
        [
            (id(0, &[0, 0, 0]), "LD_P1".into()),
            (id(0, &[0, 1, 0]), "LD_P2".into()),
            (id(0, &[0]), "LD_P3".into()),
            (id(0, &[1]), "LD_P4".into()),
            (id(0, &[2]), "LD_P4".into()),
        ]
    );
}

#[test]
fn the_rail_is_true_and_paths_ending_alike_need_no_or() {
    let st = lowered("(S X) [|A|, ] (Y);\n|A| [(X), (Y)] (Z);\nRETURN;").st;
    assert!(
        st.contains("\nX := TRUE;\nLD_P1 := A;\nY := TRUE;\n"),
        "{st}"
    );
    assert!(
        st.contains("\nLD_P2 := A;\nX := LD_P2;\nY := LD_P2;\nZ := LD_P2;\nRETURN;\n"),
        "{st}"
    );
    let power = lowered("[|A|, ] (Y);").power;
    assert_eq!(power[1].element, id(0, &[0]));
    assert_eq!(power[1].variable, "TRUE");
}

#[test]
fn edges_and_guards_read_as_written() {
    let st = lowered(
        "|A| |P B| |N C| (P X) (N Y) (R Z) T1(EN, PT := T#1s, ET => E) ADD(IN1 := N, IN2 := 1, OUT => N) RETURN;",
    )
    .st;
    let body = st.split("END_VAR\n").last().unwrap();
    assert_eq!(
        body,
        "LD_P1 := A;\nLD_P2 := LD_P1 AND B AND NOT LD_M1;\nLD_M1 := B;\nLD_P3 := LD_P2 AND NOT C AND LD_M2;\n\
         LD_M2 := C;\nX := LD_P3 AND NOT LD_M3;\nLD_M3 := LD_P3;\nY := NOT LD_P3 AND LD_M4;\nLD_M4 := LD_P3;\n\
         IF LD_P3 THEN Z := FALSE; END_IF;\nIF LD_P3 THEN T1(PT := T#1s, ET => E); END_IF;\n\
         IF LD_P3 THEN N := ADD(IN1 := N, IN2 := 1); END_IF;\nIF LD_P3 THEN RETURN; END_IF;\nEND_PROGRAM\n"
    );
}

#[test]
fn a_function_whose_result_goes_nowhere_is_left_out() {
    let st = lowered("|A| ADD(IN1 := N, IN2 := 1) (X);").st;
    assert!(
        st.ends_with("END_VAR\nLD_P1 := A;\nX := LD_P1;\nEND_PROGRAM\n"),
        "{st}"
    );
}

#[test]
fn hidden_names_avoid_every_name_the_program_writes() {
    let st = lowered("VAR ld_p1, LD_M1 : BOOL; END_VAR\n|P ld_p1| (LD_P3);").st;
    assert!(
        st.contains("    LD_P2 : BOOL;\n    LD_M2 : BOOL;\nEND_VAR\nLD_P2 := ld_p1 AND NOT LD_M2;"),
        "{st}"
    );
}

#[test]
fn a_jump_sets_a_flag_cleared_at_the_top_and_guards_the_rungs_it_skips() {
    let st = lowered("|A| >>Done;\n(X);\nDone: (Y);").st;
    let body = st.split("END_VAR\n").last().unwrap();
    assert_eq!(
        body,
        "LD_J1 := FALSE;\nLD_P1 := A;\nLD_J1 := LD_P1;\nIF NOT (LD_J1) THEN\nX := TRUE;\nEND_IF;\nY := TRUE;\n\
         END_PROGRAM\n"
    );
}

#[test]
fn a_blank_is_reported_on_its_element_and_its_rung_left_out() {
    let ld = "|A| (X);\n|A| [|B|, |?|] (Y);\n|A| T1(IN, PT := ?);\n|A| ?(IN);\n(?);\n|A| (Z);";
    assert_eq!(
        errors(ld),
        [
            (
                id(1, &[1, 1, 0]),
                "This contact has no variable yet: the ? on this rung".into()
            ),
            (
                id(2, &[1]),
                "T1's PT has no value yet: the ? on this rung".into()
            ),
            (
                id(3, &[1]),
                "This block has no name yet: the ? on this rung".into()
            ),
            (
                id(4, &[0]),
                "This coil has no variable yet: the ? on this rung".into()
            ),
        ]
    );
    let st = lower(&parse(ld).unwrap(), "Ld", &blocks).st;
    assert!(
        st.contains("X := LD_P1;\nLD_P") && st.contains("Z := LD_P"),
        "{st}"
    );
    assert!(!st.contains("Y :=") && !st.contains("T1("), "{st}");
}

#[test]
fn what_the_boxes_refuse() {
    let refused = |rung: &str| errors(rung).into_iter().map(|(_, m)| m).collect::<Vec<_>>();
    let rail = "starts a rung: nothing feeds it, so it stands at the left rail, first on the rung or first on a path \
                of a branch that is";
    for rung in [
        "|A| EQ(IN1 := N, IN2 := 3, OUT) (Q);",
        "[|A|, |B|] EQ(IN1 := N, IN2 := 3, OUT) (Q);",
        "|A| [EQ(IN1 := N, IN2 := 3, OUT), |B|] (Q);",
    ] {
        assert_eq!(refused(rung), [format!("EQ(..., OUT) {rail}")]);
    }
    let cases = [
        (
            "T1(EN := A);",
            "EN is the pin the rung feeds: write it bare, or leave it out",
        ),
        (
            "EQ(EN, IN1 := N, OUT) (Q);",
            "EQ starts the rung by OUT, so nothing feeds it and it has no EN",
        ),
        (
            "EQ(IN1 := N, OUT, OUT => A) (Q);",
            "EQ's OUT carries the rung, so it cannot also be written to A",
        ),
        (
            "|A| ADD(IN1, OUT => N);",
            "a function sits on the rung by EN and ENO, or starts it by OUT: 'IN1' needs a value (IN1 := ...)",
        ),
        (
            "|A| ADD(IN1 := N, Q => N);",
            "a function has one output, OUT: 'Q' is not it",
        ),
        (
            "|A| T1(IN, PT, Q);",
            "a box sits on the rung by one input and one output: 'T1' names IN, PT, Q",
        ),
        (
            "|A| Unknown(PT := T#1s);",
            "cannot tell which pins of 'Unknown' sit on the rung: name them bare, as in Unknown(IN, ..., Q)",
        ),
    ];
    for (rung, message) in cases {
        assert_eq!(refused(rung), [message.to_string()], "{rung}");
    }
}

#[test]
fn a_block_of_unknown_type_takes_its_bare_pins_as_input_then_output() {
    let st = lowered("|A| Unknown(Go, Speed := 3, Ready) (X);").st;
    assert!(
        st.contains("Unknown(Go := LD_P1, Speed := 3);\nLD_P2 := Unknown.Ready;\nX := LD_P2;"),
        "{st}"
    );
    let st = lowered("|A| T1(PT := T#1s) T1(EN, Q) (X);").st;
    assert!(st.contains("T1(IN := LD_P1, PT := T#1s);\nLD_P2 := T1.Q;\nIF LD_P2 THEN T1(); END_IF;\nLD_P3 := T1.Q;"), "{st}");
}

#[test]
fn what_the_labels_and_jumps_refuse() {
    let ld = "Top: |A| >>Top;\n|A| >>Nowhere;\nTop: (X);\n|A| >>Back;\nBack: (Y) >>Top;";
    assert_eq!(
        errors(ld),
        [
            (id(2, &[]), "label 'Top' is already on rung 1, so a jump to it could mean either".into()),
            (
                id(0, &[]),
                "a jump back up to 'Top' (rung 1) would run rungs again in one scan, a loop: a jump goes only down"
                    .into()
            ),
            (id(1, &[]), "there is no rung labelled 'Nowhere' to jump to".into()),
            (
                id(4, &[]),
                "a jump back up to 'Top' (rung 1) would run rungs again in one scan, a loop: a jump goes only down"
                    .into()
            ),
        ]
    );
}

#[test]
fn a_place_in_the_lowered_st_maps_back_to_its_element() {
    let ld =
        "VAR A, B : BOOL; END_VAR\n|A| >>Over;\n|B| ADD(IN1 := N, IN2 := 1, OUT => N);\nOver: (A);";
    let lowered = lowered(ld);
    let st = &lowered.st;
    let at = |text: &str| st.find(text).unwrap_or_else(|| panic!("{text} in {st}"));
    assert_eq!(lowered.origin(0), None, "PROGRAM");
    assert_eq!(
        lowered.origin(at("B : BOOL")),
        Some(Origin::Declarations(7))
    );
    assert_eq!(lowered.origin(at("    LD_P1 : BOOL")), None);
    assert_eq!(
        lowered.origin(at("LD_J1 := FALSE")),
        Some(Origin::Element(id(0, &[]), None))
    );
    assert_eq!(
        lowered.origin(at("A;\nLD_J1 :=")),
        Some(Origin::Element(id(0, &[0]), Some(Field::Operand)))
    );
    assert_eq!(
        lowered.origin(at("LD_J1 := LD_P1")),
        Some(Origin::Element(id(0, &[]), None))
    );
    assert_eq!(
        lowered.origin(at("IF NOT")),
        Some(Origin::Element(id(1, &[]), None))
    );
    assert_eq!(
        lowered.origin(at("N := ADD")),
        Some(Origin::Element(id(1, &[1]), Some(Field::Pin(2))))
    );
    assert_eq!(
        lowered.origin(at("ADD(")),
        Some(Origin::Element(id(1, &[1]), Some(Field::Operand)))
    );
    assert_eq!(
        lowered.origin(at("N, IN2")),
        Some(Origin::Element(id(1, &[1]), Some(Field::Pin(0))))
    );
    assert_eq!(
        lowered.origin(at("1);")),
        Some(Origin::Element(id(1, &[1]), Some(Field::Pin(1))))
    );
    assert_eq!(
        lowered.origin(at(" := ADD")),
        Some(Origin::Element(id(1, &[1]), None))
    );
    assert_eq!(
        lowered.origin(at("END_IF;\nEND_IF")),
        Some(Origin::Element(id(1, &[1]), None))
    );
    assert_eq!(lowered.origin(at("END_PROGRAM")), None);
}

#[test]
fn trusts_diagnostic_on_the_lowered_st_maps_back_to_the_contact() {
    let lowered = lowered("VAR Count : INT; Q : BOOL; END_VAR\n|Q| |Count| (Q);");
    let error = trust_runtime::harness::TestHarness::from_source(&lowered.st)
        .err()
        .map(|e| format!("{e:?}"))
        .expect("a contact on an INT does not compile");
    let (start, _) = error
        .rsplit_once("(at ")
        .and_then(|(_, range)| range.split_once(".."))
        .unwrap_or_else(|| panic!("{error}"));
    let start: usize = start.parse().unwrap();
    let origin = lowered
        .origin(start)
        .unwrap_or_else(|| panic!("{error} at {start} in {}", lowered.st));
    assert!(
        matches!(&origin, Origin::Element(e, _) if *e == id(0, &[1])),
        "{error}: {origin:?} in {}",
        lowered.st
    );
}
