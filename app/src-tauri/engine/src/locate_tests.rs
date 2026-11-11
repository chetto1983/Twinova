use super::*;

#[test]
fn a_span_becomes_what_its_start_maps_to_and_look_alikes_stay() {
    let at = |start: usize| format!("<{start}>");
    assert_eq!(
        rewrite_spans("e (at 2..3) f at 9..10 g", at),
        "e (at <2>) f at <9> g"
    );
    for unchanged in ["at", "at x..1", "at 1..", "at 1.2", "no span"] {
        assert_eq!(rewrite_spans(unchanged, at), unchanged);
    }
}

#[test]
fn a_line_counts_from_one_and_clamps_past_the_end() {
    assert_eq!(
        [0, 2, 3, 6, 99].map(|o| line_of("ab\ncd\n", o)),
        [1, 1, 2, 3, 3]
    );
}

#[test]
fn a_labelled_message_names_each_file_and_line() {
    let files = [
        ("plc/A.st".to_string(), "a\nbb\n".to_string()),
        ("plc/B.st".to_string(), "x\ny\nz\n".to_string()),
    ];
    let message = "plc/B.st: bad at 4..5\nplc/A.st: odd at 0..1\nplc/A.st: no span\nfile 3: elsewhere at 1..2\nplain";
    assert_eq!(
        locate_in_files(message, &files),
        "bad at plc/B.st:3\nodd at plc/A.st:1\nplc/A.st: no span\nfile 3: elsewhere at 1..2\nplain"
    );
}
