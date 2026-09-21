# TrapScript

## Creators

- Eusef Karl Blancada (eusefkarl)
- Joseph Patrick Salomeo (ysxiaixsy)

## Overview

TrapScript is a dynamically typed scripting language styled on internet slang, distinguished by a pipeline-chaining operator (>>) that lets a value be threaded through a sequence of transformations and function calls without ugly nested calls or intermediate variables.


## Host language and build

- Host language: Rust 1.97.1
- Version metadata: rust-toolchain.toml
- Build: `./build.sh`

## Running it


| Command | What it does |
|---|---|
| `./run <file>` | Executes a program. Not implemented yet (Lab 4); currently prints a usage error. |
| `./run --tokenize <file>` | Scans the file and prints its token stream to stdout, or every lexical error to stderr. |
| `./run --parse <file>` | Prints the parsed tree. Not implemented yet (Lab 2). |
| `./run --eval <file>` | Evaluates each expression and prints its value. Not implemented yet (Lab 3). |
| `./run` | Starts the REPL. Not implemented yet. |


Exit codes: `0` when the file scans cleanly, `65` when the scanner rejects the file (an unexpected character or an unterminated string), `70` for runtime errors (not used until Lab 3).

## File extension

`.trap`

## Lexical structure

### Keywords

| Keyword | Purpose |
|---|---|
| `ong` | if |
| `wait` | else if |
| `nah` | else |
| `spin` | while / for |
| `hold` | let / var |
| `locked` | const |
| `motion` | function |
| `pause` | break |
| `typeshi` | return |
| `spittin` | print |
| `cap` | boolean false |
| `nocap` | boolean true |
| [TODO] | null / nil |


### Operators


| Operator | Category | Operands | Associativity | Precedence |
|---|---|---|---|---|
| `=` | assignment | binary | right | 1 |
| `==` | comparison | binary | left | 2 |
| `!=` | comparison | binary | left | 2 |
| `<` | comparison | binary | left | 2 |
| `<=` | comparison | binary | left | 2 |
| `>` | comparison | binary | left | 2 |
| `>=` | comparison | binary | left | 2 |
| `+` | arithmetic | binary | left | 3 |
| `-` | arithmetic | binary/unary | left | 3 |
| `*` | arithmetic | binary | left | 4 |
| `/` | arithmetic | binary | left | 4 |
| `!` | logical | unary | right | [TODO — Lab 2] |
| `>>` | pipeline | binary | left | [TODO — where does chaining sit relative to arithmetic?] |

### Punctuation

| Symbol | Purpose |
|---|---|
| `,` | separates function parameters and call arguments |

### Literals


| Kind | Syntax | Produces |
|---|---|---|
| number | `42`, `3.14`. A trailing dot (`1.`) is allowed; a leading dot (`.5`) is an error. | an integer for `42`, a decimal for `3.14` and `1.` |
| string | `"hello"`. Can span multiple lines. No escape sequences: a backslash is a normal character, so `\"` ends the string. | the text between the quotes |
| boolean | `nocap`, `cap` | `nocap` is true, `cap` is false |
| [nil] | [spelling] | [what runtime value] |


### Identifiers

- Start characters: any Unicode letter (so `café` is valid), or underscore
- Continue characters: any Unicode letter or number (so `x²` is valid), or underscore
- Case-sensitive: yes

### Comments

- Mid-line comments: allowed. Everything from `//` to the end of the line is ignored, so a comment can follow code.
- Block comments: not supported
- Nesting: not supported
- [Harness note: comment_prefix in tests/lab*/manifest.json is set to the
  token above.]

## Whitespace and termination

- Whitespace significant: No. `>>` is the explicit step separator regardless of line breaks. Newlines are treated as standard whitespace, allowing pipeline chains to be written on a single line or split across multiple lines for readability.
- Statement terminator: none. Newlines are whitespace, so statement boundaries come from the grammar, not line breaks.
- Block delimiters: curly braces `{}`
- Grouping delimiters: parentheses `()`

## Token output format

```
Token {
    token_type: Hold,
    lexeme: "hold",
    line: 1,
}
```

Each token prints in Rust's pretty debug format (`{:#?}`), five lines per token:

- `token_type`: the token's category, such as `Hold`, `Identifier`, or `Num`
- `lexeme`: the token's source text; for a string, the text between the quotes
- `line`: the line the token starts on

Frozen as of Lab 1; changes are recorded in the changelog.

## Grammar

```
[Your complete context-free grammar, current as of the latest activity.
Unambiguous, with precedence and associativity encoded in rule structure.]
```

## Parse output format

```
[one line of real --parse output, e.g. (+ 1.0 (* 2.0 3.0))]
```

- Groupings print as: [form]
- Numbers print as: [form]

## Semantics

### Values and types

[What runtime values exist, and how they are represented in the host
language.]

### Value printing

- Numbers: [e.g. 5 rather than 5.0]
- Nil: [spelling]
- Strings: [with or without quotes]

### Truthiness

[The complete rule. Which values are false in a condition; everything else is
true.]

### Operator semantics

- Arithmetic: [accepted operand types]
- `+` on strings: [concatenation, error, or coercion]
- Mixed types: [what happens]
- Comparison: [accepted operand types]
- Equality across types: [false, or an error]
- Division by zero: [value produced, or runtime error]

### Scope and bindings

- Redeclaration in the same scope: [allowed or an error]
- Uninitialized variable holds: [value]
- Shadowing: [behavior]
- Undefined name: [static error with exit 65, or runtime error with exit 70]

### Control flow and functions

- Logical operators return: [booleans, or the operand]
- Dangling else binds to: [which if]
- Closure capture of a loop variable: [per iteration, or shared]
- Function with no return statement produces: [value]
- Arity mismatch: [message and exit code]

## Native functions


| Name | Arguments | Returns | Notes |
|---|---|---|---|
| [name] | [count and types] | [type] | [caveats] |


## Errors and diagnostics

Message format:

```
Error: Unexpected character '@' on line 2
Error: Unterminated string starting on line 3
```


| Failure | Exit code |
|---|---|
| lexical error: unexpected character or unterminated string | 65 |
| [syntax error] | 65 |
| [runtime error] | 70 |


## Testing conventions


| Folder | Activity | Mode | Flag |
|---|---|---|---|
| tests/lab1 | Scanner | sidecar | `--tokenize` |
| tests/lab2 | Parser | sidecar | `--parse` |
| tests/lab3 | Evaluator | inline | `--eval` |
| tests/lab4 | Context | inline | none |
| tests/lab5 | Functions | inline | none |


```
tests/lab1/booleans.trap               nocap and cap scan as booleans; capital and nocapper stay identifiers
tests/lab1/keywords/keywords.trap      every keyword together in one program
tests/lab1/keywords/blanklines.trap    blank lines between code keep line numbers right
tests/lab1/keywords/conditional.trap   ong with >= and < and a string
tests/lab1/keywords/controlchain.trap  an ong / wait / nah chain
tests/lab1/keywords/dec.trap           a decimal number
tests/lab1/keywords/false.trap         the false keyword
tests/lab1/keywords/function.trap      a motion definition with parameters and typeshi
tests/lab1/keywords/idwithkey.trap     holder, ongoing, and spinner stay identifiers, not keywords
tests/lab1/keywords/int.trap           an integer
tests/lab1/keywords/nested.trap        a spin loop with a nested ong and pause
tests/lab1/keywords/not.trap           ! before an identifier
tests/lab1/keywords/ong.trap           a single ong block with ==
tests/lab1/keywords/str.trap           locked with a string literal
tests/lab1/keywords/true.trap          the true keyword
tests/lab1/errors/errors.trap          an unexpected character and an unterminated string; exits 65
```

Run locally with:

```bash
curl -sSL https://raw.githubusercontent.com/WhiteLicorice/cmsc-124-harness/v1.1/run_tests.py -o run_tests.py
./build.sh
python3 run_tests.py tests/lab1
```

## Sample code

```
[a short program]
```

Output:

```
[its output]
```

## Design rationale

**Vocabulary.** We chose internet/social-media slang over a
generic keyword set (`if`, `else`, `var`) because we wanted something fun and recognizable--that if you saw the keywords you'd immediately know it's TrapScript. Though, this trades familiarity for personality, a newcomer(in the sense that they came from a different programming language) can't guess that `hold` means variable declaration the way they could
guess `var`, but this tradeoff is worth it because personality creates identity. Standard syntax is sterile and forgettable, but TrapScript turns writing code into an expressive, culturally distinct experience. Once you learn the slang logic, like "holding" a variable or putting a function into "motion," the syntax becomes natural and memorable. The slight learning curve gives TrapScript a unique soul instead of just being another generic Python clone.

**The pipeline operator (`>>`).** Most C-family languages express a
sequence of operations through nested function calls or reassignment
(`f(g(h(x)))` or repeated `x = ...`). We introduced `>>` so a value can be
threaded through a chain of transformations top-to-bottom, read in the
order they execute, without intermediate variables. This was inspired by Unix shell pipes (`|`), modern functional pipeline operators (like Elixir's `|>`), and the internet's greentext format (`>`), where events are chained line-by-line in chronological order. Combined with the real-life concept of moving product through a chain of processing spots, passing data left-to-right (`>>`) makes complex data flows far easier to write, read, and debug.

## Known limitations

- `>>` is not scanned yet: it comes out as two `>` tokens.
- `//` comments are not skipped yet: `/` always scans as division.
- `true` and `false` still scan as booleans alongside `nocap` and `cap`.
- `./run` with no arguments does not start a REPL yet.
- `./run <file>` without a flag prints a usage error instead of the file's contents, so `tests/lab0` fails and is left out of CI.
- Tokens do not carry a literal value yet.

## Changelog


| Activity | What changed in the language |
|---|---|
| Lab 1 | Initial token vocabulary and keyword set defined; pipeline operator (`>>`) introduced for chained method/operator calls. |
