//! The Structured Text controller of the scene: truST's harness at the pinned commit, driven one scan at a time
//! with virtual time (spec 2026-11-07-tauri-shell-design.md, "The PLC binding").

use serde::Serialize;
use trust_runtime::harness::TestHarness;
use trust_runtime::value::{Duration, Value};

pub use crate::controller::DEFAULT_SCAN_NS;
use crate::controller::scan_period;

/// The variables of `PROGRAM Main` that the scene and the UI exchange with the program.
pub const CONTRACT: [&str; 4] = ["sensor", "start", "stop", "motor"];

#[derive(Debug, Clone, Copy, Default, Serialize, PartialEq, Eq)]
pub struct Inputs {
    pub sensor: bool,
    pub start: bool,
    pub stop: bool,
}

pub struct Plc {
    harness: TestHarness,
    scan_ns: i64,
}

impl Plc {
    /// Compiles `source`, then checks that it declares every contract variable as a BOOL.
    pub fn build(source: &str) -> Result<Self, String> {
        let harness = TestHarness::from_source(source).map_err(|e| e.to_string())?;
        let missing: Vec<&str> = CONTRACT
            .iter()
            .copied()
            .filter(|name| !matches!(harness.get_output(name), Some(Value::Bool(_))))
            .collect();
        if missing.is_empty() {
            let scan_ns = scan_period(&harness)?;
            Ok(Self { harness, scan_ns })
        } else {
            Err(format!(
                "PROGRAM Main must declare these BOOL variables: {}",
                missing.join(", ")
            ))
        }
    }

    /// The period at which the program is scanned (ROADMAP 3.4): the greatest common divisor of the intervals of its
    /// periodic `TASK`s, so that every task falls on a scan, or `DEFAULT_SCAN_NS` when it declares none. At each scan
    /// truST runs only the tasks whose interval has elapsed (measured on the pinned truST, 2026-11-08).
    pub fn scan_ns(&self) -> i64 {
        self.scan_ns
    }

    /// One scan: writes the inputs, advances virtual time by the scan period, runs a cycle, returns `motor`.
    pub fn scan(&mut self, inputs: Inputs) -> Result<bool, String> {
        for (name, value) in [
            ("sensor", inputs.sensor),
            ("start", inputs.start),
            ("stop", inputs.stop),
        ] {
            self.harness
                .try_set_input(name, value)
                .map_err(|e| e.to_string())?;
        }
        self.harness
            .advance_time(Duration::from_nanos(self.scan_ns));
        let cycle = self.harness.cycle();
        if let Some(error) = cycle.errors.first() {
            return Err(error.to_string());
        }
        match self.harness.get_output("motor") {
            Some(Value::Bool(on)) => Ok(on),
            other => Err(format!("motor is not a BOOL: {other:?}")),
        }
    }

    /// The value of a variable of `PROGRAM Main`, for the I/O panel.
    pub fn bool_var(&self, name: &str) -> Option<bool> {
        match self.harness.get_output(name) {
            Some(Value::Bool(b)) => Some(b),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = "PROGRAM Main\nVAR sensor : BOOL; start : BOOL; stop : BOOL; motor : BOOL; END_VAR\n\
        IF start THEN motor := TRUE; END_IF;\nIF stop THEN motor := FALSE; END_IF;\nEND_PROGRAM\n";

    /// MINIMAL with a configuration whose tasks have the given `TASK` attributes.
    fn with_tasks(tasks: &[&str]) -> String {
        let declarations: String = tasks
            .iter()
            .enumerate()
            .map(|(i, attrs)| format!("TASK T{i}({attrs});\n"))
            .collect();
        format!(
            "{MINIMAL}CONFIGURATION Cell\nRESOURCE CPU ON PLC\n{declarations}PROGRAM P WITH T0 : Main;\nEND_RESOURCE\nEND_CONFIGURATION\n"
        )
    }

    #[test]
    fn the_scan_period_comes_from_the_program_tasks() {
        let period = |tasks: &[&str]| Plc::build(&with_tasks(tasks)).map(|p| p.scan_ns());
        assert_eq!(Plc::build(MINIMAL).unwrap().scan_ns(), DEFAULT_SCAN_NS);
        assert_eq!(period(&["INTERVAL := T#5ms, PRIORITY := 1"]), Ok(5_000_000));
        assert_eq!(
            period(&[
                "INTERVAL := T#4ms, PRIORITY := 1",
                "INTERVAL := T#6ms, PRIORITY := 2"
            ]),
            Ok(2_000_000),
            "the greatest common divisor, so every task falls on a scan"
        );
        let error = period(&["SINGLE := start, PRIORITY := 1"]).expect_err("refused");
        assert!(error.contains("SINGLE"), "{error}");
    }

    #[test]
    fn a_syntax_error_is_reported_as_text() {
        let error = Plc::build("PROGRAM Main\nVAR x : BOOL; END_VAR\nx := ;\nEND_PROGRAM\n")
            .err()
            .expect("refused");
        assert!(!error.is_empty());
    }

    #[test]
    fn missing_contract_variables_are_listed() {
        let error = Plc::build("PROGRAM Main\nVAR motor : BOOL; END_VAR\nEND_PROGRAM\n")
            .err()
            .expect("refused");
        assert!(error.contains("sensor, start, stop"), "{error}");
    }

    #[test]
    fn a_contract_variable_of_the_wrong_type_is_refused() {
        let source = "PROGRAM Main\nVAR sensor : INT; start : BOOL; stop : BOOL; motor : BOOL; END_VAR\nEND_PROGRAM\n";
        let error = Plc::build(source).err().expect("refused");
        assert!(error.contains("sensor"), "{error}");
    }

    #[test]
    fn a_runtime_fault_in_the_cycle_is_an_error() {
        let source = "PROGRAM Main\nVAR sensor : BOOL; start : BOOL; stop : BOOL; motor : BOOL;\n\
            zero : INT; result : INT; END_VAR\nresult := 10 / zero;\nEND_PROGRAM\n";
        let mut plc = Plc::build(source).expect("builds");
        let error = plc.scan(Inputs::default()).expect_err("a fault");
        assert!(error.to_lowercase().contains("division"), "{error}");
    }

    #[test]
    fn a_scan_applies_the_inputs_and_returns_the_motor() {
        let mut plc = Plc::build(MINIMAL).expect("builds");
        assert!(!plc.scan(Inputs::default()).unwrap());
        assert!(
            plc.scan(Inputs {
                start: true,
                ..Inputs::default()
            })
            .unwrap()
        );
        assert!(plc.scan(Inputs::default()).unwrap(), "the program latches");
        assert!(
            !plc.scan(Inputs {
                stop: true,
                ..Inputs::default()
            })
            .unwrap()
        );
        assert_eq!(plc.bool_var("motor"), Some(false));
    }
}
