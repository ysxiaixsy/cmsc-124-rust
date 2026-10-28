# TrapScript

## Creators

- Eusef Karl Blancada (eusefkarl)
- Joseph Patrick Salomeo (ysxiaixsy)

## Overview

TrapScript is a dynamically typed scripting language styled on internet slang.


## Host language and build

- Host language: Rust 1.97.1
- Version metadata: rust-toolchain.toml
- Build: `./build.sh`

## Running it


| Command | What it does |
|---|---|
| `./run <file>` | Executes the program. Statements run in source order and only `spittin` writes to stdout; a bare expression statement is evaluated and its value discarded. |
| `./run --tokenize <file>` | Scans the file and prints its token stream to stdout, or every lexical error to stderr. |
| `./run --parse <file>` | Parses the file and prints each expression statement in prefix form, one line per statement, or every scan or syntax error to stderr. |
| `./run --help` | Prints the supported command forms. |
| `./run --eval <file>` | Evaluates the file and prints the value of **every** expression statement, the Lab 3 contract. Kept separate from plain `./run` so the Lab 3 tests stay meaningful now that a program speaks only through `spittin`. |
| `./run` | Starts the evaluator REPL. Each input line is a complete program and must include a semicolon after every statement. Values print in statement order; errors report a diagnostic and return the prompt. End with Ctrl+Z then Enter on Windows, or Ctrl+D on Linux/macOS. |
| `./run --tokenize` | Starts the scanner REPL. |
| `./run --eval` | Starts the evaluator REPL explicitly. |
| `./run --parse` | Starts the REPL at the parser instead. Each input line is a complete program; every statement needs a semicolon. It prints one expression tree per statement. |


Exit codes: `0` when the file is accepted, `65` when it is rejected before running (a lexical error from the scanner or a syntax error from the parser), `70` for runtime errors.

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
| `nil` | absence of a value |
| `cap` | boolean false |
| `nocap` | boolean true |


### Operators


| Operator | Category | Operands | Associativity | Precedence |
|---|---|---|---|---|
| `=` | assignment | binary | right | 1 |
| `==` | equality | binary | left | 2 |
| `!=` | equality | binary | left | 2 |
| `<` | comparison | binary | left | 3 |
| `<=` | comparison | binary | left | 3 |
| `>` | comparison | binary | left | 3 |
| `>=` | comparison | binary | left | 3 |
| `+` | arithmetic | binary | left | 4 |
| `-` | arithmetic | binary | left | 4 |
| `*` | arithmetic | binary | left | 5 |
| `/` | arithmetic | binary | left | 5 |
| `!` | logical | unary | right | 6 |
| `-` | arithmetic | unary | right | 6 |

### Punctuation

| Symbol | Purpose |
|---|---|
| `,` | separates function parameters and call arguments |
| `;` | terminates an expression statement |

### Literals


| Kind | Syntax | Produces |
|---|---|---|
| number | `42`, `3.14`. A trailing dot (`1.`) is allowed; a leading dot (`.5`) is an error. | an `f64` value in all cases; token categories distinguish integer and decimal spellings |
| string | `"hello"`. Can span multiple lines. No escape sequences: a backslash is a normal character, so `\"` ends the string. | the text between the quotes |
| boolean | `nocap`, `cap` | `nocap` is true, `cap` is false |
| nil | `nil` | the absence of a value |


### Identifiers

- Start characters: any Unicode letter (so `café` is valid), or underscore
- Continue characters: any Unicode letter or number (so `x²` is valid), or underscore
- Case-sensitive: yes

### Comments

- Mid-line comments: allowed. Everything from `//` to the end of the line is ignored, so a comment can follow code.
- Block comments: not supported
- Nesting: not supported
- Harness note: the Lab 1 and Lab 2 test folders use sidecar mode, which never reads comments. From Lab 3, `comment_prefix` in the inline-mode manifests is `//`.

## Whitespace and termination

- Whitespace: spaces, tabs, and newlines outside strings separate tokens and are otherwise ignored. Newlines do not end statements.
- Statement terminator: `;` (Semicolon). Every expression statement needs one, including the final statement immediately before EOF.
- A semicolon inside a string or a `//` comment is part of that literal or comment, not a terminator.
- Expressions and parenthesized groups can span lines. Multiple statements can share a line. Blank and comment-only lines are allowed.
- Empty statements are not supported: a bare or repeated semicolon is a syntax error. An empty program is accepted.
- Block delimiters: curly braces `{}`
- Grouping delimiters: parentheses `()`

## Token output format

```
Token {
    token_type: Num,
    lexeme: "42",
    literal: Some(
        Number(
            42.0,
        ),
    ),
    line: 1,
}
```

Each token prints in Rust's pretty debug format (`{:#?}`), one field per line:

- `token_type`: the token's category, such as `Hold`, `Identifier`, or `Num`
- `lexeme`: the exact source text, in quotes. A string's lexeme keeps its own quotes, which print escaped: `"\"hi\""`
- `literal`: the value the token stands for. Numbers give `Some(Number(...))`, always with a decimal point (`42` gives `42.0`); strings give `Some(String(...))`, without the quotes; `nocap` and `cap` give `Some(Boolean(true))` and `Some(Boolean(false))`. Every other token has `None`
- `line`: the line the token starts on

A token with no literal, such as a keyword, prints on six lines:

```
Token {
    token_type: Hold,
    lexeme: "hold",
    literal: None,
    line: 1,
}
```

Frozen as of Lab 1; changes are recorded in the changelog.

## Grammar

```
program             → statement* EOF
statement           → printStatement | expressionStatement
printStatement      → "spittin" expression ";"
expressionStatement → expression ";"
expression → equality
equality   → comparison ( ( "!=" | "==" ) comparison )*
comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )*
term       → factor ( ( "-" | "+" ) factor )*
factor     → unary ( ( "/" | "*" ) unary )*
unary      → ( "!" | "-" ) unary | primary
primary    → NUM | DEC | STRING | "nocap" | "cap" | "nil" | "(" expression ")"
```

- Quoted terminals are tokens. `NUM`, `DEC`, and `STRING` are the scanner's integer, decimal, and string tokens. `true` and `false` produce the same tokens as `nocap` and `cap` for now (see known limitations).
- The rules go from loosest-binding (`equality`) to tightest (`primary`), and each rule only calls the one below it. That is what makes `!` bind tighter than `==`: `!nocap == cap` parses as `(== (! nocap) cap)`.
- The `( ... )*` loops make every binary operator left-associative: `1 != 2 != 3` parses as `(!= (!= 1.0 2.0) 3.0)`.
- `unary` calls itself, so unary operators chain and group from the right: `!!nocap` parses as `(! (! nocap))`.
- Every rule above is implemented. `primary` has no identifier branch yet, so names are not expressions until variables arrive in Lab 4.
- **Program structure:** the parser returns one `Program` root containing an ordered list of expression-statement children. Each child contains the existing expression tree. The parser consumes the complete file through EOF; it does not return independent expression roots. Newlines are whitespace, so `1 +` followed by `2;` continues one expression, and a leading `-` on a later line can continue the preceding expression when no semicolon intervenes. A multiline string remains one literal node.
- `nil` is a literal, introduced in Lab 3. Its token has type `Nil` and no payload; its AST node is `Literal::Nil`.
- **Statements:** `spittin` was reserved as the output keyword from Lab 1 and is wired up in Lab 4. One token of lookahead picks the branch, so `statement` needs no backtracking. A print statement is a statement, not an expression: `spittin` cannot appear inside an expression, and `spittin spittin 1;` is a syntax error.
- **Script mode:** running a file executes its statements in order, and only `spittin` writes to stdout. A bare expression statement is still evaluated, so its runtime errors still fire, but its value is discarded. `--eval` keeps the Lab 3 behaviour of printing every expression statement's value, which is why both entry points exist.

## Parse output format

The program root is not printed as a wrapper. The printer visits its statement children and preserves the existing prefix expression format, with one output line per expression statement. Output line breaks do not represent separate AST roots.

```
(!= (group (== 1.0 2.0)) (! cap))
```

That is the output for `(1 == 2) != !cap`.

- One line per expression statement, in prefix form: the operator comes first, then its operands.
- Groupings print as: `(group ...)`
- Numbers print as: always with a decimal point, so `5` prints as `5.0` and `3.14` as `3.14`
- Strings print in double quotes with control characters and backslashes escaped: a string containing a newline prints as `"first\nsecond"` on one output line, and a literal backslash prints as `\\`
- Booleans print as `nocap` and `cap`, even when the source said `true` or `false`

## Semantics

### Values and types

Runtime values use a Rust tagged union, `Value`, with `Number(f64)`,
`String(String)`, `Boolean(bool)`, and `Nil` variants. One return type can hold
any TrapScript value; matching a variant checks its type and extracts its data.
This fits Rust's exhaustive pattern matching and the scanner's existing `f64`
representation without downcasting or adding a second numeric type.

AST `Literal` describes source syntax; `Value` also represents computed results.
The evaluator converts literals to values and recursively evaluates child nodes
before their parent. Binary operands evaluate left to right. Runtime failures
propagate through `Result<Value, RuntimeError>` with the operator token for its line.

The evaluator consumes the `Program` root and visits its expression statements in
source order. A static scan or parse error rejects the program before evaluation
starts. A runtime error stops evaluation with exit code `70`; values from earlier
statements have already been printed.

Every operator the grammar produces now evaluates: all four literal types, grouping,
unary minus and logical not, `+ - * /`, the four comparisons, and `==` and `!=`.
Operators that need numbers share one check, so they report the same diagnostic.

### Value printing

- Numbers: Rust display formatting (`5` rather than `5.0`, `3.5` for fractions). Floating-point precision and overflow follow `f64`.
- Nil: `nil`.
- Strings: contents without quotes, preserving spaces and line breaks.
- Booleans: `nocap` and `cap`.
- Parse output keeps its existing debug representation, including `5.0` and quoted strings.

### Truthiness

Only `cap` and `nil` are falsey; all other values, including zero and the empty string, are truthy. The rule lives in one function, so `!` and every later conditional answer the same way. `!` takes any value and never reports a type error. `tests/lab3/truthiness.trap` is the executable statement of this rule.

### Operator semantics

- Arithmetic: numbers only, including unary minus. Division uses floating-point arithmetic.
- `+` on strings: concatenates two strings.
- Mixed types: runtime error; no implicit coercion.
- Comparison: numbers only, on both sides; anything else is a runtime error. Returns a boolean.
- Equality across types: any pair of values, comparing the type first, so `1 == "1"` is `cap` rather than an error. `nil` equals only `nil`, so `nil == cap` is `cap` even though both are falsey.
- Division by zero: runtime error, including negative zero.

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

When a file has errors, `--tokenize` first prints the tokens it scanned before the first error, then every error in the file. All of it goes to stderr, so stdout stays empty for a rejected file, and the exit code is `65`. The REPL shows a line with errors the same way, then gives the prompt back instead of exiting.

Syntax errors from `--parse` name the line and the token where parsing failed, or `end` when parsing reaches EOF:

```
[line 1] Error at ')': Expect expression.
[line 3] Error at '+': Expect expression.
```

The parser reports syntax errors with the token line and continues recovery at a statement boundary (a semicolon) or EOF, while ensuring it makes progress. A missing semicolon, including after the last expression at EOF, is a syntax error. A missing operand is reported at the token or EOF where parsing fails. The parser publishes no program if any syntax errors occurred. Like `--tokenize`, a rejected file prints nothing to stdout and exits `65`.


| Failure | Exit code |
|---|---|
| lexical error: unexpected character or unterminated string | 65 |
| syntax error: a missing `)`, a missing operand, or a token that can't start an expression | 65 |
| runtime type error, division by zero, or an operator deferred to Week 2 | 70 |


## Testing conventions


| Folder | Activity | Mode | Flag |
|---|---|---|---|
| tests/lab0 | Onboarding | sidecar | none |
| tests/lab1 | Scanner | sidecar | `--tokenize` |
| tests/lab2 | Parser | sidecar | `--parse` |
| tests/lab3 | Evaluator | inline | `--eval` |
| tests/lab4 | Context | inline | none |
| tests/lab5 | Functions | inline | none |


```
tests/lab1/boolean_keywords_and_identifier_prefixes.trap       nocap and cap scan as booleans; capital and nocapper stay identifiers
tests/lab1/keywords/all_keywords_and_token_categories.trap     every keyword together in one program
tests/lab1/keywords/blank_lines_preserve_line_numbers.trap     blank lines between code keep line numbers right
tests/lab1/keywords/condition_with_comparisons_and_string.trap ong with >= and < and a string
tests/lab1/keywords/if_else_if_else_keyword_chain.trap         an ong / wait / nah chain
tests/lab1/keywords/decimal_literal.trap                        a decimal number
tests/lab1/keywords/false_keyword.trap                           the false keyword
tests/lab1/keywords/function_parameters_and_return_keyword.trap a motion definition with parameters and typeshi
tests/lab1/keywords/identifiers_starting_with_keywords.trap     holder, ongoing, and spinner stay identifiers, not keywords
tests/lab1/keywords/integer_literal.trap                        an integer
tests/lab1/keywords/nested_loop_condition_and_break.trap        a spin loop with a nested ong and pause
tests/lab1/keywords/logical_not_before_identifier.trap         ! before an identifier
tests/lab1/keywords/if_block_with_equality.trap                 a single ong block with ==
tests/lab1/keywords/string_literal_in_constant_declaration.trap locked with a string literal
tests/lab1/keywords/true_keyword.trap                            the true keyword
tests/lab1/errors/unexpected_at_character.trap                  unexpected @ after valid tokens; exits 65
tests/lab1/errors/unexpected_hash_character.trap                unexpected #; exits 65
tests/lab1/errors/unterminated_multiline_string.trap            unterminated string across lines; exits 65
tests/lab1/errors/multiple_unexpected_characters.trap          scanner reports more than one error; exits 65
tests/lab1/empty_file.trap                                      empty source emits only EOF
tests/lab1/strings/empty_string.trap                            empty string literal
tests/lab1/strings/multiline_string.trap                        valid string spanning two lines
tests/lab1/strings/backslash_does_not_escape_quote.trap         backslash is ordinary; the quote ends the string
tests/lab1/operators/single_and_double_character.trap          single and double character operators together
tests/lab1/operators/division_not_comment.trap                  slash between numbers is division
tests/lab1/operators/semicolon_boundaries.trap                  semicolon token, with semicolons inside strings and comments left literal
tests/lab1/comments/inline_comment_preserves_next_line.trap    inline comment is discarded; next line is counted
tests/lab1/comments/comment_at_eof_without_newline.trap        comment at EOF is discarded without a final newline
tests/lab1/numbers/trailing_decimal_point.trap                  trailing decimal point is allowed
tests/lab1/numbers/number_followed_by_identifier.trap           number ends before an identifier
tests/lab1/numbers/leading_decimal_point_rejected.trap          leading decimal point is rejected; exits 65
tests/lab1/numbers/repeated_decimal_point_rejected.trap         second decimal point is rejected; exits 65
tests/lab1/identifiers/underscore_unicode_digit_and_case.trap  Unicode, underscores, digits, and case
tests/lab1/identifiers/keyword_case_sensitive.trap              capitalized keyword remains an identifier
tests/lab1/whitespace/tabs_and_crlf_line_counting.trap          tabs and CRLF preserve line numbers

tests/lab2/expressions/literals.trap          every literal kind, one per line, including 1. and ""
tests/lab2/expressions/equality.trap          == and != over booleans, numbers, and strings
tests/lab2/expressions/associativity.trap     equality groups from the left
tests/lab2/expressions/leftassoc.trap         - and / group from the left, where the two readings differ
tests/lab2/expressions/unary.trap             chained !
tests/lab2/expressions/unaryminus.trap        unary minus: chained, on a group, and beside *
tests/lab2/expressions/precedence.trap        ! binds tighter than ==
tests/lab2/expressions/levels.trap            three levels at once: equality over comparison over term and factor
tests/lab2/expressions/mixed.trap             four or more levels in one expression
tests/lab2/expressions/grouping.trap          parentheses that change the tree
tests/lab2/expressions/redundantgroup.trap    parentheses that do not change the tree
tests/lab2/expressions/multiline_equality.trap  an expression continues across a newline
tests/lab2/expressions/multiline_statements.trap expressions and groups span lines; statements share a line; a later-line minus continues the expression
tests/lab2/expressions/multiline_string_escaped.trap  multiline strings and backslashes print on one line
tests/lab2/expressions/empty.trap             an empty file is accepted with no output
tests/lab2/errors/unclosed.trap               ( with no closing ); exits 65
tests/lab2/errors/missingoperand.trap         a binary operator with no right operand; exits 65
tests/lab2/errors/badstart.trap               a token that cannot begin an expression; exits 65
tests/lab2/errors/sameline.trap               adjacent expressions on one line without a semicolon; exits 65
tests/lab2/errors/missing_final_semicolon.trap  final expression statement without a semicolon; exits 65
tests/lab2/errors/missing_statement_semicolon.trap adjacent statements without a semicolon; exits 65
tests/lab2/errors/empty_statements.trap        bare and repeated semicolons are rejected; exits 65
tests/lab2/errors/recovery_statements.trap      malformed statements recover at semicolons; exits 65
tests/lab2/errors/multiline_string_same_line_followup.trap  a multiline string expression followed by a token without a semicolon; exits 65
```

Run locally with:

```bash
curl -sSL https://raw.githubusercontent.com/WhiteLicorice/cmsc-124-harness/v1.1/run_tests.py -o run_tests.py
./build.sh
python3 run_tests.py tests/lab0
python3 run_tests.py tests/lab1
python3 run_tests.py tests/lab2
python3 run_tests.py tests/lab3
```

On Windows, run these from Git Bash and use `python` instead of `python3`.

## Sample code

```
2 + 3 * 4;
(2 + 3) * 4;
"hi" + " there";
nil;
```

Output from `./run --eval <file>`:

```
14
20
hi there
nil
```

## Design rationale

**Vocabulary.** We chose internet/social-media slang over a
generic keyword set (`if`, `else`, `var`) because we wanted something fun and recognizable--that if you saw the keywords you'd immediately know it's TrapScript. Though, this trades familiarity for personality, a newcomer(in the sense that they came from a different programming language) can't guess that `hold` means variable declaration the way they could
guess `var`, but this tradeoff is worth it because personality creates identity. Standard syntax is sterile and forgettable, but TrapScript turns writing code into an expressive, culturally distinct experience. Once you learn the slang logic, like "holding" a variable or putting a function into "motion," the syntax becomes natural and memorable. The slight learning curve gives TrapScript a unique soul instead of just being another generic Python clone.

## Known limitations

- `true` and `false` still scan as booleans alongside `nocap` and `cap`.
- The REPL handles each submitted line as a complete program (evaluating by default, scanning with `--tokenize`, parsing with `--parse`), so line numbers restart at 1 on every line. Parse and evaluation input requires a semicolon after every expression statement. An unfinished expression or missing semicolon reports an error and returns to the prompt; the REPL does not buffer multiple lines into one program. A string cannot continue onto the next REPL input line: a multiline string that works in a file is an unterminated-string error in the REPL.
- `--parse` has no identifiers: a name like `x` is "Expect expression" until variables arrive in Lab 4.

## Changelog


| Activity | What changed in the language |
|---|---|
| Lab 1 | Initial token vocabulary and keyword set defined. Late in Lab 1 the token output format gained the `literal` field, and a string's lexeme started keeping its quotes, so every `tests/lab1` expectation was regenerated. |
| Lab 2 | Drafted the expression grammar. `==` and `!=` now bind looser than `<`, `<=`, `>`, and `>=` (they shared one precedence level in Lab 1), and `!` and unary `-` sit at the tightest level. The whole ladder is now parsed. Expression statements end with semicolons; newlines are whitespace, so expressions and groups can span lines and multiple statements can share one line. Multiline string values print with escaped line breaks. Parser recovery proceeds at semicolons or EOF. |

### Lab 3 Week 2 progress

Every operator in the grammar evaluates, with the type checks each one needs.
`tests/lab3/` uses inline expectations for literals, precedence, associativity,
string addition, comparison and its `<` against `<=` boundary, equality across
matching and mismatched types, the truthiness rule under `!`, runtime failures,
and static rejection before any evaluation.
Runtime errors use `[line N] Runtime error: message` on stderr, stop file evaluation
with exit 70, and leave the REPL usable.

### Lab 4 Week 1 progress

Programs are now observable from a file. `spittin` is wired up as a print statement,
and plain `./run <file>` executes a program instead of echoing it, with only `spittin`
writing to stdout. `--tokenize`, `--parse`, and `--eval` are unchanged, and
`tests/lab1` through `tests/lab3` still pass.

`tests/lab4/` uses inline expectations with no flag, since plain execution is what is
under test: output of every value type, script mode discarding bare expression values,
`expect nothing` for a file that intentionally prints nothing, a runtime error that
exits 70, and a missing semicolon that exits 65.
`tests/lab0/hello.src` is now a TrapScript program that prints its own expectation.
CI runs Labs 0 through 4.

Bindings, scope, and assignment arrive in Week 2.
