Original language grammar
```
PROGRAM := EXP

EXP := FORM | ATOM

FORM := APPLICATION
      | LAMBDA
      | COND
      | BLOCK
      | LET
      | DEFINITION
	  | ASSIGNMENT

ATOM := IDENTIFIER
      | STRING
      | INTEGER

// Forms

APPLICATION := EXP '(' ARGLIST? ')'
LAMBDA := ('lambda' | 'λ') '(' PARAMETERS ')' BLOCK
COND := 'cond' CLAUSE+
BLOCK := '{' EXPLIST? '}'
LET := 'let' IDENTIFIER '=' EXP BLOCK?
DEFINITION := 'def' IDENTIFIER '=' EXP
ASSIGNMENT := IDENTIFIER '=' EXP

PARAMETERS := IDENTIFIER (',' IDENTIFIER)*
ARGLIST := EXP (';' EXP)*
CLAUSE := '(' EXP '=>' EXP ')'

// Atoms
     
IDENTIFIER := IDSTART IDCHAR*             // See restriction below
IDSTART := ICHAR except for DIGIT and PLUS and MINUS
IDCHAR := UTF8 except for DELIMITERS
DELIMITERS := WS | '"' | '(' | ')' | '{' | '}' | ',' | ';'
     
STRING := '"' (UTF8NOBS | ESCAPESEQ)* '"'
UTF8NOBS := UTF8 except for backslash ('\' codepoint 92)
ESCAPESEQ := '\' ('\' | '"' | 't' | 'n' | 'r')

INT := ('+' | '-')? DIGIT+
DIGIT := '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'

UTF8 := Any Unicode character (codepoint) encoded in UTF-8
WS := space (ASCII 32) | tab (9) | return (13) | newline (10)

// Comments and whitespace

Comments begin with '//' and extend to the end of the line (\n).
The parser is insensitive to whitespace and comments.
   
// Restriction on identifiers

An identifier cannot be a keyword.  Keywords can be found in the 
grammar above, and include: lambda, λ, cond, def, let, =, =>
```