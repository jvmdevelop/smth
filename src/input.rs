use crate::MathSystems;
use std::{fs, path::Path};

#[derive(Debug, Clone, Copy)]
pub enum MathInput<'a> {
    Text(&'a str),
    File(&'a Path),
}
impl<'a> From<&'a str> for MathInput<'a> {
    fn from(text: &'a str) -> Self {
        Self::Text(text)
    }
}
impl<'a> From<&'a String> for MathInput<'a> {
    fn from(text: &'a String) -> Self {
        Self::Text(text)
    }
}
impl<'a> From<&'a Path> for MathInput<'a> {
    fn from(path: &'a Path) -> Self {
        Self::File(path)
    }
}

pub fn msystem<'a>(input: impl Into<MathInput<'a>>) -> Result<MathSystems, String> {
    match input.into() {
        MathInput::Text(text) => MathSystems::parse(text),
        MathInput::File(path) => {
            let text = fs::read_to_string(path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            MathSystems::parse(&text)
        }
    }
}
pub fn msystem_text(text: &str) -> Result<MathSystems, String> {
    MathSystems::parse(text)
}
pub fn msystem_file(path: impl AsRef<Path>) -> Result<MathSystems, String> {
    let path = path.as_ref();
    let text =
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    MathSystems::parse(&text)
}

#[macro_export]
macro_rules! msystem {
    (file $path:expr) => {
        $crate::msystem_file($path)
    };
    ($input:expr) => {
        $crate::msystem($input)
    };
}
