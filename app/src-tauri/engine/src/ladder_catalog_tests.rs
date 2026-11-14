use super::*;

const MAIN: &str = "PROGRAM Main\nVAR x : INT; END_VAR\nEND_PROGRAM\n\
                    FUNCTION_BLOCK Valve\nVAR_INPUT Open : BOOL; END_VAR\nVAR_OUTPUT Opened : BOOL; Travel : INT; END_VAR\nEND_FUNCTION_BLOCK\n\
                    FUNCTION Twice : INT\nVAR_INPUT N : INT; END_VAR\nVAR_OUTPUT Odd : BOOL; END_VAR\nTwice := N * 2;\nEND_FUNCTION\n";

fn program(ld: &str) -> Vec<(String, String)> {
    vec![
        ("plc/Main.st".into(), MAIN.into()),
        ("plc/Pump.ld".into(), ld.into()),
    ]
}

fn pins(list: &[CatalogPin]) -> Vec<String> {
    list.iter()
        .map(|p| format!("{}:{}:{}", p.name, p.type_name, p.bool))
        .collect()
}

#[test]
fn a_box_may_be_a_standard_or_own_block_an_own_function_or_a_standard_one() {
    let found = catalog(
        &program("VAR T1 : TON; END_VAR\n|T1.Q| (T1.Q);\n"),
        &["ADD", "LIMIT", "NOPE"],
    )
    .unwrap();
    let get = |name: &str| {
        found
            .iter()
            .find(|e| e.name == name)
            .unwrap_or_else(|| panic!("{name}"))
    };
    let ton = get("TON");
    assert_eq!((ton.block, ton.own, ton.variadic), (true, false, false));
    assert_eq!(pins(&ton.inputs), ["IN:BOOL:true", "PT:TIME:false"]);
    assert_eq!(pins(&ton.outputs), ["Q:BOOL:true", "ET:TIME:false"]);
    let valve = get("Valve");
    assert_eq!((valve.block, valve.own), (true, true));
    assert_eq!(
        pins(&valve.outputs),
        ["Opened:BOOL:true", "Travel:INT:false"]
    );
    let twice = get("Twice");
    assert_eq!((twice.block, twice.own), (false, true));
    assert_eq!(pins(&twice.inputs), ["N:INT:false"]);
    assert_eq!(pins(&twice.outputs), ["OUT:INT:false", "Odd:BOOL:true"]);
    let add = get("ADD");
    assert_eq!((add.block, add.own, add.variadic), (false, false, true));
    assert_eq!(pins(&add.inputs), ["IN1::false", "IN2::false"]);
    assert_eq!(pins(&add.outputs), ["OUT::false"]);
    let limit = get("LIMIT");
    assert_eq!(
        (limit.variadic, pins(&limit.inputs)),
        (
            false,
            vec![
                "MN::false".to_string(),
                "IN::false".into(),
                "MX::false".into()
            ]
        )
    );
    assert!(
        found.iter().all(|e| e.name != "NOPE"),
        "a name the runtime has not is left out"
    );
    // the blocks first, then the project's functions, then the standard ones
    let order: Vec<&str> = found
        .iter()
        .map(|e| e.name.as_str())
        .filter(|n| ["TON", "Valve", "Twice", "ADD"].contains(n))
        .collect();
    assert_eq!(order, ["TON", "Valve", "Twice", "ADD"]);
}

#[test]
fn a_program_that_does_not_read_or_compile_has_no_catalogue() {
    assert_eq!(
        catalog(&program("|A| (B)"), &[]).unwrap_err(),
        "plc/Pump.ld:1: expected ';' to end the rung"
    );
    assert_eq!(
        catalog(&program("VAR T1 : Nowhere; END_VAR\n"), &[]).unwrap_err(),
        "the program does not compile, so its blocks are not known"
    );
}
