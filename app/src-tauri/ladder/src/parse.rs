//! The `.ld` text grammar, Open Industry Project's (NOTICE), read into the model:
//!
//! ```text
//! (* a comment, above its rung *)
//! Label: [|Start|, |Motor|] |/Stop| T1(IN, PT := T#2s) (Motor) >>Next;
//! ```
//!
//! A contact is `|x|`, `|/x|`, `|P x|` or `|N x|`; a coil `(x)`, `(/x)`, `(S x)`, `(R x)`, `(P x)` or `(N x)`; a branch
//! `[path, path, …]`, an empty path being a wire; a box a function block's instance or a function with its pins. An
//! operand or a pin's value is kept as written: truST, which compiles the lowered program, reads its expressions.

use serde::Serialize;

use crate::model::{CoilMode, ContactMode, Element, Exit, Pin, Program, Rung};

/// Where the text stops making sense, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParseError {
    pub message: String,
    /// A byte offset in the text.
    pub offset: usize,
}

pub fn parse(text: &str) -> Result<Program, ParseError> {
    let mut p = Parser { text, at: 0 };
    let declarations = p.declarations()?;
    let mut rungs = Vec::new();
    loop {
        let comment = p.comments()?;
        if p.at == text.len() {
            if comment.is_some() {
                return Err(p.error("a comment with no rung after it"));
            }
            break;
        }
        rungs.push(p.rung(comment)?);
    }
    Ok(Program {
        declarations,
        rungs,
    })
}

struct Parser<'a> {
    text: &'a str,
    at: usize,
}

const VAR_BLOCKS: [&str; 7] = [
    "VAR_INPUT",
    "VAR_OUTPUT",
    "VAR_IN_OUT",
    "VAR_TEMP",
    "VAR_GLOBAL",
    "VAR_EXTERNAL",
    "VAR",
];

impl Parser<'_> {
    fn rest(&self) -> &str {
        &self.text[self.at..]
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.into(),
            offset: self.at,
        }
    }

    fn skip_space(&mut self) {
        let trimmed = self.rest().trim_start();
        self.at = self.text.len() - trimmed.len();
    }

    /// Whether the text goes on with `word`, a whole word, in any case.
    fn word(&self, word: &str) -> bool {
        let rest = self.rest();
        rest.len() >= word.len()
            && rest[..word.len()].eq_ignore_ascii_case(word)
            && !rest[word.len()..].starts_with(is_ident_char)
    }

    fn eat(&mut self, token: &str) -> bool {
        self.skip_space();
        let found = self.rest().starts_with(token);
        if found {
            self.at += token.len();
        }
        found
    }

    fn expect(&mut self, token: &str, what: &str) -> Result<(), ParseError> {
        if self.eat(token) {
            Ok(())
        } else {
            Err(self.error(&format!("expected '{token}' {what}")))
        }
    }

    /// The `VAR … END_VAR` blocks before the first rung, as written.
    fn declarations(&mut self) -> Result<String, ParseError> {
        self.skip_space();
        let start = self.at;
        let mut end = start;
        loop {
            self.skip_space();
            if !VAR_BLOCKS.iter().any(|block| self.word(block)) {
                break;
            }
            let upper = self.rest().to_ascii_uppercase();
            let Some(close) = upper.find("END_VAR") else {
                return Err(self.error("a VAR block with no END_VAR"));
            };
            self.at += close + "END_VAR".len();
            end = self.at;
        }
        self.at = end;
        Ok(self.text[start..end].to_string())
    }

    /// The comments written before a rung, joined by lines, each line without its leading space.
    fn comments(&mut self) -> Result<Option<String>, ParseError> {
        let mut lines: Vec<String> = Vec::new();
        loop {
            self.skip_space();
            if !self.rest().starts_with("(*") {
                break;
            }
            let Some(close) = self.rest().find("*)") else {
                return Err(self.error("a comment with no '*)'"));
            };
            let body = &self.rest()[2..close];
            lines.extend(body.trim().lines().map(|l| l.trim().to_string()));
            self.at += close + 2;
        }
        Ok((!lines.is_empty()).then(|| lines.join("\n")))
    }

    fn rung(&mut self, comment: Option<String>) -> Result<Rung, ParseError> {
        self.skip_space();
        let label = self.label();
        let elements = self.series(&[";"])?;
        let exit = self.exit()?;
        self.expect(";", "to end the rung")?;
        Ok(Rung {
            comment,
            label,
            elements,
            exit,
        })
    }

    /// `Name:` opening a rung, not `Name :=` nor a box.
    fn label(&mut self) -> Option<String> {
        let start = self.at;
        let name = self.ident()?;
        self.skip_space();
        if self.rest().starts_with(':') && !self.rest().starts_with(":=") {
            self.at += 1;
            return Some(name);
        }
        self.at = start;
        None
    }

    fn exit(&mut self) -> Result<Option<Exit>, ParseError> {
        self.skip_space();
        if self.eat(">>") {
            let label = self
                .ident()
                .ok_or_else(|| self.error("a jump names a label: >>Next"))?;
            return Ok(Some(Exit::Jump { label }));
        }
        if self.word("RETURN") {
            self.at += "RETURN".len();
            return Ok(Some(Exit::Return));
        }
        Ok(None)
    }

    /// Elements up to one of `ends`, or to a rung's exit.
    fn series(&mut self, ends: &[&str]) -> Result<Vec<Element>, ParseError> {
        let mut out = Vec::new();
        loop {
            self.skip_space();
            if self.at == self.text.len() {
                return Err(self.error("expected ';' to end the rung"));
            }
            let rest = self.rest();
            if ends.iter().any(|e| rest.starts_with(e))
                || rest.starts_with(">>")
                || self.word("RETURN")
            {
                return Ok(out);
            }
            out.push(self.element()?);
        }
    }

    fn element(&mut self) -> Result<Element, ParseError> {
        let start = self.at;
        if self.eat("[") {
            let mut paths = vec![self.series(&[",", "]"])?];
            while self.eat(",") {
                paths.push(self.series(&[",", "]"])?);
            }
            self.expect("]", "to close the branch")?;
            if paths.len() < 2 {
                self.at = start;
                return Err(self.error("a branch has two paths or more: [|A|, |B|]"));
            }
            return Ok(Element::Branch { paths });
        }
        if self.eat("|") {
            let mode = match self.modifier(&['/', 'P', 'N']) {
                None => ContactMode::Open,
                Some('/') => ContactMode::Closed,
                Some('P') => ContactMode::Rising,
                Some(_) => ContactMode::Falling,
            };
            let operand = self.operand(&['|'], "a contact")?;
            self.expect("|", "to close the contact")?;
            return Ok(Element::Contact { mode, operand });
        }
        if self.eat("(") {
            let mode = match self.modifier(&['/', 'S', 'R', 'P', 'N']) {
                None => CoilMode::Normal,
                Some('/') => CoilMode::Negated,
                Some('S') => CoilMode::Set,
                Some('R') => CoilMode::Reset,
                Some('P') => CoilMode::Rising,
                Some(_) => CoilMode::Falling,
            };
            let operand = self.operand(&[')'], "a coil")?;
            self.expect(")", "to close the coil")?;
            return Ok(Element::Coil { mode, operand });
        }
        self.block()
    }

    /// A contact's or a coil's modifier: `/`, or a letter followed by a space and the operand (`|P|` alone examines a
    /// variable called P).
    fn modifier(&mut self, letters: &[char]) -> Option<char> {
        self.skip_space();
        let mut chars = self.rest().chars();
        let first = chars.next()?.to_ascii_uppercase();
        if !letters.contains(&first) {
            return None;
        }
        if first == '/' {
            self.at += 1;
            return Some('/');
        }
        let spaced = chars.next().is_some_and(char::is_whitespace);
        let then = self.rest()[1..].trim_start().chars().next();
        if spaced && then.is_some_and(|c| c == '?' || c.is_ascii_alphabetic() || c == '_') {
            self.at += 1;
            return Some(first);
        }
        None
    }

    /// An operand up to one of `ends` outside brackets: `?` for a blank, else a place as written.
    fn operand(&mut self, ends: &[char], what: &str) -> Result<Option<String>, ParseError> {
        let text = self.balanced(ends);
        match text.as_str() {
            "?" => Ok(None),
            "" => Err(self.error(&format!("{what} needs a variable, or ? while blank"))),
            _ => Ok(Some(text)),
        }
    }

    /// A box: `T1(IN, PT := T#2s, ET => elapsed)` or a function, its head blank (`?`) while unnamed.
    fn block(&mut self) -> Result<Element, ParseError> {
        self.skip_space();
        let head = if self.eat("?") {
            None
        } else {
            let text = self.balanced(&['(']);
            if text.is_empty() || !text.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
                return Err(self
                    .error("expected a contact |x|, a coil (x), a branch [..., ...] or a block"));
            }
            Some(text)
        };
        self.expect("(", "to open the block's pins")?;
        let mut pins = Vec::new();
        if !self.eat(")") {
            loop {
                pins.push(self.pin()?);
                if self.eat(")") {
                    break;
                }
                self.expect(",", "between the block's pins")?;
            }
        }
        Ok(Element::Box { head, pins })
    }

    fn pin(&mut self) -> Result<Pin, ParseError> {
        self.skip_space();
        let name = self
            .ident()
            .ok_or_else(|| self.error("a pin is named: IN, PT := T#2s or Q => done"))?;
        let out = if self.eat(":=") {
            false
        } else if self.eat("=>") {
            true
        } else {
            return Ok(Pin {
                name,
                value: None,
                out: false,
            });
        };
        let value = self.balanced(&[',', ')']);
        if value.is_empty() {
            return Err(self.error("a pin's value is missing"));
        }
        Ok(Pin {
            name,
            value: Some(value),
            out,
        })
    }

    fn ident(&mut self) -> Option<String> {
        self.skip_space();
        let rest = self.rest();
        if !rest.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
            return None;
        }
        let len = rest.find(|c: char| !is_ident_char(c)).unwrap_or(rest.len());
        let found = rest[..len].to_string();
        self.at += len;
        Some(found)
    }

    /// The text up to one of `ends` outside parentheses, brackets and string literals, trimmed.
    fn balanced(&mut self, ends: &[char]) -> String {
        self.skip_space();
        let rest = self.rest();
        let mut depth = 0usize;
        let mut quote: Option<char> = None;
        let mut len = rest.len();
        for (i, c) in rest.char_indices() {
            match (quote, c) {
                (Some(q), _) if c == q => quote = None,
                (Some(_), _) => {}
                (None, '\'' | '"') => quote = Some(c),
                (None, '(' | '[') if !(depth == 0 && ends.contains(&c)) => depth += 1,
                (None, ')' | ']') if depth > 0 => depth -= 1,
                (None, _) if depth == 0 && ends.contains(&c) => {
                    len = i;
                    break;
                }
                _ => {}
            }
        }
        let text = rest[..len].trim().to_string();
        self.at += len;
        text
    }
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
#[path = "parse_tests.rs"]
mod tests;
