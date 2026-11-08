use super::*;

#[test]
fn addresses_parse_and_print_back() {
    for text in [
        "%IX0.0", "%QX12.7", "%MX3.1", "%IB4", "%QW2", "%ID8", "%QL16",
    ] {
        assert_eq!(Address::parse(text).unwrap().to_string(), text);
    }
    let a = Address::parse("%QX12.7").unwrap();
    assert_eq!(
        (a.area, a.size, a.byte, a.bit),
        (Area::Output, Size::Bit, 12, 7)
    );
}

#[test]
fn malformed_addresses_are_refused() {
    for text in [
        "",
        "IX0.0",
        "%",
        "%I",
        "%ZX0.0",
        "%IY0",
        "%IX0",
        "%IX0.8",
        "%IX.0",
        "%IX0.",
        "%ID4.0",
        "%ID",
        "%IDx",
        "%IX+1.0",
        "%ID-4",
        "%ID+4",
        "%IB+1",
        "%IX0.0.0",
        "%ID99999999999",
    ] {
        assert!(Address::parse(text).is_err(), "{text} should be refused");
    }
}

#[test]
fn spans_cover_the_bits_of_each_size() {
    let span = |t: &str| Address::parse(t).unwrap().span();
    assert_eq!(span("%IX4.3"), (35, 36));
    assert_eq!(span("%IB4"), (32, 40));
    assert_eq!(span("%IW4"), (32, 48));
    assert_eq!(span("%ID4"), (32, 64));
    assert_eq!(span("%IL4"), (32, 96));
}

#[test]
fn overlap_needs_the_same_area_and_shared_bits() {
    let a = |t: &str| Address::parse(t).unwrap();
    assert!(a("%IX4.0").overlaps(a("%ID4")));
    assert!(a("%ID4").overlaps(a("%IX7.7")));
    assert!(a("%IW2").overlaps(a("%IB3")));
    assert!(
        !a("%ID4").overlaps(a("%IX8.0")),
        "the double word ends where byte 8 begins"
    );
    assert!(
        !a("%IX3.7").overlaps(a("%ID4")),
        "byte 3 ends where the double word begins"
    );
    assert!(
        !a("%IX4.0").overlaps(a("%QX4.0")),
        "inputs and outputs are separate areas"
    );
    assert!(!a("%IX4.0").overlaps(a("%IX4.1")));
}
