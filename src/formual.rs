use std::ops::{ Sub, Mul, Div, Add };
use std::collections::HashSet;
use std::fmt;
/*
This file will likey just be a tokenizer and parser for the formuals. The resulting structure will be an ast
that can then be executed (hopefully).
*/
#[derive(Debug)]
enum Token <'a> {
    OpenBracket, // '('
    CloseBracket, // ')'
    RangeDelimiter, // ':'
    TextQualifier, // '"'
    Comma, // ',
    Operator(char), // '+', '-', etc..
    Text(&'a [char]), // any purly text field
    EndOfFile,
}

impl<'a> PartialEq for Token<'a> {
    fn eq(&self, other: &Self) -> bool {
        use Token::*;
        match (self, other) {
            (OpenBracket, OpenBracket) => true,
            (CloseBracket, CloseBracket) => true,
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

    fn lookahead(&mut self, n: usize) -> Token<'a> {
        let original = &self.view[0..];
        if n <= 1 {
            let mut toke = self.next();
            self.view = &original;
            return toke;
        } else {
            let mut toke = self.next();
            for i in 1..n {
                toke = self.next();
            }
            self.view = &original;
            return toke

        }
    }

    fn next(&mut self) -> Token<'a> {
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
                    return Token::CloseBracket 
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

                ' ' => {
                    self.skip(1);
                    return self.next()
                }

                _ => {
                    let mut idx = 0;
                    let tmp = &self.view[0..];
                    let sp_c = ['+', '-', '/', '*', ':', '(', ')', '"', ',', ' '];
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

#[derive(Debug)]
enum Operator {
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Invalid
}

impl Operator {
    fn new(op: char) -> Self {
        match op {
            '+' => Operator::Addition,
            '-' => Operator::Subtraction,
            '*' => Operator::Multiplication,
            '/' => Operator::Division,
            _ => Operator::Invalid
        }
    }

    fn apply<T>(&self, lhs: T, rhs: T) -> Option<T> 
    where
        T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>
    {
        match self {
            Operator::Addition => Some(lhs + rhs),
            Operator::Subtraction => Some(lhs - rhs),
            Operator::Multiplication => Some(lhs * rhs),
            Operator::Division => Some(lhs / rhs),
            Operator::Invalid => None
        }
    }

    fn is_addition(&self) -> bool {
        match self {
            Operator::Addition => true,
            _ => false
        }
    }

    fn is_subtraction(&self) -> bool {
        match self {
            Operator::Subtraction => true,
            _ => false
        }
    }

    fn is_division(&self) -> bool {
        match self {
            Operator::Division => true,
            _ => false
        }
    }

    fn is_multiplication(&self) -> bool {
        match self {
            Operator::Multiplication => true,
            _ => false
        }
    }
}

#[derive(Debug)]
enum Node {
    // top level entry point of the ast
    Formula {
        expr: Box<Node>,
    }, 

     // what should be evaluated
    Expression {
        lhs: Box<Node>, // the main term to evaluate.
        op: Option<Operator>, // if performing an operation on this.
        rhs: Option<Box<Node>> // another term...
    },

    // either a value or addition / subtraction operation.
    Term {
        lhs: Box<Node>, // factor...
        op: Option<Operator>,
        rhs: Option<Box<Node>>,
    },

    // either a value or multiplication / division operation.
    Factor {
        lhs: Box<Node>, // primary...
        op: Option<Operator>,
        rhs: Option<Box<Node>>,
    },

    Primary(Box<Node>), // a value, reference, or another expression.
    Primitive(String), // prims like string boolean int or float.
    CellRef(String), // a cell reference with (col, row) as strings.
    CellRange(Box<Node>, Box<Node>), // two cell refs.
    // a function call.
    Function {
        name: String,
        args: Vec<Node>
    }, 
}

#[derive(Debug, Clone)]
enum ParseError {
    UnexpectedEndOfFile,
    UnexpectedToken { expected: String, found: String },
    InvalidExpression,
    // Add more error types as needed
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedEndOfFile => write!(f, "Unexpected end of file"),
            ParseError::UnexpectedToken { expected, found } => {
                write!(f, "Expected token '{}', but found '{}'", expected, found)
            }
            ParseError::InvalidExpression => write!(f, "Invalid expression"),
            // Handle other errors here
        }
    }
}

pub type ParseResult = Result<Node, ParseError>;

pub struct Ast<'a> {
    src: &'a str,
    func_names: HashSet<String>
}

impl<'a> Ast<'a> {
    fn new(src: &'a str, func_names: HashSet<String>) -> Self {
        Self {
            src,
            func_names
        }
    }

    fn parse(&self) -> ParseResult {
        let chars: Vec<char> = self.src.chars().collect();
        if let Some(c) = chars.get(0) {
            if *c == '=' {
                let mut tokenizer = Tokenizer::new(&chars[1..]);
                let expr = self.parse_expression(&mut tokenizer)?;
                self.expect(&mut tokenizer, Token::EndOfFile, expr)
            } else {
                Err(ParseError::InvalidExpression)
            }
        } else {
            Err(ParseError::UnexpectedEndOfFile)
        }
    }

    fn parse_expression(&self, tokenizer: &mut Tokenizer) -> ParseResult {
        let lhs = self.parse_term(tokenizer)?;
        
        match tokenizer.lookahead(1) {
            Token::Operator(op) => {
                let operator = Operator::new(op);
                let _ = tokenizer.next();
                let rhs = self.parse_term(tokenizer)?;

                Ok(Node::Expression {
                    lhs: Box::new(lhs),
                    op: Some(operator),
                    rhs: Some(Box::new(rhs))
                })
            }

            _ => {
                Ok(Node::Expression {
                    lhs: Box::new(lhs),
                    op: None,
                    rhs: None,
                })
            }
        }
    }

    fn parse_term(&self, tokenizer: &mut Tokenizer) -> ParseResult {
        let lhs = self.parse_factor(tokenizer)?;
        
        match tokenizer.lookahead(1) {
            Token::Operator(op) if op == '+' || op == '-' => {
                let operator = Operator::new(op);
                let _ = tokenizer.next();
                let rhs = self.parse_factor(tokenizer).unwrap();

                Ok(Node::Term {
                    lhs: Box::new(lhs),
                    op: Some(operator),
                    rhs: Some(Box::new(rhs))
                })
            }

            _ => {
                Ok(Node::Term {
                    lhs: Box::new(lhs),
                    op: None,
                    rhs: None,
                })
            }
        }
    }

    fn parse_factor(&self, tokenizer: &mut Tokenizer) -> ParseResult {
        let lhs = self.parse_primary(tokenizer)?;
        
        match tokenizer.lookahead(1) {
            Token::Operator(op) if op == '/' || op == '*' => {
                let operator = Operator::new(op);
                let _ = tokenizer.next();
                let rhs = self.parse_primary(tokenizer)?;

                Ok(Node::Factor {
                    lhs: Box::new(lhs),
                    op: Some(operator),
                    rhs: Some(Box::new(rhs))
                })
            }

            _ => {
                Ok(Node::Factor {
                    lhs: Box::new(lhs),
                    op: None,
                    rhs: None,
                })
            }
        }
    }

    fn parse_primary(&self, tokenizer: &mut Tokenizer) -> ParseResult {
        match tokenizer.next() {
            Token::Text(text) => {
                let as_string: String = text.iter().collect();
                
                // Check if the text is a numeric, boolean, cell reference, or a string literal
                if self.is_numeric(text) || self.is_boolean(text) {
                    Ok(Node::Primitive(as_string))
                } else if self.is_cell_ref(text) {
                    Ok(Node::CellRef(as_string))
                } else {
                    self.parse_function(tokenizer, as_string)
                }
            },

            Token::TextQualifier => {
                return self.parse_string(tokenizer);
            }

            Token::OpenBracket => {
                let expr = self.parse_expression(tokenizer)?;
                self.expect(tokenizer, Token::CloseBracket, expr)
            },

            token => Err(ParseError::UnexpectedToken {
                expected: "Text or OpenBracket".to_string(),
                found: format!("{:?}", token),
            }),
        }
    }

    fn parse_string(&self, tokenizer: &mut Tokenizer) -> ParseResult {
        let mut string = String::new();
        let mut next_token = tokenizer.next();
        let mut done = false;
        
        while !done {
            match next_token {
                Token::TextQualifier => {
                    if tokenizer.lookahead(1) == Token::TextQualifier {
                        string.push('"');
                        let _ = tokenizer.next();
                    } else {
                        done = true;
                    }
                }

                Token::Text(text) => {
                    let as_string: String = text.iter().collect();
                    string.push_str(&as_string);
                }

                Token::EndOfFile => {
                    return Err(ParseError::UnexpectedEndOfFile);
                }

                _ => {
                    return Err(ParseError::InvalidExpression);
                }
            }

            next_token = tokenizer.next();
        }
        
        Ok(Node::Primitive(string))
    }

    // for now we will only support functions that take n arguments, no cell ranges.
    fn parse_function(&self, tokenizer: &mut Tokenizer, name: String) -> ParseResult {
        // Check if the name is a valid function name
        if !self.func_names.contains(&name) {
            return Err(ParseError::InvalidExpression);
        }
        
        let mut next_token = tokenizer.next();

        if next_token != Token::OpenBracket {
            return Err(ParseError::UnexpectedToken {
                expected: "OpenBracket".to_string(),
                found: format!("{:?}", next_token),
            });
        }

        let mut args = Vec::new();
        while next_token != Token::CloseBracket {
            let arg = self.parse_expression(tokenizer)?;
            args.push(arg);
            next_token = tokenizer.next();
            if next_token != Token::Comma && next_token != Token::CloseBracket {
                return Err(ParseError::UnexpectedToken {
                    expected: "Comma or CloseBracket".to_string(),
                    found: format!("{:?}", next_token),
                });
            }
        }

        Ok(Node::Function {
            name,
            args,
        })
    }

    fn is_cell_ref(&self, chars: &'a [char]) -> bool {
        let mut has_column = false;
        let mut has_row = false;
    
        for &c in chars {
            match c {
                'A'..='Z' | 'a'..='z' if !has_row => has_column = true,
                '0'..='9' if has_column => has_row = true,
                _ => return false,  // Invalid character or sequence
            }
        }
    
        has_column && has_row  // Must have both column and row parts
    }

    fn is_numeric(&self, chars: &'a [char]) -> bool {
        for c in chars {
            if !c.is_numeric() {
                return false
            }
        }
        true
    }

    fn is_string_literal(&self, chars: &'a [char]) -> bool {
        return chars.len() > 0 && chars[0] == '"'
    }

    fn is_boolean(&self, chars: &'a [char]) -> bool {
        let as_string: String = chars.iter().collect();
        return as_string == "TRUE" || as_string == "FALSE"
    }

    fn expect(&self, tokenizer: &mut Tokenizer, expected: Token, result: Node) -> ParseResult {
        let next_token = tokenizer.next();
        if next_token == expected {
            Ok(result)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: format!("{:?}", next_token),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_tokens() {
        let input = "+-*/".chars().collect::<Vec<_>>();
        let mut tokenizer = Tokenizer::new(&input);

        assert_eq!(tokenizer.next(), Token::Operator('+'));
        assert_eq!(tokenizer.next(), Token::Operator('-'));
        assert_eq!(tokenizer.next(), Token::Operator('*'));
        assert_eq!(tokenizer.next(), Token::Operator('/'));
        assert_eq!(tokenizer.next(), Token::EndOfFile);
    }

    #[test]
    fn test_bracket_tokens() {
        let input = "()".chars().collect::<Vec<_>>();
        let mut tokenizer = Tokenizer::new(&input);

        assert_eq!(tokenizer.next(), Token::OpenBracket);
        assert_eq!(tokenizer.next(), Token::CloseBracket);
        assert_eq!(tokenizer.next(), Token::EndOfFile);
    }

    #[test]
    fn test_text_token() {
        let input = "hello".chars().collect::<Vec<_>>();
        let mut tokenizer = Tokenizer::new(&input);

        assert_eq!(tokenizer.next(), Token::Text(&input));
        assert_eq!(tokenizer.next(), Token::EndOfFile);
    }

    #[test]
    fn test_mixed_tokens() {
        let input = "(A1:B2)".chars().collect::<Vec<_>>();
        let mut tokenizer = Tokenizer::new(&input);
        assert_eq!(tokenizer.next(), Token::OpenBracket);
        assert_eq!(tokenizer.next(), Token::Text(&['A', '1']));
        assert_eq!(tokenizer.next(), Token::RangeDelimiter);
        assert_eq!(tokenizer.next(), Token::Text(&['B', '2']));
        assert_eq!(tokenizer.next(), Token::CloseBracket);
        assert_eq!(tokenizer.next(), Token::EndOfFile);
    }

    #[test]
    fn test_functions() {
        let input = "SUM(A1:C1)".chars().collect::<Vec<_>>();
        let mut tokenizer = Tokenizer::new(&input);
        assert_eq!(tokenizer.next(), Token::Text(&['S', 'U', 'M']));
        assert_eq!(tokenizer.next(), Token::OpenBracket);
        assert_eq!(tokenizer.next(), Token::Text(&['A', '1']));
        assert_eq!(tokenizer.next(), Token::RangeDelimiter);
        assert_eq!(tokenizer.next(), Token::Text(&['C', '1']));
        assert_eq!(tokenizer.next(), Token::CloseBracket);
        assert_eq!(tokenizer.next(), Token::EndOfFile);
    }

    #[test]
    fn test_parser_addition() {
        // the string 
        let input = "=IF(GREATER(a1, b1), SUM(a1, b1), 0)".to_string();
        let mut func_names = HashSet::new();
        func_names.insert("MAX".to_string());
        func_names.insert("IF".to_string());
        func_names.insert("SUM".to_string());
        func_names.insert("AVERAGE".to_string());
        func_names.insert("AND".to_string());
        func_names.insert("NOT".to_string());
        func_names.insert("OR".to_string());
        func_names.insert("GREATER".to_string());
        let mut parser = Ast::new(&input, func_names);
        let ast = parser.parse().unwrap();
        println!("{:#?}", ast);
    }
    // Additional tests for other scenarios (TextQualifier, RangeDelimiter, etc.)
}

// this 