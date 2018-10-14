use std::fmt;

#[derive(Clone, PartialEq)]
pub enum TokenType {
    Doctype(String),
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
            TokenType::Doctype(name) => write!(f, "Doctype({})", name),
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

#[derive(Clone, PartialEq)]
pub struct Token {
    ty: TokenType,
    start: usize,
    end: usize,
}
impl Token {
    pub fn get_type(&self) -> &TokenType {
        &self.ty
    }
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

    #[allow(dead_code)]
    pub fn token_source(&self, token: &Token) -> String {
        let mut printable = self.src[token.start..token.end].to_string();
        for (from, to) in &[("\t", "<Tab>"), ("\n", "<LF>")] {
            printable = printable.replace(from, to);
        }
        printable
    }

    pub fn get_tokens(&self) -> Vec<Token> {
        self.tokens.clone()
    }
    fn add_token(&mut self, ty: TokenType, start: usize, length: usize) {
        eprintln!("<{}: {}..{}>", &ty, &start, &length);
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
    fn consume_line(&mut self, p: &mut Peekable<Chars>) -> Option<String> {
        match self.consume_while(p, Box::new(|c| -> bool { c != '\n' })) {
            Some(mut s) => {
                if p.peek().is_some() {
                    s.push(self.consume_next(p));
                }
                Some(s)
            }
            None => None,
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

        let first_line: String = tmp.chars().take_while(|&x| -> bool { x != '\n' }).collect();
        if first_line.starts_with("doctype html") {
            self.consume_line(&mut c_iter);
            self.add_token(TokenType::Doctype("html".to_string()), 0, first_line.len());
        }

        'outer: loop {
            let ch = match c_iter.peek() {
                None => {
                    eprintln!("end of file");
                    break;
                }
                Some(&c) => c,
            };
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
                    let prev = if indents.len() > 0 {
                        indents[indents.len() - 1]
                    } else {
                        0
                    };
                    if level > prev {
                        // Found indent
                        self.add_token(TokenType::Indent, start, level);
                        indents.push(level);
                    } else if level < prev {
                        // Found outdent
                        let mut sz = prev;
                        while level < sz && indents.len() > 0 {
                            eprintln!("Outdent! actual level={}, indent level={}", &level, &sz);
                            indents.pop();
                            sz = indents[indents.len() - 1];
                            self.add_token(TokenType::Outdent, sz, sz - level);
                        }
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
                            eprintln!("Closing parenthesis not found");
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
                                                        eprintln!("Error: closing quote not found");
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
                    eprintln!("# Found an unexpected char: [{}]", s);
                    break;
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize_one(src: &str) -> Token {
        let mut lex = Lexer::new(src.to_string());
        lex.tokenize();
        let tokens = lex.get_tokens();
        assert_eq!(tokens.len(), 1);
        tokens[0].clone()
    }
    fn tokenize(src: &str) -> Vec<Token> {
        let mut lex = Lexer::new(src.to_string());
        lex.tokenize();
        lex.get_tokens()
    }

    #[test]
    fn lexer_works_html() {
        let src = "html";
        let token = tokenize_one(src);
        assert_eq![
            token,
            Token {
                ty: TokenType::Tag("html".to_string()),
                start: 0,
                end: 4,
            }
        ];
    }
    #[test]
    fn lexer_works_doctype() {
        let src = "doctype html";
        let token = tokenize_one(src);
        assert_eq![
            token,
            Token {
                ty: TokenType::Doctype("html".to_string()),
                start: 0,
                end: 12,
            }
        ];
    }
    #[test]
    fn lexer_works_id() {
        let src = "#abc";
        let token = tokenize_one(src);
        assert_eq![
            token,
            Token {
                ty: TokenType::Id("abc".to_string()),
                start: 0,
                end: 4,
            }
        ];
    }
    #[test]
    fn lexer_works_class() {
        let src = ".class-name1.class-name2";
        let tokens = tokenize(src);
        assert_eq![
            tokens[0],
            Token {
                ty: TokenType::Class("class-name1".to_string()),
                start: 0,
                end: 12,
            }
        ];
        assert_eq![
            tokens[1],
            Token {
                ty: TokenType::Class("class-name2".to_string()),
                start: 12,
                end: 24,
            }
        ];
    }
    #[test]
    fn lexer_works_attr() {
        let src = r#"(aa=AA bb="B B" cc="'CC'")"#;
        let tokens = tokenize(src);
        assert_eq![
            tokens[0],
            Token {
                ty: TokenType::Attr("aa".to_string(), "AA".to_string()),
                start: 1,
                end: 6,
            }
        ];
        assert_eq![
            tokens[1],
            Token {
                ty: TokenType::Attr("bb".to_string(), "B B".to_string()),
                start: 7,
                end: 15,
            }
        ];
        assert_eq![
            tokens[2],
            Token {
                ty: TokenType::Attr("cc".to_string(), "'CC'".to_string()),
                start: 16,
                end: 25,
            }
        ];
    }
    #[test]
    fn lexer_works_colon() {
        let src = r#"div: span: img"#;
        let tokens = tokenize(src);
        assert_eq![
            tokens[0],
            Token {
                ty: TokenType::Tag("div".to_string()),
                start: 0,
                end: 3,
            }
        ];
        assert_eq![
            tokens[1],
            Token {
                ty: TokenType::Colon,
                start: 3,
                end: 4,
            }
        ];
        assert_eq![
            tokens[2],
            Token {
                ty: TokenType::Tag("span".to_string()),
                start: 5,
                end: 9,
            }
        ];
        assert_eq![
            tokens[3],
            Token {
                ty: TokenType::Colon,
                start: 9,
                end: 10,
            }
        ];
        assert_eq![
            tokens[4],
            Token {
                ty: TokenType::Tag("img".to_string()),
                start: 11,
                end: 14,
            }
        ];
    }
    #[test]
    fn lexer_works_all_in_one() {
        let src = r##"doctype html
html
  head
    meta(charset="UTF-8")
    title ページタイトル
  body
    .wrapper
      #header
        .menu
          a(href="#" alt="link"): img
      #container
        ul.item-list
          li.item text1
          li.item text2
          li.item text3
"##;
        let mut lex = Lexer::new(src.to_string());
        lex.tokenize();
        let tokens = lex.get_tokens();
        assert_eq!(tokens.len(), 58);
        let expects = vec![
            TokenType::Doctype("html".to_string()),
            TokenType::Tag("html".to_string()),
            TokenType::NewLine,
            TokenType::Indent,
            TokenType::Tag("head".to_string()),
            TokenType::NewLine,
            TokenType::Indent,
            TokenType::Tag("meta".to_string()),
            TokenType::Attr("charset".to_string(), "UTF-8".to_string()),
            TokenType::NewLine,
            TokenType::Tag("title".to_string()),
            TokenType::Text("ページタイトル".to_string()),
            TokenType::NewLine,
            TokenType::Outdent,
            TokenType::Tag("body".to_string()),
            TokenType::NewLine,
            TokenType::Indent,
            TokenType::Class("wrapper".to_string()),
            TokenType::NewLine,
            TokenType::Indent,
            TokenType::Id("header".to_string()),
            TokenType::NewLine,
            TokenType::Indent,
            TokenType::Class("menu".to_string()),
            TokenType::NewLine,
            TokenType::Indent,
            TokenType::Tag("a".to_string()),
            TokenType::Attr("href".to_string(), "#".to_string()),
            TokenType::Attr("alt".to_string(), "link".to_string()),
            TokenType::Colon,
            TokenType::Tag("img".to_string()),
            TokenType::NewLine,
            TokenType::Outdent,
            TokenType::Outdent,
            TokenType::Id("container".to_string()),
            TokenType::NewLine,
            TokenType::Indent,
            TokenType::Tag("ul".to_string()),
            TokenType::Class("item-list".to_string()),
            TokenType::NewLine,
            TokenType::Indent,
            TokenType::Tag("li".to_string()),
            TokenType::Class("item".to_string()),
            TokenType::Text("text1".to_string()),
            TokenType::NewLine,
            TokenType::Tag("li".to_string()),
            TokenType::Class("item".to_string()),
            TokenType::Text("text2".to_string()),
            TokenType::NewLine,
            TokenType::Tag("li".to_string()),
            TokenType::Class("item".to_string()),
            TokenType::Text("text3".to_string()),
            TokenType::NewLine,
            TokenType::Outdent,
            TokenType::Outdent,
            TokenType::Outdent,
            TokenType::Outdent,
            TokenType::Outdent,
        ];
        let zipped = tokens.iter().zip(expects.iter());
        for (actual, expect) in zipped {
            println!("{}", actual.ty);
            println!("{}", expect);
            assert!(*actual.get_type() == *expect);
        }
    }
}
