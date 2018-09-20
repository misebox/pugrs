use regex::Regex;
use std::collections::HashMap;
use std::fmt;

enum TokenType {
    NewLine,
    Tag(String),
    Id(String),
    Class(String),
    Attr(String, String),
    Text(String),
    Colon,
    Indent,
    Outdent,
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
            TokenType::Indent => write!(f, "Indent"),
            TokenType::Outdent => write!(f, "Outdent"),
            TokenType::Slash => write!(f, "Slash"),
        }
    }
}

// #[derive(Clone, Debug, PartialEq)]
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
    pub fn add_token(&mut self, ty: TokenType, start: usize, length: usize) {
        println!("<{}: {}..{}>", &ty, &start, &length);
        self.tokens.push(Token {
            ty: ty,
            start: self.pos + start,
            end: self.pos + start + length,
        });
    }
    pub fn tokenize(&mut self) {
        let mut tmp = self.src.clone();
        let mut c_iter = tmp.chars().peekable();
        let mut indents = vec![0_usize];
        loop {
            let ch = match c_iter.peek() {
                None => {
                    println!("end of file");
                    break;
                }
                Some(&c) => c,
            };
            print!("- [{}] ", &ch);
            match ch {
                s if s.is_ascii_alphabetic() => {
                    let mut v: Vec<char> = vec![];
                    v.push(c_iter.next().unwrap());
                    while c_iter.peek().is_some() && {
                        let c = *c_iter.peek().unwrap();
                        c.is_ascii_alphanumeric() || c == '-'
                    } {
                        v.push(c_iter.next().unwrap());
                    }
                    let name = v.iter().collect::<String>();
                    let len = name.len();
                    self.add_token(TokenType::Tag(name), 0, len);
                    self.pos += len;
                    continue;
                }
                '\n' => {
                    c_iter.next();
                    self.add_token(TokenType::NewLine, 0, 1);
                    self.pos += 1;
                    let mut v: Vec<char> = vec![];
                    while c_iter.peek().is_some() && *c_iter.peek().unwrap() == ' ' {
                        v.push(c_iter.next().unwrap());
                    }
                    let level = v.len();
                    let prev = indents[indents.len() - 1];
                    if level > prev {
                        self.add_token(TokenType::Indent, 0, level);
                        println!("Indent {}", level);
                        self.pos += level;
                        indents.push(level);
                    } else if level < prev {
                        self.add_token(TokenType::Outdent, 0, level);
                        println!("Outdent {}", level);
                        self.pos += level;
                        indents.pop();
                    }
                    continue;
                }
                ' ' => {
                    let mut i = 0;
                    while c_iter.peek().is_some() && *c_iter.peek().unwrap() == ' ' {
                        c_iter.next();
                        i += 1;
                        self.pos += 1;
                    }
                    println!("# Found blank x {}", i);
                    continue;
                }
                s @ _ => {
                    println!("# Found an unexpected char: [{}]", s);
                    self.pos += 1;
                    break;
                }
            };
        }
    }
}
