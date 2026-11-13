//! A Ladder file as a program of the PLC (spec decision 2): a PROGRAM named after the file.

use crate::parse::{ParseError, parse};

/// The PROGRAM the Ladder file at `path` is: its name without the folder and the extension.
pub fn program_name(path: &str) -> Result<&str, String> {
    let file = path.rsplit('/').next().unwrap_or(path);
    let name = file.strip_suffix(".ld").unwrap_or(file);
    let identifier = name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if identifier {
        Ok(name)
    } else {
        Err(format!(
            "{path}: a Ladder file is a PROGRAM named after the file, so its name must be an identifier"
        ))
    }
}

/// The text before `text`'s declarations in `declarations_only`: they keep their lines, and on the first line move
/// right by its length.
pub fn declarations_prefix(name: &str) -> String {
    format!("PROGRAM {name} ")
}

/// The program `name` with `text`'s declarations and no rungs, each declaration on the line it has in the file: what
/// the ST frontend reads of a Ladder file, and what tells the lowering what a box's head names.
pub fn declarations_only(name: &str, text: &str) -> Result<String, ParseError> {
    let declarations = parse(text)?.declarations;
    let leading = &text[..text.len() - text.trim_start().len()];
    Ok(format!(
        "{}{leading}{declarations}\nEND_PROGRAM\n",
        declarations_prefix(name)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ladder_file_is_named_by_its_file_name_when_that_is_an_identifier() {
        assert_eq!(program_name("plc/Pump_2.ld"), Ok("Pump_2"));
        assert_eq!(program_name("_Belt.ld"), Ok("_Belt"));
        for bad in ["plc/2nd.ld", "plc/a-b.ld", "plc/.ld"] {
            assert_eq!(
                program_name(bad),
                Err(format!(
                    "{bad}: a Ladder file is a PROGRAM named after the file, so its name must be an identifier"
                ))
            );
        }
    }

    #[test]
    fn its_declarations_keep_their_lines() {
        let text = "\n\nVAR\n  a : BOOL;\nEND_VAR\n|a| (a);\n";
        let only = declarations_only("Pump", text).unwrap();
        assert_eq!(
            only,
            "PROGRAM Pump \n\nVAR\n  a : BOOL;\nEND_VAR\nEND_PROGRAM\n"
        );
        assert_eq!(only.lines().nth(3), text.lines().nth(3));
        assert_eq!(
            declarations_only("P", "|a| (a);").unwrap(),
            "PROGRAM P \nEND_PROGRAM\n"
        );
        assert!(declarations_only("P", "|a| (a)").is_err());
    }
}
