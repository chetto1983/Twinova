use super::*;
use crate::program_files::ProgramFiles;

fn ladder_line() -> Place {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../project/tests/fixtures/valid/ladder_line/plc/Main.ld"
    );
    let text = std::fs::read_to_string(path).unwrap();
    let files = [("plc/Main.ld".to_string(), text)];
    ProgramFiles::of(&files).unwrap().places().remove(0)
}

#[test]
fn a_ladder_file_maps_each_rung_s_lines_onto_its_generated_ones_and_back() {
    let place = ladder_line();
    assert!(place.steps_by_rung());
    let rungs = place.rungs.clone().unwrap();
    assert_eq!(rungs.len(), 13);
    // rung 1: its comment on line 26, itself on 27, lowered to line 52 (row 51)
    assert_eq!(
        rungs[0],
        Rung {
            from: 26,
            at: 27,
            to: 27,
            rows: (51, 51)
        }
    );
    assert_eq!((place.row_of(26), place.row_of(27)), (Some(51), Some(51)));
    assert_eq!(place.line_at(51), Some(27));
    // rung 3, its four generated lines 56 to 59, shows on its line 30
    assert_eq!(rungs[2].rows, (55, 58));
    assert_eq!(
        (55..=58).map(|row| place.line_at(row)).collect::<Vec<_>>(),
        [Some(30); 4]
    );
    // the declarations take none, and the PROGRAM's own lines show nowhere
    assert_eq!((place.row_of(1), place.row_of(25)), (None, None));
    assert_eq!(
        (place.line_at(0), place.line_at(50), place.line_at(93)),
        (None, None, None)
    );
    // every generated line of a rung shows on its rung, and every line of a rung reaches its first
    for rung in &rungs {
        for line in rung.from..=rung.to {
            assert_eq!(place.row_of(line), Some(rung.rows.0));
        }
        for row in rung.rows.0..=rung.rows.1 {
            assert_eq!(place.line_at(row), Some(rung.at));
        }
    }
    assert!(
        rungs
            .windows(2)
            .all(|w| w[0].to < w[1].from && w[0].rows.1 < w[1].rows.0)
    );
}

#[test]
fn an_st_file_maps_its_lines_one_for_one() {
    let place = Place::whole("plc/Main.st", 2, "a\nb\nc");
    assert!(!place.steps_by_rung());
    assert_eq!(
        (place.row_of(1), place.row_of(3), place.row_of(0)),
        (Some(0), Some(2), None)
    );
    assert_eq!(
        (place.line_at(0), place.line_at(2), place.line_at(3)),
        (Some(1), Some(3), None)
    );
    let block = Place {
        first_line: 10,
        ..place
    };
    assert_eq!(
        (block.row_of(1), block.line_at(10), block.line_at(9)),
        (Some(10), Some(1), None)
    );
}
