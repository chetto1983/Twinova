//! The lowered ST run in truST, scan by scan: OIP's cases (NOTICE), their expected behaviour IEC 61131-3's (8.2.3,
//! 8.2.4), derived by hand from the element definitions.

use trust_runtime::harness::TestHarness;
use trust_runtime::value::{Duration, Value};

use crate::lower::{Block, BlockPin, Lowered, lower};
use crate::parse::parse;

fn pins(list: &[(&str, bool)]) -> Vec<BlockPin> {
    list.iter()
        .map(|&(name, bool)| BlockPin {
            name: name.into(),
            bool,
        })
        .collect()
}

/// The fixtures' declarations, as the project's would resolve them.
pub(super) fn blocks(head: &str) -> Option<Block> {
    let instance = |inputs: &[(&str, bool)], outputs: &[(&str, bool)]| Block::Instance {
        inputs: pins(inputs),
        outputs: pins(outputs),
    };
    match head.to_ascii_uppercase().as_str() {
        "T1" => Some(instance(
            &[("IN", true), ("PT", false)],
            &[("Q", true), ("ET", false)],
        )),
        "C1" => Some(instance(
            &[("CU", true), ("R", true), ("PV", false)],
            &[("Q", true), ("CV", false)],
        )),
        "R1" => Some(instance(&[("CLK", true)], &[("Q", true)])),
        "L" => Some(instance(&[("Set", true), ("Clear", true)], &[("On", true)])),
        "ADD" | "GT" | "EQ" | "LT" | "MOVE" | "INRANGE" => Some(Block::Function),
        _ => None,
    }
}

pub(super) fn lowered(text: &str) -> Lowered {
    let lowered = lower(&parse(text).unwrap(), "Ld", &blocks);
    assert_eq!(lowered.errors, [], "{}", lowered.st);
    lowered
}

/// Run `sources` one 10 ms scan per step, each step setting inputs (`Start` TRUE, `!Start` FALSE, `N=3` an INT), and
/// give `watch` after each scan, BOOLs as 1 and 0: `Motor=1 Count=2`.
pub(super) fn trace(sources: &[&str], steps: &[&str], watch: &[&str]) -> Vec<String> {
    let mut harness = TestHarness::from_sources(sources)
        .unwrap_or_else(|e| panic!("{e:?}\n{}", sources.join("\n")));
    steps
        .iter()
        .map(|step| {
            for input in step.split_whitespace() {
                match input.split_once('=') {
                    Some((name, n)) => harness.set_input(name, Value::Int(n.parse().unwrap())),
                    None => match input.strip_prefix('!') {
                        Some(name) => harness.set_input(name, false),
                        None => harness.set_input(input, true),
                    },
                }
            }
            harness.cycle();
            harness.advance_time(Duration::from_millis(10));
            let values: Vec<String> = watch
                .iter()
                .map(|name| match harness.get_output(name) {
                    Some(Value::Bool(b)) => format!("{name}={}", u8::from(b)),
                    Some(Value::Int(n)) => format!("{name}={n}"),
                    other => format!("{name}={other:?}"),
                })
                .collect();
            values.join(" ")
        })
        .collect()
}

fn run(ld: &str, steps: &[&str], watch: &[&str]) -> Vec<String> {
    trace(&[&lowered(ld).st], steps, watch)
}

const BOOLS: &str = "VAR A, B, C, X, Y, Z, Q : BOOL; END_VAR\n";

fn bools(rungs: &str) -> String {
    format!("{BOOLS}{rungs}")
}

#[test]
fn a_seal_in_latches_through_its_own_contact_and_stop_drops_it() {
    let ld = "VAR Start, Stop, Motor : BOOL; END_VAR\n[|Start|, |Motor|] |/Stop| (Motor);";
    let t = run(ld, &["Start", "!Start", "", "Stop", "!Stop"], &["Motor"]);
    assert_eq!(t, ["Motor=1", "Motor=1", "Motor=1", "Motor=0", "Motor=0"]);
}

#[test]
fn a_coil_passes_its_power_on_and_an_empty_rung_does_nothing() {
    let t = run(&bools("|A| (B) (C);\n;"), &["A", "!A"], &["B", "C"]);
    assert_eq!(t, ["B=1 C=1", "B=0 C=0"]);
}

#[test]
fn a_contact_after_a_coil_reads_what_the_coil_wrote() {
    let t = run(&bools("|A| (B) |B| (C);"), &["A", "!A"], &["C"]);
    assert_eq!(t, ["C=1", "C=0"]);
}

#[test]
fn what_flows_on_is_what_the_contact_examined_though_a_coil_rewrites_it() {
    let t = run(&bools("|A| (/A) (B);"), &["A"], &["A", "B"]);
    assert_eq!(t, ["A=0 B=1"]);
}

#[test]
fn a_branch_runs_its_contact_paths_first_then_its_coil_paths_and_ors_them() {
    let t = run(
        &bools("|A| [(X), |X| (Y), |/A|] (Z);"),
        &["A", "A", "!A"],
        &["X", "Y", "Z"],
    );
    assert_eq!(t, ["X=1 Y=0 Z=1", "X=1 Y=1 Z=1", "X=0 Y=0 Z=0"]);
}

#[test]
fn paths_holding_contacts_run_top_to_bottom() {
    let t = run(&bools("|A| [|A| (X), |X| (Y)];"), &["A", "!A"], &["X", "Y"]);
    assert_eq!(t, ["X=1 Y=1", "X=0 Y=0"]);
}

#[test]
fn an_empty_path_is_a_wire() {
    let t = run(&bools("|A| [|B|, ] (Q);"), &["A", "!A B"], &["Q"]);
    assert_eq!(t, ["Q=1", "Q=0"]);
}

#[test]
fn set_and_reset_coils_hold_between_the_rungs_that_power_them() {
    let ld = bools("|A| (S Q);\n|B| (R Q);");
    let t = run(&ld, &["A", "!A", "B", "!B"], &["Q"]);
    assert_eq!(t, ["Q=1", "Q=1", "Q=0", "Q=0"]);
}

#[test]
fn a_negated_coil_writes_the_inverse_and_passes_the_power_itself_on() {
    let t = run(&bools("|A| (/X) (Y);"), &["A", "!A"], &["X", "Y"]);
    assert_eq!(t, ["X=0 Y=1", "X=1 Y=0"]);
}

#[test]
fn a_p_contact_pulses_once_per_rise_and_remembers_whatever_its_power() {
    let t = run(
        &bools("|A| |P B| (Q);"),
        &["A !B", "B", "", "!B", "!A B", "A", "!B", "B"],
        &["Q"],
    );
    let pulses: Vec<&str> = t.iter().map(|s| &s[2..]).collect();
    assert_eq!(pulses, ["0", "1", "0", "0", "0", "0", "0", "1"]);
}

#[test]
fn an_n_contact_pulses_once_per_fall() {
    let t = run(&bools("|N B| (Q);"), &["B", "", "!B", ""], &["Q"]);
    assert_eq!(t, ["Q=0", "Q=0", "Q=1", "Q=0"]);
}

#[test]
fn p_and_n_coils_pulse_on_their_power_changing_and_pass_it_on() {
    let t = run(
        &bools("|A| (P X) (N Y) (Z);"),
        &["!A", "A", "", "!A", ""],
        &["X", "Y", "Z"],
    );
    assert_eq!(
        t,
        [
            "X=0 Y=0 Z=0",
            "X=1 Y=0 Z=1",
            "X=0 Y=0 Z=1",
            "X=0 Y=1 Z=0",
            "X=0 Y=0 Z=0"
        ]
    );
}

#[test]
fn a_timer_on_the_rung_is_fed_on_in_and_carries_q_on() {
    let ld = "VAR A, Late : BOOL; T1 : TON; END_VAR\n|A| T1(IN, PT := T#30ms) (Late);";
    let t = run(ld, &["A", "", "", "", "!A"], &["Late"]);
    // ET counts from the scan IN rose on, so Q is on at the fourth 10 ms scan
    assert_eq!(t, ["Late=0", "Late=0", "Late=0", "Late=1", "Late=0"]);
}

#[test]
fn another_pin_named_bare_goes_on_the_rung() {
    // INT#2: truST at the pinned runtime refuses an untyped literal on CTU's PV, in plain ST too (measured, 2026-11-13)
    let ld = "VAR Clear, Pulse, Full : BOOL; C1 : CTU; END_VAR\n|Clear| C1(R, CU := Pulse, PV := INT#2, Q);\n|C1.Q| (Full);";
    let t = run(ld, &["Pulse", "!Pulse", "Pulse", "Clear"], &["Full"]);
    assert_eq!(t, ["Full=0", "Full=0", "Full=1", "Full=0"]);
}

#[test]
fn a_block_behind_en_runs_only_while_powered_and_eno_passes_the_power_on() {
    let ld =
        "VAR Go, Done : BOOL; T1 : TON; END_VAR\n|Go| T1(EN, IN := TRUE, PT := T#20ms) (Done);";
    let t = run(ld, &["Go", "", "!Go", "Go"], &["Done", "T1.Q"]);
    assert_eq!(
        t,
        [
            "Done=1 T1.Q=0",
            "Done=1 T1.Q=0",
            "Done=0 T1.Q=0",
            "Done=1 T1.Q=1"
        ]
    );
}

#[test]
fn a_function_behind_en_writes_its_result_where_out_says() {
    let ld = "VAR Go, Seen : BOOL; Count : INT; END_VAR\n|Go| ADD(IN1 := Count, IN2 := 1, OUT => Count) (Seen);";
    let t = run(ld, &["Go", "", "!Go", "Go"], &["Count", "Seen"]);
    assert_eq!(
        t,
        [
            "Count=1 Seen=1",
            "Count=2 Seen=1",
            "Count=2 Seen=0",
            "Count=3 Seen=1"
        ]
    );
}

#[test]
fn a_user_function_block_with_its_own_pins() {
    let latch = "FUNCTION_BLOCK Latch\nVAR_INPUT Set : BOOL; Clear : BOOL; END_VAR\nVAR_OUTPUT On : BOOL; END_VAR\n\
                 IF Set THEN On := TRUE; END_IF;\nIF Clear THEN On := FALSE; END_IF;\nEND_FUNCTION_BLOCK\n";
    let ld = lowered("VAR A, B, Lamp : BOOL; L : Latch; END_VAR\n|A| L(Set, Clear := B) (Lamp);");
    let t = trace(&[latch, &ld.st], &["A", "!A", "B"], &["Lamp"]);
    assert_eq!(t, ["Lamp=1", "Lamp=1", "Lamp=0"]);
}

const NUMBERS: &str = "VAR A, B, C, Q : BOOL; N : INT; END_VAR\n";

#[test]
fn a_function_starting_the_rung_is_the_power_and_contacts_follow_in_series() {
    let ld = format!("{NUMBERS}EQ(IN1 := N, IN2 := 3, OUT) |A| (Q);");
    let t = run(&ld, &["N=3 A", "N=4", "N=3 !A", "A"], &["Q"]);
    assert_eq!(t, ["Q=1", "Q=0", "Q=0", "Q=1"]);
}

#[test]
fn what_a_starting_function_examined_flows_on_though_a_later_box_writes_its_operand() {
    let ld = format!(
        "{NUMBERS}EQ(IN1 := N, IN2 := 0, OUT) |A| MOVE(IN := 10, OUT => N) (Q);\n\
         EQ(IN1 := N, IN2 := 10, OUT) |B| MOVE(IN := 20, OUT => N) (C);"
    );
    let t = run(&ld, &["A B", "!A !B"], &["N", "Q", "C"]);
    assert_eq!(t, ["N=20 Q=1 C=1", "N=20 Q=0 C=0"]);
}

#[test]
fn a_starting_function_may_open_any_path_of_a_branch_at_the_rail() {
    let ld = format!(
        "{NUMBERS}[GT(IN1 := N, IN2 := 5, OUT), |B|] (Q);\n[[LT(IN1 := N, IN2 := 2, OUT), |C|] |A|, |B|] (C);"
    );
    let t = run(&ld, &["N=6", "N=3", "B", "!B N=1 A"], &["Q", "C"]);
    assert_eq!(t, ["Q=1 C=0", "Q=0 C=0", "Q=1 C=1", "Q=0 C=1"]);
}

#[test]
fn a_user_function_returning_bool_starts_a_rung() {
    let in_range = "FUNCTION InRange : BOOL\nVAR_INPUT X, Lo, Hi : INT; END_VAR\n\
                    InRange := (X >= Lo) AND (X <= Hi);\nEND_FUNCTION\n";
    let ld = lowered(&format!(
        "{NUMBERS}InRange(X := N, Lo := 1, Hi := 5, OUT) (Q);"
    ));
    let t = trace(&[in_range, &ld.st], &["N=3", "N=7"], &["Q"]);
    assert_eq!(t, ["Q=1", "Q=0"]);
}

#[test]
fn and_or_xor_mod_and_not_run_as_the_operators_st_spells_them() {
    let ld = "VAR A, B, C, X, Y, Z, Q : BOOL; N, M : INT; END_VAR\n\
         AND(IN2 := B, IN1 := A, IN3 := C, OUT) (X);\nXOR(IN1 := A, IN2 := B, OUT) (Y);\n\
         |A| MOD(IN2 := 3, IN1 := N, OUT => M);\n[NOT(IN := A, OUT), OR(IN1 := B, IN2 := C, OUT)] (Z);";
    let t = run(ld, &["A B C N=7", "!B !C", "!A B"], &["X", "Y", "M", "Z"]);
    assert_eq!(t, ["X=1 Y=0 M=1 Z=1", "X=0 Y=1 M=1 Z=0", "X=0 Y=1 M=1 Z=1"]);
}

#[test]
fn a_jump_skips_the_rungs_down_to_its_label_whose_rung_runs() {
    let ld = bools("|B| >>Done;\n|A| (X);\nDone: |A| (Y);\n|A| (Z);");
    let t = run(&ld, &["A", "B !A", "!B"], &["X", "Y", "Z"]);
    assert_eq!(t, ["X=1 Y=1 Z=1", "X=1 Y=0 Z=0", "X=0 Y=0 Z=0"]);
}

#[test]
fn jumps_over_one_another_each_end_at_their_own_label() {
    let ld = bools("|A| >>One;\n|B| >>Two;\n(X);\nOne: (Y);\nTwo: (Z);");
    let t = run(&ld, &["A", "!A B"], &["X", "Y", "Z"]);
    assert_eq!(t, ["X=0 Y=1 Z=1", "X=0 Y=1 Z=1"]);
}

#[test]
fn return_ends_the_scan_while_powered() {
    let ld = bools("|B| RETURN;\n|A| (X);\n;\nRETURN;\n(Y);");
    let t = run(&ld, &["A", "B !A", "!B"], &["X", "Y"]);
    assert_eq!(t, ["X=1 Y=0", "X=1 Y=0", "X=0 Y=0"]);
}

/// The ST twin of the program below, written by hand in another way (an R_TRIG for the edge, an IF for the count):
/// the two run alike over random inputs, scan by scan (the plan's Global Constraints).
const TWIN: &str = "PROGRAM Twin\nVAR Start, Stop, Motor, Btn, Pulse, Late : BOOL; T1 : TON; R1 : R_TRIG; Count : INT; \
                    END_VAR\nMotor := (Start OR Motor) AND NOT Stop;\nR1(CLK := Btn);\nPulse := Motor AND R1.Q;\n\
                    IF Pulse THEN Count := Count + 1; END_IF;\nT1(IN := Motor, PT := T#20ms);\nLate := T1.Q;\nEND_PROGRAM\n";
const LADDER: &str = "VAR Start, Stop, Motor, Btn, Pulse, Late : BOOL; T1 : TON; Count : INT; END_VAR\n\
                      [|Start|, |Motor|] |/Stop| (Motor);\n|Motor| |P Btn| ADD(IN1 := Count, IN2 := 1, OUT => Count) \
                      (Pulse);\n|Motor| T1(IN, PT := T#20ms) (Late);";

proptest::proptest! {
    #[test]
    fn the_ladder_runs_as_its_hand_written_twin(inputs in proptest::collection::vec(0u8..8, 1..120)) {
        let steps: Vec<String> = inputs
            .iter()
            .map(|bits| {
                let flag = |bit: u8, name: &str| if bits & bit != 0 { name.to_string() } else { format!("!{name}") };
                [flag(1, "Start"), flag(2, "Stop"), flag(4, "Btn")].join(" ")
            })
            .collect();
        let steps: Vec<&str> = steps.iter().map(String::as_str).collect();
        let watch = ["Motor", "Pulse", "Late", "Count"];
        proptest::prop_assert_eq!(run(LADDER, &steps, &watch), trace(&[TWIN], &steps, &watch));
    }
}
