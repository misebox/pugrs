use std::fmt;

pub enum TokenType {
    NewLine,
    Tag(String),
    Id(String),
    Class(String),
    Attr(String, String),
    Text(String),
    Colon,
    Indent(usize),
    Outdent(usize),
    Slash,
}
impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::NewLine => write!(f, "NewLine"),
            TokenType::Tag(name) => write!(f, "Tag({})", name),
            TokenType::Id(name) => write!(f, "Id({})", name),
            TokenType::Class(name) => write!(f, "Class({})", name),
            TokenType::Attr(name, value) => write!(f, "Attr({}, {})", name, value),
            TokenType::Text(body) => write!(f, "Text({})", body),
            TokenType::Colon => write!(f, "Colon"),
            TokenType::Indent(level) => write!(f, "Indent({})", level),
            TokenType::Outdent(level) => write!(f, "Outdent({})", level),
            TokenType::Slash => write!(f, "Slash"),
        }
    }
}

pub struct Token {
    ty: TokenType,
    start: usize,
    end: usize,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}: {}..{}>", self.ty, self.start, self.end)
    }
}

pub struct Lexer {
    src: String,
    tokens: Vec<Token>,
    pos: usize,
}

use std::boxed::Box;
use std::iter::Peekable;
use std::str::Chars;

type CharCond = Fn(char) -> bool;

impl Lexer {
    pub fn new(src: String) -> Lexer {
        Lexer {
            src: src,
            tokens: vec![],
            pos: 0,
        }
    }

    pub fn token_source(&self, token: &Token) -> String {
        let mut printable = self.src[token.start..token.end].to_string();
        for (from, to) in &[("\t", "<Tab>"), ("\n", "<LF>")] {
            printable = printable.replace(from, to);
        }
        printable
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }
    fn add_token(&mut self, ty: TokenType, start: usize, length: usize) {
        println!("<{}: {}..{}>", &ty, &start, &length);
        self.tokens.push(Token {
            ty: ty,
            start: start,
            end: start + length,
        });
    }
    fn consume_next(&mut self, p: &mut Peekable<Chars>) -> char {
        self.pos += 1;
        p.next().unwrap()
    }
    fn consume_while(&mut self, p: &mut Peekable<Chars>, cb: Box<CharCond>) -> Option<String> {
        let mut v: Vec<char> = vec![];
        while p.peek().is_some() && {
            let c = *p.peek().unwrap();
            cb(c)
        } {
            v.push(self.consume_next(p));
        }
        if v.len() > 0 {
            Some(v.iter().collect::<String>())
        } else {
            None
        }
    }
    fn consume_whitespaces(&mut self, p: &mut Peekable<Chars>) -> Option<String> {
        self.consume_while(p, Box::new(|c| -> bool { c.is_ascii_whitespace() }))
    }
    fn consume_name(&mut self, p: &mut Peekable<Chars>) -> Option<String> {
        self.consume_while(
            p,
            Box::new(|c: char| -> bool { c.is_ascii_alphanumeric() || c == '-' || c == '_' }),
        )
    }
    fn consume_quoted(&mut self, p: &mut Peekable<Chars>) -> Option<String> {
        let sign = match p.peek() {
            Some(&'"') | Some(&'\'') => self.consume_next(p),
            _ => {
                return None;
            }
        };
        let mut v: Vec<char> = vec![];
        while p.peek().is_some() && *p.peek().unwrap() != sign {
            v.push(self.consume_next(p));
        }
        self.consume_next(p);
        Some(v.iter().collect::<String>())
    }
    pub fn tokenize(&mut self) {
        let tmp = self.src.clone();
        let mut c_iter = tmp.chars().peekable();
        let mut indents = vec![0_usize];
        'outer: loop {
            let ch = match c_iter.peek() {
                None => {
                    println!("end of file");
                    break;
                }
                Some(&c) => c,
            };
            match &ch {
                c if c.is_ascii_control() => {
                    print!("- [{}] ", &ch.escape_unicode());
                }
                c => {
                    print!("- [{}] ", c);
                }
            }
            match ch {
                s if s.is_ascii_alphabetic() => {
                    // Found Tag
                    let start = self.pos;
                    let name = self.consume_name(&mut c_iter).unwrap();
                    let len = name.len();
                    self.add_token(TokenType::Tag(name), start, len);
                    continue;
                }
                '\n' => {
                    // Found new line
                    let start = self.pos;
                    self.add_token(TokenType::NewLine, start, 1);
                    self.consume_next(&mut c_iter);
                    let start = self.pos;
                    let level = match self
                        .consume_while(&mut c_iter, Box::new(|c: char| -> bool { c == ' ' }))
                    {
                        Some(s) => s.len(),
                        None => 0,
                    };
                    let prev = indents[indents.len() - 1];
                    if level > prev {
                        // Found indent
                        self.add_token(TokenType::Indent(level), start, level);
                        indents.push(level);
                    } else if level < prev {
                        // Found outdent
                        self.add_token(TokenType::Outdent(level), start, level);
                        indents.pop();
                    }
                    continue;
                }
                ' ' | '|' => {
                    // Found text (" text" or "| text" or "|text")
                    let start = self.pos;
                    let mut len = 1;
                    let first = self.consume_next(&mut c_iter);
                    if first == '|' && *c_iter.peek().unwrap() == ' ' {
                        self.consume_next(&mut c_iter);
                        len += 1;
                    }
                    if let Some(body) =
                        self.consume_while(&mut c_iter, Box::new(|c: char| -> bool { c != '\n' }))
                    {
                        len += body.len();
                        self.add_token(TokenType::Text(body), start, len);
                    }
                    continue;
                }
                '#' => {
                    // Found id
                    let start = self.pos;
                    self.consume_next(&mut c_iter);
                    if let Some(name) = self.consume_name(&mut c_iter) {
                        let len = self.pos - start;
                        self.add_token(TokenType::Id(name), start, len);
                    };
                    continue;
                }
                '.' => {
                    // Found class
                    let start = self.pos;
                    self.consume_next(&mut c_iter);
                    if let Some(name) = self.consume_name(&mut c_iter) {
                        let len = self.pos - start;
                        self.add_token(TokenType::Class(name), start, len);
                    }
                    continue;
                }
                '(' => {
                    // Found attrs
                    self.consume_next(&mut c_iter);
                    loop {
                        if !c_iter.peek().is_some() {
                            println!("Closing parenthesis not found");
                            break 'outer;
                        }
                        match *c_iter.peek().unwrap() {
                            ')' => {
                                self.consume_next(&mut c_iter);
                                break;
                            }
                            c if c.is_ascii_whitespace() => {
                                self.consume_whitespaces(&mut c_iter);
                                continue;
                            }
                            c if c.is_ascii_alphabetic() => {
                                // Found an attribute
                                let start = self.pos;
                                let name = self.consume_name(&mut c_iter).unwrap();
                                let value: String = match c_iter.peek() {
                                    Some(&c) if c.is_ascii_whitespace() => {
                                        self.consume_whitespaces(&mut c_iter);
                                        "".to_string()
                                    }
                                    Some(&'=') => {
                                        self.consume_next(&mut c_iter);
                                        match c_iter.peek() {
                                            Some(&c) if c.is_ascii_whitespace() => {
                                                self.consume_whitespaces(&mut c_iter);
                                                "".to_string()
                                            }
                                            Some(&'"') | Some(&'\'') => {
                                                match self.consume_quoted(&mut c_iter) {
                                                    Some(body) => body,
                                                    None => {
                                                        println!("Error: closing quote not found");
                                                        break;
                                                    }
                                                }
                                            }
                                            Some(&_) => self
                                                .consume_while(
                                                    &mut c_iter,
                                                    Box::new(|c: char| -> bool {
                                                        !c.is_ascii_whitespace() && c != ')'
                                                    }),
                                                ).unwrap(),
                                            None => "".to_string(),
                                        }
                                    }
                                    _ => "".to_string(),
                                };
                                let len = self.pos - start;
                                self.add_token(TokenType::Attr(name, value), start, len);
                                continue;
                            }
                            _ => break,
                        }
                    }
                    continue;
                }
                '/' => {
                    // Found slash
                    let start = self.pos;
                    self.consume_next(&mut c_iter);
                    self.add_token(TokenType::Slash, start, 1);
                    continue;
                }
                ':' => {
                    // Found colon
                    let start = self.pos;
                    self.add_token(TokenType::Colon, start, 1);
                    // consume ':'
                    self.consume_next(&mut c_iter);
                    // consume ' ' after ':'
                    self.consume_while(&mut c_iter, Box::new(|c| -> bool { c == ' ' }));
                }
                s => {
                    println!("# Found an unexpected char: [{}]", s);
                    break;
                }
            };
        }
    }
}
