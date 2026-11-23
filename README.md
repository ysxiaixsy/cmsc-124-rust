<h1 align="center">
  <img src="images/trapscript-wordmark.svg" alt="TrapScript" width="360">
</h1>

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
| `./run` | Starts the evaluator REPL. Each input line is a complete program and must include a semicolon after every statement. Values print in statement order; errors report a diagnostic and return the prompt. Bindings persist across lines for the whole session, including after an error. End with Ctrl+Z then Enter on Windows, or Ctrl+D on Linux/macOS. |
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
| `\|\|` | logical | binary | left | 2 |
| `&&` | logical | binary | left | 3 |
| `==` | equality | binary | left | 4 |
| `!=` | equality | binary | left | 4 |
| `<` | comparison | binary | left | 5 |
| `<=` | comparison | binary | left | 5 |
| `>` | comparison | binary | left | 5 |
| `>=` | comparison | binary | left | 5 |
| `+` | arithmetic | binary | left | 6 |
| `-` | arithmetic | binary | left | 6 |
| `*` | arithmetic | binary | left | 7 |
| `/` | arithmetic | binary | left | 7 |
| `!` | logical | unary | right | 8 |
| `-` | arithmetic | unary | right | 8 |
| `( )` | call | a callee and any number of arguments | left | 9 |

A higher number binds tighter. `&&` and `||` exist only doubled: a lone `&` or `|` is an unexpected character.

### Punctuation

| Symbol | Purpose |
|---|---|
| `,` | separates function parameters and call arguments |
| `;` | terminates a statement or declaration |

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
- Statement terminator: `;` (Semicolon). Every statement and declaration needs one, including the final one immediately before EOF.
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
program             → declaration* EOF
declaration         → motionDeclaration | holdDeclaration | lockedDeclaration | statement
motionDeclaration   → "motion" IDENTIFIER "(" parameters? ")" block
parameters          → IDENTIFIER ( "," IDENTIFIER )*
holdDeclaration     → "hold" IDENTIFIER ( "=" expression )? ";"
lockedDeclaration   → "locked" IDENTIFIER "=" expression ";"
statement           → block | ongStatement | spinStatement | pauseStatement | returnStatement
                    | printStatement | expressionStatement
block               → "{" declaration* "}"
ongStatement        → "ong" "(" expression ")" statement
                      ( "wait" "(" expression ")" statement )* ( "nah" statement )?
spinStatement       → "spin" "(" expression ")" statement
pauseStatement      → "pause" ";"
returnStatement     → "typeshi" expression? ";"
printStatement      → "spittin" expression ";"
expressionStatement → expression ";"
expression → assignment
assignment → IDENTIFIER "=" assignment | logicOr
logicOr    → logicAnd ( "||" logicAnd )*
logicAnd   → equality ( "&&" equality )*
equality   → comparison ( ( "!=" | "==" ) comparison )*
comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )*
term       → factor ( ( "-" | "+" ) factor )*
factor     → unary ( ( "/" | "*" ) unary )*
unary      → ( "!" | "-" ) unary | call
call       → primary ( "(" arguments? ")" )*
arguments  → expression ( "," expression )*
primary    → NUM | DEC | STRING | "nocap" | "cap" | "nil" | IDENTIFIER | "(" expression ")"
```

- Quoted terminals are tokens. `NUM`, `DEC`, and `STRING` are the scanner's integer, decimal, and string tokens. `true` and `false` produce the same tokens as `nocap` and `cap` for now (see known limitations).
- **Control flow:** `ong` and `spin` take a parenthesized condition and a single statement, which is usually a block. Because the branch is a statement, it nests anywhere a statement can.
- **`wait` is sugar.** Each `wait` becomes the else branch of the `ong` before it, holding another `ong`, so the tree carries only condition, then, and else. `ong (a) {…} wait (b) {…} nah {…}` prints as `(ong a (block …) (ong b (block …) (block …)))`. Chaining a `nah` onto another `ong` by hand means the same thing.
- **Dangling else:** a `nah` binds to the nearest unmatched `ong`. That falls out of recursive descent rather than a rule, because the innermost unfinished `ong` call is the one holding the parser when `nah` appears.
- **Conditions use the truthiness rule from Semantics:** only `cap` and `nil` are falsey, so `ong (0)` and `ong ("")` both take the then branch.
- **Only the chosen branch is executed.** The other is never walked, so a runtime error sitting in it cannot fire. `tests/lab5/conditionals/only_chosen_branch.trap` is the executable proof.
- `spin` tests its condition before every pass, so a false condition runs the body zero times. It is the first construct that can loop forever; the harness kills a test file after 15 seconds.
- **Logical operators:** `||` and `&&` sit between assignment and equality, with `&&` the tighter of the two, so `a || b && c` parses as `(|| a (&& b c))` and `x = a || b` stores the whole expression. Both group from the left. They get a tree node of their own, `Logical`, because the evaluator may skip the right operand, which it never does for a `Binary`.
- **Functions:** `motion` is a declaration like `hold`, so it is legal wherever a statement is, including inside a block or another motion. The body must be a block. Parameter names must be identifiers and must differ from each other: `motion f(a, a) {}` is a syntax error.
- **Calls:** a call is a postfix on any primary, and the rule loops, so a call's result can be called in turn: `f(1)(2)`. Calls bind tighter than unary operators, so `-f(x)` negates the result. An argument is any expression, including an assignment. A call is not an assignment target: `f() = 1;` is rejected with exit `65`.
- **`typeshi`** takes an optional expression and is only legal inside a motion body. Anywhere else, including a block or a loop at the top level, is a syntax error, so the file is rejected before it runs.
- **`pause`** is only legal inside a `spin` body, at any depth of blocks and `ong`s within it. A motion body starts with no loops around it, even when the `motion` declaration itself sits inside a `spin`, because a `pause` in the body would run during a call, not during that loop. Anywhere else is the syntax error `Can't use 'pause' outside a spin.`, exit `65`.
- **`locked`** declares like `hold`, but the value is required: a locked name can never be assigned later, so one declared without a value would be stuck at `nil`. `locked x;` is the syntax error `Expect '=' after locked name.`, exit `65`.
- The rules go from loosest-binding (`assignment`) to tightest (`primary`), and each rule only calls the one below it. That is what makes `!` bind tighter than `==`: `!nocap == cap` parses as `(== (! nocap) cap)`.
- The `( ... )*` loops make every binary operator left-associative: `1 != 2 != 3` parses as `(!= (!= 1.0 2.0) 3.0)`.
- `unary` calls itself, so unary operators chain and group from the right: `!!nocap` parses as `(! (! nocap))`.
- Every rule above is implemented. An identifier in an expression reads that binding.
- **Where bindings are legal:** `hold` is a declaration, so it can begin a statement but cannot appear inside an expression: `spittin hold x = 1;` is a syntax error. The initializer is optional, and the name must be an identifier, so a keyword cannot be declared.
- **Assignment** is an expression, legal anywhere an expression is, and it produces the assigned value: `spittin x = 2;` prints `2`, and `hold y = (x = 3);` binds both. `assignment` calls itself on the right, so `a = b = 1` groups as `a = (b = 1)`.
- **Assignment targets:** the parser reads the left side as an ordinary expression, then checks that it is a bare name when it finds `=`. `1 = 2;`, `(a) = 2;`, `-a = 2;`, and `a + b = 3;` are all rejected before anything runs, with exit `65`.
- **Nesting limit:** an expression's tree can be at most 10,000 levels deep. A group, a unary operator, an assignment, a call, and each binary or logical operator add a level, so `1 + 1 + 1` is two levels deep because `+` groups from the left, and `(a) + (b)` counts only its deeper side plus one. A call counts one more than the deepest of its callee and its arguments, so `f(g(1))` and `f()()` are both two levels deep. Each enclosing block also adds a level, and a motion body is a block, so blocks can nest at most 10,000 deep, and a statement's expression starts counting from the depth of the blocks around it. Each top-level statement starts again from zero. Past the limit is a syntax error with exit `65`. The parser, evaluator, and printer all recurse once per level, so the interpreter runs on a 256 MB stack, and the limit keeps every expression well inside it instead of letting the host stack overflow and abort the process.
- **Program structure:** the parser returns one `Program` root containing an ordered list of statement children. Each child contains the existing expression tree. The parser consumes the complete file through EOF; it does not return independent expression roots. Newlines are whitespace, so `1 +` followed by `2;` continues one expression, and a leading `-` on a later line can continue the preceding expression when no semicolon intervenes. A multiline string remains one literal node.
- `nil` is a literal, introduced in Lab 3. Its token has type `Nil` and no payload; its AST node is `Literal::Nil`.
- **Statements:** `spittin` was reserved as the output keyword from Lab 1 and is wired up in Lab 4. One token of lookahead picks the branch, so `statement` needs no backtracking. A print statement is a statement, not an expression: `spittin` cannot appear inside an expression, and `spittin spittin 1;` is a syntax error.
- **Blocks:** `{` begins a block, which holds any number of declarations and statements and ends at the matching `}`. A block is itself a statement, so blocks nest, and no semicolon follows the `}`. `{}` is an empty block. A block that reaches EOF without its `}` is a syntax error.
- **Script mode:** running a file executes its statements in order, and only `spittin` writes to stdout. A bare expression statement is still evaluated, so its runtime errors still fire, but its value is discarded. `--eval` keeps the Lab 3 behaviour of printing every expression statement's value, which is why both entry points exist.

## Parse output format

The program root is not printed as a wrapper. The printer visits its statement children and preserves the existing prefix expression format, with one output line per statement. Output line breaks do not represent separate AST roots.

```
(!= (group (== 1.0 2.0)) (! cap))
```

That is the output for `(1 == 2) != !cap`.

- One line per statement, in prefix form: the operator comes first, then its operands.
- Statements with a keyword lead with it: `(spittin 1.0)`, `(hold x 1.0)`, and `(hold x)` when there is no initializer. `locked x = 1;` prints as `(locked x 1.0)` and `pause;` as `(pause)`.
- Names print bare, and assignment prints as an operator: `(= x (+ x 1.0))`.
- A block prints on one line with its statements in order: `{ hold x = 1; spittin x; }` prints as `(block (hold x 1.0) (spittin x))`, and `{}` prints as `(block)`.
- A motion prints its name, its parameters in their own parentheses, and its body as a block: `motion add(a, b) { typeshi a + b; }` prints as `(motion add (a b) (block (typeshi (+ a b))))`. With no parameters the slot stays, as `()`. A bare `typeshi;` prints as `(typeshi)`.
- A call prints its callee and then its arguments: `add(1, 2)` prints as `(call add 1.0 2.0)`, `f()` as `(call f)`, and `f(1)(2)` as `(call (call f 1.0) 2.0)`.
- `&&` and `||` print the way the other two-operand operators do: `(|| a (&& b c))`.
- Groupings print as: `(group ...)`
- Numbers print as: always with a decimal point, so `5` prints as `5.0` and `3.14` as `3.14`
- Strings print in double quotes with control characters and backslashes escaped: a string containing a newline prints as `"first\nsecond"` on one output line, and a literal backslash prints as `\\`
- Booleans print as `nocap` and `cap`, even when the source said `true` or `false`

## Semantics

### Values and types

Runtime values use a Rust tagged union, `Value`, with `Number(f64)`,
`String(String)`, `Boolean(bool)`, `Nil`, `Function`, `Native`, and `Stash` variants. One return type can hold
any TrapScript value; matching a variant checks its type and extracts its data.
This fits Rust's exhaustive pattern matching and the scanner's existing `f64`
representation without downcasting or adding a second numeric type.

A function value is a shared reference to the `motion` declaration plus the scope
that declaration ran in. Binding it to a second name copies the reference, so both
names call the same function.

A stash is an ordered collection of any values, made with `stash()` and changed with
`stack` and `put` (see Native functions). It is shared the same way a function is:
binding it to a second name or passing it to a motion hands over the same stash, so a
change made through one name is seen through every other. It is the only value that can
change after it is made, which is why it is the only one shared rather than copied.

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
- Functions: `<motion name>`, with the name from the declaration even when the value is bound to another name. A native function prints as `<native name>`.
- Stashes: their items in brackets, separated by `, `: `[1, "two", nocap, nil]`. A string inside a stash keeps its quotes, so `["1"]` and `[1]` look different. A stash that holds itself prints `[...]` at that point instead of printing forever.
- Parse output keeps its existing debug representation, including `5.0` and quoted strings.

### Truthiness

Only `cap` and `nil` are falsey; all other values, including zero, the empty string, every function, and every stash, empty or not, are truthy. The rule lives in one function, so `!`, the conditionals, and the logical operators all answer the same way. `!` takes any value and never reports a type error. `tests/lab3/values/truthiness.trap` is the executable statement of this rule.

### Operator semantics

- Arithmetic: numbers only, including unary minus. Division uses floating-point arithmetic.
- `+` on strings: concatenates two strings.
- Mixed types: runtime error; no implicit coercion.
- Comparison: numbers only, on both sides; anything else is a runtime error. Returns a boolean.
- Equality across types: any pair of values, comparing the type first, so `1 == "1"` is `cap` rather than an error. `nil` equals only `nil`, so `nil == cap` is `cap` even though both are falsey.
- Equality of functions: by identity. A function equals itself under any name it is bound to. Two declarations with the same text are different functions, and so are the two functions made by running one declaration twice.
- Equality of stashes: by identity, like functions. A stash equals itself under any name, and two stashes with the same items are still unequal. Comparing items would also never finish on a stash that holds itself.
- Division by zero: runtime error, including negative zero.
- `||` and `&&`: any operands, never a type error. `||` evaluates its left operand and produces it if it is truthy; otherwise it evaluates and produces its right operand. `&&` produces its left operand if that is falsey, and otherwise its right. See Control flow and functions.

### Scope and bindings

- Paradigm: imperative. `hold` creates a binding, `=` changes it in place, and `spittin` makes results observable.
- Mutability: a `hold` binding is mutable. Typing is dynamic, so assignment can change both a binding's value and its type. A `locked` binding is not: assigning to it is the runtime error `Can't assign to locked 'x'.`, and declaring the same name again in the same scope, with `hold`, `locked`, or `motion`, is `Can't redeclare locked 'x'.`, both exit `70`. Locking belongs to one binding, not to the name: an inner block or a parameter can declare its own `x` and change it freely, and the locked one is back after the block. A locked value is worked out once, when the declaration runs, and changing what it was built from afterwards does not reach it. Locking a stash fixes which stash the name holds, not what is in it.
- Declaring against assigning: `hold` binds in the innermost environment without searching and cannot fail. `=` only updates an existing binding and never creates one, so a misspelled name is an error rather than a new variable.
- Redeclaration in the same scope: allowed, unless the name is locked. A later `hold` replaces the earlier binding, which keeps retyping a line at the REPL painless. The initializer runs first, so `hold x = x + 1;` reads the old `x`, and `hold x;` resets it to `nil`.
- Uninitialized variable holds: `nil`.
- Visibility: a binding becomes visible once its `hold` finishes, so a first declaration cannot read itself in its own initializer.
- Scope: static (lexical). The program starts in one global environment, and each block runs in a new environment whose enclosing environment is the one the block sits in. A read or an assignment searches the innermost environment first, then each enclosing one out to global, and stops at the first binding of that name; if none has it, the name is undefined. The search follows how the source nests, never the order of calls, so a function looks up names from where it was declared, not from where it was called. A call runs in a new environment whose enclosing environment is the one the `motion` declaration ran in.
- Why static: what a name in a function body means can be read off the source around the declaration, and no caller can change it by happening to have a local of the same name. The cost is that a function cannot pick up a value from whoever called it; anything it needs from the caller has to be passed as an argument. Dynamic scope would allow that implicit passing, but every nonlocal name would then depend on the chain of active calls. `tests/lab5/functions/static_scope.trap` is the distinguishing test: a caller and the declaration site bind the same name differently, and the function prints the declaration site's. `tests/lab5/errors/runtime/no_dynamic_scope.trap` shows the other half, a caller's local that the function cannot see at all.
- Shadowing: a `hold` inside a block always creates a new binding in that block, even when an enclosing environment already has the name. The inner binding hides the outer one until the block ends; the outer one stays alive and unchanged, and is visible again after the `}`. Assigning with `=` inside the block changes whichever binding the search finds first, so it changes the outer binding only when the block has not shadowed it.
- Shadowing an outer name with itself: inside a block, `hold x = x;` reads the outer `x` and then creates the inner one, because the initializer runs before the name is bound. This follows the same rule as `hold x = x + 1;` at one level, so there is no special case and no separate resolution pass. `hold y = y;` still fails when no environment has a `y`.
- Binding time: a name binds to storage and to its first value when `hold` executes, and to a new value at each assignment. Its type is never declared; it follows whichever value it currently holds.
- Scope against lifetime: a block's binding can be named from its `hold` to the block's `}`. Its storage exists while that block is running and is released when the block exits, so reading it after the `}` is `Undefined variable`. A shadowed binding shows the difference: it is alive inside the inner block but cannot be named there.
- Storage lifetime: a global binding lasts from the moment `hold` executes until the program ends, or until the session ends in the REPL, so it is static storage in the sense of lasting for the program. A block's bindings are stack-dynamic: created when their `hold` runs and released when the block exits, including when it exits through a runtime error. A call's parameters and locals behave the same way and are released when the call returns. The one exception is a scope that a function was declared in: the function keeps that scope, so its bindings live for as long as the function can still be called, which is what makes closures work. Environments are reference-counted to allow this.
- Values are copied on binding: after `hold b = a;`, assigning to `a` leaves `b` unchanged. A stash is the exception: `b` holds the same stash, so `stack(a, 1)` is seen through `b`. Assigning a different value to `a` still leaves `b` alone.
- Undefined name: runtime error with exit 70 and `Undefined variable 'a'.`, for both reading and assigning. The check happens when the name is evaluated, so code that never runs never reports, and mutually recursive functions need no separate resolution pass: a body can mention a function declared further down, as long as that declaration has run by the time of the call. The right side of an assignment runs first, so `z = 1 / 0;` reports the division by zero even if `z` is undefined.

### Control flow and functions

- Logical operators return: the operand that settled the answer, not a boolean. `nil || "guest"` is `guest`, `0 || 1` is `0` because zero is truthy, and `cap && 1` is `cap`.
- Short-circuiting: the right operand is evaluated only when the left one does not settle the answer. When it is skipped, its side effects do not happen and its runtime errors do not fire: `nocap || 1 / 0` is `nocap`.
- Dangling else binds to: the nearest unmatched `ong`. `tests/lab5/conditionals/dangling_else.trap` runs both unbraced cases where the two readings differ.
- Locals across calls: every call gets a new environment, so a `hold` in a body starts over on each call. State that should persist lives outside the call, in a global or in a captured scope. `tests/lab5/functions/fresh_locals.trap` shows both, and that a caller's bindings are intact after the callee returns early.
- Declaring a function: `motion name(params) { ... }` binds `name` in the innermost environment, exactly as `hold` would, to a new function value. It can sit inside a block or another function, and a later declaration of the same name in the same scope replaces the earlier one. Nothing in the body runs until a call.
- Functions are values: a function can be printed, bound to another name, passed as an argument, and returned as a result.
- Calling: the callee is evaluated, then each argument from left to right, then the callee is checked. Each argument is bound to the parameter in its position, in a new environment that the body's statements also run in, so a parameter and a `hold` in the body share one scope. Arguments are passed by value for numbers, strings, booleans, and `nil`, and by reference for functions and stashes, so a motion handed a stash can change the caller's stash. Assigning to the parameter itself never reaches the caller.
- `typeshi` ends the call it runs in and gives the call its value, from however many blocks, `ong`s, and `spin`s it sits inside. A `typeshi;` with no expression gives `nil`.
- `pause` ends the nearest `spin` around it and nothing further out: in nested loops only the inner one stops, and the statement after that loop runs next. Like `typeshi`, it passes up through every block and `ong` between itself and its loop, so the rest of the body is skipped. `tests/lab5/repetition/pause*.trap` cover a single loop, nested loops, and a loop inside a motion.
- Function with no return statement produces: `nil`.
- Recursion: a function can call itself, and two functions can call each other, because a name is looked up when the call runs.
- Closures: a function keeps the environment it was declared in, and reads and assigns the live bindings there, not copies. A function returned from a call keeps that call's parameters and locals alive, and each call makes a separate set.
- Closure capture of a loop variable: `spin` has no loop variable of its own. A `hold` inside the loop body is new on every pass, so a function declared in the body captures that pass's binding. A counter declared outside the loop is one binding, shared by every pass and every function that reads it.
- Arity mismatch: runtime error `Expected 2 arguments but got 1.`, exit `70`. The body does not start. A missing argument is not filled with `nil` and a spare one is not dropped.
- Calling something that is not a function: runtime error `Can only call motions.`, exit `70`.
- Call depth: calls run on the interpreter's own 256 MB stack. A call is refused with the runtime error `Stack overflow.`, exit `70`, once three quarters of that stack is in use, so unbounded recursion ends in a diagnostic instead of crashing the process. The limit is measured in stack, not in calls, so the depth it allows depends on how deeply each call nests and on the build: a minimal recursive function gets about 200,000 calls deep in the release build, and `tests/lab5/functions/recursion.trap` holds every build to at least 10,000.
- An error inside a function reports the line it happened on, not the line of the call, and stops the whole program. A failed call reports the line of its `(`.

## Native functions


| Name | Arguments | Returns | Notes |
|---|---|---|---|
| `spill` | none | a string, or `nil` | Reads the next line of standard input and returns it without its line ending (`\n` or `\r\n`). Nothing else is trimmed, so an empty line is `""`. Returns `nil` once the input has run out. The result is always a string, even when the line spells a number. Input that is not valid UTF-8 is a runtime error, exit `70`. |
| `digits` | one string, or `nil` | a number, or `nil` | Returns the number the string spells, or `nil` when it does not spell one. The spelling is the one number literals use (`42`, `3.14`, `1.`), with spaces around it ignored and a leading `-` allowed. So `.5`, `1e3`, `+5`, `1.2.3`, `12abc`, `inf`, and `""` are all `nil`. `digits(nil)` is `nil`, so `digits(spill())` is `nil` at the end of the input. Any other argument (a number, a boolean, a function) is the runtime error `Argument to digits must be a string.`, exit `70`. |
| `sayit` | any value | a string | Returns the text `spittin` would print for the value: `sayit(10)` is `"10"`, `sayit(nocap)` is `"nocap"`, and a string comes back unchanged. This is the only way a number becomes text, so `"score: " + sayit(10)` works while `"score: " + 10` is still a runtime error. `digits(sayit(n))` is `n` again. |
| `size` | a string or a stash | a number | The number of characters in a string (characters, not bytes, so `size("é")` is `1`), or the number of values in a stash. Anything else is `Argument to size must be a string or a stash.` |
| `slice` | a string, then two positions | a string | `slice(text, from, to)` is the characters from position `from` up to, not including, `to`, counting from `0`. `slice(text, 0, size(text))` is the whole string, and equal positions give `""`. Positions must be whole and not negative (`Slice positions must be whole numbers.`), and must fit: `slice("trap", 2, 9)` is `Can't slice 2 to 9 from a string of 4 characters.` rather than a shorter string. |
| `has` | two strings | a boolean | Whether the second string appears anywhere in the first, matching case exactly. Every string has `""`. Anything other than two strings is `Arguments to has must be two strings.` |
| `stash` | none | a new stash | Every call makes a separate, empty stash. |
| `stack` | a stash, then any value | `nil` | Puts the value on the end of the stash, changing the stash itself. Answers `nil`, because the change is the point. |
| `grab` | a stash, then a position | the value there | The value at that position, counting from `0`. A position with nothing there is `Position 3 is out of range for a stash of 3.`, not `nil`, because `nil` could be a value the program stacked. |
| `put` | a stash, a position, then any value | `nil` | Replaces the value at a position that already holds one. It never grows the stash, so a wrong position is an error instead of a gap; `stack` is the only way to add. |

```
hold total = 0;
hold line;
spin (line = spill()) {
  hold n = digits(line);
  ong (n == nil) { spittin "skipped: " + line; } nah { total = total + n; }
}
spittin total;
```

Run as `./run sum.trap < numbers.txt`, this adds up the lines that are numbers and names the
ones that are not. The loop ends at the end of the input because `nil` is falsey, and it does
not end at a blank line because the empty string is truthy. When the text itself is not
needed, the two calls chain: `digits(spill())` is the next line as a number, and `nil` both
for a line that is not a number and at the end of the input.

- A native is implemented in Rust and bound in the global environment before any user code runs. It is called exactly like a motion, and its argument count is checked the same way and before it runs, so `spill(1)` reports `Expected 0 arguments but got 1.` without reading anything.
- A native is a value: it prints as `<native spill>`, can be bound to another name or passed as an argument, is truthy, and equals only itself.
- `digits` answers `nil` for a string that is not a number so that a program can check its input, and `nil` is the only falsey thing it returns: `digits("0")` is `0`, which is truthy, so `digits(text) || 10` supplies a default without swallowing a zero.
- The names are not reserved. A program can shadow `spill` in a block or replace it with a `hold` or a `motion` of the same name.
- Every native that takes a position (`slice`, `grab`, `put`) refuses one that is not a whole number, rather than rounding it, and refuses one that does not fit, rather than cutting it down or answering `nil`. Asking for something that is not there is a bug in the program, and quietly handing back less would hide it.
- The stash natives name themselves when given something that is not a stash: `stack("text", 1)` is `First argument to stack must be a stash.`
- Output stays a statement: `spittin` is the only thing that writes to stdout, and no native prints.

```
hold prices = stash();
stack(prices, 5);
stack(prices, 15);
stack(prices, 20);
hold total = 0;
hold i = 0;
spin (i < size(prices)) {
  total = total + grab(prices, i);
  i = i + 1;
}
spittin "total: " + sayit(total);
```

This prints `total: 40`. A `spin` walks a stash by position from `0` up to its size, and
`sayit` is what lets the number join the string.
- In the REPL, `spill` reads from the same input as the prompt, so it takes the next line typed.


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
[line 128] Error at '(': Expression nested too deeply (limit is 10000 levels).
[line 4] Error at end: Expect '}' after block.
[line 1] Error at '{': Blocks nested too deeply (limit is 10000 levels).
[line 2] Error at 'a': Duplicate parameter name 'a'.
[line 1] Error at ';': Expect ')' after arguments.
[line 4] Error at 'typeshi': Can't use 'typeshi' outside a motion.
[line 1] Error at 'pause': Can't use 'pause' outside a spin.
[line 1] Error at ';': Expect '=' after locked name.
```

Runtime errors name the line and stop the program:

```
[line 3] Runtime error: Expected 2 arguments but got 1.
[line 4] Runtime error: Can only call motions.
[line 5] Runtime error: Stack overflow.
[line 3] Runtime error: Can't assign to locked 'x'.
[line 4] Runtime error: Position 1 is out of range for a stash of 1.
```

The parser reports syntax errors with the token line and continues recovery at a statement boundary, or EOF, while ensuring it makes progress. A statement boundary is just after a semicolon, or just before a token that starts a statement (`hold`, `locked`, `motion`, `typeshi`, `pause`, `spittin`, `{`) or ends a block (`}`). Stopping before `}` means an error inside a block is recorded and the block still parses to its own closing brace, so one mistake does not turn the rest of the block, or its `}`, into further errors. A `}` with no open block is reported once and skipped. A block nested past the limit is reported once and skipped through its matching `}`. A motion whose header is malformed is reported once and its body is skipped through its matching `}`: parsed as a top-level block instead, every `typeshi` inside it would be reported again as being outside a motion. A missing semicolon, including after the last expression at EOF, is a syntax error. A missing operand is reported at the token or EOF where parsing fails. The parser publishes no program if any syntax errors occurred. Like `--tokenize`, a rejected file prints nothing to stdout and exits `65`.


| Failure | Exit code |
|---|---|
| lexical error: unexpected character (including a lone `&` or `\|`) or unterminated string | 65 |
| syntax error: a missing `)`, a missing operand, a token that can't start an expression, a malformed `hold`, `locked`, or `motion`, a duplicate parameter name, a `typeshi` outside a motion, a `pause` outside a `spin`, an invalid assignment target, an unclosed block or a stray `}`, or an expression or block nested past the limit | 65 |
| runtime type error, division by zero, or an undefined variable | 70 |
| assigning to a locked binding, or redeclaring one in its own scope | 70 |
| a call with the wrong number of arguments, a call of something that is not a function, or calls nested past the stack limit | 70 |
| `spill` failing to read standard input, such as input that is not valid UTF-8, or `digits` given something that is neither a string nor `nil` | 70 |
| a native given the wrong kind of argument, or a position that is not whole or does not fit (`slice`, `grab`, `put`) | 70 |


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
tests/lab1/keywords/boolean_keywords_and_identifier_prefixes.trap nocap and cap scan as booleans; capital and nocapper stay identifiers
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
tests/lab1/whitespace/empty_file.trap                           empty source emits only EOF
tests/lab1/literals/source_text_and_values.trap                 lexemes and literal values for numbers, strings, and booleans
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
tests/lab2/statements/block.trap               nested and empty blocks print as (block ...) on one line

tests/lab3/values/literals.trap                     every literal evaluates to its value, including the empty string
tests/lab3/values/truthiness.trap                   only cap and nil are falsey under !; zero and "" are truthy
tests/lab3/operators/arithmetic.trap                + - * / with precedence, left associativity, unary minus, and string +
tests/lab3/operators/comparison.trap                the four comparisons, including the < against <= boundary
tests/lab3/operators/equality.trap                  == and != across matching and mismatched types
tests/lab3/operators/precedence_chain.trap          term over comparison over equality in one expression
tests/lab3/statements/multiline_statements.trap     statements span and share lines; semicolons in strings and comments stay literal
tests/lab3/statements/comment_only.trap             a file of only comments prints nothing
tests/lab3/nesting/parens_at_limit.trap             10000 nested groups still run
tests/lab3/nesting/unary_at_limit.trap              10000 chained ! still run
tests/lab3/nesting/chain_at_limit.trap              10000 + operators still run
tests/lab3/nesting/siblings_do_not_add_up.trap      two sides 6000 deep count as the deeper one, not their sum
tests/lab3/nesting/limit_is_per_statement.trap      two statements 6000 deep each count from zero
tests/lab3/errors/runtime/division_by_zero.trap     dividing by negative zero stops evaluation but keeps earlier output; exits 70
tests/lab3/errors/runtime/mixed_addition.trap       + on a string and a number; exits 70
tests/lab3/errors/runtime/wrong_arithmetic_type.trap - on a string; exits 70
tests/lab3/errors/runtime/wrong_comparison_type.trap < on a string; exits 70
tests/lab3/errors/runtime/wrong_negation_type.trap  unary minus on nil; exits 70
tests/lab3/errors/syntax/missing_final_semicolon.trap a final expression with no semicolon rejects the whole file; exits 65
tests/lab3/errors/syntax/recovery_statements.trap   three malformed statements all report after recovery; exits 65
tests/lab3/errors/syntax/static_error.trap          an unclosed group rejects the file before anything evaluates; exits 65
tests/lab3/errors/syntax/nesting_parens_over_limit.trap 10001 nested groups are rejected and nothing runs; exits 65
tests/lab3/errors/syntax/nesting_unary_over_limit.trap 10001 chained ! are rejected; exits 65
tests/lab3/errors/syntax/nesting_equality_over_limit.trap 10001 == operators are rejected; exits 65
tests/lab3/errors/syntax/nesting_comparison_over_limit.trap 10001 < operators are rejected; exits 65
tests/lab3/errors/syntax/nesting_term_over_limit.trap 10001 + operators are rejected; exits 65
tests/lab3/errors/syntax/nesting_factor_over_limit.trap 10001 * operators are rejected; exits 65
tests/lab3/errors/syntax/nesting_grows_through_group.trap a chain inside a group plus a chain outside it pass the limit together; exits 65
tests/lab3/errors/syntax/nesting_right_operand_counts.trap a deep right operand counts toward its operator's depth; exits 65

tests/lab4/output/hello.trap                                  spittin prints every value type
tests/lab4/output/script_mode.trap                            bare expression values are discarded between spittin statements
tests/lab4/output/silent_expressions.trap                     a file of bare expressions prints nothing
tests/lab4/bindings/hold_and_lookup.trap                      hold binds every value type; names read back alone and inside expressions
tests/lab4/bindings/hold_without_initializer.trap             hold with no initializer holds nil until assigned
tests/lab4/bindings/redeclaration.trap                        a later hold replaces the binding, and its initializer reads the old value
tests/lab4/bindings/names_are_exact.trap                      case, digits, underscores, and keyword prefixes make distinct names
tests/lab4/assignment/updates_binding.trap                    = changes a binding's value and its type
tests/lab4/assignment/as_expression.trap                      assignment produces its value, chains from the right, and nests in expressions
tests/lab4/assignment/copies_value.trap                       assigning to one name leaves a name bound from it unchanged
tests/lab4/assignment/chain_at_limit.trap                     10000 chained assignments still run
tests/lab4/scopes/shadowing.trap                              an inner hold hides the outer name, which is unchanged after the block
tests/lab4/scopes/assign_outer.trap                           = inside nested blocks updates the outer binding, and the change lasts
tests/lab4/scopes/nested_lookup.trap                          lookup searches outward and stops at the first match
tests/lab4/scopes/hold_reads_outer.trap                       hold x = x + 1 in a block reads the outer x, then shadows it
tests/lab4/scopes/empty_and_nested.trap                       empty blocks and blocks nested three deep
tests/lab4/scopes/blocks_at_limit.trap                        10000 nested blocks still run
tests/lab4/errors/runtime/block_binding_ends.trap             a block's binding is undefined after its }; exits 70
tests/lab4/errors/runtime/assign_in_block_does_not_create.trap = in a block never creates a binding; exits 70
tests/lab4/errors/runtime/print_runtime_error.trap            a runtime error in spittin keeps earlier output; exits 70
tests/lab4/errors/runtime/undefined_read.trap                 reading an undeclared name; exits 70
tests/lab4/errors/runtime/undefined_assignment.trap           assigning an undeclared name; exits 70
tests/lab4/errors/runtime/undefined_in_own_initializer.trap   a first hold cannot read itself; exits 70
tests/lab4/errors/runtime/undefined_keeps_earlier_output.trap an undefined read stops the program but keeps earlier output; exits 70
tests/lab4/errors/runtime/undefined_misspelled_assignment.trap a misspelled assignment is an error, not a new variable; exits 70
tests/lab4/errors/syntax/missing_semicolon.trap               spittin with no semicolon; exits 65
tests/lab4/errors/syntax/hold_missing_semicolon.trap          hold with no semicolon; exits 65
tests/lab4/errors/syntax/hold_missing_name.trap               hold with no name; exits 65
tests/lab4/errors/syntax/hold_keyword_as_name.trap            a keyword cannot be declared; exits 65
tests/lab4/errors/syntax/hold_missing_initializer.trap        = with no initializer; exits 65
tests/lab4/errors/syntax/hold_inside_expression.trap          hold cannot appear inside an expression; exits 65
tests/lab4/errors/syntax/invalid_assignment_literal.trap      a literal cannot be assigned; exits 65
tests/lab4/errors/syntax/invalid_assignment_grouped.trap      a grouped name cannot be assigned; exits 65
tests/lab4/errors/syntax/invalid_assignment_binary.trap       a binary expression cannot be assigned, and nothing runs first; exits 65
tests/lab4/errors/syntax/invalid_assignment_unary.trap        a unary expression cannot be assigned; exits 65
tests/lab4/errors/syntax/nesting_assignment_over_limit.trap   10001 chained assignments are rejected; exits 65
tests/lab4/errors/syntax/nesting_blocks_over_limit.trap       10001 nested blocks are rejected once, with no stray } errors; exits 65
tests/lab4/errors/syntax/unclosed_block.trap                  a block with no closing }; exits 65
tests/lab4/errors/syntax/stray_brace.trap                     a } with no open block; exits 65
tests/lab4/errors/syntax/recovery_in_block.trap               errors inside a block are reported and the block still closes; exits 65
tests/lab4/errors/syntax/recovery_at_keyword.trap             recovery resumes at spittin and hold after a missing semicolon; exits 65
```

Run locally with:

```bash
curl -sSL https://raw.githubusercontent.com/WhiteLicorice/cmsc-124-harness/v1.1/run_tests.py -o run_tests.py
./build.sh
python3 run_tests.py tests/lab0
python3 run_tests.py tests/lab1
python3 run_tests.py tests/lab2
python3 run_tests.py tests/lab3
python3 run_tests.py tests/lab4
python3 run_tests.py tests/lab5
```

On Windows, run these from Git Bash and use `python` instead of `python3`.

## Sample code

```
hold label = "global";
hold count = 1;
{
  hold label = "shadowed";
  spittin label;
  count = count + 1;
}
spittin label;
spittin count;
```

Output from `./run <file>`:

```
shadowed
global
2
```

The inner `hold` shadows `label` until the `}`, while `=` changes the outer `count` because the
block never held its own.

```
motion make_counter() {
  hold count = 0;
  motion next() {
    count = count + 1;
    typeshi count;
  }
  typeshi next;
}
hold tick = make_counter();
spittin tick();
spittin tick() > 1 && "counting";
spittin nil || "guest";
```

Output from `./run <file>`:

```
1
counting
guest
```

`next` keeps `count` alive after `make_counter` returns, and `&&` and `||` hand back one of
their operands rather than a boolean.

Longer programs live in `examples/`: `scopes.trap` covers nested blocks and shadowing,
`control_flow.trap` covers `ong`, `wait`, `nah`, and `spin`, and `functions.trap` covers
`motion`, `typeshi`, recursion, closures, and the logical operators. Each file lists its
expected output at the top.

## Design rationale

**Vocabulary.** We chose internet/social-media slang over a
generic keyword set (`if`, `else`, `var`) because we wanted something fun and recognizable--that if you saw the keywords you'd immediately know it's TrapScript. Though, this trades familiarity for personality, a newcomer(in the sense that they came from a different programming language) can't guess that `hold` means variable declaration the way they could
guess `var`, but this tradeoff is worth it because personality creates identity. Standard syntax is sterile and forgettable, but TrapScript turns writing code into an expressive, culturally distinct experience. Once you learn the slang logic, like "holding" a variable or putting a function into "motion," the syntax becomes natural and memorable. The slight learning curve gives TrapScript a unique soul instead of just being another generic Python clone.

## Known limitations

- `true` and `false` still scan as booleans alongside `nocap` and `cap`.
- The REPL handles each submitted line as a complete program (evaluating by default, scanning with `--tokenize`, parsing with `--parse`), so line numbers restart at 1 on every line. Parse and evaluation input requires a semicolon after every expression statement. An unfinished expression or missing semicolon reports an error and returns to the prompt; the REPL does not buffer multiple lines into one program. A string cannot continue onto the next REPL input line: a multiline string that works in a file is an unterminated-string error in the REPL.
- A stash has no literal syntax and no `[ ]` indexing: one is built with `stash()` and `stack`, and read with `grab`. There is no way to remove an item or shorten a stash.
- Redeclaring a locked name in the same scope is caught when the second declaration runs, not before the program starts, so code before it has already run.
- The test harness does not supply standard input: a test's interpreter inherits whatever the harness was started with, which is a terminal when run by hand. A test that called `spill` would wait for typing and time out, so the tests cover everything about `spill` except the read itself, which was checked by piping input in.
- There are no anonymous functions: every user function comes from a `motion` declaration.
- Parser recovery stops at semicolons and before `hold`, `locked`, `motion`, `typeshi`, `pause`, `spittin`, `{`, and `}`. A missing semicolon before a statement that begins with an expression still hides any error in that statement, because nothing marks where it starts. The body of a motion with a malformed header is skipped, so errors inside it are not reported until the header is fixed.
- A function and the scope it was declared in refer to each other, and scopes are reference-counted, so a function declared inside a block or a call keeps that scope's memory until the program exits. A loop that declares a function on every pass grows memory with every pass. A stash stacked into itself, directly or through another stash, is never freed for the same reason.
- The call depth limit is a share of the host stack rather than a fixed number of calls, so the deepest recursion a program can reach differs between builds and platforms.
- A REPL line is a complete program, so a block must open and close on one line.
- Expressions can nest at most 10,000 levels. The `tests/lab3/nesting/` and `nesting_*_over_limit` tests are written against that exact number, so changing it means regenerating them.

## Changelog


| Activity | What changed in the language |
|---|---|
| Lab 1 | Initial token vocabulary and keyword set defined. Late in Lab 1 the token output format gained the `literal` field, and a string's lexeme started keeping its quotes, so every `tests/lab1` expectation was regenerated. |
| Lab 2 | Drafted the expression grammar. `==` and `!=` now bind looser than `<`, `<=`, `>`, and `>=` (they shared one precedence level in Lab 1), and `!` and unary `-` sit at the tightest level. The whole ladder is now parsed. Expression statements end with semicolons; newlines are whitespace, so expressions and groups can span lines and multiple statements can share one line. Multiline string values print with escaped line breaks. Parser recovery proceeds at semicolons or EOF. |
| Lab 3 | Expressions evaluate. Runtime values are numbers, strings, booleans, and `nil`. Only `cap` and `nil` are falsey. There is no implicit coercion: mixed types are a runtime error, `1 == "1"` is `cap` rather than an error, and division by zero is an error rather than infinity. Runtime errors exit `70`. `--eval` prints every expression statement's value. |
| Lab 4 | Programs run. Plain `./run <file>` executes and only `spittin` writes to stdout. `hold` creates bindings and `=` updates them without ever creating one, so a misspelled name is an error. Blocks bring nested static scopes and shadowing. Parser recovery resumes before statement keywords as well as after semicolons. Expressions and blocks nest at most 10,000 levels. |
| Lab 5 | Control flow and functions. `ong`/`wait`/`nah` choose, `spin` repeats, and `pause` ends the nearest `spin`. `&&` and `||` short-circuit and produce the deciding operand. `motion` declares a function, calls are a postfix on any expression, `typeshi` returns, and functions are values with closures over static scope. `locked` declares a binding that can't be assigned or redeclared. Environments became reference-counted. Natives: `spill`, `digits`, `sayit`, `size`, `slice`, `has`, and the stash natives `stash`, `stack`, `grab`, and `put`, which bring the first value that can change after it is made. |

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

Bindings and assignment arrive in Week 2, and nested scopes in Week 3.

### Lab 4 Week 2 progress

Programs can now remember things. `hold` creates a binding, an identifier reads one, and
`=` updates one. Assignment is a right-associative expression that produces its value, and
its target is checked in the parser. All of it lives in one global `Environment`, created
once per file run or REPL session, so REPL variables survive across lines and errors.

`tests/lab4/` gains binding creation and lookup, `hold` with no initializer, assignment
and its value inside expressions, redeclaration, value copying, and exact name matching,
then one error per file: undefined reads and assignments at exit 70, and malformed `hold`
declarations and invalid assignment targets at exit 65.

Deep nesting now reports a syntax error instead of overflowing the host stack: the interpreter
runs on a 256 MB stack, and the parser rejects any expression deeper than 10,000 levels.

Nested scopes, shadowing, and recovery beyond semicolons are Week 3.

### Lab 4 Week 3 progress

Blocks bring nested scopes. `{ ... }` is a statement whose declarations run in a new
environment linked to the one around it, so scope is static: a read or `=` searches the
innermost environment, then each enclosing one, and stops at the first match. `hold` always
binds in the innermost environment, which makes shadowing possible, and `=` still never
creates a binding. A block's bindings are released when it exits, and the enclosing
environment comes back even after a runtime error, so a REPL session never stays inside a
failed block. Inside a block, `hold x = x;` reads the outer `x` before shadowing it.

Parser recovery is synchronized: after a syntax error it resumes after a semicolon or before
`hold`, `spittin`, `{`, or `}`, so an error inside a block no longer leaves a stray `}`
behind it, and a missing semicolon before a `hold` or `spittin` no longer hides that
statement's own errors. Blocks count toward the 10,000-level nesting limit.

`tests/lab4/scopes/` covers shadowing, assignment through enclosing scopes, outward lookup,
`hold` reading the outer name, empty and nested blocks, and the nesting limit. Runtime errors
cover a binding read after its block ends and `=` that would create a binding; syntax errors
cover an unclosed block, a stray `}`, recovery inside a block and at keywords, and blocks
past the limit. `tests/lab2/statements/` covers the `(block ...)` parse output.

`build.sh` and `run` are now stored as executable, and `examples/` holds two runnable
programs.

### Lab 5 Week 1 progress

Programs can now choose and repeat. `ong` takes a parenthesized condition and a statement,
with any number of `wait` clauses and an optional `nah`. `spin` takes the same shape and
re-tests its condition before every pass. Both read truth through the Lab 3 rule, so only
`cap` and `nil` are falsey.

`wait` needs no node of its own: the parser folds a chain into nested `Ong` nodes, each
`wait` becoming the else branch of the one before it. The tree therefore carries only a
condition, a then branch, and an optional else, and the evaluator needs one arm rather than
a chain. A `nah` binds to the nearest unmatched `ong`, which recursive descent gives for free.

Only the chosen branch is executed, so a runtime error in the untaken branch never fires.
`spin` runs its body zero times when the condition starts false.

`tests/lab5/conditionals/` covers each branch, a `wait` chain reaching its middle and its
`nah`, nested conditionals, a missing `nah`, the truthiness rule in a condition, and the
proof that the untaken branch is never walked. `tests/lab5/repetition/` covers a loop that
runs three times and one that runs zero times. CI runs Labs 0 through 5.

Logical operators, functions, calls, and results arrive in Week 2.

### Lab 5 Week 2 progress

Programs can now combine conditions and name their own operations. `&&` and `||` are new
tokens, new precedence levels between assignment and equality, and a new `Logical` node.
They short-circuit and produce the operand that settled the answer. `motion` declares a
function, a call runs it, and `typeshi` ends the call and hands back a value, or `nil` when
there is none.

Functions are values and scope is static, which meant changing how environments are stored.
A block used to push a scope and pop it on the way out; a function has to keep the scope it
was declared in after that block has ended, so an environment is now a reference-counted
handle and a block or a call simply makes a child of the one it runs in. That one change is
what gives recursion, functions passed and returned as values, and closures.

`typeshi` unwinds without host exceptions: every statement reports whether it finished or
returned, and blocks, `ong`, and `spin` pass a return straight up to the call. A `typeshi`
outside a motion is a syntax error. Unbounded recursion is a runtime error, `Stack overflow.`,
instead of a crashed interpreter, and call expressions count toward the 10,000-level nesting
limit.

Repetition needed no new grammar: `spin` is unchanged from Week 1. Its new tests cover nested
loops, compound `&&` and `||` conditions, an unbraced body, and a `typeshi` leaving a loop.

`tests/lab5/logical/` covers which operand each operator produces, short-circuiting of side
effects and of runtime errors, precedence against each other and against the neighbouring
levels, and use as conditions. `tests/lab5/functions/` covers declaration and redeclaration,
parameters and argument order, results and the `nil` default, `typeshi` unwinding out of
nested statements, recursion and mutual recursion, functions as values, static scope, and
closures. `tests/lab5/nesting/` holds call nesting at the limit. `tests/lab5/errors/runtime/`
has one error per file: arity in both directions, calling a non-function, undefined and
not-yet-declared names, an error inside a function, locals ending with the call, no dynamic
scope, and the stack limit. `tests/lab5/errors/syntax/` covers malformed declarations and
calls, `typeshi` outside a motion, a lone `&` and `|`, recovery, and call nesting past the
limit. `tests/lab1/operators/` and `tests/lab2/` gain the new tokens and the new parse output.

Two native functions are in: `spill()` reads a line of standard input, and `digits(text)`
turns a string into the number it spells, or `nil`. Natives are a second kind of callable
value, bound in the global environment before the program starts and dispatched from the same
call expression as motions, with the same arity check. `tests/lab5/natives/` covers `spill`
as a value and every spelling `digits` accepts and rejects. `tests/lab5/errors/runtime/` adds
both arity errors and `digits` given a number.

`pause` and `locked` are still to come.

### Lab 5 Week 3 progress

The Week 3 checklist was already met by Week 2's work: results reaching the caller
(`functions/results.trap`, `typeshi_unwinds.trap`), functions passed and returned
(`functions/first_class.trap`), independent captures from one factory and loop capture
(`functions/closures.trap`), both natives, arity errors for motions and natives, and
`examples/functions.trap`, with all six folders green in CI.

This week finishes the vocabulary that was reserved but unusable, and adds what a real
program needs. Every addition keeps the language's rule that nothing is converted, filled
in, or cut down behind the program's back.

- `pause` ends the nearest `spin`. It is a third kind of statement result alongside finishing
  and returning, so it unwinds the same way `typeshi` does but stops at the loop instead of
  the call. The parser rejects one outside a `spin`, and a motion body starts outside any loop.
- `locked` declares a binding that can't be assigned or redeclared in its scope. The lock is
  kept beside the scope's map, on the binding rather than the name, so shadowing still works.
- `sayit` turns any value into its text. `+` still refuses to mix a number and a string, so a
  number joins a sentence only when the program asks for it.
- `size`, `slice`, and `has` read text. Positions must be whole and must fit; nothing is
  rounded or clipped.
- The stash is the first collection, and the first value that can change after it is made,
  so it is shared by reference like a function. `stash`, `stack`, `grab`, and `put` build,
  grow, read, and replace; `size` counts. A missing position is an error rather than `nil`.

`tests/lab5/repetition/` and `tests/lab5/locked/` cover `pause` and `locked`,
`tests/lab5/natives/` covers `sayit` and the text tools, and `tests/lab5/stash/` covers
building, walking, sharing, capturing, and nesting stashes. `tests/lab5/errors/` gains one
file per new rejection, and `tests/lab2/statements/` gains the `(pause)` and `(locked ...)`
parse output.
