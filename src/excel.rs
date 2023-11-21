use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};
use std::fmt;
use crate::csv::Parser;

#[derive(Debug)]
pub enum FieldValue {
    Integer(i32),
    Float(f32),
    String(String),
    Formula(String)
}

impl FieldValue {
    fn new(item: String) -> Self {
        // to-do, what if its the string "123" and not an int?
        if let Ok(int) = item.parse::<i32>() {
            FieldValue::Integer(int)
        } else if let Ok(float) = item.parse::<f32>() {
            FieldValue::Float(float)
        } else {
            let chars = item.chars().collect::<Vec<_>>();
            if chars.len() > 0 && chars[0] == '=' {
                return FieldValue::Formula(item);
            }
            return FieldValue::String(item);
        }
    }

    fn fmt_string_field(&self, item: &str) -> String {
        let mut out = String::new();
        let special_chars = ['"', ',', '\n', '\r'];

        for sp_c in special_chars {
            if item.contains(sp_c) {
                out.push('"');
                for rest in item.chars() {
                    if rest == '"' {
                        out.push('"');
                        out.push('"');
                    } else {
                        out.push(rest);
                    }
                }
                out.push('"');
                return out;
            }
        }

        return item.to_string();
    }
}

impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldValue::Integer(i) => write!(f, "{}", i),
            FieldValue::Float(fl) => write!(f, "{}", fl),
            FieldValue::String(s) => write!(f, "{}", self.fmt_string_field(&s)),
            FieldValue::Formula(s) => write!(f, "{}", self.fmt_string_field(&s)),
        }
    }
}


#[derive(Debug)]
pub struct Field {
    pub val: FieldValue
}

impl Field {
    fn new(item: String) -> Self {
        let val = FieldValue::new(item);
        Self {
            val
        }
    }
}

pub struct Excel {
    src: String,
    pub rows: Vec<Vec<Field>>,
}

impl Excel {
    pub fn new(src: String) -> Self {
        let mut parser = Parser::new(&src);
        let csv = parser.parse();
        
        let mut rows = Vec::new();
        let mut curr_row = Vec::new();

        for row in csv {
            for item in row {
                curr_row.push(Field::new(item));
            }
            rows.push(curr_row);
            curr_row = Vec::new();
        }

        rows.push(curr_row);

        Self {
            src,
            rows
        }
    }

    pub fn from_path(path: &str) -> std::io::Result<Self> {
        let file_contents = read_to_string(&path)?;
        Ok(Self::new(file_contents))
    }

    pub fn to_file(&self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for row in &self.rows {
            let mut row_output = Vec::new();
            for field in row {
                row_output.extend_from_slice(field.val.to_string().as_bytes());
                row_output.push(b',');
            }
            writer.write_all(&row_output)?;
            writer.write(&[b'\n'])?;
        }
        writer.flush();
        Ok(())
    }
}