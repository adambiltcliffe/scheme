pub(crate) enum Token {
    LBracket,
    RBracket,
    Dot,
    Tick,
    Value(String),
}

pub(crate) fn tokenize(input: &str) -> Vec<Token> {
    let mut result = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(ch) = iter.next() {
        match ch {
            c if c.is_whitespace() => continue,
            '(' => result.push(Token::LBracket),
            ')' => result.push(Token::RBracket),
            '.' => result.push(Token::Dot),
            '\'' => result.push(Token::Tick),
            _ => {
                let mut s = String::new();
                s.push(ch);
                while iter.peek().is_some()
                    && !iter.peek().unwrap().is_whitespace()
                    && *iter.peek().unwrap() != '('
                    && *iter.peek().unwrap() != ')'
                {
                    s.push(iter.next().unwrap())
                }
                result.push(Token::Value(s))
            }
        }
    }
    result
}
