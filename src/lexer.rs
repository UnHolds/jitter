use logos::Logos;
use std::num::ParseIntError;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexingError {
    InvalidInteger(String),
    #[default]
    NonAsciiCharacter,
}


impl From<ParseIntError> for LexingError {
    fn from(err: ParseIntError) -> Self {
        use std::num::IntErrorKind::*;
        match err.kind() {
            PosOverflow | NegOverflow => LexingError::InvalidInteger("overflow error".to_owned()),
            _ => LexingError::InvalidInteger("other error".to_owned()),
        }
    }
}


#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(error = LexingError)]
#[logos(skip r"([ \t\n\r\f]+|(\/\/.*))")]
pub enum Token {
    #[token("{")]
    OpeningCurlyBracket,

    #[token("}")]
    ClosingCurlyBracket,

    #[token("(")]
    OpeningRoundBracket,

    #[token(")")]
    ClosingRoundBracket,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[token("if")]
    IfStatement,

    #[token("while")]
    WhileLoop,

    #[token("fun")]
    Function,

    #[regex(r"[A-z]([A-z]|[0-9])*", |lex| lex.slice().to_owned())]
    Identifier(String),

    #[token("&&")]
    LogicAnd,

    #[token("||")]
    LogicOr,

    #[token("=")]
    Assignment,

    #[token("==")]
    Equals,

    #[token("!=")]
    NotEquals,

    #[token(">")]
    Greater,

    #[token(">=")]
    GreaterEquals,

    #[token("<")]
    Less,

    #[token("<=")]
    LessEquals,

    #[token("+")]
    Addition,

    #[token("-")]
    Subtraction,

    #[token("*")]
    Multiplication,

    #[token("/")]
    Division,

    #[token("%")]
    Modulo,

    #[token("return")]
    Return,

    #[regex("-?[0-9]+", |lex| lex.slice().parse())]
    #[regex("'[ -~]'", |lex| lex.slice().as_bytes()[1] as i64)]
    Number(i64),

}

// use this function to lex the code
pub fn lex(code: &str) -> std::iter::Peekable<logos::Lexer<'_, Token>> {
    Token::lexer(code).peekable()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_curly_bracket() {
        let mut lex = Token::lexer("{}");
        assert_eq!(Token::OpeningCurlyBracket, lex.next().unwrap().unwrap());
        assert_eq!(Some(Ok(Token::ClosingCurlyBracket)), lex.next());
    }

    #[test]
    fn simple_program_1() {
        let code = "
        fun test1() {
            test2();
        }";
        let mut lex = Token::lexer(code);
        assert_eq!(Some(Ok(Token::Function)), lex.next());
        assert_eq!(Some(Ok(Token::Identifier("test1".to_owned()))), lex.next());
        assert_eq!(Some(Ok(Token::OpeningRoundBracket)), lex.next());
        assert_eq!(Some(Ok(Token::ClosingRoundBracket)), lex.next());
        assert_eq!(Some(Ok(Token::OpeningCurlyBracket)), lex.next());
        assert_eq!(Some(Ok(Token::Identifier("test2".to_owned()))), lex.next());
        assert_eq!(Some(Ok(Token::OpeningRoundBracket)), lex.next());
        assert_eq!(Some(Ok(Token::ClosingRoundBracket)), lex.next());
        assert_eq!(Some(Ok(Token::Semicolon)), lex.next());
        assert_eq!(Some(Ok(Token::ClosingCurlyBracket)), lex.next());
    }
}
