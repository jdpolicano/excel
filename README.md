
## Program Structure


## Structs


## Grammar
Below is the rough sketch of the grammar definitions that are (hopefully) implemented in the parser. The grammar is written in a modified BNF notation. The grammar is not complete, but it should give you a rough idea of what the parser can handle.

| Component      | Definition                                                      |
|----------------|-----------------------------------------------------------------|
| `Formula`      | `'=' Expression`                                                |
| `Expression`   | `Term \| Expression Operator Term`                              |
| `Term`         | `Factor \| Term '+' Factor \| Term '-' Factor`                  |
| `Factor`       | `Primary \| Factor '*' Primary \| Factor '/' Primary`           |
| `Primary`      | `Primitive \| CellRef \| '(' Expression ')' \| Function`        |
| `Primitive`    | `Number \| String \| Boolean`                                   |
| `Function`     | `FunctionName '(' [ Expression { ',' Expression } ] ')'`        |
| `FunctionName` | `'SUM' \| 'AVERAGE' \| 'MAX' \| 'MIN' \| ... \| 'IF' \| ...`    |
| `Operator`     | `'+' \| '-' \| '*' \| '/' \| '^' \| '&' \| '=' \| '<>' \| ...`  |
| `Number`       | `[0-9]+ ('.' [0-9]+)?`                                          |
| `String`       | `'"' [^"]* '"'`                                                 |
| `Boolean`      | `'TRUE' \| 'FALSE'`                                             |
| `CellRef`      | `ColumnRef RowRef`                                              |
| `ColumnRef`    | `[A-Z]+`                                                        |
| `RowRef`       | `[0-9]+`                                                        |
