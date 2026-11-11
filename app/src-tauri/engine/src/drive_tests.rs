use super::*;

/// OIP's VFD scale (4.4b acceptance 3): 30 at `speed_at_30`, in proportion below it, 0 stopped, signed in reverse.
#[test]
fn the_velocity_reads_thirty_at_speed_at_30() {
    let drive = Feedback::new(0.5, 0.0254);
    assert_eq!(drive.velocity(0.5), 30.0);
    assert_eq!(drive.velocity(0.0), 0.0);
    assert_eq!(drive.velocity(-0.25), -15.0);
    assert_eq!(Feedback::new(1.0, 0.0254).velocity(0.5), 15.0);
    assert_eq!(
        Feedback::new(0.0, 0.0254).velocity(0.5),
        0.0,
        "no scale, no reading"
    );
}

/// One pulse per pitch of travel, for one millisecond, whichever way the surface runs.
#[test]
fn the_pulse_comes_once_per_pitch_of_travel_either_way() {
    for direction in [1.0_f32, -1.0] {
        let mut drive = Feedback::new(0.5, 0.0254);
        let (mut travel, mut pulses) = (0.0_f64, 0);
        for ms in 0..10_000 {
            // A 400 ms ramp, then the rated speed.
            let speed = direction * 0.5 * (ms as f32 / 400.0).min(1.0);
            travel += f64::from(speed.abs()) * 0.001;
            pulses += usize::from(drive.pulse(speed));
        }
        assert_eq!(
            pulses,
            (travel / 0.0254).floor() as usize,
            "{direction}: {travel} m"
        );
    }
}

/// At 0.5 m/s a 0.5 mm pitch is exactly one millisecond's travel: a pulse every millisecond, none lost to rounding
/// (OIP: "twelve twelfths of an inch do not sum to exactly one").
#[test]
fn a_pitch_of_one_milliseconds_travel_pulses_every_millisecond() {
    let mut drive = Feedback::new(0.5, 0.0005);
    assert!((0..1000).all(|_| drive.pulse(0.5)));
}

/// A millisecond that covers several pitches gives one pulse and keeps only what is left of the last pitch.
#[test]
fn a_fast_surface_pulses_once_a_millisecond_and_keeps_the_remainder() {
    let mut drive = Feedback::new(0.5, 0.001);
    // 2.5 mm a millisecond over a 1 mm pitch: a pulse now, 0.5 mm kept; then 3 mm reached, nothing kept.
    assert!(drive.pulse(2.5));
    assert!(
        (drive.since_pulse - 0.0005).abs() < 1e-9,
        "{}",
        drive.since_pulse
    );
    assert!(drive.pulse(2.5));
    assert!(drive.since_pulse.abs() < 1e-9, "{}", drive.since_pulse);
    assert!(!drive.pulse(0.0));
}

#[test]
fn a_drive_without_a_pitch_never_pulses() {
    let mut drive = Feedback::new(0.5, 0.0);
    assert!((0..1000).all(|_| !drive.pulse(0.5)));
}

#[test]
fn the_feedback_takes_its_scale_and_pitch_from_the_parameters() {
    use std::path::Path;
    use twinova_project::model::ParamValue;
    use twinova_project::storage::open_folder;
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/demo_m3");
    let mut project = open_folder(&dir).unwrap().opened.project;
    assert_eq!(Feedback::of(&project, "belt_a"), Feedback::new(0.5, 0.0254));
    let belt = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "belt_a")
        .unwrap();
    belt.parameters
        .insert("speed_at_30".into(), ParamValue::Real(1.5));
    belt.parameters
        .insert("pulse_pitch".into(), ParamValue::Real(0.1));
    assert_eq!(Feedback::of(&project, "belt_a"), Feedback::new(1.5, 0.1));
    // A family without the drive's parameters (the sink) has no scale and no pitch.
    assert_eq!(Feedback::of(&project, "sink"), Feedback::new(0.0, 0.0));
}
