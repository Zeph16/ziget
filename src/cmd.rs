use core::fmt;
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    path::Path
};

use inkwell::context::Context;

use crate::{
    codegen::{elf::MachineCodeGenerator, ir::CodeGenerator},
    lexing::{lexer::Lexer, token::{Token, TokenType}},
    parsing::{node::ProgramNode, parser::Parser, semantic_analyzer::SemanticAnalyzer},
};

pub struct Config<'a> {
    pub input_file: &'a Path,
    pub tokens_file: Option<&'a str>,
    pub tree_file: Option<&'a str>,
    pub symbol_table_file: Option<&'a str>,
    pub ir_file: &'a str,
    pub exe_file: &'a str,
}

pub fn read_input_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;
    Ok(input)
}

pub fn process_tokens(config: &Config, input: String) -> Result<Vec<Token>, Box<dyn Error>> {
    let lexer = Lexer::new(input.chars().peekable());
    let tokens: Vec<Token> = lexer.collect();

    let mut has_invalid_tokens = false;
    for token in &tokens {
        if token.token_type == TokenType::Invalid {
            eprintln!(
                "Invalid token at line {}, column {}: {}",
                token.line,
                token.column,
                token.lexeme
            );
            has_invalid_tokens = true;
        }
    }

    if has_invalid_tokens {
        return Err(Box::new(fmt::Error));
    }


    if let Some(tokens_file) = &config.tokens_file {
        let mut file = File::create(tokens_file)?;
        println!("================================================");
        println!("Writing tokens to file");
        for token in &tokens {
            writeln!(file, "{:?}", token)?;
        }
        println!("Tokens written to file: {}", tokens_file);
    }

    Ok(tokens)
}

pub fn parse_ast(tokens: Vec<Token>) -> Result<ProgramNode, Box<dyn Error>> {
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(val) => val,
        Err(e) => {
            println!("================================================");
            println!("Parsing errors:\n{}", e);
            return Err(Box::new(fmt::Error));
        }
    };
    Ok(ast)
}

pub fn analyze_ast(ast: &mut ProgramNode) -> Result<SemanticAnalyzer, Box<dyn Error>> {
    let mut analyzer = SemanticAnalyzer::new();
    match analyzer.analyze(ast) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("================================================");
            eprintln!("Semantic Analysis errors:\n{}", e);
            return Err(Box::new(fmt::Error));
        }
    };
    if !analyzer.warnings.is_empty() {
        eprintln!("================================================");
        eprintln!("Semantic Analysis warnings:\n");
        for warning in &analyzer.warnings {
            eprintln!("Warning: {}", warning);
        }
    }
    Ok(analyzer)
}

pub fn generate_ir(ast: &ProgramNode, filename: &str) -> Result<(), Box<dyn Error>> {
    let llvmcontext = Context::create();
    let mut ir_generator = CodeGenerator::new("ziget", &llvmcontext);

    ir_generator.generate_code(ast);

    println!("================================================");
    println!("Writing IR to file");
    ir_generator.write_to_file(filename);
    println!("IR written to file: {}", filename);

    Ok(())
}

pub fn write_parse_tree(ast: &ProgramNode, config: &Config) -> Result<(), Box<dyn Error>> {
    if let Some(tree_file) = &config.tree_file {
        println!("================================================");
        println!("Writing Parse Tree to file");
        let mut file = File::create(tree_file)?;
        writeln!(file, "{:#?}", ast)?;
        println!("Parse Tree written to file: {}", tree_file);
    }
    Ok(())
}
pub fn write_symbol_table(analyzer: &SemanticAnalyzer, config: &Config) -> Result<(), Box<dyn Error>> {
    if let Some(symbol_table_file) = &config.symbol_table_file {
        println!("Writing symbol tables to file...");
        let mut file = File::create(symbol_table_file)?;
        for table in &analyzer.symbol_tables {
            writeln!(file, "{:#?}", table)?;
        }
        println!("Symbol tables written to file: {}", symbol_table_file);
    }
    Ok(())
}

pub fn compile_and_link(config: &Config) -> Result<(), Box<dyn Error>> {
    let obj_filename = format!("{}.o", &config.ir_file.trim_end_matches(".ll"));
    let asm_filename = format!("{}.s", &config.ir_file.trim_end_matches(".ll"));
    let elf_generator = MachineCodeGenerator::new();
    elf_generator.generate_assembly_file(&config.ir_file, &asm_filename);
    elf_generator.generate_object_file(&asm_filename, &obj_filename);
    elf_generator.link_executable(&obj_filename, &config.exe_file);
    Ok(())
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("Reading input file...");
    let input = read_input_file(config.input_file)?;

    println!("Lexing input...");
    let tokens = process_tokens(&config, input)?;

    println!("Parsing tokens...");
    let mut ast = parse_ast(tokens)?;

    println!("Analyzing parse tree...");
    let analyzer = analyze_ast(&mut ast)?;


    write_parse_tree(&ast, &config)?;
    write_symbol_table(&analyzer, &config)?;


    println!("Generating intermediate code...");
    generate_ir(&ast, &config.ir_file)?;

    println!("Generating machine code...");
    compile_and_link(&config)?;
    println!("Compiled successfully to {}!", &config.exe_file);

    Ok(())
}
