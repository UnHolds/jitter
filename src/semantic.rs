use crate::parser::{self, Expression, Function, Program};

#[derive(Debug, PartialEq)]
pub enum SemanticError {
    DuplicateParameter(String),
    DuplicateFunction(String),
    VariableUsedBeforeInit
}

type SemanticResult = Result<(), SemanticError>;

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateParameter(fun) =>
            write!(f, "duplicated parametes in function: {}", fun),
            Self::DuplicateFunction(fun) =>
            write!(f, "duplicated function: {}", fun),
            Self::VariableUsedBeforeInit =>
            write!(f, "variable used before init"),
        }
    }
}

fn check_duplicate_parameters(function: &parser::Function) -> SemanticResult {
    let mut uniq = std::collections::HashSet::new();
    if (&function.parameters).into_iter().all(move |x| uniq.insert(x)) {
        Ok(())
    }else{
        Err(SemanticError::DuplicateParameter(function.name.to_owned()))
    }
}

fn check_duplicate_functions(program: &parser::Program) -> SemanticResult {
    let mut uniq = std::collections::HashSet::new();
    for function in &program.functions {
        if uniq.insert(function.name.to_owned()) == false {
            return Err(SemanticError::DuplicateFunction(function.name.to_owned()))
        }
    }
    Ok(())
}

fn unbox<T>(value: &Box<T>) -> &T {
    &**value
}

fn contains_var(vars: &Vec<Vec<String>>, var: String) -> bool{
    for stack in vars {
        for v in stack {
            if v.to_owned() == var {
                return true;
            }
        }
    }
    false
}

fn check_vars_in_expression(vars: &mut Vec<Vec<String>>, expression: &Expression) -> SemanticResult{
    match expression {
        Expression::Number(_) => Ok(()),
        Expression::Variable(v) => {
            if contains_var(vars, v.to_owned()){
                Ok(())
            }else{
                Err(SemanticError::VariableUsedBeforeInit)
            }
        },
        Expression::FunctionCall(fc) => {
            for arg in &fc.arguments {
                check_vars_in_expression(vars, arg)?;
            }
            Ok(())
        },
        Expression::Addition(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
        Expression::Subtraction(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
        Expression::Multiplication(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
        Expression::Division(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
        Expression::Modulo(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
        Expression::Greater(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
        Expression::GreaterEquals(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
        Expression::Less(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
        Expression::LessEquals(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
        Expression::Equals(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
        Expression::NotEquals(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
        Expression::LogicAnd(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
        Expression::LogicOr(b) => {
            check_vars_in_expression(vars, &b.0)?;
            check_vars_in_expression(vars, &b.1)?;
            Ok(())
        },
    }
}

fn check_variable_use_before_init(known_vars: &mut Vec<Vec<String>>, block: &parser::Block) -> SemanticResult {
    for statement in block {
        match statement {
            parser::Statement::Assignment(v) => {
                check_vars_in_expression(known_vars, &v.expression)?;
                let mut vars = known_vars.pop().unwrap();
                vars.push(v.variable_name.to_owned());
                known_vars.push(vars);
            }
            parser::Statement::FunctionCall(f) => {
                for arg in &f.arguments {
                    check_vars_in_expression(known_vars, arg)?;
                }
            }
            parser::Statement::IfStatement(s) => {
                check_vars_in_expression(known_vars, &s.condition)?;
                known_vars.push(vec![]);
                check_variable_use_before_init(known_vars, &s.block)?;
                known_vars.pop();
            }
            parser::Statement::WhileLoop(l) => {
                check_vars_in_expression(known_vars, &l.condition)?;
                known_vars.push(vec![]);
                check_variable_use_before_init(known_vars, &l.block)?;
                known_vars.pop();
            }
        }
    }
    Ok(())
}

pub fn check(program: &parser::Program) -> SemanticResult {
    check_duplicate_functions(program)?;
    for function in &program.functions {
        check_duplicate_parameters(function)?;
        let mut vars = vec![function.parameters.clone()];
        check_variable_use_before_init(&mut vars, &function.block)?;
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;
    use logos::Logos;

    #[test]
    fn check_use_before_init() {
        let code = "
        fun main() {
            b = a;
        }
        ";
        let mut lex = lexer::Token::lexer(code).peekable();
        let program = parser::parse(&mut lex).unwrap();
        assert!(check(&program).is_err_and(|e| e == SemanticError::VariableUsedBeforeInit))
    }

    #[test]
    fn check_duplicate_parameter() {
        let code = "
        fun main(a,b,a) {
        }
        ";
        let mut lex = lexer::Token::lexer(code).peekable();
        let program = parser::parse(&mut lex).unwrap();
        assert!(check(&program).is_err_and(|e| e == SemanticError::DuplicateParameter("main".to_owned())))
    }

    #[test]
    fn check_duplicate_function() {
        let code = "
        fun test1() {}
        fun test1(a, b) {}
        ";
        let mut lex = lexer::Token::lexer(code).peekable();
        let program = parser::parse(&mut lex).unwrap();
        assert!(check(&program).is_err_and(|e| e == SemanticError::DuplicateFunction("test1".to_owned())))
    }

}
