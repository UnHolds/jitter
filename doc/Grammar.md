Program = Function Program | None
Function = Identifier Parameter Block
Paramter = Identifier Parameter | None
Block = Statements Block | None
Statements = Assignment | IfStatement | WhileLoop | FunctionCall

Assignment = Identifier Expression
IfStatement = Expression Block
WhileLoop = Expression Block
FunctionCall = Identifier Arguments
Arguments = Expression Arguments | None

Expression = Identifier | Addition | Subtraction | Multiplication | Division | Modulo | Number | FunctionCall

Addition = Expression Expression
Subtraction = Expression Expression
Multiplication = Expression Expression
Division = Expression Expression
Modulo = Expression Expression

TODO

- Semantic check if num or args is okay
  Predecende

1. parenthesis
2. muliplication division modulo
3. addition subtraction
