use std::collections::HashSet;
use std::fmt;

use crate::model::{Flag, FlagFile};

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.message)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Str(String),
    Int(i64),
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Equals,
    Comma,
    Percent,
    Eof,
}

#[derive(Debug, Clone)]
struct Token {
    tok: Tok,
    line: usize,
    col: usize,
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic()
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'
}

fn tokenize(src: &str) -> Result<Vec<Token>, ParseError> {
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0usize;
    let mut line = 1usize;
    let mut col = 1usize;
    let mut tokens = Vec::new();

    macro_rules! advance {
        () => {{
            if chars[i] == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            i += 1;
        }};
    }

    while i < chars.len() {
        let c = chars[i];

        if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                advance!();
            }
            continue;
        }
        if c.is_whitespace() {
            advance!();
            continue;
        }

        let start_line = line;
        let start_col = col;

        match c {
            '{' => {
                tokens.push(Token { tok: Tok::LBrace, line: start_line, col: start_col });
                advance!();
            }
            '}' => {
                tokens.push(Token { tok: Tok::RBrace, line: start_line, col: start_col });
                advance!();
            }
            '[' => {
                tokens.push(Token { tok: Tok::LBracket, line: start_line, col: start_col });
                advance!();
            }
            ']' => {
                tokens.push(Token { tok: Tok::RBracket, line: start_line, col: start_col });
                advance!();
            }
            '=' => {
                tokens.push(Token { tok: Tok::Equals, line: start_line, col: start_col });
                advance!();
            }
            ',' => {
                tokens.push(Token { tok: Tok::Comma, line: start_line, col: start_col });
                advance!();
            }
            '%' => {
                tokens.push(Token { tok: Tok::Percent, line: start_line, col: start_col });
                advance!();
            }
            '"' => {
                advance!();
                let mut s = String::new();
                loop {
                    if i >= chars.len() {
                        return Err(ParseError {
                            line: start_line,
                            col: start_col,
                            message: "unterminated string".to_string(),
                        });
                    }
                    let c2 = chars[i];
                    if c2 == '"' {
                        advance!();
                        break;
                    }
                    if c2 == '\\' {
                        advance!();
                        if i >= chars.len() {
                            return Err(ParseError {
                                line: start_line,
                                col: start_col,
                                message: "unterminated string".to_string(),
                            });
                        }
                        let esc = chars[i];
                        let mapped = match esc {
                            'n' => '\n',
                            't' => '\t',
                            '"' => '"',
                            '\\' => '\\',
                            other => {
                                return Err(ParseError {
                                    line,
                                    col,
                                    message: format!("unknown escape sequence '\\{}'", other),
                                });
                            }
                        };
                        s.push(mapped);
                        advance!();
                        continue;
                    }
                    s.push(c2);
                    advance!();
                }
                tokens.push(Token { tok: Tok::Str(s), line: start_line, col: start_col });
            }
            c if c.is_ascii_digit() => {
                let mut num = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    num.push(chars[i]);
                    advance!();
                }
                let value: i64 = num.parse().map_err(|_| ParseError {
                    line: start_line,
                    col: start_col,
                    message: format!("invalid number '{}'", num),
                })?;
                tokens.push(Token { tok: Tok::Int(value), line: start_line, col: start_col });
            }
            c if is_ident_start(c) => {
                let mut ident = String::new();
                while i < chars.len() && is_ident_char(chars[i]) {
                    ident.push(chars[i]);
                    advance!();
                }
                tokens.push(Token { tok: Tok::Ident(ident), line: start_line, col: start_col });
            }
            other => {
                return Err(ParseError {
                    line: start_line,
                    col: start_col,
                    message: format!("unexpected character '{}'", other),
                });
            }
        }
    }

    tokens.push(Token { tok: Tok::Eof, line, col });
    Ok(tokens)
}

fn is_valid_flag_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(is_ident_char)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        let t = self.peek();
        ParseError { line: t.line, col: t.col, message: message.into() }
    }

    fn error_at(&self, tok: &Token, message: impl Into<String>) -> ParseError {
        ParseError { line: tok.line, col: tok.col, message: message.into() }
    }

    fn expect(&mut self, expected: Tok) -> Result<Token, ParseError> {
        let t = self.advance();
        if std::mem::discriminant(&t.tok) == std::mem::discriminant(&expected) {
            Ok(t)
        } else {
            Err(self.error_at(&t, format!("expected {:?}, found {:?}", expected, t.tok)))
        }
    }

    fn parse_flag(&mut self, seen: &mut HashSet<String>) -> Result<Flag, ParseError> {
        let kw = self.advance();
        match kw.tok {
            Tok::Ident(ref s) if s == "flag" => {}
            _ => return Err(self.error_at(&kw, "expected 'flag'")),
        }

        let name_tok = self.advance();
        let name = match name_tok.tok {
            Tok::Ident(ref s) => s.clone(),
            _ => return Err(self.error_at(&name_tok, "expected flag name")),
        };
        if !is_valid_flag_name(&name) {
            return Err(self.error_at(
                &name_tok,
                format!(
                    "invalid flag name '{}': must start with a letter and contain only letters, digits, '.', '_' or '-'",
                    name
                ),
            ));
        }
        if !seen.insert(name.clone()) {
            return Err(self.error_at(&name_tok, format!("duplicate flag name '{}'", name)));
        }

        self.expect(Tok::LBrace)?;

        let mut enabled: Option<bool> = None;
        let mut description: Option<String> = None;
        let mut rollout: Option<u8> = None;
        let mut tags: Vec<String> = Vec::new();

        while self.peek().tok != Tok::RBrace {
            if self.peek().tok == Tok::Eof {
                return Err(self.error("unexpected end of file inside flag block"));
            }
            let field_tok = self.advance();
            let field_name = match field_tok.tok {
                Tok::Ident(ref s) => s.clone(),
                _ => return Err(self.error_at(&field_tok, "expected field name")),
            };
            self.expect(Tok::Equals)?;

            match field_name.as_str() {
                "enabled" => {
                    let v = self.advance();
                    enabled = Some(match v.tok {
                        Tok::Ident(ref s) if s == "true" => true,
                        Tok::Ident(ref s) if s == "false" => false,
                        _ => return Err(self.error_at(&v, "expected 'true' or 'false'")),
                    });
                }
                "description" => {
                    let v = self.advance();
                    match v.tok {
                        Tok::Str(s) => description = Some(s),
                        _ => return Err(self.error_at(&v, "expected a string")),
                    }
                }
                "rollout" => {
                    let v = self.advance();
                    let n = match v.tok {
                        Tok::Int(n) => n,
                        _ => {
                            return Err(self.error_at(
                                &v,
                                "expected an integer percentage, e.g. rollout = 25%",
                            ))
                        }
                    };
                    self.expect(Tok::Percent)?;
                    if !(0..=100).contains(&n) {
                        return Err(self.error_at(
                            &v,
                            format!("rollout must be between 0 and 100, got {}", n),
                        ));
                    }
                    rollout = Some(n as u8);
                }
                "tags" => {
                    self.expect(Tok::LBracket)?;
                    if self.peek().tok != Tok::RBracket {
                        loop {
                            let t = self.advance();
                            let tag = match t.tok {
                                Tok::Ident(ref s) => s.clone(),
                                _ => return Err(self.error_at(&t, "expected a tag name")),
                            };
                            if tags.contains(&tag) {
                                return Err(self.error_at(&t, format!("duplicate tag '{}'", tag)));
                            }
                            tags.push(tag);
                            if self.peek().tok == Tok::Comma {
                                self.advance();
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(Tok::RBracket)?;
                }
                other => {
                    return Err(self.error_at(&field_tok, format!("unknown field '{}'", other)));
                }
            }
        }
        self.expect(Tok::RBrace)?;

        let enabled = enabled.ok_or_else(|| {
            self.error_at(&name_tok, format!("flag '{}' is missing required field 'enabled'", name))
        })?;

        Ok(Flag { name, enabled, description, rollout, tags })
    }
}

pub fn parse(src: &str) -> Result<FlagFile, ParseError> {
    let tokens = tokenize(src)?;
    let mut parser = Parser { tokens, pos: 0 };
    let mut seen = HashSet::new();
    let mut flags = Vec::new();

    while parser.peek().tok != Tok::Eof {
        flags.push(parser.parse_flag(&mut seen)?);
    }

    Ok(FlagFile { flags })
}
