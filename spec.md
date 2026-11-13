### math library

input|each token wrapped into msystem!() macros |: `math_tokens|expressions,equations,inequality|`
output|depend on type|: `math system`

Lexer -> Tokens -> Parser -> AST -> MathSystem

### math tokens format determination (mtf(NOT FUCKING MICROSOFT TAPE FORMAT)):

| tokens | value |
| ---- | ----- |
| **   | degree|
| _()    | index |
| *    | mul   |
| /    | div   |
| +    | plus  |
| -    | minus |
| n_rt | any_root |
|  .   | plot requirement |
|  ..  | any_equation .. another_any_equation - consistent system of eq|
| n_vec | any_vector  |
| n_m_tens |  any_tensor |
| =        |  equals     |
| !=       | not_equals  |
| par parametr_name: name = eq(parametr_name) | parametrization |
| basis name coordinates | basis defining. Coordinates its tuple (number,number,number,number,...) |
| number   | for any number       |
| variable | varibles            |
| e        | 2.7....|
| pi       | 3.14....|
| fact()  | factorial| 
| ! | not |
| || | or |
| & | and |
| condition <expr/eq> exprected_value(true/false) | condition |



```mtf

```
