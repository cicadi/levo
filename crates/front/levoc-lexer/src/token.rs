#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    Normal,
    OuterDoc,
    InnerDoc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hexadecimal = 16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LitKind {
    Int { base: Base, empty: bool },
    Float { base: Base, exp_empty: bool },

    Char { terminated: bool },
    Str { terminated: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Punc {
    Plus,  // +
    Minus, // -
    Star,  // *
    Slash, // /
    Perc,  // %

    Amp,   // &
    Bar,   // |
    Caret, // ^
    Bang,  // !

    Eq, // =
    Lt, // <
    Gt, // >

    Dot,   // .
    Comma, // ,
    Colon, // :
    Semi,  // ;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delim {
    Paren, // ( )
    Brack, // [ ]
    Brace, // { }
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub len: usize,
    pub kind: TokenKind,
}

impl Token {
    pub fn new(kind: TokenKind, len: usize) -> Self {
        Self { len, kind }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Unknown,

    Space,
    Newline,

    LineComment { kind: CommentKind },
    BlockComment { kind: CommentKind, terminated: bool },

    Ident,
    Lit { kind: LitKind },

    Punc(Punc),
    Open(Delim),
    Close(Delim),
}
