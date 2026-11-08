use std::fmt;

/// One problem found in a project, located by file and by the path inside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: String,
    pub path: String,
    pub message: String,
}

impl Diagnostic {
    pub fn new(file: &str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            file: file.to_string(),
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}: {}", self.file, self.message)
        } else {
            write!(f, "{}: {}: {}", self.file, self.path, self.message)
        }
    }
}
