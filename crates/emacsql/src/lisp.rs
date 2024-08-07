use std::str::FromStr;

#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    String(String),
    Cons(Box<Value>, Box<Value>),
    Integer(i64),
    Real(f64),
    Symbol(String),
    // emacslisp contains both: List and Vector, I parse it as the same things
    List(Vec<Value>),
    Nil,
}

impl Value {
    fn text(self) -> Option<String> {
        match self {
            Self::Symbol(s) | Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_symbol(self) -> Option<String> {
        match self {
            Self::Symbol(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_real(self) -> Option<f64> {
        match self {
            Self::Real(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_list(self) -> Option<Vec<Value>> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn is_nil(self) -> bool {
        match self {
            Self::Nil => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    UnbalancedExpr,
    EndOfInput,
    UnexpectedDot,
    InvalidNumber,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub fn parse_string(s: &str) -> Result<Value> {
    let mut parser = Parser::new(s);
    parser.parse()
}

impl FromStr for Value {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_string(s)
    }
}

#[derive(Debug)]
pub enum Event {
    Parsed(Value),
    Skipped,
    ErrorHappened(Error),
    End,
}

impl From<Result<Value>> for Event {
    fn from(value: Result<Value>) -> Self {
        match value {
            Ok(val) => Self::Parsed(val),
            Err(e) => Self::ErrorHappened(e),
        }
    }
}

impl Into<Result<Value>> for Event {
    fn into(self) -> Result<Value> {
        match self {
            Self::Parsed(val) => Ok(val),
            Self::ErrorHappened(err) => Err(err),
            Self::End => Err(Error::EndOfInput),
            Self::Skipped => todo!(),
        }
    }
}

impl Event {
    #[allow(dead_code)]
    fn parsed(self) -> Option<Value> {
        use Event::*;

        match self {
            Parsed(val) => Some(val),
            ErrorHappened(_) | Skipped | End => None,
        }
    }
}

enum ListType {
    Vec,
    List,
}

impl ListType {
    /// Try guess the type of list by the its opening paren.
    fn from_open(op: char) -> Option<Self> {
        match op {
            '(' => Some(Self::List),
            '[' => Some(Self::Vec),
            _ => None,
        }
    }

    /// Returns the opening parenthesis of this [`ListType`].
    #[allow(dead_code)]
    fn open(&self) -> char {
        match self {
            Self::List => '(',
            Self::Vec => '[',
        }
    }

    /// Returns the closing parenthesis of this [`ListType`].
    fn close(&self) -> char {
        match self {
            Self::List => ')',
            Self::Vec => ']',
        }
    }
}

struct Parser<'a> {
    cursor: usize,
    len: usize,
    src: &'a [u8],
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Parser {
            src: src.as_bytes(),
            cursor: 0,
            len: src.len(),
        }
    }

    fn parse(&mut self) -> Result<Value> {
        loop {
            let ev = self.next_event();

            match ev {
                Event::Parsed(val) => return Ok(val),
                Event::Skipped => continue,
                Event::End => return Err(Error::UnbalancedExpr),
                Event::ErrorHappened(err) => return Err(err),
            }
        }
    }

    fn next_event(&mut self) -> Event {
        if self.ch().is_none() {
            return Event::End;
        }

        let ch = self.ch().unwrap();

        use Event::*;
        match ch {
            ch if ch.is_whitespace() => {
                self.chop_spaces();
                Event::Skipped
            }
            ch if ch.is_numeric() || ch == '-' => return self.parse_numeric_or_symbol().into(),
            '"' => return self.parse_string().into(),
            '[' | '(' => return self.parse_list_or_cons().into(),
            '.' => return ErrorHappened(Error::UnexpectedDot),
            ')' | ']' => return ErrorHappened(Error::UnbalancedExpr),
            _ => return self.parse_symbol().into(),
        }
    }

    fn ch(&self) -> Option<char> {
        self.is_not_empty().then(|| self.src[self.cursor] as char)
    }

    fn chop_word(&mut self) -> String {
        self.take_while(|c| c != ' ' && c != ')' && c != ']' && c != '\n')
    }

    const fn is_not_empty(&self) -> bool {
        self.cursor < self.len
    }

    fn chop_spaces(&mut self) {
        self.chop_while(char::is_whitespace);
    }

    fn parse_string(&mut self) -> Result<Value> {
        assert_eq!(self.ch(), Some('"'));
        self.chop(1);
        let mut str = String::new();
        loop {
            match self.ch().ok_or(Error::UnbalancedExpr)? {
                '"' => {
                    self.chop(1);
                    break;
                }
                '\\' => {
                    self.chop(1);
                    str.push(escape_char(self.chop_ch().ok_or(Error::UnbalancedExpr)?));
                }
                ch => {
                    str.push(ch);
                    self.chop(1);
                }
            }
        }
        Ok(Value::String(str))
    }

    fn parse_list_or_cons(&mut self) -> Result<Value> {
        let op = self.ch().ok_or(Error::UnbalancedExpr)?;
        self.chop(1);
        let kind = ListType::from_open(op).unwrap();
        let cl = kind.close();

        let mut lst: Vec<Value> = Vec::new();

        loop {
            let ch = self.ch();
            if ch == Some(cl) {
                self.chop(1);
                break;
            } else if ch == Some('.') {
                self.chop(1);
                let car = if lst.len() == 1 {
                    lst[0].clone()
                } else {
                    Value::List(lst)
                };
                let cdr = self.next_parsed().or(Err(Error::UnexpectedDot))?;
                return Ok(Value::Cons(Box::new(car), Box::new(cdr)));
            }
            let ev = self.next_event();
            match ev {
                Event::Parsed(val) => lst.push(val),
                Event::Skipped => continue,
                Event::End => return Err(Error::UnbalancedExpr),
                Event::ErrorHappened(err) => return Err(err),
            }
        }

        return Ok(Value::List(lst));
    }

    fn parse_symbol(&mut self) -> Result<Value> {
        assert!(self.ch().is_some() && is_identifier_char(self.ch().unwrap()));
        let name = self.take_while(is_identifier_char);
        Ok(Value::Symbol(name))
    }

    fn next_parsed(&mut self) -> Result<Value> {
        loop {
            let ev = self.next_event();
            match ev {
                Event::Skipped => continue,
                Event::Parsed(val) => return Ok(val),
                Event::End => return Err(Error::EndOfInput),
                Event::ErrorHappened(err) => return Err(err),
            }
        }
    }

    fn parse_numeric_or_symbol(&mut self) -> Result<Value> {
        let beg = self.cursor;
        self.parse_numeric().or_else(|_| {
            self.cursor = beg;
            let symbol = self.parse_symbol()?;
            let name = symbol.text().unwrap();
            if name.contains('.') {
                Err(Error::InvalidNumber)
            } else {
                Ok(Value::Symbol(name))
            }
        })
    }

    fn parse_numeric(&mut self) -> Result<Value> {
        let w = self.chop_word();
        if w.contains('.') {
            return w.parse().map(Value::Real).map_err(|_| Error::InvalidNumber);
        }
        w.parse()
            .map(Value::Integer)
            .map_err(|_| Error::InvalidNumber)
    }

    fn take_while(&mut self, f: fn(char) -> bool) -> String {
        let beg = self.cursor;
        self.chop_while(f);
        let end = self.cursor;
        self.substr(beg, end)
    }

    fn chop_while(&mut self, f: fn(char) -> bool) {
        while self.ch().map(f).unwrap_or(false) {
            self.chop(1);
        }
    }

    fn chop(&mut self, n: usize) {
        self.cursor += n;
    }

    fn chop_ch(&mut self) -> Option<char> {
        let res = self.ch();
        self.chop(1);
        res
    }

    fn substr(&mut self, beg: usize, end: usize) -> String {
        String::from_utf8(self.src[beg..end].to_vec()).unwrap()
    }

    #[allow(dead_code)]
    fn progress(&self) {
        println!("{}", String::from_utf8(self.src.to_vec()).unwrap());
        for _ in 0..self.cursor {
            print!(" ");
        }
        println!("|");
    }
}

fn is_identifier_char(ch: char) -> bool {
    !(ch.is_whitespace()
        || ch == '['
        || ch == ']'
        || ch == '('
        || ch == ')'
        || ch == '.'
        || ch == '"'
        || ch == '\'')
}

fn escape_char(ch: char) -> char {
    match ch {
        '"' => '"',
        'n' => '\n',
        't' => '\t',
        ch if ch.is_numeric() => ('\0' as u8 + ch.to_digit(10).unwrap() as u8) as char,
        _ => ch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let src = "   \"ur son!\"";
        let res = src.parse().unwrap();
        if let Value::String(s) = res {
            assert_eq!(s, "ur son!".to_string())
        } else {
            assert!(false);
        };
    }

    #[test]
    fn test_parse_string_list() {
        let src = "(\"a\" \"b\")";
        let res = src.parse().unwrap();
        if let Value::List(s) = res {
            let mut s = s.iter();
            assert_eq!(s.next().unwrap().clone().text().unwrap(), "a".to_string());
            assert_eq!(s.next().unwrap().clone().text().unwrap(), "b".to_string())
        } else {
            assert!(false);
        };
    }

    #[test]
    fn test_parse_num() {
        let src = "(1 2 3 4.5 -6.7)";
        let res = src.parse().unwrap();
        if let Value::List(lst) = res {
            let mut nums = lst.iter();
            assert_eq!(nums.next(), Some(&Value::Integer(1)));
            assert_eq!(nums.next(), Some(&Value::Integer(2)));
            assert_eq!(nums.next(), Some(&Value::Integer(3)));
            assert_eq!(nums.next(), Some(&Value::Real(4.5)));
            assert_eq!(nums.next(), Some(&Value::Real(-6.7)));
            assert_eq!(nums.next(), None);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parse_inner_lists() {
        let src = "(1 2 3 4 (5 6 [ 7 8 ]))";
        let res = src.parse().unwrap();
        if let Value::List(lst) = res {
            let mut nums = lst.iter();
            assert_eq!(nums.next(), Some(&Value::Integer(1)));
            assert_eq!(nums.next(), Some(&Value::Integer(2)));
            assert_eq!(nums.next(), Some(&Value::Integer(3)));
            assert_eq!(nums.next(), Some(&Value::Integer(4)));
            assert_eq!(
                nums.next(),
                Some(&Value::List(vec![
                    Value::Integer(5),
                    Value::Integer(6),
                    Value::List(vec![Value::Integer(7), Value::Integer(8)])
                ]))
            );
            assert_eq!(nums.next(), None);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parse_cons() {
        let src = "(1 2 3 4 . 7)";
        let res: Value = src.parse().unwrap();
        assert_eq!(
            res,
            Value::Cons(
                Box::new(Value::List(vec![
                    Value::Integer(1),
                    Value::Integer(2),
                    Value::Integer(3),
                    Value::Integer(4)
                ])),
                Box::new(Value::Integer(7))
            )
        );
    }

    #[test]
    fn test_parse_cons_from_2values() {
        let src = "(1 . 2)";
        let res: Value = src.parse().unwrap();
        assert_eq!(
            res,
            Value::Cons(Box::new(Value::Integer(1)), Box::new(Value::Integer(2)))
        );
    }

    #[test]
    fn test_parse_symbol() {
        let src = " myman1 ";
        let res: Value = src.parse().unwrap();
        assert_eq!(res, Value::Symbol("myman1".to_string()));
    }

    #[test]
    fn test_parse_symbol_that_starts_with_num() {
        let src = " 2drots ";
        let res: Value = src.parse().unwrap();
        assert_eq!(res, Value::Symbol("2drots".to_string()));
    }

    #[test]
    fn test_string_with_escaping() {
        //' baba"papa '
        let src = "\" baba\\\"papa \" ";
        let res: Value = src.parse().unwrap();
        assert_eq!(res, Value::String(" baba\"papa ".to_string()));
    }

    #[test]
    fn test_unbalanced_expr() {
        let src = " ( jdeidje ] )";
        let actual = src.parse::<Value>().unwrap_err();
        assert_eq!(actual, Error::UnbalancedExpr)
    }

    #[test]
    fn test_unbalanced_list_with_paren_at_end() {
        let src = " ( jdeidje ";
        let actual = src.parse::<Value>().unwrap_err();
        assert_eq!(actual, Error::UnbalancedExpr)
    }

    #[test]
    fn test_unexcepted_dot() {
        let src = " ( [ 1 2 3 4 . ] )";
        let actual = src.parse::<Value>().unwrap_err();
        assert_eq!(actual, Error::UnexpectedDot)
    }
}
