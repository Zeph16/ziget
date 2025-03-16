use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, PointerValue};
use inkwell::types::{BasicType, BasicTypeEnum, VoidType};
use std::collections::HashMap;

use crate::parsing::node::{ProgramNode, TypeNode};


pub struct CodeGenerator<'ctx> {
    pub llvm_context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub function: Option<FunctionValue<'ctx>>,
    pub variables: HashMap<String, (PointerValue<'ctx>, TypeNode)>,
    pub loop_start_block: Option<inkwell::basic_block::BasicBlock<'ctx>>,
    pub loop_end_block: Option<inkwell::basic_block::BasicBlock<'ctx>>,
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn new(module_name: &str, llvm_context: &'ctx Context) -> Self {
        let module = llvm_context.create_module(module_name);
        let builder = llvm_context.create_builder();

        CodeGenerator {
            llvm_context,
            builder,
            module,
            function: None,
            variables: HashMap::new(),
            loop_start_block: None,
            loop_end_block: None
        }
    }

    pub fn generate_code(&mut self, program: &ProgramNode) {
        self.declare_printf();
        program.generate_code(self);
    }

    pub fn write_to_file(&self, file_name: &str) {
        std::fs::write(file_name, self.module.print_to_string().to_string()).expect("Unable to write file");
    }

    pub fn void_type(&self) -> VoidType<'ctx> {
        self.llvm_context.void_type()
    }

    pub fn to_basic_type(&self, ziget_type: &TypeNode) -> BasicTypeEnum<'ctx> {
        match ziget_type {
            TypeNode::NumberType => self.llvm_context.f64_type().as_basic_type_enum(),
            TypeNode::BooleanType => self.llvm_context.bool_type().as_basic_type_enum(),
            TypeNode::StringType => self.llvm_context.ptr_type(inkwell::AddressSpace::default()).as_basic_type_enum(),
            _ => panic!("to_basic_type called for void type, call void_type instead"),
        }
    }

    fn declare_printf(&self) -> FunctionValue<'ctx> {
        let i8_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
        let printf_type = self.llvm_context.i32_type().fn_type(&[i8_ptr_type.into()], true);
        self.module.add_function("printf", printf_type, None)
    }
}

