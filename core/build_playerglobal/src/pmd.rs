use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::{BufRead, BufReader, Error, ErrorKind, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Pmd {
    #[serde(rename = "$value")]
    files: Option<Vec<File>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    name: String,
    #[serde(rename = "$value")]
    violations: Vec<Violation>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Violation {
    #[serde(rename = "beginline")]
    begin_line: usize,
    #[serde(rename = "endline")]
    end_line: usize,

    rule: String,
    #[serde(rename = "$value")]
    message: String,
}

impl Pmd {
    pub fn open(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let file = std::fs::File::open(path.into())?;
        serde_xml_rs::from_reader(file)
            .map_err(|_| Error::new(ErrorKind::Other, "Failed to parse XML data"))
    }

    pub fn contains_violations(&self) -> bool {
        self.files.is_some()
    }

    pub fn violation_count(&self) -> usize {
        let mut count = 0;
        if let Some(files) = self.files.as_ref() {
            for file in files.iter() {
                count += file.violations.len();
            }
        }
        count
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut file = std::fs::File::open(&self.name).map_err(|_| fmt::Error)?;
        for violation in self.violations.iter() {
            writeln!(f, "{}: {}", "warning".yellow(), violation.message)?;
            let max_digits = violation.end_line.to_string().len();
            writeln!(
                f,
                "{}{} {}",
                " ".repeat(max_digits),
                "-->".blue().bold(),
                self.name.bold()
            )?;
            file.seek(SeekFrom::Start(0)).map_err(|_| fmt::Error)?;
            let reader = BufReader::new(&file);
            for (num, text) in reader
                .lines()
                .enumerate()
                .skip(violation.begin_line - 1)
                .take(violation.end_line - violation.begin_line + 1)
            {
                let mut text = text.map_err(|_| fmt::Error)?;
                if text.len() > 117 {
                    text.truncate(117);
                    text.push_str("...");
                }
                let num_str = (num + 1).to_string();
                writeln!(
                    f,
                    "{}{}{} {}",
                    num_str.blue().bold(),
                    " ".repeat(max_digits - num_str.len() + 1),
                    "|".blue().bold(),
                    text
                )?;
            }
            writeln!(
                f,
                "{}{} {}: rule {} is enabled\n",
                " ".repeat(max_digits + 1),
                "=".blue().bold(),
                "note".bold(),
                violation.rule.bold()
            )?;
        }
        Ok(())
    }
}

impl fmt::Display for Pmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(files) = self.files.as_ref() {
            for file in files.iter() {
                writeln!(f, "{}", file)?;
            }
            write!(f, "{} total warnings emitted", self.violation_count())?;
        }
        Ok(())
    }
}
