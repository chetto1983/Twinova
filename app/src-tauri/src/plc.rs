//! The Structured Text controller of the scene: truST's harness at the pinned commit, driven one scan at a time
//! with virtual time (spec 2026-11-07-tauri-shell-design.md, "The PLC binding").

use serde::Serialize;
use trust_runtime::harness::TestHarness;
use trust_runtime::value::{Duration, Value};

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
            Ok(Self { harness })
        } else {
            Err(format!(
                "PROGRAM Main must declare these BOOL variables: {}",
                missing.join(", ")
            ))
        }
    }

    /// One scan: writes the inputs, advances virtual time by `scan_ms`, runs a cycle, returns `motor`.
    pub fn scan(&mut self, inputs: Inputs, scan_ms: i64) -> Result<bool, String> {
        for (name, value) in [
            ("sensor", inputs.sensor),
            ("start", inputs.start),
            ("stop", inputs.stop),
        ] {
            self.harness
                .try_set_input(name, value)
                .map_err(|e| e.to_string())?;
        }
        self.harness.advance_time(Duration::from_millis(scan_ms));
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
        let error = plc.scan(Inputs::default(), 10).expect_err("a fault");
        assert!(error.to_lowercase().contains("division"), "{error}");
    }

    #[test]
    fn a_scan_applies_the_inputs_and_returns_the_motor() {
        let mut plc = Plc::build(MINIMAL).expect("builds");
        assert!(!plc.scan(Inputs::default(), 10).unwrap());
        assert!(
            plc.scan(
                Inputs {
                    start: true,
                    ..Inputs::default()
                },
                10
            )
            .unwrap()
        );
        assert!(
            plc.scan(Inputs::default(), 10).unwrap(),
            "the program latches"
        );
        assert!(
            !plc.scan(
                Inputs {
                    stop: true,
                    ..Inputs::default()
                },
                10
            )
            .unwrap()
        );
        assert_eq!(plc.bool_var("motor"), Some(false));
    }
}
