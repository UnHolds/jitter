//intermidiate representation

use crate::parser::{self, Expression};

pub type Label = String;
pub type Function = String;
pub type ResultVariable = String;
pub type Arguments = Vec<Data>;

pub struct NameFactory {
    count_label: u64,
    count_var: u64,
}

impl NameFactory {
    pub fn new() -> Self {
        NameFactory {
            count_label: 0,
            count_var: 0
        }
    }

    pub fn get_label(&mut self) -> String {
        self.count_label += 1;
        return format!("#label{}", self.count_label)
    }

    pub fn get_variable(&mut self) -> String {
        self.count_var += 1;
        return format!("#var{}", self.count_var)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Data {
    Variable(String),
    Number(i64),
}

#[derive(Debug, PartialEq)]
pub enum IrInstruction {
    Jump(Label),
    JumpFalse(Data, Label),
    Label(String),
    FunctionCall(ResultVariable, Function, Arguments),
    Addition(ResultVariable, Data, Data),
    Subtraction(ResultVariable, Data, Data),
    Multiplication(ResultVariable, Data, Data),
    Division(ResultVariable, Data, Data),
    Modulo(ResultVariable, Data, Data),
    Greater(ResultVariable, Data, Data),
    GreaterEquals(ResultVariable, Data, Data),
    Less(ResultVariable, Data, Data),
    LessEquals(ResultVariable, Data, Data),
    Equals(ResultVariable, Data, Data),
    NotEquals(ResultVariable, Data, Data),
    LogicAnd(ResultVariable, Data, Data),
    LogicOr(ResultVariable, Data, Data),
    Assignment(ResultVariable, Data)
}

fn handle_binary_expression(b: &Box<(Expression, Expression)> , name_factory: &mut NameFactory) -> (Data, Data, Vec<IrInstruction>) {
    let mut instructions = vec![];
    let (left_res, mut left_inst) = transform_expression(&b.0, name_factory);
    let (right_res, mut right_inst) = transform_expression(&b.1, name_factory);
    instructions.append(&mut left_inst);
    instructions.append(&mut right_inst);
    (left_res, right_res, instructions)
}
fn transform_expression(expression: &parser::Expression, name_factory: &mut NameFactory) -> (Data, Vec<IrInstruction>) {
    let result = &name_factory.get_variable();
    match expression {
        parser::Expression::Number(n) => (Data::Number(n.to_owned()), vec![]),
        parser::Expression::Variable(v) => (Data::Variable(v.to_owned()), vec![]),
        parser::Expression::Addition(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::Addition(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::Subtraction(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::Subtraction(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::Multiplication(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::Multiplication(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::Division(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::Division(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::Modulo(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::Modulo(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::Greater(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::Greater(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::GreaterEquals(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::GreaterEquals(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::Less(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::Less(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::LessEquals(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::LessEquals(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::Equals(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::Equals(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::NotEquals(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::NotEquals(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::LogicAnd(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::LogicAnd(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::LogicOr(b) => {
            let (left_res, right_res, mut instructions) = handle_binary_expression(b, name_factory);
            instructions.push(IrInstruction::LogicOr(result.to_owned(), left_res, right_res));
            (Data::Variable(result.to_owned()), instructions)
        },
        parser::Expression::FunctionCall(f) => {
            let mut instructions: Vec<IrInstruction> = vec![];
            let result = &name_factory.get_variable();
            let mut arguments = vec![];
            for arg in &f.arguments {
                let (res_var, mut inst) = transform_expression(arg, name_factory);
                arguments.push(res_var);
                instructions.append(&mut inst);
            }
            instructions.push(IrInstruction::FunctionCall(result.to_owned(), f.name.to_owned(), arguments));
            (Data::Variable(result.to_owned()), instructions)
        }
    }
}

fn transform_if_statement(if_statement: &parser::IfStatement, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
    let mut instructions: Vec<IrInstruction> = vec![];
    let (result, mut condition_ir) = transform_expression(&if_statement.condition, name_factory);
    instructions.append(&mut condition_ir);
    let end_if_lable = &name_factory.get_label();
    instructions.push(IrInstruction::JumpFalse(result, end_if_lable.to_owned()));
    instructions.append(&mut transform_block(&if_statement.block, name_factory));
    instructions.push(IrInstruction::Label(end_if_lable.to_owned()));
    instructions
}

fn transform_assignment(assignment: &parser::Assignment, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
    let mut instructions: Vec<IrInstruction> = vec![];
    let (result, mut expression_ir) = transform_expression(&assignment.expression, name_factory);
    instructions.append(&mut expression_ir);
    instructions.push(IrInstruction::Assignment(assignment.variable_name.to_owned(), result));
    instructions
}

fn transform_while_loop(while_loop: &parser::WhileLoop, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
    let mut instructions: Vec<IrInstruction> = vec![];
    let start_loop_lable = &name_factory.get_label();
    let end_loop_lable = &name_factory.get_label();

    instructions.push(IrInstruction::Label(start_loop_lable.to_owned()));

    let (result, mut condition_ir) = transform_expression(&while_loop.condition, name_factory);
    instructions.append(&mut condition_ir);

    instructions.push(IrInstruction::JumpFalse(result, end_loop_lable.to_owned()));
    instructions.append(&mut transform_block(&while_loop.block, name_factory));
    instructions.push(IrInstruction::Jump(start_loop_lable.to_owned()));
    instructions.push(IrInstruction::Label(end_loop_lable.to_owned()));
    instructions
}

fn transform_function_call(function_call: &parser::FunctionCall, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
    let mut instructions: Vec<IrInstruction> = vec![];
    let mut arguments: Arguments = vec![];
    for arg in &function_call.arguments {
        let (res, mut inst) = transform_expression(arg, name_factory);
        arguments.push(res);
        instructions.append(&mut inst);
    }

    instructions.push(IrInstruction::FunctionCall(name_factory.get_variable(), function_call.name.to_owned(), arguments));

    instructions
}

fn transform_statement(statement: &parser::Statement, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
    match statement {
        parser::Statement::Assignment(a) => transform_assignment(a, name_factory),
        parser::Statement::IfStatement(s) => transform_if_statement(s, name_factory),
        parser::Statement::FunctionCall(f) => transform_function_call(f, name_factory),
        parser::Statement::WhileLoop(l) => transform_while_loop(l, name_factory)
    }
}

fn transform_block(block: &parser::Block, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
    let mut instructions: Vec<IrInstruction> = vec![];
    for statement in block {
        instructions.append(&mut transform_statement(statement, name_factory));
    }
    instructions
}

pub fn transform(function: &parser::Function) -> Vec<IrInstruction> {
    transform_block(&function.block, &mut NameFactory::new())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    #[test]
    fn ir_test_assignment() {
        let code = "
            fun test() {
                a = 1;
            }
        ";
        let prog = parser::parse(&mut lexer::lex(&code)).unwrap();
        let ir = transform(&prog.functions[0]);
        assert_eq!(ir, [IrInstruction::Assignment("a".to_owned(), Data::Number(1))])
    }

    #[test]
    fn ir_test_function_call() {
        let code = "
            fun test() {
                abc();
            }
        ";
        let prog = parser::parse(&mut lexer::lex(&code)).unwrap();
        let ir = transform(&prog.functions[0]);
        assert_eq!(ir, [IrInstruction::FunctionCall("#var1".to_owned(), "abc".to_owned(), vec![])])
        //println!("{:?}", ir);
    }

    #[test]
    fn ir_test_if_statement() {
        let code = "
            fun test() {
                if(1){
                    if(2){

                    }
                }
            }
        ";
        let prog = parser::parse(&mut lexer::lex(&code)).unwrap();
        let ir = transform(&prog.functions[0]);
        assert_eq!(ir, [IrInstruction::JumpFalse(Data::Number(1), "#label1".to_owned()), IrInstruction::JumpFalse(Data::Number(2), "#label2".to_owned()), IrInstruction::Label("#label2".to_owned()), IrInstruction::Label("#label1".to_owned())])
    }

    #[test]
    fn ir_test_function_call_with_expressions_as_arguments_and_assignment() {
        let code = "
            fun test() {
                c = abc(a, 1, 3 && 4, b);
            }
        ";
        let prog = parser::parse(&mut lexer::lex(&code)).unwrap();
        let ir = transform(&prog.functions[0]);
        assert_eq!(ir, [IrInstruction::LogicAnd("#var5".to_owned(), Data::Number(3), Data::Number(4)), IrInstruction::FunctionCall("#var2".to_owned(), "abc".to_owned(), [Data::Variable("a".to_owned()), Data::Number(1), Data::Variable("#var5".to_owned()), Data::Variable("b".to_owned())].to_vec()), IrInstruction::Assignment("c".to_owned(), Data::Variable("#var2".to_owned()))])
    }
}
