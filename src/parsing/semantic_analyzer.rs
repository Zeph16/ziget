use std::collections::{HashMap, HashSet};

use crate::parsing::node::*;
use crate::lexing::token::Operator;

use super::symbol_table::{SymbolInfo, SymbolTable, SymbolType};

pub struct SemanticAnalyzer {
    pub symbol_tables: HashMap<usize, SymbolTable>,
    errors: Vec<String>,
    pub warnings: Vec<String>,
    in_loop: usize,
    current_procedure_return_type: TypeNode,
    current_procedure_did_return: bool,
    pub current_table_id: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_tables: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            in_loop: 0,
            current_table_id: 0,
            current_procedure_return_type: TypeNode::VoidType,
            current_procedure_did_return: false,
        }
    }

    pub fn create_symbol_table(&mut self, parent_id: Option<usize>) -> usize {
        let new_id = self.symbol_tables.len();
        let new_table = SymbolTable::new(new_id, parent_id);

        if let Some(parent_id) = parent_id {
            if let Some(parent_table) = self.symbol_tables.get_mut(&parent_id) {
                parent_table.children.push(new_id);
            }
        }

        self.symbol_tables.insert(new_id, new_table);
        new_id
    }

    pub fn analyze(&mut self, program: &mut ProgramNode) -> Result<(), String> {
        self.analyze_program(program);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.join("\n"))
        }
    }

    fn analyze_program(&mut self, program: &mut ProgramNode) {
        let local_table_id = self.create_symbol_table(None);
        let local_table = self.symbol_tables.get_mut(&local_table_id).unwrap();
        local_table.insert("print".into(), SymbolInfo { symbol_type: SymbolType::Procedure { return_type: TypeNode::VoidType, param_types: vec![TypeNode::StringType] }, used: true, initialized: true });
        for procedure in &mut program.procedures {
            self.analyze_procedure_declaration(procedure, local_table_id);
        }

        self.analyze_main_procedure(&mut program.main, local_table_id);

        self.traverse_and_check(0);
    }

    fn analyze_procedure_declaration(&mut self, procedure: &mut ProcedureNode, parent_table_id: usize) {
        let parent_table = self.symbol_tables.get(&parent_table_id).unwrap();
        if parent_table.lookup(&procedure.name, &self.symbol_tables).is_some() {
            self.errors.push(format!("Procedure '{}' is already declared.", procedure.name));
            return;
        }

        let mut_parent_table = self.symbol_tables.get_mut(&parent_table_id).unwrap();

        mut_parent_table.insert(
            procedure.name.clone(),
            SymbolInfo {
                symbol_type: SymbolType::Procedure {
                    return_type: procedure.return_type.clone(),
                    param_types: procedure.params.iter().map(|p| p.param_type.clone()).collect(),
                },
                used: false,
                initialized: true,
            },
        );

        let local_table_id = self.create_symbol_table(Some(parent_table_id));
        let local_symbol_table = self.symbol_tables.get_mut(&local_table_id).unwrap();

        for param in &procedure.params {
            local_symbol_table.insert(
                param.name.clone(),
                SymbolInfo {
                    symbol_type: SymbolType::Variable(param.param_type.clone()),
                    used: false,
                    initialized: true,
                },
            );
        }

        self.current_procedure_return_type = procedure.return_type.clone();
        self.analyze_block(&mut procedure.body, local_table_id);

        let return_statement = match procedure.return_type {
            TypeNode::NumberType => ReturnNode { value: Some(ExpressionNode::Literal(LiteralNode { value: LiteralValue::NumberValue(0.0) })) },
            TypeNode::BooleanType => ReturnNode { value: Some(ExpressionNode::Literal(LiteralNode { value: LiteralValue::BooleanValue(false) })) },
            TypeNode::StringType => ReturnNode { value: Some(ExpressionNode::Literal(LiteralNode { value: LiteralValue::StringValue("".to_string()) })) },
            TypeNode::VoidType => ReturnNode { value: None },
        };
        procedure.body.statements.push(StatementNode::Return(return_statement));

        self.current_procedure_return_type = TypeNode::VoidType;
    }

    fn analyze_main_procedure(&mut self, main: &mut MainProcedureNode, parent_table_id: usize) {
        let local_table_id = self.create_symbol_table(Some(parent_table_id));
        self.analyze_block(&mut main.body, local_table_id);
    }

    fn analyze_block(&mut self, block: &mut BlockNode, parent_table_id: usize) {
        let local_table_id = self.create_symbol_table(Some(parent_table_id));

        let mut code_is_reachable = true;

        for statement in &mut block.statements {
            if !code_is_reachable {
                self.warnings.push("Unreachable code detected.".to_string());
                break;
            }

            self.analyze_statement(statement, local_table_id);

            if matches!(statement, StatementNode::Return(_) | StatementNode::Break | StatementNode::Continue) {
                code_is_reachable = false;
            }
        }
    }

    fn analyze_statement(&mut self, statement: &mut StatementNode, parent_table_id: usize) {
        match statement {
            StatementNode::VariableDeclaration(var_decl) => self.analyze_variable_declaration(var_decl, parent_table_id),
            StatementNode::Assignment(assign) => self.analyze_assignment(assign, parent_table_id),
            StatementNode::Expression(expression) => { self.analyze_expression(expression, parent_table_id); },
            StatementNode::Return(ret) => self.analyze_return_statement(ret, parent_table_id),
            StatementNode::Loop(loop_node) => self.analyze_loop(loop_node, parent_table_id),
            StatementNode::Conditional(cond) => self.analyze_conditional(cond, parent_table_id),
            StatementNode::Break => self.analyze_break(),
            StatementNode::Continue => self.analyze_continue(),
        }
    }

    fn analyze_variable_declaration(&mut self, var_decl: &mut VariableDeclarationNode, parent_table_id: usize) {
        let symbol_table = self.symbol_tables.get(&parent_table_id).unwrap();
        if symbol_table.lookup(&var_decl.name, &self.symbol_tables).is_some() {
            self.errors.push(format!("Variable '{}' is already declared in this scope.", var_decl.name));
            return;
        }

        let initializer_type = self.analyze_expression(&mut var_decl.initializer, parent_table_id);

        if initializer_type == TypeNode::VoidType {
            self.errors.push("Cannot assign void to a variable.".to_string());
            return;
        }

        if let Some(var_type) = &var_decl.var_type {
            if &initializer_type != var_type {
                self.errors.push(format!(
                    "Type mismatch: expected {:?}, found {:?} for variable '{}'.",
                    var_type, initializer_type, var_decl.name
                ));
            }
        } else {
            var_decl.var_type = Some(initializer_type.clone());
        }

        let mut_symbol_table = self.symbol_tables.get_mut(&parent_table_id).unwrap();
        mut_symbol_table.insert(
            var_decl.name.clone(),
            SymbolInfo {
                symbol_type: SymbolType::Variable(initializer_type),
                used: false,
                initialized: true,
            },
        );
    }

    fn analyze_assignment(&mut self, assign: &mut AssignmentNode, parent_table_id: usize) {
        let (var_type_opt, symbol_name) = {
            let symbol_table = self.symbol_tables.get(&parent_table_id).unwrap();
            if let Some(symbol_info) = symbol_table.lookup(&assign.name, &self.symbol_tables) {
                match &symbol_info.symbol_type {
                    SymbolType::Variable(var_type) => (Some(var_type.clone()), assign.name.clone()),
                    _ => {
                        self.errors.push(format!("'{}' is not a variable and cannot be assigned to.", assign.name));
                        (None, assign.name.clone())
                    }
                }
            } else {
                self.errors.push(format!("Variable '{}' is not declared.", assign.name));
                (None, assign.name.clone())
            }
        };

        if let Some(var_type) = var_type_opt {
            let value_type = self.analyze_expression(&mut assign.value, parent_table_id);
            if value_type != var_type {
                self.errors.push(format!(
                    "Type mismatch in assignment: expected {:?}, found {:?} for variable '{}'.",
                    var_type, value_type, symbol_name
                ));
            }
        }
    }

    fn analyze_return_statement(&mut self, ret: &mut ReturnNode, parent_table_id: usize) {
        if let Some(return_value) = &mut ret.value {
            let return_type = self.analyze_expression(return_value, parent_table_id);
            if return_type != self.current_procedure_return_type {
                self.errors.push(format!(
                    "Type mismatch in 'yield' statement: expected {:?}, found {:?}.",
                    self.current_procedure_return_type, return_type
                ));
            }
        } else {
            if self.current_procedure_return_type != TypeNode::VoidType {
                self.errors.push(format!(
                    "Type mismatch in 'yield' statement: expected {:?}, found void.",
                    self.current_procedure_return_type
                ));
            }
        }
    }

    fn analyze_loop(&mut self, loop_node: &mut LoopNode, parent_table_id: usize) {
        self.in_loop += 1;
        self.analyze_block(&mut loop_node.body, parent_table_id);
        self.in_loop -= 1;
    }

    fn analyze_conditional(&mut self, cond: &mut ConditionalNode, parent_table_id: usize) {
        let condition_type = self.analyze_expression(&mut cond.condition, parent_table_id);
        if condition_type != TypeNode::BooleanType {
            self.errors.push(format!("Conditional expression must be of type 'boolean'."));
        }

        self.analyze_block(&mut cond.consequence, parent_table_id);

        if let Some(alternative) = &mut cond.alternative {
            self.analyze_block(alternative, parent_table_id);
        }

        if cond.consequence.statements.iter().any(|s| matches!(s, StatementNode::Return(_)))
            && cond.alternative.as_ref().map_or(false, |block| block.statements.iter().any(|s| matches!(s, StatementNode::Return(_))))
        {
            self.current_procedure_did_return = true;
        }
    }

    fn analyze_break(&mut self) {
        if self.in_loop == 0 {
            self.errors.push("'leave' statement used outside of a loop.".to_string());
        }
    }

    fn analyze_continue(&mut self) {
        if self.in_loop == 0 {
            self.errors.push("'repeat' statement used outside of a loop.".to_string());
        }
    }

    fn analyze_expression(&mut self, expr: &mut ExpressionNode, parent_table_id: usize) -> TypeNode {
        match expr {
            ExpressionNode::BinaryOperation(bin_op) => {
                let left_type = self.analyze_expression(&mut bin_op.left, parent_table_id);
                let right_type = self.analyze_expression(&mut bin_op.right, parent_table_id);

                if left_type != right_type {
                    self.errors.push(format!(
                        "Type mismatch in binary operation: {:?} vs {:?}.",
                        left_type, right_type
                    ));
                    return left_type;
                }

                match bin_op.operator {
                    Operator::Plus | Operator::Minus | Operator::Times | Operator::Over | Operator::Mod => {
                        if left_type != TypeNode::NumberType {
                            self.errors.push(format!(
                                "Operator {:?} expects number types.",
                                bin_op.operator
                            ));
                        }
                        TypeNode::NumberType
                    }
                    Operator::Lt | Operator::Gt | Operator::Lte | Operator::Gte => {
                        if left_type != TypeNode::NumberType || right_type != TypeNode::NumberType {
                            self.errors.push(format!(
                                "Operator {:?} expects number types.",
                                bin_op.operator
                            ));
                        }
                        TypeNode::BooleanType
                    }
                    Operator::And | Operator::Or => {
                        if left_type != TypeNode::BooleanType || right_type != TypeNode::BooleanType {
                            self.errors.push(format!(
                                "Operator {:?} expects boolean types.",
                                bin_op.operator
                            ));
                        }
                        TypeNode::BooleanType
                    }
                    Operator::Is | Operator::Isnt => {
                        // operands should be boolean or float
                        if left_type != TypeNode::BooleanType && left_type != TypeNode::NumberType {
                            self.errors.push(format!(
                                "Operator {:?} expects boolean or number types.",
                                bin_op.operator
                            ));
                        } else if right_type != TypeNode::BooleanType && right_type != TypeNode::NumberType {
                            self.errors.push(format!(
                                "Operator {:?} expects boolean or number types.",
                                bin_op.operator
                            ));
                        }
                        TypeNode::BooleanType
                    },
                    _ => left_type,
                }
            }
            ExpressionNode::UnaryOperation(unary_op) => {
                let operand_type = self.analyze_expression(&mut unary_op.operand, parent_table_id);
                match unary_op.operator {
                    Operator::Minus => {
                        if operand_type != TypeNode::NumberType {
                            self.errors.push(format!("Unary minus expects a number type."));
                        }
                        TypeNode::NumberType
                    }
                    _ => operand_type,
                }
            }
            ExpressionNode::Literal(literal) => match &literal.value {
                LiteralValue::NumberValue(_) => TypeNode::NumberType,
                LiteralValue::BooleanValue(_) => TypeNode::BooleanType,
                LiteralValue::StringValue(_) => TypeNode::StringType,
            },
            ExpressionNode::Variable(var_name) => {
                let symbol_info = {
                    let symbol_table = self.symbol_tables.get(&parent_table_id).unwrap();
                    symbol_table.lookup(var_name, &self.symbol_tables)
                };

                if let Some(symbol_info) = symbol_info {
                    if let SymbolType::Variable(var_type) = symbol_info.symbol_type.clone() {


                        let mut current_table_id = Some(parent_table_id);

                        while let Some(table_id) = current_table_id {
                            if let Some(table) = self.symbol_tables.get_mut(&table_id) {
                                if let Some(symbol) = table.symbols.get_mut(var_name) {
                                    symbol.used = true;
                                    break;
                                } else {
                                    current_table_id = table.parent_id;
                                }
                            } else {
                                break;
                            }
                        }


                        var_type
                    } else {
                        self.errors.push(format!("'{}' is not a variable.", var_name));
                        TypeNode::VoidType
                    }
                } else {
                    self.errors.push(format!("Variable '{}' is not declared.", var_name));
                    TypeNode::VoidType
                }
            }
            ExpressionNode::ProcedureCall(proc_call) => {
                if proc_call.name == "print" {
                    self.analyze_print_call(proc_call, parent_table_id);
                    return TypeNode::VoidType;
                }
                let (return_type, param_types) = {
                    let (return_type, param_types) = {
                        let symbol_info = {
                            let symbol_table = self.symbol_tables.get(&parent_table_id).unwrap();
                            symbol_table.lookup(&proc_call.name, &self.symbol_tables)
                        };

                        if let Some(symbol_info) = symbol_info {
                            if let SymbolType::Procedure { return_type, param_types } = &symbol_info.symbol_type {
                                (return_type.clone(), param_types.clone())
                            } else {
                                self.errors.push(format!("'{}' is not a procedure.", proc_call.name));
                                return TypeNode::VoidType;
                            }
                        } else {
                            self.errors.push(format!("Procedure '{}' is not declared.", proc_call.name));
                            return TypeNode::VoidType;
                        }
                    };

                    let mut current_table_id = Some(parent_table_id);
                    while let Some(table_id) = current_table_id {
                        if let Some(table) = self.symbol_tables.get_mut(&table_id) {
                            if let Some(symbol) = table.symbols.get_mut(&proc_call.name) {
                                symbol.used = true;
                                break;
                            } else {
                                current_table_id = table.parent_id;
                            }
                        } else {
                            break;
                        }
                    }

                    (return_type, param_types)
                };

                if proc_call.args.len() != param_types.len() {
                    self.errors.push(format!(
                        "Procedure '{}' expects {} arguments, but {} were provided.",
                        proc_call.name, param_types.len(), proc_call.args.len()
                    ));
                } else {
                    for (arg, expected_type) in proc_call.args.iter_mut().zip(param_types) {
                        let arg_type = self.analyze_expression(arg, parent_table_id);
                        if &arg_type != &expected_type {
                            self.errors.push(format!(
                                "Type mismatch in procedure call: expected {:?}, found {:?}.",
                                expected_type, arg_type
                            ));
                        }
                    }

                }

                return_type
            }
        }
    }


    fn traverse_and_check(&mut self, table_id: usize) {
        let mut stack = vec![table_id];
        let mut visited = HashSet::new();

        while let Some(current_id) = stack.pop() {
            if visited.contains(&current_id) {
                panic!("Cycle detected in symbol table graph.");
            }
            visited.insert(current_id);

            if let Some(symbol_table) = self.symbol_tables.get(&current_id) {
                symbol_table.check_unused_symbols(&mut self.warnings);

                for &child_id in &symbol_table.children {
                    stack.push(child_id);
                }
            }
        }
    }

    fn analyze_print_call(&mut self, proc_call: &mut ProcedureCallNode, parent_table_id: usize) {
        if proc_call.args.is_empty() {
            self.errors.push("Print statement requires at least one argument".into());
            return;
        }

        let mut format_string = String::new();
        if let ExpressionNode::Literal(LiteralNode{ value: LiteralValue::StringValue(ref string_literal) }) = proc_call.args[0] {
            let mut expr_index = 0;

            let mut chars = string_literal.chars().peekable();


            let mut args = proc_call.args[1..].to_vec();
            while let Some(c) = chars.next() {
                if c == '{' && chars.peek() == Some(&'}') {
                    chars.next();

                    if expr_index < args.len() {
                        let expr = &mut args[expr_index];
                        let expr_type = self.analyze_expression(expr, parent_table_id);
                        match expr_type {
                            TypeNode::NumberType => format_string.push_str("%.2f"),
                            TypeNode::BooleanType => format_string.push_str("%d"),
                            TypeNode::StringType => format_string.push_str("%s"),
                            TypeNode::VoidType => self.errors.push("Cannot print void type".into()),
                        }
                        expr_index += 1;
                    } else {
                        self.errors.push("Mismatched number of interpolation expressions and placeholders in print statement!".into());
                        return;
                    }
                } else {
                    format_string.push(c);
                }
            }
            format_string.push('\n');

            if expr_index != args.len() {
                self.errors.push("Mismatched number of interpolation expressions and placeholders in print statement!".into());
                return;
            }
            proc_call.args[0] = ExpressionNode::Literal(LiteralNode{ value: LiteralValue::StringValue(format_string) });
        } else {
            for expr in proc_call.args.iter_mut() {
                match self.analyze_expression(expr, parent_table_id) {
                    TypeNode::NumberType => format_string.push_str("%.2f "),
                    TypeNode::BooleanType => format_string.push_str("%d "),
                    TypeNode::StringType => format_string.push_str("%s "),
                    TypeNode::VoidType => self.errors.push("Cannot print void type".into()),
                }
            }
            format_string.push('\n');
            proc_call.args.insert(0, ExpressionNode::Literal(LiteralNode{ value: LiteralValue::StringValue(format_string) }));
        }

    }
}
