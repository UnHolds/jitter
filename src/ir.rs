//intermidiate representation

use crate::parser::{self, Assignment, Expression, VariableName};
use crate::ssa;
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
        return format!("#label_{}", self.count_label)
    }

    pub fn get_variable(&mut self) -> String {
        self.count_var += 1;
        return format!("#var_{}", self.count_var)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Data {
    Variable(String),
    Number(i64),
}

#[derive(Debug, PartialEq, Clone)]
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
    Assignment(ResultVariable, Data),
    Return(Data)
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

fn transform_if_statement(if_statement: &ssa::SsaIfStatement, phi_nodes: &ssa::PhiNodes, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
    let mut instructions: Vec<IrInstruction> = vec![];
    let (result, mut condition_ir) = transform_expression(&if_statement.condition, name_factory);
    instructions.append(&mut condition_ir);
    let false_if_label = &name_factory.get_label();
    let true_if_label = &name_factory.get_label();
    instructions.push(IrInstruction::JumpFalse(result, false_if_label.to_owned()));
    instructions.append(&mut transform_block(&if_statement.block, name_factory));
    //fix inner phi nodes
    for phi in phi_nodes{
        instructions.push(IrInstruction::Assignment(phi.result_var.to_owned(), Data::Variable(phi.inner_option.to_owned())));
    }
    instructions.push(IrInstruction::Jump(true_if_label.to_owned()));
    instructions.push(IrInstruction::Label(false_if_label.to_owned()));
    //fix outer phi nodes
    for phi in phi_nodes{
        instructions.push(IrInstruction::Assignment(phi.result_var.to_owned(), Data::Variable(phi.outer_option.to_owned())));
    }
    instructions.push(IrInstruction::Label(true_if_label.to_owned()));
    instructions
}

fn transform_assignment(assignment: &ssa::SsaAssignment, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
    let mut instructions: Vec<IrInstruction> = vec![];
    let (result, mut expression_ir) = transform_expression(&assignment.expression, name_factory);
    instructions.append(&mut expression_ir);
    instructions.push(IrInstruction::Assignment(assignment.variable_name.to_owned(), result));
    instructions
}

fn transform_while_loop(while_loop: &ssa::SsaWhileLoop, phi_nodes: &ssa::PhiNodes, loop_phi_nodes: &ssa::LoopPhiNodes, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
    let mut instructions: Vec<IrInstruction> = vec![];
    let start_loop_label = &name_factory.get_label();
    let end_loop_label = &name_factory.get_label();
    let init_false_loop_label = &name_factory.get_label();
    let inner_loop_label = &name_factory.get_label();
    let total_end_label = &name_factory.get_label();



    let (result, mut condition_ir) = transform_expression(&while_loop.condition, name_factory);
    instructions.append(&mut condition_ir.clone());
    instructions.push(IrInstruction::JumpFalse(result.clone(), init_false_loop_label.to_owned()));
    //init inner block vars
    for phi in phi_nodes {
        instructions.push(IrInstruction::Assignment(phi.inner_option.to_owned(), Data::Variable(phi.outer_option.to_owned())));
    }
    instructions.push(IrInstruction::Jump(inner_loop_label.to_owned()));
    instructions.push(IrInstruction::Label(start_loop_label.to_owned()));
    instructions.append(&mut condition_ir);
    instructions.push(IrInstruction::JumpFalse(result, end_loop_label.to_owned()));
    instructions.push(IrInstruction::Label(inner_loop_label.to_owned()));
    instructions.append(&mut transform_block(&while_loop.block, name_factory));
    //condition phi nodes
    for loop_phi in loop_phi_nodes {
        instructions.push(IrInstruction::Assignment(loop_phi.condition_var.to_owned(), Data::Variable(loop_phi.inner_var.to_owned())));
        instructions.push(IrInstruction::Assignment(loop_phi.condition_var.to_owned(), Data::Variable(loop_phi.condition_var.to_owned())));
    }
    instructions.push(IrInstruction::Jump(start_loop_label.to_owned()));
    instructions.push(IrInstruction::Label(init_false_loop_label.to_owned()));
    //outer nodes
    for phi in phi_nodes {
        instructions.push(IrInstruction::Assignment(phi.result_var.to_owned(), Data::Variable(phi.outer_option.to_owned())));
    }
    instructions.push(IrInstruction::Jump(total_end_label.to_owned()));
    instructions.push(IrInstruction::Label(end_loop_label.to_owned()));
    //inner nodes
    for phi in phi_nodes {
        instructions.push(IrInstruction::Assignment(phi.result_var.to_owned(), Data::Variable(phi.inner_option.to_owned())));
    }
    instructions.push(IrInstruction::Label(total_end_label.to_owned()));

    instructions
}

fn transform_function_call(function_call: &ssa::SsaFunctionCall, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
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

fn transform_return(expression: &parser::Expression, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
    let mut instructions: Vec<IrInstruction> = vec![];
    let (res, mut inst) = transform_expression(expression, name_factory);
    instructions.append(&mut inst);
    instructions.push(IrInstruction::Return(res));
    instructions
}


fn transform_statement(statement: &ssa::SsaStatement, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
    match statement {
        ssa::SsaStatement::Assignment(a) => transform_assignment(a, name_factory),
        ssa::SsaStatement::IfStatement(s, phi) => transform_if_statement(s, phi, name_factory),
        ssa::SsaStatement::FunctionCall(f) => transform_function_call(f, name_factory),
        ssa::SsaStatement::WhileLoop(l, phi, loop_phi) => transform_while_loop(l, phi, loop_phi, name_factory),
        ssa::SsaStatement::Return(e) =>transform_return(e, name_factory)
    }
}

fn transform_block(block: &ssa::SsaBlock, name_factory: &mut NameFactory) -> Vec<IrInstruction> {
    let mut instructions: Vec<IrInstruction> = vec![];
    for statement in block {
        instructions.append(&mut transform_statement(statement, name_factory));
    }
    instructions
}

pub fn transform(function: &ssa::SsaFunction) -> Vec<IrInstruction> {
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
        let prog = ssa::convert(&parser::parse(&mut lexer::lex(&code)).unwrap());
        let ir = transform(&prog.functions[0]);
        assert_eq!(ir, [IrInstruction::Assignment("#var_a_#0".to_owned(), Data::Number(1))])
    }

    #[test]
    fn ir_test_function_call() {
        let code = "
            fun test() {
                abc();
            }
        ";
        let prog = ssa::convert(&parser::parse(&mut lexer::lex(&code)).unwrap());
        let ir = transform(&prog.functions[0]);
        assert_eq!(ir, [IrInstruction::FunctionCall("#var_1".to_owned(), "abc".to_owned(), vec![])])
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
        let prog = ssa::convert(&parser::parse(&mut lexer::lex(&code)).unwrap());
        let ir = transform(&prog.functions[0]);
        assert_eq!(ir, [IrInstruction::JumpFalse(Data::Number(1), "#label_1".to_owned()), IrInstruction::JumpFalse(Data::Number(2), "#label_3".to_owned()), IrInstruction::Jump("#label_4".to_owned()), IrInstruction::Label("#label_3".to_owned()), IrInstruction::Label("#label_4".to_owned()), IrInstruction::Jump("#label_2".to_owned()), IrInstruction::Label("#label_1".to_owned()), IrInstruction::Label("#label_2".to_owned())] )
    }

    #[test]
    fn ir_test_function_call_with_expressions_as_arguments_and_assignment() {
        let code = "
            fun test() {
                c = abc(a, 1, 3 && 4, b);
            }
        ";
        let prog = ssa::convert(&parser::parse(&mut lexer::lex(&code)).unwrap());
        let ir = transform(&prog.functions[0]);
        assert_eq!(ir, [IrInstruction::LogicAnd("#var_5".to_owned(), Data::Number(3), Data::Number(4)), IrInstruction::FunctionCall("#var_2".to_owned(), "abc".to_owned(), [Data::Variable("#var_a_#0".to_owned()), Data::Number(1), Data::Variable("#var_5".to_owned()), Data::Variable("#var_b_#0".to_owned())].to_vec()), IrInstruction::Assignment("#var_c_#0".to_owned(), Data::Variable("#var_2".to_owned()))])
    }
}
