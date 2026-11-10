use super::*;
use crate::field::behaving;
use std::path::Path;
use twinova_project::Project;
use twinova_project::storage::open_folder;

fn fixture(name: &str) -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("project/tests/fixtures/valid")
        .join(name);
    open_folder(&dir).unwrap().opened.project
}

/// Review focus: a private address never collides with a bound one, nor with another private one. The curve line
/// binds curve A's velocity and encoder in the middle of the other devices' signals.
#[test]
fn private_addresses_lie_past_every_bound_byte_and_never_overlap() {
    let mut project = fixture("curve_line");
    let belt = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "belt_c")
        .unwrap();
    belt.parameters
        .insert("speed".into(), ParamValue::Real(0.75));
    let devices = behaving(&project);
    let placed = place(&devices).unwrap();
    let signals: usize = devices.iter().map(|(_, d)| d.signals.len()).sum();
    assert_eq!(
        placed.len(),
        signals,
        "every signal of every device is configured"
    );
    for (i, a) in placed.iter().enumerate() {
        for b in &placed[i + 1..] {
            assert!(
                !a.address.overlaps(b.address),
                "{} and {}",
                a.address,
                b.address
            );
        }
    }
    let bound_end = |area: Area| {
        placed
            .iter()
            .filter(|p| p.instance.signals.contains_key(&p.signal.name) && p.address.area == area)
            .map(|p| p.address.span().1)
            .max()
            .unwrap()
    };
    for p in placed
        .iter()
        .filter(|p| !p.instance.signals.contains_key(&p.signal.name))
    {
        assert!(p.signal.optional, "{}.{}", p.instance.id, p.signal.name);
        assert!(
            p.address.span().0 >= bound_end(p.address.area),
            "{}",
            p.address
        );
        let fallback = match p.signal.name.as_str() {
            "speed_ref" if p.instance.id == "belt_c" => Some(Value::DWord(0.75_f32.to_bits())),
            "speed_ref" => Some(Value::DWord(0.5_f32.to_bits())),
            _ => None,
        };
        assert_eq!(p.fallback, fallback, "{}.{}", p.instance.id, p.signal.name);
    }
    let velocity = placed
        .iter()
        .find(|p| p.instance.id == "curve_a" && p.signal.name == "velocity")
        .unwrap();
    assert_eq!(
        velocity.address.to_string(),
        "%QD8",
        "a bound signal at its mirror"
    );
    // Each area keeps its own count: the controller's outputs end at byte 1 of the field's inputs, its inputs at byte
    // 16 of the field's outputs (%ID12), so the first private input is %ID1 and the first private output byte 16.
    let first = |area: Area| {
        placed
            .iter()
            .find(|p| p.address.area == area && !p.instance.signals.contains_key(&p.signal.name))
            .map(|p| p.address.to_string())
            .unwrap()
    };
    assert_eq!(first(Area::Input), "%ID1");
    assert_eq!(first(Area::Output), "%QD16");
}

#[test]
fn a_signal_reads_from_the_area_its_direction_gives() {
    assert_eq!(area(Direction::PlcOutput), Area::Input);
    assert_eq!(area(Direction::PlcInput), Area::Output);
}
