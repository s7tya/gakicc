#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Reserved,
    Ident,
    Num(i32),
    Eof,
}

#[derive(Debug)]
pub struct Token<'src> {
    pub kind: TokenKind,
    pub raw_str: &'src str,
}

pub struct Lexer<'src> {
    source: &'src str,
    cursor: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { source, cursor: 0 }
    }

    pub fn lex(&mut self) -> Vec<Token<'src>> {
        let mut tokens = vec![];

        'outer: while self.cursor < self.source.len() {
            let c = self.source[self.cursor..].chars().next().unwrap();

            if c.is_whitespace() {
                self.cursor += 1;
                continue;
            }

            for punct in [
                "==", "!=", "<=", ">=", "+", "-", "*", "/", "(", ")", "<", ">", ";", "=", "return",
            ] {
                if self.source[self.cursor..].starts_with(punct) {
                    tokens.push(Token {
                        kind: TokenKind::Reserved,
                        raw_str: &self.source[self.cursor..(self.cursor + punct.len())],
                    });
                    self.cursor += punct.len();
                    continue 'outer;
                }
            }

            if c.is_ascii_alphabetic() {
                let start = self.cursor;

                self.cursor += 1;

                while let Some(ch) = self.source[self.cursor..].chars().next() {
                    if !ch.is_ascii_alphanumeric() {
                        break;
                    }
                    self.cursor += 1;
                }

                tokens.push(Token {
                    kind: TokenKind::Ident,
                    raw_str: &self.source[start..self.cursor],
                });
                continue;
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
                    kind: TokenKind::Num(self.source[start..self.cursor].parse::<i32>().unwrap()),
                    raw_str: &self.source[start..self.cursor],
                });
                continue;
            }

            panic!("トークナイズできません: {}", &self.source[self.cursor..]);
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            raw_str: "",
        });

        tokens
    }
}
