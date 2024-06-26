use crate::parser::{self, Expression, FunctionCall, Statement};

#[derive(Debug, PartialEq)]
pub enum SemanticError {
    DuplicateParameter(String),
    DuplicateFunction(String),
    FunctionArgumentCountMissmatch(String),
    VariableUsedBeforeInit,
    FunctionDoesNotExist(String),
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
            Self::FunctionDoesNotExist(fun) =>
            write!(f, "function does not exist: {}", fun),
            Self::FunctionArgumentCountMissmatch(fun) =>
            write!(f, "function call has invalid number of arguments: {}", fun),
        }
    }
}

fn check_duplicate_parameters(function: &parser::InternalFunction) -> SemanticResult {
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
        match function {
            parser::Function::Internal(f) => {
                if uniq.insert(f.name.to_owned()) == false {
                    return Err(SemanticError::DuplicateFunction(f.name.to_owned()))
                }
            }
            parser::Function::External(f) => {
                if uniq.insert(f.name.to_owned()) == false {
                    return Err(SemanticError::DuplicateFunction(f.name.to_owned()))
                }
            }
        }
    }
    Ok(())
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
            parser::Statement::Return(e) => {
                check_vars_in_expression(known_vars, e)?;
            }
        }
    }
    Ok(())
}

fn check_if_function_exist_in_expression(declared_function_names_and_arg_count: &Vec<(String, u64)>, expression: &Expression) -> SemanticResult {
    match expression {
        Expression::Number(_) => Ok(()),
        Expression::Variable(_) => Ok(()),
        Expression::FunctionCall(fc) => {
            check_function(declared_function_names_and_arg_count, fc)?;
            for arg in &fc.arguments {
                check_if_function_exist_in_expression(declared_function_names_and_arg_count, arg)?;
            }
            Ok(())
        },
        Expression::Addition(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
        Expression::Subtraction(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
        Expression::Multiplication(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
        Expression::Division(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
        Expression::Modulo(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
        Expression::Greater(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
        Expression::GreaterEquals(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
        Expression::Less(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
        Expression::LessEquals(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
        Expression::Equals(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
        Expression::NotEquals(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
        Expression::LogicAnd(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
        Expression::LogicOr(b) => {
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.0)?;
            check_if_function_exist_in_expression(declared_function_names_and_arg_count, &b.1)?;
            Ok(())
        },
    }
}



fn check_if_function_exist_on_call(declared_function_names_and_arg_count: &Vec<(String, u64)>, block: &parser::Block) -> SemanticResult {
    for statement in block {
        match statement {
            Statement::FunctionCall(f) => {
                check_function(declared_function_names_and_arg_count, f)?;
                for arg in &f.arguments {
                    check_if_function_exist_in_expression(declared_function_names_and_arg_count, arg)?;
                }
            },
            Statement::Assignment(a) => {
                check_if_function_exist_in_expression(declared_function_names_and_arg_count, &a.expression)?;
            },
            Statement::IfStatement(s) => {
                check_if_function_exist_in_expression(declared_function_names_and_arg_count, &s.condition)?;
                check_if_function_exist_on_call(declared_function_names_and_arg_count, &s.block)?
            },
            Statement::WhileLoop(l) => {
                check_if_function_exist_in_expression(declared_function_names_and_arg_count, &l.condition)?;
                check_if_function_exist_on_call(declared_function_names_and_arg_count, &l.block)?
            },
            Statement::Return(e) => {
                check_if_function_exist_in_expression(declared_function_names_and_arg_count, e)?;
            }
        }
    }
    Ok(())
}

fn check_function(declared_function_names_and_arg_count: &Vec<(String, u64)>, function_call: &FunctionCall) -> SemanticResult  {
    let function = declared_function_names_and_arg_count.iter().find(|(n, _)| n.to_owned() == function_call.name);
    match function {
        None => Err(SemanticError::FunctionDoesNotExist(function_call.name.to_owned())),
        Some((_, n_args)) => {
            if n_args.to_owned() != function_call.arguments.len() as u64 {
                return Err(SemanticError::FunctionArgumentCountMissmatch(function_call.name.to_owned()))
            }
            Ok(())
        }
    }
}

pub fn check(program: &parser::Program) -> SemanticResult {
    check_duplicate_functions(program)?;
    let declared_function_names_and_arg_count: Vec<(String, u64)> = program.functions.iter().map(|fun|
        match fun {
            parser::Function::Internal(f) => (f.name.to_owned(), f.parameters.len() as u64),
            parser::Function::External(f) => (f.name.to_owned(), f.parameters.len() as u64)
        }
    ).collect();
    for function in &program.functions {
        match function {
            parser::Function::Internal(f) => {
                check_duplicate_parameters(f)?;
                let mut vars = vec![f.parameters.clone()];
                check_variable_use_before_init(&mut vars, &f.block)?;
                check_if_function_exist_on_call(&declared_function_names_and_arg_count, &f.block)?;
            }
            parser::Function::External(_) => ()
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;
    use logos::Logos;

    #[test]
    fn check_non_existant_function_expression() {
        let code = "
        fun main() {
            a = 1 && test();
        }
        ";
        let mut lex = lexer::Token::lexer(code).peekable();
        let program = parser::parse(&mut lex).unwrap();
        assert!(check(&program).is_err_and(|e| e == SemanticError::FunctionDoesNotExist("test".to_owned())))
    }

    #[test]
    fn check_non_existant_function_statement() {
        let code = "
        fun main() {
            test();
        }
        ";
        let mut lex = lexer::Token::lexer(code).peekable();
        let program = parser::parse(&mut lex).unwrap();
        assert!(check(&program).is_err_and(|e| e == SemanticError::FunctionDoesNotExist("test".to_owned())))
    }

    #[test]
    fn check_use_before_init() {
        let code = "
        fun main() {
            a = b;
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

    #[test]
    fn check_wrong_number_arguments_function() {
        let code = "
        fun test1() {}
        fun main(a) {
            test1(a);
        }
        ";
        let mut lex = lexer::Token::lexer(code).peekable();
        let program = parser::parse(&mut lex).unwrap();
        assert!(check(&program).is_err_and(|e| e == SemanticError::FunctionArgumentCountMissmatch("test1".to_owned())))
    }

}
