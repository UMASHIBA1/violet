use crate::dom;
use std::collections::HashMap;

struct Parser {
    pos: usize,
    input: String,
}

pub fn parse(source: String) -> dom::Node {
    let mut nodes = Parser {pos: 0, input: source}.parse_nodes();

    if nodes.len() == 1 {
        nodes.swap_remove(0)
    } else {
        dom::elem("html".to_string(), HashMap::new(), nodes)
    }
}

impl Parser {

    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn eof(&self)->bool {
        self.pos >= self.input.len()
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        // NOTE: += next_posがなぞ
        self.pos += next_pos;
        return cur_char;
    }

    fn consume_while<F>(&mut self, test: F) -> String where F: Fn(char) -> bool {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        return result;
    }

    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert!(self.consume_char() == open_quote);
        return value;
    }

    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        assert!(self.consume_char() == '=');
        let value = self.parse_attr_value();
        return (name, value);
    }

    // Consume and discard zero or more whitespace characters.
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            };
            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        };
        return attributes;
    }

    // Parse a tag or attribute name.
    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => true,
            _ => false
        })
    }

    fn parse_text(&mut self) -> dom::Node {
        dom::text(self.consume_while(|c| c != '<'))
    }

    // 一つのエレメントノードをパースする
    fn parse_element(&mut self) -> dom::Node {
        assert!(self.consume_char() == '<', "the element does not start with <");
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();
        assert!(self.consume_char() == '>');

        let children = self.parse_nodes();

        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name, "start tag name and end tag name is not equal");
        assert!(self.consume_char() == '>');

        return dom::elem(tag_name, attrs, children);
    }

    fn consume_comment(&mut self) {
        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '!');
        assert!(self.consume_char() == '-');
        assert!(self.consume_char() == '-');

        while !self.eof() && !self.starts_with("-->") {
            self.consume_char();
        };

        assert!(self.consume_char() == '-');
        assert!(self.consume_char() == '-');
        self.consume_whitespace();
        assert!(self.consume_char() == '>');
    }

    // NOTE: 一つのノードをパースする
    fn parse_node(&mut self) -> dom::Node {
        if self.starts_with("<!--") {
            self.consume_comment();
        }
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text()
        }
    }

    // NOTE: 複数のノードをパースする
    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        return nodes;
    }

}


#[cfg(test)]
mod tests {
    use super::parse;
    use crate::dom::{elem, Node, text};
    use std::collections::HashMap;

    fn create_div_element() -> Node {
        elem("div".to_string(), HashMap::new(), vec![])
    }

    #[test]
    fn parse_only_html_tag() {
        let target_str = "<html></html>".to_string();
        let parsed_dom = parse(target_str);
        let expected_dom = elem("html".to_string(), HashMap::new(),vec![]);
        assert_eq!(parsed_dom, expected_dom);
    }

    #[test]
    fn parse_html_and_body() {
        let target_str = "<html><body></body></html>".to_string();
        let parsed_dom = parse(target_str);
        let expected_dom = elem("html".to_string(), HashMap::new(),vec![elem("body".to_string(), HashMap::new(), vec![])]);
        assert_eq!(parsed_dom, expected_dom);
    }

    #[test]
    fn parse_one_div_element_dom() {
        let target_str = "<html><body><div></div></body></html>".to_string();
        let parsed_dom = parse(target_str);
        let expected_dom = elem("html".to_string(), HashMap::new(),vec![elem("body".to_string(), HashMap::new(), vec![elem("div".to_string(), HashMap::new(), vec![])])]);
        assert_eq!(parsed_dom, expected_dom);
    }

    #[test]
    fn parse_multi_div_element_dom() {
        let target_str = "<html><body><div></div><div></div><div></div></body></html>".to_string();
        let parsed_dom = parse(target_str);
        let expected_dom = elem("html".to_string(), HashMap::new(),vec![elem("body".to_string(), HashMap::new(), vec![create_div_element(), create_div_element(), create_div_element()])]);
        assert_eq!(parsed_dom, expected_dom);
    }

    #[test]
    fn parse_text_node_dom() {
        let target_str = "<html><body><div>sample text</div></body></html>".to_string();
        let parsed_dom = parse(target_str);
        let expected_dom = elem("html".to_string(), HashMap::new(),vec![
            elem("body".to_string(), HashMap::new(), vec![
                elem("div".to_string(), HashMap::new(), vec![
                    text("sample text".to_string())
                ]),
            ])
        ]);
        assert_eq!(parsed_dom, expected_dom);
    }

    #[test]
    fn parse_comment_node_dom() {
        let target_str = "<html><body><!-- sample comment --><div></div></body></html>".to_string();
        let parsed_dom = parse(target_str);
        let expected_dom = elem("html".to_string(), HashMap::new(),vec![elem("body".to_string(), HashMap::new(), vec![elem("div".to_string(), HashMap::new(), vec![])])]);
        assert_eq!(parsed_dom, expected_dom);
    }

}

// let html_string = "<html><body><h1>Title</h1><div id=\"main\" class=\"test\"><p>Hello <em>world</em>!</p></div></body></html>";
