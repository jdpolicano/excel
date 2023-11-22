/*
This file will likey just be a tokenizer and parser for the formuals. The resulting structure will be an ast
that can then be executed (hopefully).
*/
#[derive(Debug)]
enum Token <'a> {
    OpenBracket, // '('
    CloseBraket, // ')'
    RangeDelimiter, // ':'
    TextQualifier, // '"'
    Comma, // ','
    Operator(char), // '+', '-', etc..
    Text(&'a [char]), // any purly text field
    EndOfFile,
}

impl<'a> PartialEq for Token<'a> {
    fn eq(&self, other: &Self) -> bool {
        use Token::*;
        match (self, other) {
            (OpenBracket, OpenBracket) => true,
            (CloseBraket, CloseBraket) => true,
            (RangeDelimiter, RangeDelimiter) => true,
            (TextQualifier, TextQualifier) => true,
            (Comma, Comma) => true,
            (EndOfFile, EndOfFile) => true,
            (Text(a), Text(b)) => a == b,
            (Operator(a), Operator(b)) => a == b,
            _ => false,
        }
    }
}


// this is copy pasted from the tokenizer for the csv parser, this probably could have been a 
// trait or some other generic type maybe?
struct Tokenizer<'a> {
   view: &'a [char]
}

impl<'a> Tokenizer<'a> {
    pub fn new(view: &'a [char]) -> Self {
        Self {
            view
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

    fn get_formula_token(&mut self) -> Token<'a> {
        if self.empty() {
            return Token::EndOfFile;
        }

        if let Some(c) = self.peek() {
            match c {
                '+' | '-' | '/' | '*' =>  { 
                    let ret = Token::Operator(*c);
                    self.skip(1);
                    return ret;
                },

                '(' =>  {
                    self.skip(1); 
                    return Token::OpenBracket 
                },

                ')' => { 
                    self.skip(1);
                    return Token::CloseBraket 
                },

                ':' => {
                    self.skip(1);
                    return Token::RangeDelimiter
                }

                '"' => {
                    self.skip(1);
                    return Token::TextQualifier
                }

                ',' => {
                    self.skip(1);
                    return Token::Comma
                }

                _ => {
                    let mut idx = 0;
                    let tmp = &self.view[0..];
                    let sp_c = ['+', '-', '/', '*', ':', '(', ')', '"', ','];
                    while !self.next_is_one_of(&sp_c) && !self.empty() {
                        self.skip(1);
                        idx += 1;
                    }

                    return Token::Text(&tmp[0..idx]);
                }

            }
        }

        Token::EndOfFile
    }
}


enum NodeType {
    Formula, // top level entry point of the ast
    Expression, // what should be evaluated
    Term, // arguments for the expression
    Primary, // a value, reference, or another expression,
    Primitive, // prims like string boolean int or float
    Function, // a function to eval with arguments.
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_tokens() {
        let input = "+-*/".chars().collect::<Vec<_>>();
        let mut tokenizer = Tokenizer::new(&input);

        assert_eq!(tokenizer.get_formula_token(), Token::Operator('+'));
        assert_eq!(tokenizer.get_formula_token(), Token::Operator('-'));
        assert_eq!(tokenizer.get_formula_token(), Token::Operator('*'));
        assert_eq!(tokenizer.get_formula_token(), Token::Operator('/'));
        assert_eq!(tokenizer.get_formula_token(), Token::EndOfFile);
    }

    #[test]
    fn test_bracket_tokens() {
        let input = "()".chars().collect::<Vec<_>>();
        let mut tokenizer = Tokenizer::new(&input);

        assert_eq!(tokenizer.get_formula_token(), Token::OpenBracket);
        assert_eq!(tokenizer.get_formula_token(), Token::CloseBraket);
        assert_eq!(tokenizer.get_formula_token(), Token::EndOfFile);
    }

    #[test]
    fn test_text_token() {
        let input = "hello".chars().collect::<Vec<_>>();
        let mut tokenizer = Tokenizer::new(&input);

        assert_eq!(tokenizer.get_formula_token(), Token::Text(&input));
        assert_eq!(tokenizer.get_formula_token(), Token::EndOfFile);
    }

    #[test]
    fn test_mixed_tokens() {
        let input = "(A1:B2)".chars().collect::<Vec<_>>();
        let mut tokenizer = Tokenizer::new(&input);
        assert_eq!(tokenizer.get_formula_token(), Token::OpenBracket);
        assert_eq!(tokenizer.get_formula_token(), Token::Text(&['A', '1']));
        assert_eq!(tokenizer.get_formula_token(), Token::RangeDelimiter);
        assert_eq!(tokenizer.get_formula_token(), Token::Text(&['B', '2']));
        assert_eq!(tokenizer.get_formula_token(), Token::CloseBraket);
        assert_eq!(tokenizer.get_formula_token(), Token::EndOfFile);
    }

    #[test]
    fn test_functions() {
        let input = "SUM(A1:C1)".chars().collect::<Vec<_>>();
        let mut tokenizer = Tokenizer::new(&input);
        assert_eq!(tokenizer.get_formula_token(), Token::Text(&['S', 'U', 'M']));
        assert_eq!(tokenizer.get_formula_token(), Token::OpenBracket);
        assert_eq!(tokenizer.get_formula_token(), Token::Text(&['A', '1']));
        assert_eq!(tokenizer.get_formula_token(), Token::RangeDelimiter);
        assert_eq!(tokenizer.get_formula_token(), Token::Text(&['C', '1']));
        assert_eq!(tokenizer.get_formula_token(), Token::CloseBraket);
        assert_eq!(tokenizer.get_formula_token(), Token::EndOfFile);
    }
    // Additional tests for other scenarios (TextQualifier, RangeDelimiter, etc.)
}