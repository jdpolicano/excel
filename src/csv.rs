#[derive(Debug)]
pub enum Token<'a> {
    Value(&'a str), // a full field
    Delimiter, // comma
    Newline, // "\n" | "\r\n"
    TextQualifier, // '"'
    WhiteSpace, // whitespace not inside a text qualified string...
    Escaped, // '""' for quotations inside a qualified string.
    EndOfFile,
}


pub struct Tokenizer<'a> {
    view: &'a str,
}

// this turned into more of a parse3r than a tokenizer...
impl<'a> Tokenizer<'a> {
    pub fn new(view: &'a str) -> Self {
        Self {
            view,
        }
    }

    fn skip(&mut self) {
        let mut char_idx = self.view.char_indices();
        let next_idx = char_idx.nth(1).map_or(self.view.len(), |(idx, _)| idx);
        self.view = &self.view[next_idx..];
    }

    fn skip_and_slice(&mut self, nth: usize) -> &'a str {
        let mut char_idx = self.view.char_indices();
        let next_idx = char_idx.nth(nth).map_or(self.view.len(), |(idx, _)| idx);
        let slice = &self.view[0..next_idx];
        self.view = &self.view[next_idx..];
        slice
    }

    fn nth_index(&mut self, n: usize) -> usize {
        let mut char_idx = self.view.char_indices();
        char_idx.nth(n).map_or(self.view.len(), |(idx, _)| idx) 
    }

    fn peek(&self) -> Option<char> {
        self.view.chars().next()
    }

    fn nth_is(&self, idx: usize, cmp: char) -> bool {
        if let Some(c) = self.view.chars().nth(idx) {
            return c == cmp;
        }
        false
    }

    fn nth_is_one_of(&self, idx: usize, sp_cmp: &[char]) -> bool {
        if let Some(c) = self.view.chars().nth(idx) {
            for cmp in sp_cmp {
                if c == *cmp {
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
                    self.skip();
                    if self.nth_is(0, '"') {
                        self.skip();
                        return Token::Escaped;
                    }
                    return Token::TextQualifier;
                },
    
                ',' => {
                    self.skip();
                    return Token::Delimiter; 
                },
    
                ' ' => {
                    self.skip();
                    return Token::WhiteSpace; 
                }
    
    
                '\r' if self.nth_is(1, '\n') => {
                    self.skip();
                    self.skip();
                    return Token::Newline;
                }
    
                '\n' => {
                    self.skip();
                    return Token::Newline;
                }
    
                _ => {
                    let mut idx = 0;
                    let sp_c = ['\n', '\r', ',', '"', ' '];
                    while !self.nth_is_one_of(idx, &sp_c) && idx < self.view.len() {
                        idx += 1;
                    }
                    return Token::Value(self.skip_and_slice(idx));
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
    pub fn new(source: &'a str) -> Self {
        Self {
            tokenizer: Tokenizer::new(source)
        }
    }

    pub fn parse(&mut self) -> Vec<Vec<String>> {
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        let mut current_field = String::new();
        let mut in_qualifier = false;
    
        while let token = self.tokenizer.get_csv_token() {
            match token {
                Token::Value(v) => {
                    current_field.push_str(v);
                }
                Token::Delimiter => {
                    if !in_qualifier {
                        current_row.push(current_field);
                        current_field = String::new();
                    } else {
                        current_field.push(',');
                    }
                }
                Token::Newline => {
                    if !in_qualifier {
                        current_row.push(current_field);
                        rows.push(current_row);
                        current_row = Vec::new();
                        current_field = String::new()
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