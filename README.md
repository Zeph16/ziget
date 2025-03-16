# Introduction

Ziget is a custom language designed to be concise and simple while leveraging the power of LLVM for efficient backend code generation. It features a unique syntax with influences from popular languages, mainly Rust, but with its own distinct flavor. It is designed to be minimal yet powerful enough to demonstrate key programming concepts, all while being compiled down to efficient machine code using LLVM.

# Installation

**Prerequisites** - Install docker and make sure the daemon (or engine) is up and running: [https://www.docker.com](https://www.docker.com/)

1. Open a terminal inside the directory containing the Dockerfile.

2. Build Docker image:
```bash
docker compose build
```

(This might take a significant amount of time, resulting image can go upto 3 GB in size.)

Done!

# Usage

1. Edit the `main.zg` Ziget file in the playground/ folder on your local machine.

This folder is mapped to the container, allowing you to edit the `.zg` files directly.  

2. Open a terminal in the directory with the  `docker-compose.yml` file. Compile and run the `main.zg` Ziget file using Docker Compose:

```bash
docker compose up
```
  
3. Intermediate outputs like tokens (`main-tokens.txt`), AST (`main-tree.txt`), symbol tables (`main-symbol_tables.txt`), the intermediate representation (`main.ll`), assembly code (`main.s`) and the object code (`main.s`), as well as the linked executable itself (`main.out`) are accessible in the `playground/` folder. These files provide a transparent view of each stage in the compilation process.The final executable can only be run within the compilation environment which is inside the container. That’s why the container automatically runs the executable after compiling in order to show the output if any.
  

# Overview

The entire frontend, including lexical analysis, parsing, and semantic checking, is custom-built, with a DFA-based lexer, a recursive descent parser, and a symbol table for tracking types and variables. Once the code is parsed and validated, the compiler generates LLVM Intermediate Representation (IR) through a self-crafted code generator, via LLVM's tools for optimization and final machine code generation. Everything from tokens to the final executable is handled within a Docker container for consistent operability.

## Language Features

Procedures: `procedure`
Control Flow: `when, otherwise, loop, leave, repeat`
Variable Declarations: `define`
Types: `number, boolean, string`
Booleans: `yes, no`
Operators: `+, -, *, /, %, is, isnt, and, or, <, >, <=, >=`
Return Values: `yield`
Console output: `print`
One line comments: `#`

### Variables, Assignments and Types

Variables are declared using the define keyword. Assignments use := instead of =. Variable types can be specified, or left out for type inference.

Ziget uses three primary types:
	**number**: Represents floating-point numbers. Ziget does not have integers; all numbers are floats.
	**boolean**: Represents true or false values (yes and no).
	**string**: Represents text data.

Type declarations are optional. Ziget can infer types, but you can annotate them explicitly using the -> symbol.
```ziget
   define a := 42;         # Inferred as number**
   define b -> number := 3.14; # Explicitly typed as number**
```

### Operators

Ziget provides basic operators for arithmetic and logical operations:

**Arithmetic Operators**:
+: Addition
-: Subtraction or negation
\*: Multiplication
\/: Division
%: Modulo

**Logical Operators**:
and: Logical AND
or: Logical OR
is: Equality comparison
isnt: Inequality comparison
<, >, <=, >=: Relational comparisons

```ziget
define sum := 5 + 10;
define isEqual := sum is 15;
```

### Control Flow

Conditional statements follow the when...otherwise pattern, similar to if...else but with a more readable syntax. There is no “else if”.
```ziget
when score > 90 {
	print("Excellent");
} otherwise {
	print("Keep trying");
}
```

  

The loop keyword is used for creating loops, where you explicitly manage loop control (no for or while). You can control the loop using leave or repeat.

```ziget
define count := 0;
loop {
    when count is 10 {
        leave;
	}
	print(count);
    count := count + 1;
}
```

### Booleans

Ziget uses yes and no for true and false.

```ziget
define active := yes;
when active is yes {
	print("System is active.");
}
```

### Procedures

All procedure are declared with the procedure keyword, followed by its name and a set of parameters in parenthesis. Parenthesis can be **omitted in declarations** if the procedure has no parameters. You can call a procedure by its name and pass parameters in parentheses.

```ziget
procedure add(x -> number, y -> number) -> number {
    yield x + y;
}

procedure main {
    define sum := add(1, 2);
}
```


Procedures in Ziget return values using the yield keyword, which is equivalent to return in other languages. They can return values and have typed parameters. Procedures can also be declared without return types if not required.

```ziget
procedure greet(name -> string) {
	print("Hello, {}!", name);
}
```

### Print Statement

The print procedure is used to output messages to the console. It can take multiple arguments and is a wrapper around the C printf function. Ziget replaces %d, %i and %s with a singular {}.

```ziget
procedure main() {
	define name := "Ziget";
	print("Hello, {}!", name);
}
```


If the first argument isn’t a string then all arguments are printed sequentially separated by a space.

## Example Program

```ziget
procedure factorial(n -> number) -> number {
    when n is 0 {
        yield 1;
    } otherwise {
        define result := factorial(n - 1);
        yield n * result;
    }
}
procedure fibonacci(n -> number) -> number {
    define a := 0;
    define b := 1;
    define i := 0;
    
	when n is 0 {
        yield a;
	}
    
   when n is 1 {
        yield b;
    }
    
	loop {
        when i is n - 1 {
            leave;
        }
        define temp := a;
        a := b;
        b := temp + b;
        i := i + 1;
    }
    
	yield b;
}
procedure main {
	define num := 5;
    define fact := factorial(num);
    print("The factorial of {} is {}", num, fact);
    
	define fib_num := 10;
    define fib_result := fibonacci(fib_num);
    print("The Fibonacci number at position {} is {}", fib_num, fib_result);
}
```

This example demonstrates recursion, conditionals, loops, and printing. It computes the factorial of 5 and prints the result.

# Ziget Compiler: How It Works

The Ziget Compiler is built from the ground up, with special care given to crafting a modular architecture that takes the code all the way from raw source to LLVM-optimized machine code. Several of the core parts—like the lexer, parser, and code generation—are self-implemented, meaning there was total control over the behavior and design. Here's an overview look at how the system works under the hood.

1, **Lexer**: The DFA-based lexer reads the source code and converts it into tokens.

2. **Parser**: A recursive descent parser converts tokens into an AST.

3. **Semantic Analysis**: The AST is checked for logical and type errors, with warnings and errors reported.

4. **LLVM IR Generation**: The AST is transformed into LLVM IR via a self-implemented code generator.

5. **Optimization**: LLVM applies various optimizations to the IR.

6. **Compilation**: The optimized IR is linked and compiled into machine code.

## Lexical Analysis (Lexer)

The lexer converts raw source code into tokens, the smallest meaningful elements of the language. Unlike generic tools, Ziget's lexer uses a **Deterministic Finite Automaton (DFA)** implemented manually, located in `src/lexing/state_transition_table.rs` . This DFA defines specific state transitions for various token types, such as identifiers, keywords, and literals.

The custom-built DFA gives full flexibility to the structure of tokens in Ziget and allows optimizations that a generated lexer might miss. For instance, the Ziget lexer can efficiently handle overlapping tokens like keywords and identifiers (_is_ vs. _island_), managing conflicts on a state-by-state basis.

## Recursive Descent Parser

The parser is a **recursive descent parser**, meaning it follows a top-down parsing approach that processes the Ziget code according to the predefined grammar. This parser is also self-implemented, handling the full grammar of Ziget, including procedures, conditional statements, loops, and expressions.

Each construct, such as a procedure or conditional, is represented in the parse tree by a corresponding node, like `ProcedureNode` or `ConditionalNode`, all defined in `src/parsing/nodes.rs`. The parser processes input tokens, builds the **Abstract Syntax Tree (AST)**, and validates the structural integrity of the code. The AST is not just a passive data structure; it's designed for modularity and ease of traversal, to make future code transformations simpler. IR code generation is done directly on the AST nodes’ themselves.

## Semantic Analysis

Semantic analysis is performed right after the AST is built. At this stage, the compiler checks for logical errors in the code, such as:

- Variables being used before they are declared.

- Variables not being used.

- Variables being used out of scope.

- Type mismatches in expressions.

- Procedure calls with incorrect argument count or incorrect argument types.

Ziget’s **symbol table** tracks declared variables, procedures, and types. If an issue is found, the system generates warnings and errors, such as when a variable is unused or when type mismatches occur, respectively. **Warnings** are issued in non-critical cases (e.g., unused variables) while **critical errors** halt compilation.

## Code Generation (LLVM IR)

Once the code passes semantic analysis, the AST is traversed to generate **LLVM Intermediate Representation (IR).** This part of the compiler uses the **Inkwell library** to interface with LLVM. Despite using `inkwell`, the code generation logic itself is entirely self-implemented in `src/codegen/generators.rs`, with LLVM serving as the backend.

The beauty of LLVM IR is that it is both platform-agnostic and highly optimizable, so this compiler can create fast, efficient machine code across a wide variety of architectures. Ziget compiles common constructs like procedures, conditionals, loops, and arithmetic expressions directly into LLVM IR.

Even though Inkwell helps with interfacing, all transformation logic from the AST to LLVM IR is custom-built, meaning there's fine control over every aspect of the process. This means that features like control flow (via basic blocks) and loop handling are fully aligned with Ziget's unique syntax.

Example:

```llvm
define i32 @main() {
    %1 = alloca i32, align 4
    store i32 10, i32* %1, align 4
    ret i32 0
}
```


## Optimization & Compilation

LLVM’s optimizers then take over and apply several transformations to make the code as efficient as possible. Once the LLVM IR is generated and optimized, the final machine code is produced. This final phase uses **Clang** for linking which finally turns the optimized IR into executable code.

# Summary

The **Ziget Compiler** project is an exploration of compiler design, of implementing a unique language while demonstrating expertise in systems programming, language theory, and software engineering. Instead of relying on off the shelf libraries for the frontend, I took on the challenge of building the lexer, parser, and semantic analyzer from scratch. The lexer operates as a deterministic finite automaton (DFA), which allows for both efficiency and accuracy in tokenization. The recursive descent parser constructs an abstract syntax tree (AST), while semantic analysis enforces the language’s type system and scope rules through custom error and warning handling.

Beyond the frontend, the integration with LLVM helped me get familiar with advanced compiler infrastructure. The project does not rely on LLVM’s default IR generation but instead produces entirely custom LLVM IR before passing it through LLVM’s optimization stages. This approach definitely gave me a deeper understanding of low-level compilation mechanics.

The entire compilation pipeline, from parsing to final binary, runs inside a Docker container for portability and ease of use. While the project remains highly technical, the goal was not to push the boundaries of compiler design or anything big like that, it was to create a practical solution that balances academic rigor with real world applicability. This combination of creativity and technical depth made the project a great personal feat, and an application of problem-solving and software craftsmanship.

# TODO
- User input support, most probably using `libc` same as with output support
- Sockets. Just pipe `nc` to executables, stick with `scanf` and call it a day. Or actually make a proper standard library and handle sockets.`libuv` for easier cross platform, or don't be lazy and juggle `winsock` with Berkley sockets.
- Standard Library
- Arrays and structures at the very least
- Heap memory access with a `malloc\free` abstraction would also be nice... but I have to make a custom garbage collector at that point. TBD.
