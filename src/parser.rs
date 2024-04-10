use crate::lexer::{self, Token};


#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken(lexer::Token, lexer::Token),
    UnexpectedToken2(Vec<lexer::Token>, lexer::Token),
    TooFewTokens,
    LexingError(lexer::LexingError)
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken(expected, found) =>
                write!(f, "Unexpected Token: Expected {:?}. Found {:?}.", expected, found),
            Self::UnexpectedToken2(expected, found ) =>
                write!(f, "Unexpected Token: Expected {:?}. Found {:?}.", expected, found),
            Self::TooFewTokens =>
                write!(f, "Too few tokens"),
            Self::LexingError(e) =>
            write!(f, "Lexing error: {:?}", e),
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
pub type VariableName = String;
pub type Arguments = Vec<Expression>;
pub type Block = Vec<Statement>;


#[derive(Debug, PartialEq)]
pub struct Assignment {
    pub variable_name: VariableName,
    pub expression: Expression
}

#[derive(Debug, PartialEq)]
pub struct IfStatement {
    pub condition: Expression,
    pub block: Block
}

#[derive(Debug, PartialEq)]
pub struct WhileLoop {
    pub condition: Expression,
    pub block: Block
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    pub name: FunctionIdentifier,
    pub arguments: Arguments
}


#[derive(Debug, PartialEq)]
pub enum Expression {
    Number(i64),
    Variable(VariableName),
    Addition(Box<(Expression, Expression)>),
    Subtraction(Box<(Expression, Expression)>),
    Multiplication(Box<(Expression, Expression)>),
    Division(Box<(Expression, Expression)>),
    Modulo(Box<(Expression, Expression)>),
    Greater(Box<(Expression, Expression)>),
    GreaterEquals(Box<(Expression, Expression)>),
    Less(Box<(Expression, Expression)>),
    LessEquals(Box<(Expression, Expression)>),
    Equals(Box<(Expression, Expression)>),
    NotEquals(Box<(Expression, Expression)>),
    LogicAnd(Box<(Expression, Expression)>),
    LogicOr(Box<(Expression, Expression)>),
    FunctionCall(FunctionCall)
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    IfStatement(IfStatement),
    WhileLoop(WhileLoop),
    FunctionCall(FunctionCall)
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: FunctionIdentifier,
    pub parameters: Parameters,
    pub block: Block
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>
}

fn get_identifier(token: lexer::Token) -> ParseResult<String>{
    match token {
        Token::Identifier(v) => Ok(v),
        _ => Err(ParseError::UnexpectedToken(lexer::Token::Identifier("".to_owned()), token))
    }
}

fn check_token(token: lexer::Token, expected_token: lexer::Token)-> ParseResult<()> {
    if token != expected_token {
        return Err(ParseError::UnexpectedToken(expected_token, token));
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

fn parse_expression_p1(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Expression>{
    let token = get_peak_token(lex.peek())?;
    match token {
        Token::Identifier(id) => {
            let mut tmp_lex = lex.clone();
            tmp_lex.next();
            let next_token = get_token(tmp_lex.next())?;
            return match next_token {
                Token::OpeningRoundBracket => Ok(Expression::FunctionCall(parse_function_call(lex)?)),
                _ => {
                    lex.next();
                    Ok(Expression::Variable(id))
                }
            };
        },
        Token::Number(n) => {
            lex.next();
            Ok(Expression::Number(n))
        },
        Token::OpeningRoundBracket => {
            lex.next();
            let expr = parse_expression_p3(lex)?;
            let closing_token = get_token(lex.next())?;
            if closing_token != Token::ClosingRoundBracket {
                return Err(ParseError::UnexpectedToken(Token::ClosingRoundBracket, closing_token));
            }
            return Ok(expr);
        },
        t => Err(ParseError::UnexpectedToken2(vec![Token::Identifier("".to_owned()), Token::Number(0), Token::OpeningRoundBracket], t))
    }
}

fn parse_expression_p2(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Expression>{
    let mut left_side = parse_expression_p1(lex)?;
    loop {
        let token = get_peak_token(lex.peek())?;
        match token {
            Token::Multiplication => {
                lex.next();
                let right_side = parse_expression_p1(lex)?;
                left_side = Expression::Multiplication(Box::new((left_side, right_side)));
            },
            Token::Division => {
                lex.next();
                let right_side = parse_expression_p1(lex)?;
                left_side = Expression::Division(Box::new((left_side, right_side)));
            },
            Token::Modulo => {
                lex.next();
                let right_side = parse_expression_p1(lex)?;
                left_side = Expression::Modulo(Box::new((left_side, right_side)));
            },
            _ => return Ok(left_side)
        }
    }
}

fn parse_expression_p3(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Expression>{
    let mut left_side = parse_expression_p2(lex)?;
    loop {
        let token = get_peak_token(lex.peek())?;
        match token {
            Token::Addition => {
                lex.next();
                let right_side = parse_expression_p2(lex)?;
                left_side = Expression::Addition(Box::new((left_side, right_side)));
            },
            Token::Subtraction => {
                lex.next();
                let right_side = parse_expression_p2(lex)?;
                left_side = Expression::Subtraction(Box::new((left_side, right_side)));
            },
            _ => return Ok(left_side)
        }
    }
}

fn parse_expression_p4(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Expression>{
    let mut left_side = parse_expression_p3(lex)?;
    loop {
        let token = get_peak_token(lex.peek())?;
        match token {
            Token::Greater => {
                lex.next();
                let right_side = parse_expression_p3(lex)?;
                left_side = Expression::Greater(Box::new((left_side, right_side)));
            },
            Token::GreaterEquals => {
                lex.next();
                let right_side = parse_expression_p3(lex)?;
                left_side = Expression::GreaterEquals(Box::new((left_side, right_side)));
            },
            Token::Less => {
                lex.next();
                let right_side = parse_expression_p3(lex)?;
                left_side = Expression::Less(Box::new((left_side, right_side)));
            },
            Token::LessEquals => {
                lex.next();
                let right_side = parse_expression_p3(lex)?;
                left_side = Expression::LessEquals(Box::new((left_side, right_side)));
            },
            _ => return Ok(left_side)
        }
    }
}


fn parse_expression_p5(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Expression>{
    let mut left_side = parse_expression_p4(lex)?;
    loop {
        let token = get_peak_token(lex.peek())?;
        match token {
            Token::Equals => {
                lex.next();
                let right_side = parse_expression_p4(lex)?;
                left_side = Expression::Equals(Box::new((left_side, right_side)));
            },
            Token::NotEquals => {
                lex.next();
                let right_side = parse_expression_p4(lex)?;
                left_side = Expression::NotEquals(Box::new((left_side, right_side)));
            },
            _ => return Ok(left_side)
        }
    }
}

fn parse_expression_p6(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Expression>{
    let left_side = parse_expression_p5(lex)?;
    let token = get_peak_token(lex.peek())?;
    match token {
        Token::LogicAnd => {
            lex.next();
            let right_side = parse_expression_p5(lex)?;
            Ok(Expression::LogicAnd(Box::new((left_side, right_side))))
        },
        _ => Ok(left_side)
    }
}

fn parse_expression_p7(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Expression>{
    let left_side = parse_expression_p6(lex)?;
    let token = get_peak_token(lex.peek())?;
    match token {
        Token::LogicOr => {
            lex.next();
            let right_side = parse_expression_p6(lex)?;
            Ok(Expression::LogicOr(Box::new((left_side, right_side))))
        },
        _ => Ok(left_side)
    }
}

fn parse_expression(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Expression>{
    parse_expression_p7(lex)
}

fn parse_if_statement(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Statement>{
    check_token(get_token(lex.next())?, Token::IfStatement)?;
    check_token(get_token(lex.next())?, Token::OpeningRoundBracket)?;
    let condition = parse_expression(lex)?;
    check_token(get_token(lex.next())?, Token::ClosingRoundBracket)?;
    check_token(get_token(lex.next())?, Token::OpeningCurlyBracket)?;
    let block = parse_block(lex)?;
    check_token(get_token(lex.next())?, Token::ClosingCurlyBracket)?;
    Ok(Statement::IfStatement(IfStatement { condition, block }))
}

fn parse_while_loop(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Statement>{
    check_token(get_token(lex.next())?, Token::WhileLoop)?;
    check_token(get_token(lex.next())?, Token::OpeningRoundBracket)?;
    let condition = parse_expression(lex)?;
    check_token(get_token(lex.next())?, Token::ClosingRoundBracket)?;
    check_token(get_token(lex.next())?, Token::OpeningCurlyBracket)?;
    let block = parse_block(lex)?;
    check_token(get_token(lex.next())?, Token::ClosingCurlyBracket)?;
    Ok(Statement::WhileLoop(WhileLoop { condition, block }))
}

fn parse_arguments(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Arguments>{
    let mut arguments:Arguments = Vec::new();
    loop {
        let token = get_peak_token(lex.peek())?;
        match token {
            Token::ClosingRoundBracket => return Ok(arguments),
            _ => {
                let expression = parse_expression(lex)?;
                let next_token = get_peak_token(lex.peek())?;
                match next_token {
                    Token::Comma => lex.next(),
                    Token::ClosingRoundBracket => None,
                    _ => return Err(ParseError::UnexpectedToken2(vec![Token::Comma, Token::ClosingRoundBracket], token))
                };
                arguments.push(expression);
            }
        }
    }
}


fn parse_function_call(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<FunctionCall>{
    let token = get_token(lex.next())?;
    let name = match token {
        Token::Identifier(id) => id,
        _ => return Err(ParseError::UnexpectedToken(Token::Identifier("".to_owned()), token))
    };
    check_token(get_token(lex.next())?, Token::OpeningRoundBracket)?;
    let arguments = parse_arguments(lex)?;
    check_token(get_token(lex.next())?, Token::ClosingRoundBracket)?;
    Ok(FunctionCall { name, arguments })
}

fn parse_function_call_statement(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Statement>{
    let function_call = parse_function_call(lex)?;
    check_token(get_token(lex.next())?, Token::Semicolon)?;
    Ok(Statement::FunctionCall(function_call))
}


fn parse_statement(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Statement>{
    let token = get_peak_token(lex.peek())?;
    match token {
        Token::Identifier(_) => {
            let mut tmp_lex = lex.clone();
            tmp_lex.next();
            let next_token = get_token(tmp_lex.next())?;
            match next_token {
                Token::Assignment => return parse_assignment(lex),
                Token::OpeningRoundBracket => parse_function_call_statement(lex),
                //expect assignment or function call
                _ => return Err(ParseError::UnexpectedToken2(vec![Token::Assignment, Token::OpeningRoundBracket], next_token))
            }
        },
        Token::IfStatement => parse_if_statement(lex),
        Token::WhileLoop => parse_while_loop(lex),
        _ => return Err(ParseError::UnexpectedToken2(vec![Token::Identifier("".to_owned()), Token::IfStatement, Token::WhileLoop], token))
    }
}

fn parse_block(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Block>{
    let mut block: Block = Vec::new();
    loop {
        let token = get_peak_token(lex.peek())?;
        match token {
            Token::ClosingCurlyBracket => return Ok(block),
            _ => {
                let statement = parse_statement(lex)?;
                block.push(statement);
            }
        };
    }
}

fn parse_assignment(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Statement>{
    let variable_name = get_identifier(get_token(lex.next())?)?;
    check_token(get_token(lex.next())?, lexer::Token::Assignment)?;
    let expression = parse_expression(lex)?;
    check_token(get_token(lex.next())?, lexer::Token::Semicolon)?;
    Ok(Statement::Assignment(Assignment { variable_name, expression }))
}

fn parse_function(lex: &mut std::iter::Peekable<logos::Lexer<'_, lexer::Token>>) -> ParseResult<Function>{
    check_token(get_token(lex.next())?, lexer::Token::Function)?;
    let name = get_identifier(get_token(lex.next())?)?;
    check_token(get_token(lex.next())?, lexer::Token::OpeningRoundBracket)?;
    let parameters = parse_argument(lex)?;
    check_token(get_token(lex.next())?, lexer::Token::ClosingRoundBracket)?;
    check_token(get_token(lex.next())?, lexer::Token::OpeningCurlyBracket)?;
    let block = parse_block(lex)?;
    check_token(get_token(lex.next())?, lexer::Token::ClosingCurlyBracket)?;
    Ok(Function { name, parameters, block})
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

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    #[test]
    fn parser_expression_and_or_1() {
        let code = "1 && 2 || 3;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::LogicOr(Box::new((Expression::LogicAnd(Box::new((Expression::Number(1), Expression::Number(2)))), Expression::Number(3))))))
    }

    #[test]
    fn parser_expression_and_or_2() {
        let code = "3 || 1 && 2;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::LogicOr(Box::new((Expression::Number(3), Expression::LogicAnd(Box::new((Expression::Number(1), Expression::Number(2)))))))))
    }

    #[test]
    fn parser_expression_or() {
        let code = "1 || 2;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::LogicOr(Box::new((Expression::Number(1), Expression::Number(2))))))
    }

    #[test]
    fn parser_expression_and() {
        let code = "1 && 2;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::LogicAnd(Box::new((Expression::Number(1), Expression::Number(2))))))
    }

    #[test]
    fn parser_expression_equals_and_not_equals() {
        let code = "1 == 2 != 3;";
        let mut lex = lexer::Token::lexer(code).peekable();
        //println!("{:?}", parse_expression(&mut lex))
        assert_eq!(parse_expression(&mut lex), Ok(Expression::NotEquals(Box::new((Expression::Equals(Box::new((Expression::Number(1), Expression::Number(2)))), Expression::Number(3))))))
    }

    #[test]
    fn parser_expression_equals() {
        let code = "1 == 2;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::Equals(Box::new((Expression::Number(1), Expression::Number(2))))))
    }

    #[test]
    fn parser_expression_less_and_greater() {
        let code = "1 < 2 > 3;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::Greater(Box::new((Expression::Less(Box::new((Expression::Number(1), Expression::Number(2)))), Expression::Number(3))))))
    }

    #[test]
    fn parser_expression_less() {
        let code = "1 < 2;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::Less(Box::new((Expression::Number(1), Expression::Number(2))))))
    }


    #[test]
    fn parser_prog_1() {
        let code = "
        fun main(){}

        fun test1(a,b){}

        fun test2(a,b){
            a = 1 + 5;
        }
        ";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert!(parse(&mut lex).is_ok())
    }

    #[test]
    fn parser_function_with_args() {
        let code = "fun test(a, b){}";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_function(&mut lex), Ok(Function { name: "test".to_owned(), parameters: vec!["a".to_owned(), "b".to_owned()], block: vec![] }))
    }

    #[test]
    fn parser_function() {
        let code = "fun test(){}";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_function(&mut lex), Ok(Function { name: "test".to_owned(), parameters: vec![], block: vec![] }))
    }

    #[test]
    fn parser_function_call_statement_with_args() {
        let code = "test(a, 1+2);";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_function_call_statement(&mut lex), Ok(Statement::FunctionCall(FunctionCall { name: "test".to_owned(), arguments: vec![Expression::Variable("a".to_owned()), Expression::Addition(Box::new((Expression::Number(1), Expression::Number(2))))] })))
    }


    #[test]
    fn parser_function_call_expression() {
        let code = "test();";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::FunctionCall(FunctionCall { name: "test".to_owned(), arguments: vec![] })))
    }

    #[test]
    fn parser_function_call_statement() {
        let code = "test();";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_function_call_statement(&mut lex), Ok(Statement::FunctionCall(FunctionCall { name: "test".to_owned(), arguments: vec![] })))
    }

    #[test]
    fn parser_if_statement_simple() {
        let code = "if(1){}";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_if_statement(&mut lex), Ok(Statement::IfStatement(IfStatement { condition: Expression::Number(1), block: vec![] })))
    }

    #[test]
    fn parser_while_loop_simple() {
        let code = "while(1){}";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_while_loop(&mut lex), Ok(Statement::WhileLoop(WhileLoop { condition: Expression::Number(1), block: vec![] })))
    }

    #[test]
    fn parser_block() {
        let st1 = "a = 5;";
        let st2 = "b = 6 - 4;";
        let mut code = "".to_owned();
        code.push_str(st1);
        code.push_str(st2);
        code.push_str("}");
        let mut lex = lexer::Token::lexer(&code).peekable();
        let mut st1_lex = lexer::Token::lexer(st1).peekable();
        let mut st2_lex = lexer::Token::lexer(st2).peekable();
        let st1 = parse_statement(&mut st1_lex).unwrap();
        let st2 = parse_statement(&mut st2_lex).unwrap();
        assert_eq!(parse_block(&mut lex), Ok(vec![st1, st2]))
    }

    #[test]
    fn parser_statement() {
        let code = "b = 6 - 4;";
        let mut lex = lexer::Token::lexer(&code).peekable();
        assert_eq!( parse_statement(&mut lex),
            Ok(Statement::Assignment(Assignment { variable_name: "b".to_owned(), expression: Expression::Subtraction(Box::new((Expression::Number(6), Expression::Number(4)))) })))
    }

    #[test]
    fn parser_assignment() {
        let expr = "(4 - 1) * 6;";
        let mut code = "abc = ".to_owned();
        code.push_str(expr);
        let mut lex = lexer::Token::lexer(&code).peekable();
        let mut expr_lex = lexer::Token::lexer(expr).peekable();
        let expr = parse_expression(&mut expr_lex).unwrap();
        assert_eq!(parse_assignment(&mut lex), Ok(Statement::Assignment(Assignment { variable_name: "abc".to_owned(), expression: expr })))
    }

    #[test]
    fn parser_expression_additon() {
        let code = "1 + 2;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::Addition(Box::new((Expression::Number(1), Expression::Number(2))))))
    }

    #[test]
    fn parser_expression_subtraction() {
        let code = "3 - 4;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::Subtraction(Box::new((Expression::Number(3), Expression::Number(4))))))
    }

    #[test]
    fn parser_expression_multiplication() {
        let code = "5 * 6;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::Multiplication(Box::new((Expression::Number(5), Expression::Number(6))))))
    }

    #[test]
    fn parser_expression_division() {
        let code = "8 / 4;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::Division(Box::new((Expression::Number(8), Expression::Number(4))))))
    }

    #[test]
    fn parser_expression_modulo() {
        let code = "911 % 10;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::Modulo(Box::new((Expression::Number(911), Expression::Number(10))))))
    }

    #[test]
    fn parser_expression_1() {
        let code = "7 - 5 + 1;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(parse_expression(&mut lex), Ok(Expression::Addition(Box::new((Expression::Subtraction(Box::new((Expression::Number(7), Expression::Number(5)))), Expression::Number(1))))))
    }

    #[test]
    fn parser_expression_2() {
        let code = "8 / 4 * 5;";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(Ok(Expression::Multiplication(Box::new((Expression::Division(Box::new((Expression::Number(8), Expression::Number(4)))), Expression::Number(5))))), parse_expression(&mut lex))
    }

    #[test]
    fn parser_expression_3() {
        let code = "a + 2 * (b + 4);";
        let mut lex = lexer::Token::lexer(code).peekable();
        assert_eq!(
            parse_expression(&mut lex),
            Ok(Expression::Addition(
            Box::new((
                Expression::Variable("a".to_owned()),
                Expression::Multiplication(
                    Box::new((
                        Expression::Number(2),
                        Expression::Addition(
                            Box::new((
                                Expression::Variable("b".to_owned()),
                                Expression::Number(4)
                            ))
                        )
                    ))
                )
            ))
        )))
    }
}
