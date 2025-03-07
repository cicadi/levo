use super::cursor::Cursor;
use crate::token::{Base::*, CommentKind::*, Delim::*, LitKind::*, Punc::*, Token, TokenKind, TokenKind::*};

trait CharExt {
    fn is_newline(self) -> bool;
    fn is_space(self) -> bool;

    fn is_ident_start(self) -> bool;
    fn is_ident_continue(self) -> bool;
}

impl CharExt for char {
    fn is_newline(self) -> bool {
        matches!(self, '\n' | '\r' | '\u{0085}' | '\u{2028}' | '\u{2029}')
    }

    fn is_space(self) -> bool {
        !self.is_newline() && self.is_whitespace()
    }

    fn is_ident_start(self) -> bool {
        self == '_' || unicode_ident::is_xid_start(self)
    }

    fn is_ident_continue(self) -> bool {
        unicode_ident::is_xid_continue(self)
    }
}

impl Cursor<'_> {
    pub fn next_token(&mut self) -> Option<Token> {
        let kind = match self.bump()? {
            ch if ch.is_whitespace() => self.eat_space(ch),
            ch if ch.is_ident_start() => self.eat_ident(),
            ch if ch.is_ascii_digit() => self.eat_num_lit(ch),

            '/' => match self.peek() {
                Some('/') => self.eat_line_comment(),
                Some('*') => self.eat_block_comment(),
                _ => Punc(Slash),
            },

            '"' => self.eat_str_lit(),
            '\'' => self.eat_char_lit(),

            '+' => Punc(Plus),
            '-' => Punc(Minus),
            '*' => Punc(Star),
            '%' => Punc(Perc),

            '&' => Punc(Amp),
            '|' => Punc(Bar),
            '^' => Punc(Caret),
            '!' => Punc(Bang),

            '=' => Punc(Eq),
            '<' => Punc(Lt),
            '>' => Punc(Gt),

            '.' => Punc(Dot),
            ',' => Punc(Comma),
            ':' => Punc(Colon),
            ';' => Punc(Semi),

            '(' => Open(Paren),
            '[' => Open(Brack),
            '{' => Open(Brace),
            ')' => Close(Paren),
            ']' => Close(Brack),
            '}' => Close(Brace),

            _ => Unknown,
        };

        let token = Token::new(kind, self.pos());
        self.rebase();
        Some(token)
    }

    fn eat_space(&mut self, first: char) -> TokenKind {
        debug_assert!(self.prev().is_some_and(|ch| ch.is_whitespace()));
        if first.is_newline() {
            if first == '\r' && self.peek().is_some_and(|ch| ch == '\n') {
                _ = self.bump();
            }

            Newline
        } else {
            self.bump_while(|ch| ch.is_space());
            Space
        }
    }

    fn eat_line_comment(&mut self) -> TokenKind {
        debug_assert!(self.prev() == Some('/') && self.peek() == Some('/'));
        _ = self.bump();
        let kind = match self.peek() {
            Some('/') => OuterDoc,
            Some('!') => InnerDoc,
            _ => Normal,
        };

        self.bump_while(|ch| !ch.is_newline());
        LineComment { kind }
    }

    fn eat_block_comment(&mut self) -> TokenKind {
        debug_assert!(self.prev() == Some('/') && self.peek() == Some('*'));
        _ = self.bump();

        let kind = match self.peek() {
            Some('*') => {
                if self.peek_nth(1) == Some('/') {
                    _ = self.bump_nth(1);
                    return BlockComment { kind: Normal, terminated: true };
                } else {
                    OuterDoc
                }
            }
            Some('!') => InnerDoc,
            _ => Normal,
        };

        let mut depth: u32 = 1;
        loop {
            self.bump_while(|ch| !matches!(ch, '*' | '/'));
            match (self.bump(), self.peek()) {
                (None, _) | (_, None) => break,
                (Some('/'), Some('*')) => {
                    _ = self.bump();
                    depth += 1
                }
                (Some('*'), Some('/')) => {
                    _ = self.bump();
                    depth -= 1;

                    if depth <= 0 {
                        break;
                    }
                }
                _ => {}
            }
        }

        BlockComment { kind, terminated: depth <= 0 }
    }

    fn eat_ident(&mut self) -> TokenKind {
        debug_assert!(self.prev().is_some_and(|ch| ch.is_ident_start()));
        self.bump_while(|ch| ch.is_ident_continue());
        Ident
    }

    fn eat_num_lit(&mut self, first: char) -> TokenKind {
        debug_assert!(self.prev().is_some_and(|ch| ch.is_ascii_digit()));
        let mut base = Decimal;
        if first == '0' {
            match self.peek() {
                Some('b') => {
                    base = Binary;
                    _ = self.bump();
                    if self.eat_decimal_digits() {
                        return Lit { kind: Int { base, empty: true } };
                    }
                }
                Some('o') => {
                    base = Octal;
                    _ = self.bump();
                    if self.eat_decimal_digits() {
                        return Lit { kind: Int { base, empty: true } };
                    }
                }
                Some('x') => {
                    base = Hexadecimal;
                    _ = self.bump();
                    if self.eat_hexadecimal_digits() {
                        return Lit { kind: Int { base, empty: true } };
                    }
                }
                _ => _ = self.eat_decimal_digits(),
            }
        } else {
            _ = self.eat_decimal_digits();
        }

        let kind = match self.peek() {
            Some('.') if self.peek_nth(1).is_some_and(|ch| ch.is_ascii_digit()) => {
                _ = self.bump();
                _ = self.eat_decimal_digits();
                match self.peek() {
                    Some('e' | 'E') => {
                        _ = self.bump();
                        Float { base, exp_empty: self.eat_float_lit_exp() }
                    }
                    _ => Float { base, exp_empty: false },
                }
            }
            Some('e' | 'E') => {
                _ = self.bump();
                Float { base, exp_empty: self.eat_float_lit_exp() }
            }
            _ => Int { base, empty: false },
        };

        Lit { kind }
    }

    fn eat_decimal_digits(&mut self) -> bool {
        let mut is_empty = true;
        while let Some(ch) = self.peek() {
            match ch {
                ch if ch.is_ascii_digit() => {
                    _ = self.bump();
                    is_empty = false;
                }
                '_' => _ = self.bump(),

                _ => break,
            }
        }

        is_empty
    }

    fn eat_hexadecimal_digits(&mut self) -> bool {
        let mut is_empty = true;
        while let Some(ch) = self.peek() {
            match ch {
                ch if ch.is_ascii_hexdigit() => {
                    _ = self.bump();
                    is_empty = false;
                }
                '_' => _ = self.bump(),

                _ => break,
            }
        }

        is_empty
    }

    fn eat_float_lit_exp(&mut self) -> bool {
        debug_assert!(self.prev().is_some_and(|ch| matches!(ch, 'e' | 'E')));
        if matches!(self.peek(), Some('+' | '-')) {
            _ = self.bump()
        }

        self.eat_decimal_digits()
    }

    fn eat_char_lit(&mut self) -> TokenKind {
        debug_assert!(matches!(self.prev(), Some('\'')));
        if self.peek_nth(1) == Some('\'') && self.peek() != Some('\\') {
            self.bump_nth(1);
            return Lit { kind: Char { terminated: true } };
        }

        let terminated = loop {
            if let Some(ch) = self.peek() {
                match ch {
                    '\\' => _ = self.bump_nth(1),
                    '\n' => break false,
                    '\'' => {
                        _ = self.bump();
                        break true;
                    }

                    _ => _ = self.bump(),
                }
            } else {
                break false;
            }
        };

        Lit { kind: Char { terminated } }
    }

    fn eat_str_lit(&mut self) -> TokenKind {
        debug_assert!(matches!(self.prev(), Some('"')));
        let terminated = loop {
            if let Some(ch) = self.peek() {
                match ch {
                    '\\' => _ = self.bump_nth(1),
                    '\n' => break false,
                    '"' => {
                        _ = self.bump();
                        break true;
                    }

                    _ => _ = self.bump(),
                }
            } else {
                break false;
            }
        };

        Lit { kind: Str { terminated } }
    }
}
