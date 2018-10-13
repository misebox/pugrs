use lex::{Token, TokenType};

pub enum Node {
    Empty,
    Element(Box<HTMLElement>),
    Text(String),
    Comment,
}

pub struct HTMLElement {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<(Node)>,
}

impl HTMLElement {
    fn new(name: String) -> HTMLElement {
        HTMLElement {
            name: name,
            attrs: vec![],
            children: vec![],
        }
    }
    fn push_attr(&mut self, name: String, value: String) {
        self.attrs.push((name, value));
    }
    fn push_child(&mut self, child: Node) {
        self.children.push(child);
    }
    pub fn render(&self, indent: usize) -> String {
        let indent_unit = "  ";
        let mut html = "".to_string();
        html.push_str(&indent_unit.repeat(indent));
        html.push('<');
        html.push_str(&self.name);
        for (name, value) in &self.attrs {
            html.push(' ');
            // TODO HTML ESCAPE
            html.push_str(&name);
            html.push_str(r#"=""#);
            // TODO HTML ESCAPE
            html.push_str(&value);
            html.push('"');
        }
        html.push('>');
        //
        if self.children.len() > 0 || self.attrs.len() > 0 {
            html.push('\n');
        }
        match &self.name[0..] {
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta"
            | "param" | "source" | "track" | "wbr" => {
                // No need close tag
                html.push('\n');
            }
            _ => {
                for child in &self.children {
                    let str = match child {
                        Node::Element(e) => {
                            html.push_str(&e.render(indent + 1)[0..]);
                        }
                        Node::Text(body) => {
                            html.push_str(&indent_unit.repeat(indent + 1));
                            html.push_str(&body[0..]);
                            html.push('\n');
                        }
                        _ => continue,
                    };
                }

                // Close tag
                html.push_str(&indent_unit.repeat(indent));
                html.push_str("</");
                html.push_str(&self.name);
                html.push_str(">\n");
            }
        }
        html
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
    nest: usize,
}

use std::boxed::Box;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,
            index: 0,
            nest: 0,
        }
    }
    fn peek(&mut self) -> Option<Token> {
        if self.tokens.len() > self.index {
            let token = self.tokens[self.index].clone();
            Some(token)
        } else {
            None
        }
    }
    fn next(&mut self) -> Option<Token> {
        if self.tokens.len() > self.index {
            let token = self.tokens[self.index].clone();
            self.index += 1;
            eprintln!("{}- {:?}, ", " ".repeat(self.nest), &token);
            Some(token)
        } else {
            eprintln!("end of tokens");
            None
        }
    }
    fn create_element(&mut self, name: String) -> HTMLElement {
        let mut element = HTMLElement::new(name);
        loop {
            if let Some(t) = self.peek() {
                match t.get_type() {
                    TokenType::Id(value) => {
                        self.next();
                        element.attrs.push(("id".to_string(), value.to_string()))
                    }
                    TokenType::Class(value) => {
                        self.next();
                        element.attrs.push(("class".to_string(), value.to_string()))
                    }
                    TokenType::Attr(name, value) => {
                        self.next();
                        element.attrs.push((name.to_string(), value.to_string()))
                    }
                    TokenType::Text(body) => {
                        self.next();
                        element.children.extend(vec![Node::Text(body.to_string())]);
                    }
                    TokenType::NewLine => {
                        self.next();
                        if let Some(t) = self.peek() {
                            match t.get_type() {
                                TokenType::NewLine | TokenType::Tag(_) => {
                                    break;
                                }
                                _ => continue,
                            }
                        } else {
                            break;
                        }
                    }
                    TokenType::Indent => {
                        self.next();
                        self.nest += 1;
                        eprintln!("start parse children {}", self.nest);
                        element.children.extend(self.parse());
                        eprintln!("end parse children {}", self.nest);
                        self.nest -= 1;
                    }
                    TokenType::Colon => {
                        self.next();
                        self.nest += 1;
                        eprintln!("start parse child {}", self.nest);
                        element.children.extend(vec![self.parse_one()]);
                        eprintln!("end parse child {}", self.nest);
                        self.nest -= 1;
                    }
                    TokenType::Outdent | TokenType::Slash => {
                        break;
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }
        element
    }

    pub fn parse_one(&mut self) -> Node {
        let node = match self.next() {
            Some(t) => match t.get_type() {
                TokenType::Text(body) => Node::Text(body.to_string()),
                TokenType::Tag(name) => {
                    Node::Element(Box::new(self.create_element(name.to_string())))
                }
                TokenType::Id(_id) => {
                    let mut element = self.create_element("div".to_string());
                    element.push_attr("id".to_string(), _id.to_string());
                    Node::Element(Box::new(element))
                }
                TokenType::Class(name) => {
                    let mut element = self.create_element("div".to_string());
                    element.push_attr("class".to_string(), name.to_string());
                    Node::Element(Box::new(element))
                }
                tt => {
                    eprintln!("Parse Error {}", tt);
                    return Node::Empty;
                }
            },
            None => Node::Empty,
        };
        node
    }

    pub fn parse(&mut self) -> Vec<Node> {
        let mut nodes: Vec<Node> = vec![];
        loop {
            let node = match self.peek() {
                Some(t) => {
                    match t.get_type() {
                        TokenType::Outdent => {
                            self.next();
                            break;
                        }
                        TokenType::NewLine | TokenType::Slash => {
                            self.next();
                            continue;
                        }
                        _ => (),
                    };
                    self.parse_one()
                }
                None => break,
            };
            nodes.push(node);
        }
        nodes
    }
}
