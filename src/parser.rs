use crate::Expr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenType {
    Number(String),
    Variable(String),
    Euler,
    Pi,
    Basis,
    Plus,
    Minus,
    Multiply,
    Divide,
    Degree,
    Equals,
    NotEquals,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    PlotRequirement,
    SystemOfEq,
    LogicalNot,
    LogicalOr,
    LogicalAnd,
    AnyRoot(usize),
    Vector(usize),
    Tensor(usize, usize),
    Parametrization,
    Factorial,
    Condition,
    LParen,
    RParen,
    Underscore,
    Comma,
    Colon,
}

pub(crate) struct Lexer<'s> {
    text: &'s str,
}
impl<'s> Lexer<'s> {
    pub(crate) fn new(text: &'s str) -> Self {
        Self { text }
    }
    pub(crate) fn lex(&mut self) -> Result<Vec<TokenType>, String> {
        let mut out = Vec::new();
        let chars: Vec<char> = self.text.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i].is_whitespace() {
                i += 1;
                continue;
            }
            let start = i;
            if chars[i].is_ascii_digit()
                || (chars[i] == '.' && chars.get(i + 1).is_some_and(|c| c.is_ascii_digit()))
            {
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                if chars.get(i) == Some(&'.') {
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                if chars.get(i).is_some_and(|c| *c == 'e' || *c == 'E') {
                    i += 1;
                    if chars.get(i).is_some_and(|c| *c == '+' || *c == '-') {
                        i += 1;
                    }
                    let exponent_start = i;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    if i == exponent_start {
                        return Err(format!("expected exponent digits at character {start}"));
                    }
                }
                if chars.get(i) == Some(&'_') {
                    i += 1;
                    while i < chars.len() && chars[i].is_alphanumeric() {
                        i += 1;
                    }
                    while chars.get(i) == Some(&'_')
                        && chars.get(i + 1).is_some_and(|c| c.is_alphanumeric())
                    {
                        i += 1;
                        while i < chars.len() && chars[i].is_alphanumeric() {
                            i += 1;
                        }
                    }
                }
            } else if chars[i].is_alphabetic() || chars[i] == '_' {
                i += 1;
                while i < chars.len() {
                    if chars[i].is_alphanumeric() {
                        i += 1;
                    } else if chars[i] == '_'
                        && chars.get(i + 1).is_some_and(|c| c.is_alphanumeric())
                    {
                        i += 2;
                        while i < chars.len() && chars[i].is_alphanumeric() {
                            i += 1;
                        }
                    } else {
                        break;
                    }
                }
                if chars[start..i].iter().collect::<String>() == "fact"
                    && chars.get(i) == Some(&'(')
                    && chars.get(i + 1) == Some(&')')
                {
                    i += 2;
                }
            } else if (chars[i] == '<' || chars[i] == '>') && chars.get(i + 1) == Some(&'=') {
                i += 2;
            } else if chars[i] == '*' && chars.get(i + 1) == Some(&'*') {
                i += 2;
            } else if chars[i] == '!' && chars.get(i + 1) == Some(&'=') {
                i += 2;
            } else if chars[i] == '.' && chars.get(i + 1) == Some(&'.') {
                i += 2;
            } else if chars[i] == '|' && chars.get(i + 1) == Some(&'|') {
                i += 2;
            } else {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            out.push(
                self.get_token(&word)
                    .map_err(|_| format!("unknown token {word:?} at character {start}"))?,
            );
        }
        Ok(out)
    }
    fn get_token(&self, word: &str) -> Result<TokenType, ()> {
        Ok(match word {
            "+" => TokenType::Plus,
            "-" => TokenType::Minus,
            "*" => TokenType::Multiply,
            "/" => TokenType::Divide,
            "**" => TokenType::Degree,
            "=" => TokenType::Equals,
            "!=" => TokenType::NotEquals,
            "<" => TokenType::Less,
            "<=" => TokenType::LessEqual,
            ">" => TokenType::Greater,
            ">=" => TokenType::GreaterEqual,
            "." => TokenType::PlotRequirement,
            ".." => TokenType::SystemOfEq,
            "!" => TokenType::LogicalNot,
            "||" => TokenType::LogicalOr,
            "&" => TokenType::LogicalAnd,
            "(" => TokenType::LParen,
            ")" => TokenType::RParen,
            "_" => TokenType::Underscore,
            "," => TokenType::Comma,
            ":" => TokenType::Colon,
            "e" => TokenType::Euler,
            "pi" => TokenType::Pi,
            "basis" => TokenType::Basis,
            "par" => TokenType::Parametrization,
            "fact" | "fact()" => TokenType::Factorial,
            "condition" => TokenType::Condition,
            s if s.ends_with("_rt") => {
                TokenType::AnyRoot(s.strip_suffix("_rt").ok_or(())?.parse().map_err(|_| ())?)
            }
            s if s.ends_with("_vec") => {
                TokenType::Vector(s.strip_suffix("_vec").ok_or(())?.parse().map_err(|_| ())?)
            }
            s if s.ends_with("_tens") => {
                let dims = s.strip_suffix("_tens").ok_or(())?;
                let (r, c) = dims.split_once('_').ok_or(())?;
                if c.contains('_') {
                    return Err(());
                }
                TokenType::Tensor(r.parse().map_err(|_| ())?, c.parse().map_err(|_| ())?)
            }
            s if s.parse::<f64>().is_ok() => TokenType::Number(s.to_owned()),
            s if !s.is_empty() => TokenType::Variable(s.to_owned()),
            _ => return Err(()),
        })
    }
}

pub(crate) struct Parser {
    tokens: Vec<TokenType>,
    pos: usize,
}
impl Parser {
    pub(crate) fn new(tokens: Vec<TokenType>) -> Self {
        Self { tokens, pos: 0 }
    }
    fn peek(&self) -> Option<&TokenType> {
        self.tokens.get(self.pos)
    }
    fn take(&mut self) -> Option<TokenType> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
    fn expect(&mut self, expected: TokenType) -> Result<(), String> {
        if self.peek() == Some(&expected) {
            self.take();
            Ok(())
        } else {
            Err(format!("expected {expected:?}, got {:?}", self.peek()))
        }
    }
    pub(crate) fn parse(&mut self) -> Result<Expr, String> {
        let e = self.expression(0)?;
        if self.peek().is_some() {
            return Err(format!("unexpected token {:?}", self.peek()));
        }
        Ok(e)
    }
    fn expression(&mut self, min: u8) -> Result<Expr, String> {
        let mut left = self.prefix()?;
        loop {
            if min == 0 && self.peek() == Some(&TokenType::PlotRequirement) {
                self.take();
                left = Expr::PlotRequirement(vec![left].into_boxed_slice());
                continue;
            }
            if self.peek() == Some(&TokenType::Underscore) {
                self.take();
                let parenthesized = self.peek() == Some(&TokenType::LParen);
                if parenthesized {
                    self.take();
                }
                let index = match self.take() {
                    Some(TokenType::Number(n)) => n
                        .parse::<usize>()
                        .map_err(|_| "index must be a non-negative integer")?,
                    _ => return Err("expected integer index after '_'".into()),
                };
                if parenthesized {
                    self.expect(TokenType::RParen)?;
                }
                left = Expr::Index(Box::new(left), index);
                continue;
            }
            let (prec, op) = match self.peek() {
                Some(TokenType::Plus) => (3, 0),
                Some(TokenType::Minus) => (3, 1),
                Some(TokenType::Multiply) => (4, 2),
                Some(TokenType::Divide) => (4, 3),
                Some(TokenType::Degree) => (5, 4),
                Some(TokenType::Equals) => (2, 5),
                Some(TokenType::NotEquals) => (2, 6),
                Some(TokenType::Less) => (2, 11),
                Some(TokenType::LessEqual) => (2, 12),
                Some(TokenType::Greater) => (2, 13),
                Some(TokenType::GreaterEqual) => (2, 14),
                Some(TokenType::LogicalAnd) => (1, 7),
                Some(TokenType::LogicalOr) => (0, 8),
                Some(TokenType::SystemOfEq) => (0, 9),
                _ => break,
            };
            if prec < min {
                break;
            }
            self.take();
            let right = self.expression(match op {
                4 => prec,
                9 => 0,
                _ => prec + 1,
            })?;
            left = match op {
                0 => Expr::Plus(Box::new(left), Box::new(right)),
                1 => Expr::Minus(Box::new(left), Box::new(right)),
                2 => Expr::Multiply(Box::new(left), Box::new(right)),
                3 => Expr::Divide(Box::new(left), Box::new(right)),
                4 => {
                    let exponent = match &right {
                        Expr::Number(n) => n
                            .parse::<u8>()
                            .map_err(|_| "power must be an integer literal from 0 to 255")?,
                        _ => return Err("power must be an integer literal from 0 to 255".into()),
                    };
                    Expr::Degree(Box::new(left), exponent)
                }
                5 => Expr::Equals(
                    vec![left].into_boxed_slice(),
                    vec![right].into_boxed_slice(),
                ),
                6 => Expr::NotEquals(
                    vec![left].into_boxed_slice(),
                    vec![right].into_boxed_slice(),
                ),
                11 => Expr::Less(Box::new(left), Box::new(right)),
                12 => Expr::LessEqual(Box::new(left), Box::new(right)),
                13 => Expr::Greater(Box::new(left), Box::new(right)),
                14 => Expr::GreaterEqual(Box::new(left), Box::new(right)),
                7 => Expr::LogicalAnd(Box::new(left), Box::new(right)),
                8 => Expr::LogicalOr(Box::new(left), Box::new(right)),
                9 => Expr::SystemOfEq(
                    vec![left].into_boxed_slice(),
                    vec![right].into_boxed_slice(),
                ),
                _ => Expr::PlotRequirement(vec![left, right].into_boxed_slice()),
            };
        }
        Ok(left)
    }
    fn prefix(&mut self) -> Result<Expr, String> {
        let tok = self
            .take()
            .ok_or("expected expression, reached end of input")?;
        Ok(match tok {
            TokenType::Number(n) => Expr::Number(n),
            TokenType::Variable(n) => Expr::Variable(n),
            TokenType::Euler => Expr::Euler,
            TokenType::Pi => Expr::Pi,
            TokenType::Minus => Expr::Minus(
                Box::new(Expr::Number("0".into())),
                Box::new(self.expression(5)?),
            ),
            TokenType::LogicalNot => Expr::LogicalNot(Box::new(self.expression(3)?)),
            TokenType::LParen => {
                let e = self.expression(0)?;
                self.expect(TokenType::RParen)?;
                e
            }
            TokenType::AnyRoot(n) => Expr::AnyRoot(
                vec![Expr::Number(n.to_string()), self.expression(3)?].into_boxed_slice(),
            ),
            TokenType::Vector(n) => {
                let values = if self.peek() == Some(&TokenType::LParen) {
                    self.expression_list()?
                } else {
                    vec![self.expression(3)?]
                };
                if values.len() != n {
                    return Err(format!(
                        "vector dimension {n} needs {n} values, got {}",
                        values.len()
                    ));
                }
                Expr::Vector(n, values.into_boxed_slice())
            }
            TokenType::Factorial => Expr::Factorial(vec![self.expression(3)?].into_boxed_slice()),
            TokenType::Basis => {
                let name = match self.take() {
                    Some(TokenType::Variable(name)) => name,
                    _ => return Err("expected basis name after 'basis'".into()),
                };
                let coordinates = self.expression_list()?;
                Expr::Basis {
                    name,
                    coordinates: coordinates.into_boxed_slice(),
                }
            }
            TokenType::Condition => {
                let expression = self.expression(1)?;
                let expected = match self.take() {
                    Some(TokenType::Variable(s)) if s == "true" => true,
                    Some(TokenType::Variable(s)) if s == "false" => false,
                    _ => return Err("condition must end in true or false".into()),
                };
                Expr::Condition {
                    expression: Box::new(expression),
                    expected_value: expected,
                }
            }
            TokenType::Tensor(r, c) => {
                let values = self.expression_list()?;
                if values.len() != r.saturating_mul(c) {
                    return Err(format!(
                        "tensor {r}x{c} needs {} values, got {}",
                        r.saturating_mul(c),
                        values.len()
                    ));
                }
                Expr::Tensor(r, c, values.into_boxed_slice())
            }
            TokenType::Parametrization => {
                let parameter_name = match self.take() {
                    Some(TokenType::Variable(v)) => v,
                    _ => return Err("expected parameter name after par".into()),
                };
                self.expect(TokenType::Colon)?;
                let variable_name = match self.take() {
                    Some(TokenType::Variable(v)) => v,
                    _ => return Err("expected variable name after ':'".into()),
                };
                self.expect(TokenType::Equals)?;
                let equation = self.expression(1)?;
                Expr::Parametrization {
                    parameter_name,
                    variable_name,
                    equations: vec![equation].into_boxed_slice(),
                }
            }
            other => return Err(format!("expected a value, got {other:?}")),
        })
    }

    fn expression_list(&mut self) -> Result<Vec<Expr>, String> {
        self.expect(TokenType::LParen)?;
        let mut values = Vec::new();
        if self.peek() == Some(&TokenType::RParen) {
            self.take();
            return Ok(values);
        }
        loop {
            values.push(self.expression(0)?);
            match self.take() {
                Some(TokenType::Comma) => continue,
                Some(TokenType::RParen) => break,
                other => return Err(format!("expected ',' or ')', got {other:?}")),
            }
        }
        Ok(values)
    }
}
