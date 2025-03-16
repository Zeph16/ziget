use crate::lexing::token::Operator;
use crate::parsing::node::{
    ProgramNode, ProcedureNode, MainProcedureNode, BlockNode, StatementNode, VariableDeclarationNode,
    AssignmentNode, ReturnNode, LoopNode, ConditionalNode, ExpressionNode, BinaryOperationNode,
    UnaryOperationNode, LiteralNode, ProcedureCallNode, TypeNode, LiteralValue,
};

use super::ir::CodeGenerator;

use inkwell::types::BasicType;
use inkwell::values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum};

impl<'ctx> ProgramNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) {
        for procedure in &self.procedures {
            procedure.generate_code(generator);
        }

        self.main.generate_code(generator);
    }
}

impl<'ctx> ProcedureNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) {
        let params: Vec<_> = self.params.iter().map(|param| generator.to_basic_type(&param.param_type).into()).collect();

        let fn_type = match self.return_type {
            TypeNode::VoidType => generator.void_type().fn_type(&params, false),
            _ => generator.to_basic_type(&self.return_type).fn_type(&params, false)
        };

        let function = generator.module.add_function(&self.name, fn_type, None);
        generator.function = Some(function);

        let entry_block = generator.llvm_context.append_basic_block(function, "entry");
        generator.builder.position_at_end(entry_block);

        for (i, param) in self.params.iter().enumerate() {
            let alloca = generator.builder.build_alloca(generator.to_basic_type(&param.param_type), &param.name).unwrap();
            generator.builder.build_store(alloca, function.get_nth_param(i as u32).unwrap()).unwrap();
            generator.variables.insert(param.name.clone(), (alloca, param.param_type.clone()));
        }

        self.body.generate_code(generator);

        if self.return_type == TypeNode::VoidType {
            generator.builder.build_return(None).unwrap();
        }

        generator.function = None;
    }
}

impl<'ctx> MainProcedureNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) {
        let fn_type = generator.llvm_context.void_type().fn_type(&[], false);
        let function = generator.module.add_function("main", fn_type, None);
        generator.function = Some(function);

        let entry_block = generator.llvm_context.append_basic_block(function, "entry");
        generator.builder.position_at_end(entry_block);

        self.body.generate_code(generator);

        generator.builder.build_return(None).unwrap();
        generator.function = None;
    }
}

impl<'ctx> BlockNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) {
        for statement in &self.statements {
            statement.generate_code(generator);
        }
    }
}

impl<'ctx> StatementNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) {
        match self {
            StatementNode::VariableDeclaration(var_decl) => var_decl.generate_code(generator),
            StatementNode::Assignment(assign) => assign.generate_code(generator),
            StatementNode::Expression(expr) => { expr.generate_code(generator); },
            StatementNode::Return(ret) => ret.generate_code(generator),
            StatementNode::Loop(loop_node) => loop_node.generate_code(generator),
            StatementNode::Conditional(cond) => cond.generate_code(generator),
            StatementNode::Break => {
                if let Some(loop_end) = generator.loop_end_block {
                    generator.builder.build_unconditional_branch(loop_end).unwrap();
                } else {
                    panic!("`leave` statement used outside of a loop context");
                }
            },
            StatementNode::Continue => {
                if let Some(loop_header) = generator.loop_start_block {
                    generator.builder.build_unconditional_branch(loop_header).unwrap();
                } else {
                    panic!("`repeat` statement used outside of a loop context");
                }
            }
        }
    }
}

impl<'ctx> VariableDeclarationNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) {
        let var_type = generator.to_basic_type(self.var_type.as_ref().unwrap());
        let alloca = generator.builder.build_alloca(var_type, &self.name).unwrap();
        let init_val = self.initializer.generate_code(generator).unwrap();
        generator.builder.build_store(alloca, init_val).unwrap();
        generator.variables.insert(self.name.clone(), (alloca, self.var_type.clone().unwrap()));
    }
}

impl<'ctx> AssignmentNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) {
        if let Some(var) = generator.variables.get(&self.name) {
            let var_clone = (*var).clone();
            let val = self.value.generate_code(generator).unwrap();
            generator.builder.build_store(var_clone.0, val).unwrap();
        }
    }
}

impl<'ctx> ReturnNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) {
        if let Some(expr) = &self.value {
            let ret_val = expr.generate_code(generator).unwrap();
            generator.builder.build_return(Some(&ret_val)).unwrap();
        } else {
            generator.builder.build_return(None).unwrap();
        }
    }
}

impl<'ctx> LoopNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) {
        let loop_bb = generator.llvm_context.append_basic_block(generator.function.unwrap(), "loop");
        let after_loop_bb = generator.llvm_context.append_basic_block(generator.function.unwrap(), "afterloop");

        generator.loop_start_block = Some(loop_bb);
        generator.loop_end_block = Some(after_loop_bb);

        generator.builder.build_unconditional_branch(loop_bb).unwrap();
        generator.builder.position_at_end(loop_bb);

        self.body.generate_code(generator);

        generator.builder.build_unconditional_branch(loop_bb).unwrap();
        generator.builder.position_at_end(after_loop_bb);

        generator.loop_start_block = None;
        generator.loop_end_block = None;
    }
}

impl<'ctx> ConditionalNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) {
        let condition_val = self.condition.generate_code(generator).unwrap();
        let condition_bool = generator.builder.build_int_compare(inkwell::IntPredicate::EQ, condition_val.into_int_value(), generator.llvm_context.bool_type().const_int(1, false), "cond");

        let then_bb = generator.llvm_context.append_basic_block(generator.function.unwrap(), "then");
        let else_bb = generator.llvm_context.append_basic_block(generator.function.unwrap(), "else");
        let merge_bb = generator.llvm_context.append_basic_block(generator.function.unwrap(), "merge");

        generator.builder.build_conditional_branch(condition_bool.unwrap(), then_bb, else_bb).unwrap();

        generator.builder.position_at_end(then_bb);
        self.consequence.generate_code(generator);
        generator.builder.build_unconditional_branch(merge_bb).unwrap();

        generator.builder.position_at_end(else_bb);
        if let Some(alternative) = &self.alternative {
            alternative.generate_code(generator);
        }
        generator.builder.build_unconditional_branch(merge_bb).unwrap();

        generator.builder.position_at_end(merge_bb);
    }
}

impl<'ctx> ExpressionNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        match self {
            ExpressionNode::BinaryOperation(bin_op) => bin_op.generate_code(generator),
            ExpressionNode::UnaryOperation(un_op) => un_op.generate_code(generator),
            ExpressionNode::Literal(lit) => lit.generate_code(generator),
            ExpressionNode::Variable(var_name) => {
                if let Some(var) = generator.variables.get(var_name) {
                    Some(generator.builder.build_load(generator.to_basic_type(&(*var).1),(*var).0, var_name).unwrap())
                } else {
                    None
                }
            }
            ExpressionNode::ProcedureCall(proc_call) => proc_call.generate_code(generator),
        }
    }
}

impl<'ctx> ProcedureCallNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        if self.name == "print" {
            return self.generate_print_code(generator);
        }
        let function = generator.module.get_function(&self.name).expect("Unknown function called");

        let args: Vec<BasicMetadataValueEnum<'ctx>> = self.args.iter()
            .map(|arg| arg.generate_code(generator).unwrap().into())
            .collect();

        let call = generator.builder.build_call(function, &args, "calltmp");

        call.unwrap().try_as_basic_value().left()
    }

    fn generate_print_code(&self, generator: &mut CodeGenerator<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let printf_function = generator.module.get_function("printf")
            .expect("printf function not declared");

        let format_str_expr = &self.args[0];
        let format_string = format_str_expr.generate_code(generator).unwrap();

        let mut args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
        args.push(format_string.into());

        for expr in self.args.iter().skip(1) {
            let generated_arg = expr.generate_code(generator).unwrap();
            args.push(generated_arg.into());
        }

        generator.builder.build_call(printf_function, &args, "printtmp").unwrap();
        
        None
    }
}

impl<'ctx> BinaryOperationNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let lhs = self.left.generate_code(generator)?;
        let rhs = self.right.generate_code(generator)?;

        let result = match self.operator {
            Operator::Plus => generator.builder.build_float_add(lhs.into_float_value(), rhs.into_float_value(), "addtmp").unwrap().as_basic_value_enum(),
            Operator::Minus => generator.builder.build_float_sub(lhs.into_float_value(), rhs.into_float_value(), "subtmp").unwrap().as_basic_value_enum(),
            Operator::Times => generator.builder.build_float_mul(lhs.into_float_value(), rhs.into_float_value(), "multmp").unwrap().as_basic_value_enum(),
            Operator::Over => generator.builder.build_float_div(lhs.into_float_value(), rhs.into_float_value(), "divtmp").unwrap().as_basic_value_enum(),
            Operator::Mod => generator.builder.build_float_rem(lhs.into_float_value(), rhs.into_float_value(), "modtmp").unwrap().as_basic_value_enum(),

            Operator::And => generator.builder.build_and(lhs.into_int_value(), rhs.into_int_value(), "andtmp").unwrap().as_basic_value_enum(),
            Operator::Or => generator.builder.build_or(lhs.into_int_value(), rhs.into_int_value(), "ortmp").unwrap().as_basic_value_enum(),

            Operator::Is => generator.builder.build_float_compare(inkwell::FloatPredicate::OEQ, lhs.into_float_value(), rhs.into_float_value(), "eqtmp").unwrap().as_basic_value_enum(),
            Operator::Isnt => generator.builder.build_float_compare(inkwell::FloatPredicate::ONE, lhs.into_float_value(), rhs.into_float_value(), "netmp").unwrap().as_basic_value_enum(),
            Operator::Lt => generator.builder.build_float_compare(inkwell::FloatPredicate::OLT, lhs.into_float_value(), rhs.into_float_value(), "lttmp").unwrap().as_basic_value_enum(),
            Operator::Gt => generator.builder.build_float_compare(inkwell::FloatPredicate::OGT, lhs.into_float_value(), rhs.into_float_value(), "gttmp").unwrap().as_basic_value_enum(),
            Operator::Lte => generator.builder.build_float_compare(inkwell::FloatPredicate::OLE, lhs.into_float_value(), rhs.into_float_value(), "ltetmp").unwrap().as_basic_value_enum(),
            Operator::Gte => generator.builder.build_float_compare(inkwell::FloatPredicate::OGE, lhs.into_float_value(), rhs.into_float_value(), "gtetmp").unwrap().as_basic_value_enum(),
            _ => unimplemented!(),
        };

        Some(result)
    }
}

impl<'ctx> UnaryOperationNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let operand = self.operand.generate_code(generator)?;

        let result = match self.operator {
            Operator::Minus => generator.builder.build_float_neg(operand.into_float_value(), "negtmp"),
            _ => unimplemented!(),
        };

        Some(result.unwrap().as_basic_value_enum())
    }
}

impl<'ctx> LiteralNode {
    pub fn generate_code(&self, generator: &mut CodeGenerator<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let value = match &self.value {
            LiteralValue::NumberValue(n) => generator.llvm_context.f64_type().const_float(*n).as_basic_value_enum(),
            LiteralValue::BooleanValue(b) => generator.llvm_context.bool_type().const_int(*b as u64, false).as_basic_value_enum(),
            LiteralValue::StringValue(s) => {
                let str_val = generator.builder.build_global_string_ptr(s, "str");
                str_val.unwrap().as_basic_value_enum()
            }
        };

        Some(value)
    }
}
