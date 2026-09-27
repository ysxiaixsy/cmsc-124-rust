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
| `./run <file>` | Executes a program. Not implemented yet (Lab 4); currently prints a usage error. |
| `./run --tokenize <file>` | Scans the file and prints its token stream to stdout, or every lexical error to stderr. |
| `./run --parse <file>` | Parses the file and prints one tree per expression in prefix form, or every scan or syntax error to stderr. So far it handles literals, grouping, `!`, `==`, and `!=` (Lab 2 week 1). |
| `./run --eval <file>` | Evaluates each expression and prints its value. Not implemented yet (Lab 3). |
| `./run` | Starts the REPL. Type code across as many lines as you like: Enter starts a new line (shown with a `... ` prompt), and a blank line submits the whole entry. Its tokens are printed, or its errors if it has any, and the prompt comes back. Line numbers count from 1 within each entry. The session ends when input closes with Ctrl+C. |


Exit codes: `0` when the file is accepted, `65` when it is rejected before running (a lexical error from the scanner or a syntax error from the parser), `70` for runtime errors (not used until Lab 3).

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

- Whitespace significant: No. Newlines are treated as standard whitespace.
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
expression → equality
equality   → comparison ( ( "!=" | "==" ) comparison )*
comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )*
term       → factor ( ( "-" | "+" ) factor )*
factor     → unary ( ( "/" | "*" ) unary )*
unary      → ( "!" | "-" ) unary | primary
primary    → NUM | DEC | STRING | "nocap" | "cap" | "(" expression ")"
```

- Quoted terminals are tokens. `NUM`, `DEC`, and `STRING` are the scanner's integer, decimal, and string tokens. `true` and `false` produce the same tokens as `nocap` and `cap` for now (see known limitations).
- The rules go from loosest-binding (`equality`) to tightest (`primary`), and each rule only calls the one below it. That is what makes `!` bind tighter than `==`: `!nocap == cap` parses as `(== (! nocap) cap)`.
- The `( ... )*` loops make every binary operator left-associative: `1 != 2 != 3` parses as `(!= (!= 1.0 2.0) 3.0)`.
- `unary` calls itself, so unary operators chain and group from the right: `!!nocap` parses as `(! (! nocap))`.
- **Implemented so far (Lab 2 week 1):** `expression`, `equality`, the `"!"` branch of `unary`, and `primary`. Until `comparison`, `term`, and `factor` exist, `equality` calls `unary` directly.
- **Splitting a file into expressions:** an expression ends where the grammar says it ends, and the next one has to start on a new line. An unfinished expression continues onto the next line, so `1 ==` followed by `2` on the next line is one expression. An empty file has no expressions and is accepted.
- TrapScript has no nil, so `primary` has no nil literal.

## Parse output format

```
(!= (group (== 1.0 2.0)) (! cap))
```

That is the output for `(1 == 2) != !cap`.

- One line per expression, in prefix form: the operator comes first, then its operands.
- Groupings print as: `(group ...)`
- Numbers print as: always with a decimal point, so `5` prints as `5.0` and `3.14` as `3.14`
- Strings print in double quotes: `"hi"`
- Booleans print as `nocap` and `cap`, even when the source said `true` or `false`

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

When a file has errors, `--tokenize` first prints the tokens it scanned before the first error, then every error in the file. All of it goes to stderr, so stdout stays empty for a rejected file, and the exit code is `65`. The REPL shows an entry with errors the same way.

Syntax errors from `--parse` name the line and the token where parsing failed, or `end` for the end of the file:

```
[line 1] Error at ')': Expect expression.
[line 3] Error at '+': Expect expression.
```

The parser reports every syntax error in the file: after an error it skips the rest of that line and carries on with the next one. Like `--tokenize`, a rejected file prints nothing to stdout and exits `65`.


| Failure | Exit code |
|---|---|
| lexical error: unexpected character or unterminated string | 65 |
| syntax error: a missing `)`, a missing operand, or a token that can't start an expression | 65 |
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
tests/lab1/comments/inline_comment_preserves_next_line.trap    inline comment is discarded; next line is counted
tests/lab1/comments/comment_at_eof_without_newline.trap        comment at EOF is discarded without a final newline
tests/lab1/numbers/trailing_decimal_point.trap                  trailing decimal point is allowed
tests/lab1/numbers/number_followed_by_identifier.trap           number ends before an identifier
tests/lab1/numbers/leading_decimal_point_rejected.trap          leading decimal point is rejected; exits 65
tests/lab1/numbers/repeated_decimal_point_rejected.trap         second decimal point is rejected; exits 65
tests/lab1/identifiers/underscore_unicode_digit_and_case.trap  Unicode, underscores, digits, and case
tests/lab1/identifiers/keyword_case_sensitive.trap              capitalized keyword remains an identifier
tests/lab1/whitespace/tabs_and_crlf_line_counting.trap          tabs and CRLF preserve line numbers
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

## Known limitations

- `true` and `false` still scan as booleans alongside `nocap` and `cap`.
- `./run <file>` without a flag prints a usage error instead of the file's contents, so `tests/lab0` fails and is left out of CI.
- Tokens do not carry a literal value yet.
- `--parse` only handles literals, grouping, `!`, `==`, and `!=` so far. `<`, `+`, `*`, unary `-`, and the rest are a syntax error until their grammar levels are added.

## Changelog


| Activity | What changed in the language |
|---|---|
| Lab 1 | Initial token vocabulary and keyword set defined. |
| Lab 2 | Drafted the expression grammar. `==` and `!=` now bind looser than `<`, `<=`, `>`, and `>=` (they shared one precedence level in Lab 1), and `!` and unary `-` sit at the tightest level. |
