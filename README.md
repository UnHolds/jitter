# Jitter

Jitter is an on method call just-in-time compiler for a self defined language.
It has no real use case besides learning how jit-compiler work. The JIT-Compiler
was developed during the lecture "Dynamic Compilation" @ TuWien.

## Capabilities

- Converts AST into SSA-from (single assignment form)
- Semantic checker
- Variable lifetime checker
- LSR (linear scan register allocation)
- Constant Evaluation (Planned)
- Constant Propagation (Planned)

## Usage

```
Jitter 0.1.0
Usage of the jitter JIT compiler

USAGE:
    jitter.exe [FLAGS] [OPTIONS] <file> [args]...

FLAGS:
    -h, --help           Prints help information
    -a, --print-asm      prints the decoded bytes (assembly)
    -i, --print-ir       prints the converted ir form of the functions
    -p, --print-parse    print the parse output of the program
    -s, --print-ssa      prints the converted ssa form of the progrm
    -V, --version        Prints version information

OPTIONS:
    -l, --level <log-level>    The log level of the application [default: info]

ARGS:
    <file>       The file that contains the source code
    <args>...    arguments for the passed program
```

### Language

Every program always needs to have a function called `main`!

Example program:

```
fun fun1(a, b) {
    c = a + b;
    return c / 3;
}

fun main(a, b) {
    c = test(10, 15);
    if(c % 2) {
        return 4;
    }
    return c * 5;
}
```

- Functions => `fun <name>(<parameter>) { <block> }`
- Statements `<statement>`
  - If-Statement => `if(<expr>){<block>}`
  - While-Loop => `while(<expr>){<block>}`
  - Assignment => `<variable> = <expr>;`
  - Function Call => eg. `fun1(<arguments>);`
  - return => `return <expr>;`
- Expressions `<expr>`
  - Number => eg. `4`
  - Variable => eg. `a`
  - Addition => ``<expr> + <expr>`
  - Subtraction => `<expr> - <expr>`
  - Multiplication => `<expr> * <expr>`
  - Division => `<expr> / <expr>`
  - Modulo => `<expr> % <expr>`
  - Greater => `<expr> > <expr>`
  - Greater Equals => `<expr> >= <expr>`
  - Less => `<expr> < <expr>`
  - Less equals => `<expr> <= <expr>`
  - Equals => `<expr> == <expr>`
  - Not equals => `<expr> != <expr>`
  - Logic And => `<expr> && <expr>`
  - Logic Or => `<expr> || <expr>`
  - Function Call => eg. `fun1(<parameters>)`
- Block `<block>` => just a bunch of `<statements>`

## Predefined Functions

- `cool()` - Prints the string `"cool\n"`
- `print_num(num)` - Prints the passed parameter `num` as an integer
- `println_num(num)` - same as `print_num(num)` but adds a `\n` at the end
- `print_char(char)` - prints the passed parameter `char` as a character (needs to be printable ascii value otherwise it will crash)
- `println_char(char)` - same as `print_char(char)` but adds a `\n` at the end

## Contributing

1. Found a problem?
   1. Create an issue
2. Contribute Changes
   1. Fork the project and make changes
   2. Create a pull request

## License

This project is licensed under the EUPL license (see [LICENSE](https://github.com/UnHolds/jitter/LICENSE) for more)
