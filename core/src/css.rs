// NOTE: 参考: https://limpet.net/mbrubeck/2014/08/13/toy-layout-engine-3-css.html
// https://github.com/mbrubeck/robinson/blob/master/src/css.rs

#[derive(Debug, PartialEq, Clone)]
pub struct Stylesheet {
    pub rules: Vec<Rule>
}

// 一個のセレクタとdeclaration達の塊
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>
}

// NOTE: 今はSimpleSelectorだけだけど今後[href="example.com"]とか追加できるようになる
#[derive(Clone, Debug, PartialEq)]
pub enum Selector {
    Simple(SimpleSelector)
}

// NOTE: #id, .class, bodyみたいな部分
#[derive(Clone, Debug, PartialEq)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>
}

// NOTE: margin: auto;
#[derive(Clone,Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: Value
}

// NOTE: margin: auto; のautoの部分
#[derive(Clone,Debug, PartialEq)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    Percentage(f32),
    ColorValue(Color)
}

impl Value {
    pub fn to_px(&self) -> f32 {
        match *self {
            Value::Length(f, Unit::Px) => f,
            _ => 0.0
        }
    }
}

// NOTE: 現在pxのみだけど本来はvwとかemとか入る
#[derive(Clone, Debug, PartialEq)]
pub enum Unit {
    Px
}

// NOTE: 色の構造体
#[derive(Clone,Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

pub type Specificity = (usize, usize, usize);

pub fn parse(source: String) -> Stylesheet {
    let mut parser = Parser {pos: 0, input: source};
    Stylesheet {rules: parser.parse_rules()}
}


impl Selector {
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        (a,b,c)
    }
}



struct Parser {
    pos: usize,
    input: String
}



impl Parser {

    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() {break};
            rules.push(self.parse_rule());
        }
        rules
    }

    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations()
        }
    }

    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {self.consume_char(); self.consume_whitespace();}
                '{' => break,
                c => panic!("Unexpected character {} in selector list", c)
            }
        }
        selectors.sort_by(|a,b| b.specificity().cmp(&a.specificity()));
        selectors
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        assert_eq!(self.consume_char(), '{');
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }
        declarations
    }

    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {tag_name: None, id: None, class: Vec::new()};
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break
            }
        }
        selector
    }



    fn parse_declaration(&mut self) -> Declaration {
        let property_name = self.parse_identifier();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ':');
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ';');

        Declaration {
            name: property_name,
            value,
        }
    }

    fn parse_value(&mut self) -> Value {
        match self.next_char() {
            '0'..='9' => self.parse_start_with_num_value(),
            '#' => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier())
        }
    }

    fn parse_start_with_num_value(&mut self) -> Value {
        let num_value = self.parse_float();
        match self.next_char() {
            '%' => {
                self.consume_char();
                Value::Percentage(num_value)
            },
            _ => Value::Length(num_value, self.parse_unit())
        }
    }

    fn parse_float(&mut self) -> f32 {
        let s = self.consume_while(|c| match c {
            '0'..='9' | '.' => true,
            _ => false
        });
        s.parse().unwrap()
    }

    fn parse_unit(&mut self) -> Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => Unit::Px,
            _ => panic!("unrecognized unit")
        }
    }

    fn parse_color(&mut self) -> Value {
        assert_eq!(self.consume_char(), '#');
        Value::ColorValue(Color {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255
        })
    }

    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos..self.pos + 2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
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
        result
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }
}

fn valid_identifier_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse, Stylesheet, Rule, SimpleSelector, Declaration, Value, Selector, Unit};
    use crate::css::Color;

    #[test]
    fn parse_id_selector() {
        let target_str = "#id {margin: auto;}".to_string();
        let parsed_css = parse(target_str);
        let selector = Selector::Simple(SimpleSelector{tag_name: None, id: Some("id".to_string()), class: vec![]});
        let declaration = Declaration {name: "margin".to_string(), value: Value::Keyword("auto".to_string())};
        let expected_css = Stylesheet {rules: vec![Rule {selectors: vec![selector], declarations: vec![declaration]}]};
        assert_eq!(parsed_css, expected_css);
    }

    #[test]
    fn parse_class_selector() {
        let target_str = ".class {margin: auto;}".to_string();
        let parsed_css = parse(target_str);
        let selector = Selector::Simple(SimpleSelector{tag_name: None, id: None, class: vec!["class".to_string()]});
        let declaration = Declaration {name: "margin".to_string(), value: Value::Keyword("auto".to_string())};
        let expected_css = Stylesheet {rules: vec![Rule {selectors: vec![selector], declarations: vec![declaration]}]};
        assert_eq!(parsed_css, expected_css);
    }

    #[test]
    fn parse_asterisk_selector() {
        let target_str = "* {margin: auto;}".to_string();
        let parsed_css = parse(target_str);
        let selector = Selector::Simple(SimpleSelector{tag_name: None, id: None, class: vec![]});
        let declaration = Declaration {name: "margin".to_string(), value: Value::Keyword("auto".to_string())};
        let expected_css = Stylesheet {rules: vec![Rule {selectors: vec![selector], declarations: vec![declaration]}]};
        assert_eq!(parsed_css, expected_css);
    }

    #[test]
    fn parse_tag_name_selector() {
        let target_str = "input {margin: auto;}".to_string();
        let parsed_css = parse(target_str);
        let selector = Selector::Simple(SimpleSelector{tag_name: Some("input".to_string()), id: None, class: vec![]});
        let declaration = Declaration {name: "margin".to_string(), value: Value::Keyword("auto".to_string())};
        let expected_css = Stylesheet {rules: vec![Rule {selectors: vec![selector], declarations: vec![declaration]}]};
        assert_eq!(parsed_css, expected_css);
    }

    #[test]
    fn parse_keyword_declaration() {
        let target_str = "#id {display: flex;}".to_string();
        let parsed_css = parse(target_str);
        let selector = Selector::Simple(SimpleSelector{tag_name: None, id: Some("id".to_string()), class: vec![]});
        let declaration = Declaration {name: "display".to_string(), value: Value::Keyword("flex".to_string())};
        let expected_css = Stylesheet {rules: vec![Rule {selectors: vec![selector], declarations: vec![declaration]}]};
        assert_eq!(parsed_css, expected_css);
    }

    #[test]
    fn parse_length_declaration() {
        let target_str = "#id {font-size: 16px;}".to_string();
        let parsed_css = parse(target_str);
        let selector = Selector::Simple(SimpleSelector{tag_name: None, id: Some("id".to_string()), class: vec![]});
        let declaration = Declaration {name: "font-size".to_string(), value: Value::Length(16.0, Unit::Px)};
        let expected_css = Stylesheet {rules: vec![Rule {selectors: vec![selector], declarations: vec![declaration]}]};
        assert_eq!(parsed_css, expected_css);
    }

    #[test]
    fn parse_color_declaration() {
        let target_str = "#id {color: #FFFF00;}".to_string();
        let parsed_css = parse(target_str);
        let selector = Selector::Simple(SimpleSelector{tag_name: None, id: Some("id".to_string()), class: vec![]});
        let declaration = Declaration {name: "color".to_string(), value: Value::ColorValue(Color {r: 255, g: 255, b: 0, a: 255})};
        let expected_css = Stylesheet {rules: vec![Rule {selectors: vec![selector], declarations: vec![declaration]}]};
        assert_eq!(parsed_css, expected_css);
    }

    #[test]
    fn parse_percentage_declaration() {
        let target_str = "#id {width: 100%;}".to_string();
        let parsed_css = parse(target_str);
        let selector = Selector::Simple(SimpleSelector{tag_name: None, id: Some("id".to_string()), class: vec![]});
        let declaration = Declaration {name: "width".to_string(), value: Value::Percentage(100.0)};
        let expected_css = Stylesheet {rules: vec![Rule {selectors: vec![selector], declarations: vec![declaration]}]};
        assert_eq!(parsed_css, expected_css);
    }

    #[test]
    fn parse_multi_rules() {
        let target_str = "#id {margin: auto;} .class {margin: auto;}".to_string();
        let parsed_css = parse(target_str);
        let id_selector = Selector::Simple(SimpleSelector{tag_name: None, id: Some("id".to_string()), class: vec![]});
        let class_selector = Selector::Simple(SimpleSelector{tag_name: None, id: None, class: vec!["class".to_string()]});
        let declaration = Declaration {name: "margin".to_string(), value: Value::Keyword("auto".to_string())};
        let id_rule = Rule {selectors: vec![id_selector], declarations: vec![declaration.clone()]};
        let class_rule = Rule {selectors: vec![class_selector], declarations: vec![declaration]};
        let expected_css = Stylesheet {rules: vec![id_rule, class_rule]};
        assert_eq!(parsed_css, expected_css);
    }

}
