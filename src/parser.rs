use std::fmt::Arguments;

use crate::lexer::{self, Token};
use logos::Logos;
use nom::{self, Err};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken(lexer::Token, lexer::Token),
    TooFewTokens,
    LexingError(lexer::LexingError)
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken(expected, found) =>
                write!(f, "Unexpected Token: Expected {:?}. Found {:?}.", expected, found),
            Self::TooFewTokens =>
                write!(f, "Too few tokens"),
            Self::LexingError(e) =>
            write!(f, "Lexing error"),
        }
    }
}

type ParseResult<T> = Result<T, ParseError>;

fn get_token(lex_token: Option<Result<lexer::Token, lexer::LexingError>>) -> ParseResult<lexer::Token> {
    match lex_token {
        None => return Err(ParseError::TooFewTokens),
        Some(r) => match r {
            Err(e) => return Err(ParseError::LexingError(e)),
            Ok(v) => Ok(v)
        }
    }
}

fn get_peak_token(lex_token: Option<&Result<lexer::Token, lexer::LexingError>>) -> ParseResult<lexer::Token> {
    match lex_token {
        None => return Err(ParseError::TooFewTokens),
        Some(r) => match r {
            Err(e) => return Err(ParseError::LexingError(e.to_owned())),
            Ok(v) => Ok(v.to_owned())
        }
    }
}

pub type Parameter = String;
pub type FunctionIdentifier = String;
pub type Parameters = Vec<Parameter>;

#[derive(Debug, PartialEq)]
pub struct Function {
    name: FunctionIdentifier,
    parameters: Parameters
}

#[derive(Debug, PartialEq)]
pub struct Program {
    functions: Vec<Function>
}

fn get_identifier(token: lexer::Token) -> ParseResult<String>{
    match token {
        Token::Identifier(v) => Ok(v),
        _ => Err(ParseError::UnexpectedToken(lexer::Token::Identifier("".to_owned()), token))
    }
}

fn check_token(token: lexer::Token, expected_token: lexer::Token)-> ParseResult<()> {
    if token != expected_token {
        return Err(ParseError::UnexpectedToken(token, expected_token));
    }
    Ok(())
}

fn parse_argument(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Parameters>{

    let mut parameters: Parameters = Vec::new();
    loop {
        let token = get_peak_token(lex.peek())?;
        match token {
            Token::Identifier(v) => parameters.push(v),
            _ => return Ok(parameters),

        };
        lex.next();
        let res = check_token(get_peak_token(lex.peek())?, Token::Comma);
        match res {
            Err(_) => return Ok(parameters),
            Ok(_) => ()
        };
        lex.next();
    }
}

fn parse_function(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Function>{
    check_token(get_token(lex.next())?, lexer::Token::Function)?;
    let name = get_identifier(get_token(lex.next())?)?;
    check_token(get_token(lex.next())?, lexer::Token::OpeningRoundBracket)?;
    let parameters = parse_argument(lex)?;
    check_token(get_token(lex.next())?, lexer::Token::ClosingRoundBracket)?;
    check_token(get_token(lex.next())?, lexer::Token::OpeningCurlyBracket)?;
    //todo block parser
    check_token(get_token(lex.next())?, lexer::Token::ClosingCurlyBracket)?;
    Ok(Function { name: name, parameters: parameters})
}

pub fn parse(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Program> {
    let mut functions: Vec<Function> = Vec::new();
    loop {
        if lex.peek().is_none() {
            return Ok(Program {functions: functions});
        }
        functions.push(parse_function(lex)?);
    }
}



pub fn test() {
    let code = "
        fun test1(test,test2) {
        }";
    let mut lex = lexer::Token::lexer(code).peekable();
    println!("{:?}", parse(&mut lex));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_simple() {

    }
}
