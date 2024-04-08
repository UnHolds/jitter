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
