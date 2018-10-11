use parse::{HTMLElement, Node};

pub fn render(nodes: Vec<Node>) -> String {
    let mut output = "".to_string();
    for node in nodes {
        match node {
            Node::Element(e) => {
                output.push_str(&e.render(0));
            }
            Node::Text(body) => {
                // Consider about indent
                // TODO Escape
                output.push_str(&body);
            } // TODO Implement Comment
            _ => continue,
        }
    }
    output
}
