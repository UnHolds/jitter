//single static assignment

use std::collections::HashMap;
use itertools;
use crate::parser::{self, VariableName};

struct VariableTracker {
    vars: HashMap<String, u64>
}

impl VariableTracker {
    pub fn new() -> Self {
        VariableTracker {
            vars: std::collections::HashMap::new()
        }
    }

    pub fn get_current(&mut self, name: &str) -> String {
        let num = match self.vars.get(name) {
            Some(n) => {
                n.to_owned()
            },
            None => {
                self.vars.insert(name.to_owned(), 0);
                0
            },
        };
        format!("#var_{}_#{}",name, num)
    }

    pub fn get_new(&mut self, name: &str) -> String {
        let num = match self.vars.get(name) {
            Some(n) => {
                n.to_owned() + 1
            },
            None => {
                0
            },
        };
        self.vars.insert(name.to_owned(), num);
        format!("#var_{}_#{}", name, num)
    }
}

fn convert_expression(expression: &parser::Expression, var_tracker: &mut VariableTracker) -> parser::Expression {
    match expression {
        parser::Expression::Number(n) => parser::Expression::Number(n.to_owned()),
        parser::Expression::Variable(v) => parser::Expression::Variable(var_tracker.get_current(v)),
        parser::Expression::FunctionCall(fc) => {
            let new_arguments = fc.arguments.iter().map(|arg| convert_expression(arg, var_tracker)).collect();
            parser::Expression::FunctionCall(parser::FunctionCall{name: fc.name.to_owned(), arguments: new_arguments})
        },
        parser::Expression::Addition(b) => {
            parser::Expression::Addition(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
        parser::Expression::Subtraction(b) => {
            parser::Expression::Subtraction(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
        parser::Expression::Multiplication(b) => {
            parser::Expression::Multiplication(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
        parser::Expression::Division(b) => {
            parser::Expression::Division(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
        parser::Expression::Modulo(b) => {
            parser::Expression::Modulo(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
        parser::Expression::Greater(b) => {
            parser::Expression::Greater(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
        parser::Expression::GreaterEquals(b) => {
            parser::Expression::GreaterEquals(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
        parser::Expression::Less(b) => {
            parser::Expression::Less(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
        parser::Expression::LessEquals(b) => {
            parser::Expression::LessEquals(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
        parser::Expression::Equals(b) => {
            parser::Expression::Equals(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
        parser::Expression::NotEquals(b) => {
            parser::Expression::NotEquals(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
        parser::Expression::LogicAnd(b) => {
            parser::Expression::LogicAnd(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
        parser::Expression::LogicOr(b) => {
            parser::Expression::LogicOr(Box::new((convert_expression(&b.0, var_tracker), convert_expression(&b.1, var_tracker))))
        },
    }
}

fn get_assigned_variables_in_block(block: &parser::Block) -> Vec<VariableName> {
    let mut vars = vec![];
    for statement in block {
        match statement {
            parser::Statement::PhiNode(_, _, _) => panic!("PhiNodes can't exist before the ssa transformation"),
            parser::Statement::Assignment(a) => vars.push(a.variable_name.to_owned()),
            parser::Statement::FunctionCall(_) => (),
            parser::Statement::IfStatement(s) => {
                vars.append(&mut get_assigned_variables_in_block(&s.block));
            },
            parser::Statement::WhileLoop(l) => {
                vars.append(&mut get_assigned_variables_in_block(&l.block));
            },
        }
    }
    vars
}

fn convert_block(block: &parser::Block, var_tracker: &mut VariableTracker) -> parser::Block {
    let mut new_block = vec![];
    for statement in block {
        match statement {
            parser::Statement::PhiNode(_, _, _) => panic!("PhiNodes can't exist before the ssa transformation"),
            parser::Statement::Assignment(a) => {
                let new_var = var_tracker.get_new(&a.variable_name.to_owned());
                let new_inner_block = convert_expression(&a.expression, var_tracker);
                new_block.push(parser::Statement::Assignment(parser::Assignment { variable_name: new_var, expression: new_inner_block }));
            },
            parser::Statement::FunctionCall(f) => {
                let mut new_args = vec![];
                for arg in &f.arguments {
                    new_args.push(convert_expression(arg, var_tracker));
                }
                new_block.push(parser::Statement::FunctionCall(parser::FunctionCall { name: f.name.to_owned(), arguments: new_args }));
            },
            parser::Statement::IfStatement(s) => {
                let new_condition = convert_expression(&s.condition, var_tracker);
                let assigned_vars: Vec<VariableName> = get_assigned_variables_in_block(&s.block);
                let pre_var_names: Vec<VariableName> = assigned_vars.iter().map(|v| var_tracker.get_current(v)).collect();
                let new_inner_block = convert_block(&s.block, var_tracker);
                let post_var_names: Vec<VariableName> = assigned_vars.iter().map(|v| var_tracker.get_current(v)).collect();

                let mut phi_nodes = vec![];
                for (var, pre, post) in itertools::izip!(assigned_vars, pre_var_names, post_var_names) {
                    phi_nodes.push(parser::Statement::PhiNode(var_tracker.get_new(&var), pre, post))
                }

                new_block.push(parser::Statement::IfStatement(parser::IfStatement {condition: new_condition, block: new_inner_block}));
                new_block.append(&mut phi_nodes);
            },
            parser::Statement::WhileLoop(l) => {
                let new_condition = convert_expression(&l.condition, var_tracker);
                let assigned_vars: Vec<VariableName> = get_assigned_variables_in_block(&l.block);
                let pre_var_names: Vec<VariableName> = assigned_vars.iter().map(|v| var_tracker.get_current(v)).collect();
                let new_inner_block = convert_block(&l.block, var_tracker);
                let post_var_names: Vec<VariableName> = assigned_vars.iter().map(|v| var_tracker.get_current(v)).collect();

                let mut phi_nodes = vec![];
                for (var, pre, post) in itertools::izip!(assigned_vars, pre_var_names, post_var_names) {
                    phi_nodes.push(parser::Statement::PhiNode(var_tracker.get_new(&var), pre, post))
                }

                new_block.push(parser::Statement::IfStatement(parser::IfStatement {condition: new_condition, block: new_inner_block}));
                new_block.append(&mut phi_nodes);
            }
        }
    }
    new_block
}


pub struct SsaProgram {
    pub functions: Vec<SsaFunction>
}

pub struct SsaFunction {
    pub name: parser::FunctionIdentifier,
    pub parameters: parser::Parameters,
    pub block: parser::Block
}

pub fn convert(program: &parser::Program) -> SsaProgram {
    let mut new_function = vec![];
    let mut var_tracker = VariableTracker::new();
    for function in &program.functions {
        let new_parameters = function.parameters.iter().map(|p| var_tracker.get_new(p)).collect();
        let new_block = convert_block(&function.block, &mut var_tracker);
        new_function.push(SsaFunction{name: function.name.to_owned(), block: new_block, parameters: new_parameters});
    }

    SsaProgram { functions: new_function }
}