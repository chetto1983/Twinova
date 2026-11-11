use super::*;
use crate::field::behaving;
use std::path::Path;
use twinova_project::Project;
use twinova_project::storage::open_folder;

fn fixture(name: &str) -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../project/tests/fixtures/valid")
        .join(name);
    open_folder(&dir).unwrap().opened.project
}

/// Review focus: a private address never collides with a bound one, nor with another private one; and a signal the
/// native model writes (the curve line binds curve A's velocity and encoder) is not the field's at all.
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
    // Five conveyors, each with run, running and speed_ref; their velocity, encoder and pulse are native.
    let names: Vec<String> = placed
        .iter()
        .map(|p| format!("{}.{}", p.instance.id, p.signal.name))
        .collect();
    assert_eq!(placed.len(), 5 * 3, "{names:?}");
    for native in ["velocity", "encoder", "pulse"] {
        assert!(placed.iter().all(|p| p.signal.name != native), "{native}");
    }
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
    let bound = |p: &&Placed| p.instance.signals.contains_key(&p.signal.name);
    let bound_end = placed
        .iter()
        .filter(bound)
        .filter(|p| p.address.area == Area::Input)
        .map(|p| p.address.span().1)
        .max()
        .unwrap();
    let private: Vec<&Placed> = placed.iter().filter(|p| !bound(p)).collect();
    assert_eq!(private.len(), 5, "each conveyor's speed_ref");
    for p in &private {
        assert_eq!(
            (p.signal.name.as_str(), p.signal.optional, p.address.area),
            ("speed_ref", true, Area::Input)
        );
        assert!(p.address.span().0 >= bound_end, "{}", p.address);
        let speed: f32 = if p.instance.id == "belt_c" { 0.75 } else { 0.5 };
        assert_eq!(
            p.fallback,
            Some(Value::DWord(speed.to_bits())),
            "{}",
            p.instance.id
        );
    }
    // The controller's outputs end at byte 1 of the field's inputs: the first private input is %ID1.
    assert_eq!(private[0].address.to_string(), "%ID1");
    let running = placed
        .iter()
        .find(|p| p.instance.id == "curve_a" && p.signal.name == "running")
        .unwrap();
    assert_eq!(
        running.address.to_string(),
        "%QX0.2",
        "a bound signal at its mirror"
    );
}

/// An unbound optional output of a block gets a private address in the field's output area, past its bound outputs:
/// the chain transfer's raised_fb, made optional and left unbound.
#[test]
fn a_private_output_lies_past_the_bound_outputs() {
    let mut project = fixture("two_belts_diverter");
    let def = project
        .definitions
        .get_mut("twinova.chain_transfer@1")
        .unwrap();
    def.signals
        .iter_mut()
        .find(|s| s.name == "raised_fb")
        .unwrap()
        .optional = true;
    let transfer = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "chain_transfer")
        .unwrap();
    transfer.signals.remove("raised_fb");
    let devices = behaving(&project);
    let placed = place(&devices).unwrap();
    let raised = placed
        .iter()
        .find(|p| p.signal.name == "raised_fb")
        .unwrap();
    // The controller's inputs the blocks write end with roller_r.running at %IX0.5: byte 1 is free.
    assert_eq!(
        (raised.address.to_string(), &raised.fallback),
        ("%QX1.0".to_string(), &None)
    );
}

#[test]
fn a_signal_reads_from_the_area_its_direction_gives() {
    assert_eq!(area(Direction::PlcOutput), Area::Input);
    assert_eq!(area(Direction::PlcInput), Area::Output);
}
