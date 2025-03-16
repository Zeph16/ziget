use std::{error::Error, path::Path};
pub mod lexing;
pub mod parsing;
pub mod codegen;

mod cmd;
use clap::Parser;
use cmd::{run, Config};


#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Specify the input .zg file
    #[arg()]
    input_file: String,

    /// Specify the output file
    #[arg(short, long, default_value_t = format!("a.out"))]
    pub output: String,

    /// Flag to save the parse tree to a file
    #[arg(short, long, default_value_t = false)]
    pub parser_output: bool,
    
    /// Flag to save the list of tokens to a file
    #[arg(short, long, default_value_t = false)]
    pub lexer_output: bool,

    /// Flag to save the relational symbol tables to a file
    #[arg(short, long, default_value_t = false)]
    pub symbol_output: bool,

}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if !Path::new(&args.input_file).exists() {
        eprintln!("Error: input file does not exist");
        return Ok(());
    }

    let tokens_file_name = format!("{}-tokens.txt", &args.input_file.trim_end_matches(".zg"));
    let tree_file_name =  format!("{}-tree.txt", &args.input_file.trim_end_matches(".zg"));
    let symbol_table_file_name =  format!("{}-symbol_tables.txt", &args.input_file.trim_end_matches(".zg"));
    let ir_file_name = format!("{}.ll", &args.input_file.trim_end_matches(".zg"));


    let exe_file_name = if args.output == "a.out" {
        format!("{}.out", &args.input_file.trim_end_matches(".zg"))
    } else {
        args.output.clone()
    };

    run(Config {
        input_file: Path::new(&args.input_file),
        tokens_file: if args.lexer_output { Some(&tokens_file_name) } else { None },
        tree_file: if args.lexer_output { Some(&tree_file_name) } else { None },
        symbol_table_file: if args.lexer_output { Some(&symbol_table_file_name) } else { None },
        ir_file: &ir_file_name,
        exe_file: &exe_file_name
    })?;

    Ok(())
}
