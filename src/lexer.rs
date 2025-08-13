use crate::escape::unescape;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    Reserved,
    Ident,
    Num(i32),
    String(String),
    Char(char),
    Eof,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer<'src> {
    source: &'src str,
    cursor: usize,
}

fn is_ident_first(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_follow(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { source, cursor: 0 }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        'outer: while self.cursor < self.source.len() {
            let c = self.source[self.cursor..].chars().next().unwrap();

            if self.source[self.cursor..].starts_with("//") {
                self.cursor += 2;
                while !self.source[self.cursor..].starts_with('\n') {
                    self.cursor += 1;
                }

                continue;
            }

            if self.source[self.cursor..].starts_with("/*") {
                self.cursor += 2;
                if let Some(offset) = self.source[self.cursor..].find("*/") {
                    self.cursor += offset + 2;
                } else {
                    panic!("unclosed block comment");
                }

                continue;
            }

            if c.is_whitespace() {
                self.cursor += 1;
                continue;
            }

            for keyword in [
                "return", "if", "else", "for", "while", "int", "sizeof", "char",
            ] {
                if let Some(rest) = self.source[self.cursor..].strip_prefix(keyword) {
                    if rest.is_empty() || !is_ident_follow(rest.chars().next().unwrap()) {
                        tokens.push(Token {
                            kind: TokenKind::Reserved,
                            span: Span {
                                lo: self.cursor,
                                hi: self.cursor + keyword.len(),
                            },
                        });
                        self.cursor += keyword.len();
                        continue 'outer;
                    }
                }
            }

            for punct in [
                "==", "!=", "<=", ">=", "||", "&&", "+", "-", "*", "/", "{", "}", "(", ")", "<",
                ">", ";", "=", "&", ",", "[", "]", "%",
            ] {
                if self.source[self.cursor..].starts_with(punct) {
                    tokens.push(Token {
                        kind: TokenKind::Reserved,
                        span: Span {
                            lo: self.cursor,
                            hi: self.cursor + punct.len(),
                        },
                    });
                    self.cursor += punct.len();
                    continue 'outer;
                }
            }

            if c.is_ascii_digit() {
                let start = self.cursor;
                while self.cursor < self.source.len()
                    && self.source[self.cursor..]
                        .chars()
                        .next()
                        .unwrap()
                        .is_ascii_digit()
                {
                    self.cursor += 1;
                }

                tokens.push(Token {
                    kind: TokenKind::Num(
                        self.source[start..self.cursor]
                            .parse::<i32>()
                            .expect("数字へのパースに失敗"),
                    ),
                    span: Span {
                        lo: start,
                        hi: self.cursor,
                    },
                });
                continue;
            }

            if c == '"' {
                let start = self.cursor;
                self.cursor += 1;

                while let Some(c) = self.source[self.cursor..].chars().next() {
                    if self.source[self.cursor..].starts_with("\\\\")
                        || self.source[self.cursor..].starts_with("\\\"")
                    {
                        self.cursor += 2;
                        continue;
                    }

                    if c == '"' {
                        break;
                    }

                    self.cursor += 1;
                }

                tokens.push(Token {
                    kind: TokenKind::String(unescape(&self.source[(start + 1)..self.cursor])),
                    span: Span {
                        lo: start,
                        hi: (self.cursor + 1),
                    },
                });

                self.cursor += 1;
                continue;
            }

            if c == '\'' {
                let start = self.cursor;
                self.cursor += 1;

                while let Some(c) = self.source[self.cursor..].chars().next() {
                    if self.source[self.cursor..].starts_with(r"\'") {
                        self.cursor += 2;
                        continue;
                    }

                    if c == '\'' {
                        break;
                    }

                    self.cursor += 1;
                }

                tokens.push(Token {
                    kind: TokenKind::Char(
                        unescape(&self.source[(start + 1)..self.cursor])
                            .chars()
                            .next()
                            .unwrap(),
                    ),
                    span: Span {
                        lo: start,
                        hi: (self.cursor + 1),
                    },
                });

                self.cursor += 1;
                continue;
            }

            if is_ident_first(c) {
                let start = self.cursor;
                self.cursor += 1;

                while let Some(ch) = self.source[self.cursor..].chars().next() {
                    if !is_ident_follow(ch) {
                        break;
                    }
                    self.cursor += 1;
                }

                tokens.push(Token {
                    kind: TokenKind::Ident,
                    span: Span {
                        lo: start,
                        hi: self.cursor,
                    },
                });
                continue;
            }

            panic!("トークナイズできません: {}", &self.source[self.cursor..]);
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span {
                lo: self.source.len(),
                hi: self.source.len(),
            },
        });

        tokens
    }
}
