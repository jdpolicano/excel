use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};
use std::fmt;

#[derive(Debug)]
enum Token<'a> {
    Value(&'a [char]), // a full field
    Delimiter, // comma
    Newline, // "\n" | "\r\n"
    TextQualifier, // '"'
    WhiteSpace,// whitespace not inside a text qualified string...
    Escaped, // '""' for quotations inside a qualified string.
    EndOfFile,
}


pub struct Tokenizer<'a> {
    view: &'a [char],
}

// this turned into more of a parse3r than a tokenizer...
impl<'a> Tokenizer<'a> {
    pub fn new(view: &'a [char]) -> Self {
        Self {
            view,
        }
    }

    fn skip(&mut self, n: usize) {
        self.view = &self.view[n.min(self.view.len())..];
    }

    fn peek(&self) -> Option<&char> {
        self.view.get(0)
    }

    fn next_is(&self, cmp: char) -> bool {
        if let Some(c) = self.peek() {
            return *c == cmp;
        }
        false
    }

    fn next_is_one_of(&self, sp_cmp: &[char]) -> bool {
        if let Some(c) = self.peek() {
            for cmp in sp_cmp {
                if *c == *cmp {
                    return true
                }
            }
        }
        false
    }

    fn empty(&self) -> bool {
        self.view.is_empty()
    }

    // to-do, could this be a zero copy operation instead?
    fn get_csv_token(&mut self) -> Token {
        if self.empty() {
            return Token::EndOfFile;
        }

        if let Some(c) = self.peek() {
            match c {
                '"' => {
                    self.skip(1);
                    if self.next_is('"') {
                        self.skip(1);
                        return Token::Escaped;
                    }

                    return Token::TextQualifier
                }
    
                ',' => {
                    self.skip(1);
                    return Token::Delimiter; 
                }
    
                ' ' => {
                    self.skip(1);
                    return Token::WhiteSpace; 
                }
    
    
                '\r'  => {
                    self.skip(2);
                    return Token::Newline;
                }
    
                '\n' => {
                    self.skip(1);
                    return Token::Newline
                }
    
                _ => {
                    let tmp = &self.view[0..];
                    let mut idx = 0;
                    let sp_c = ['\n', '\r', ',', '"', ' '];
                    while !self.next_is_one_of(&sp_c) {
                        self.skip(1);
                        idx += 1;
                    }
                    return Token::Value(&tmp[0..idx]);
                }
            }
        }

        Token::EndOfFile
    }
}

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [char]) -> Self {
        Self {
            tokenizer: Tokenizer::new(source)
        }
    }

    pub fn parse(&mut self) -> Vec<Vec<String>> {
        let mut max_field_size = 0;
        let mut max_row_width = 0;
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        let mut current_field = String::new();
        let mut in_qualifier = false;

        while let token = self.tokenizer.get_csv_token() {
            match token {
                Token::Value(val) => {
                    for c in val {
                        current_field.push(*c);
                    }
                }
                Token::Delimiter => {
                    if !in_qualifier {
                        max_field_size = max_field_size.max(current_field.len());
                        current_row.push(current_field);
                        current_field = String::with_capacity(max_field_size);
                    } else {
                        current_field.push(',');
                    }
                }
                Token::Newline => {
                    if !in_qualifier {
                        max_field_size = max_field_size.max(current_field.len());
                        max_row_width = max_row_width.max(current_row.len());
                        current_row.push(current_field);
                        rows.push(current_row);
                        current_row = Vec::with_capacity(max_row_width);
                        current_field = String::with_capacity(max_field_size);
                    } else {
                        current_field.push('\n');
                    }
                }
                Token::TextQualifier => {
                    in_qualifier = !in_qualifier;
                }
    
                Token::WhiteSpace => {
                    if in_qualifier {
                        current_field.push(' ');
                    }
                }
    
                Token::Escaped => {
                    if in_qualifier {
                        current_field.push('"');
                    }
                }
                Token::EndOfFile => {
                    break;
                }
            }
        }
    
        // Handle the last row if not empty
        if !current_field.is_empty() {
            current_row.push(current_field);
        }
        if !current_row.is_empty() {
            rows.push(current_row);
        }
    
        rows
    }
}

