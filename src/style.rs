// NOTE: https://limpet.net/mbrubeck/2014/08/23/toy-layout-engine-4-style.html

use std::collections::{HashMap};
use crate::css::{Value, Selector, SimpleSelector, Specificity, Rule, Stylesheet};
use crate::dom::{Node, ElementData, NodeType};

pub type PropertyMap = HashMap<String, Value>;

#[derive(Clone, Debug, PartialEq)]
pub struct StyledNode<'a> {
    node: &'a Node,
    specified_values: PropertyMap,
    children: Vec<StyledNode<'a>>,
}


pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    StyledNode {
        node: root,
        specified_values: match root.node_type {
            NodeType::Element(ref elem) => specified_values(elem, stylesheet),
            NodeType::Text(_) => HashMap::new()
        },
        children: root.children.iter().map(|child| style_tree(child, stylesheet)).collect(),
    }
}

// その要素に渡すDeclarationのプロパティ名と値のマップを返す
fn specified_values(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);

    rules.sort_by(|&(a, _), &(b,_)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    return values;
}

type MatchedRule<'a> = (Specificity, &'a Rule);

//NOTE: ルールの配列に対してその要素に対応するかをそれぞれ判定
fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    stylesheet.rules.iter().filter_map(|rule| match_rule(elem, rule)).collect()
}



// そのルールの持つセレクタに要素が合致するか判定
fn match_rule<'a>(elem: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    rule.selectors.iter()
        .find(|selector| matches(elem, *selector))
        .map(|selector| (selector.specificity(), rule))
}


// NOTE: そのセレクタがそのElementに合致するか判定
fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match *selector {
        Selector::Simple(ref simple_selector) => matches_simple_selector(elem, simple_selector)
    }
}

fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    // tag_name.iter(): Optionのiterでtag_nameの存在確認 -> anyにより存在していたうえでtag_nameと合致するかを確認、合致しなければreturn false
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    let elem_classes = elem.classes();
    if selector.class.iter().any(|class| !elem_classes.contains(&**class)) {
        return false;
    }

    return true;
}


// NOTE: 処理の手順を自分なりにまとめます
// 目標: そのNodeに対応したCSSのDeclarationを付与した要素のツリー(StyledNode)を作成する

// 手順:
// 以下を子ノードに対して再帰的に繰り返す
// 1. Rulesのセレクタの中からそのノードに一致するセレクタを探し、一致するRuleを配列にする
// 2. そのRuleの配列をセレクタの優先順位の合計に沿ってソートする
// 3. Ruleの配列からDeclarationのプロパティ名とプロパティの値をHashMapに代入しそれを配列化する
// 4. 配列にしたDeclarationをspecified_valueとしてNodeのプロパティに入れる.

#[cfg(test)]
mod tests {
    use super::style_tree;
    use crate::dom::{Node, NodeType, AttrMap, ElementData};
    use crate::css::{Stylesheet, Rule, Selector, SimpleSelector, Value, Declaration, Unit};
    use crate::style::{StyledNode, PropertyMap};


    fn create_element_node(tag_name: String, attributes: AttrMap, children: Vec<Node>) -> Node {
        let this_element = NodeType::Element(ElementData {tag_name, attributes});
        Node {node_type: this_element, children}
    }

    // fn create_styled_element_node(tag_name: String, attributes: AttrMap, children: Vec<Node>) {
    //     let this_element = StyledNode {
    //         node: create_element_node(tag_name, attributes, children),
    //         specified_values:
    //     }
    // }

    // fn create_rule(tag_name: Option<String>, id: Option<String>, class: Vec<String>, declarations: Vec<Declaration>) -> Rule {
    //     Rule {selectors: vec![Selector::Simple(SimpleSelector {tag_name, id, class})], declarations}
    // }
    //
    // fn setup<'a>() -> (Node, StyledNode<'a>) {
    //     let body = create_element_node("body".to_string(), AttrMap::new(), vec![]);
    //     let styled_body: StyledNode<'a> = StyledNode {node: &body.clone(), specified_values: PropertyMap::new(), children: vec![]};
    //     let html = create_element_node("html".to_string(), AttrMap::new(), vec![body]);
    //     let styled_html: StyledNode<'a> = StyledNode {node: &html.clone(), specified_values: PropertyMap::new(), children: vec![styled_body]};
    //
    //     (html, styled_html)
    // }
    //
    // fn add_to_node_body(added_node: &Node, html_node: &mut Node) {
    //     html_node.children[0].children.push(added_node.clone());
    // }

    // fn add_to_styled_body(added_styled_node: &StyledNode, styled_html_node: &mut StyledNode) {
    //     styled_html_node.children[0].children.push(added_styled_node.clone());
    // }

    #[test]
    fn test_merge_one_div_and_one_rule() {

        // let aaa =   StyledNode {
        //     node: Node {
        //         children: [
        //             Node {
        //                 children: [Node { children: [], node_type: Element(ElementData { tag_name: "div", attributes: {} }) }],
        //                 node_type: Element(ElementData { tag_name: "body", attributes: {} }) }
        //         ],
        //         node_type: Element(ElementData { tag_name: "html", attributes: {} }) },
        //     specified_values: {},
        //     children: [
        //         StyledNode {
        //             node: Node {
        //                 children: [Node { children: [], node_type: Element(ElementData { tag_name: "div", attributes: {} }) }],
        //                 node_type: Element(ElementData { tag_name: "body", attributes: {} }) },
        //             specified_values: {},
        //             children: [StyledNode {
        //                 node: Node { children: [], node_type: Element(ElementData { tag_name: "div", attributes: {} }) },
        //                 specified_values: {"padding": Length(4.0, Px), "margin": Keyword("auto")},
        //                 children: []
        //             }]
        //         }]
        // };

        let target_element = create_element_node("div".to_string(), AttrMap::new(), vec![]);
        let body = create_element_node("body".to_string(), AttrMap::new(), vec![target_element.clone()]);
        let html = create_element_node("html".to_string(), AttrMap::new(), vec![body.clone()]);

        let target_stylesheet = Stylesheet {rules: vec![
            Rule {
                selectors: vec![ Selector::Simple(SimpleSelector {tag_name: Some("div".to_string()), id: None, class: vec![]})],
                declarations: vec![
                    Declaration {name: "margin".to_string(), value: Value::Keyword("auto".to_string())},
                    Declaration {name: "padding".to_string(), value: Value::Length(4.0, Unit::Px)}
                ]
            }
        ]};
        let styled_html = style_tree(&html, &target_stylesheet);
        
        let mut expected_property_map = PropertyMap::new();
        expected_property_map.insert("margin".to_string(), Value::Keyword("auto".to_string()));
        expected_property_map.insert("padding".to_string(), Value::Length(4.0, Unit::Px));
        let expected_styled_target_node = StyledNode {node: &target_element.clone(), specified_values: expected_property_map, children: vec![]};

        let expected_styled_body: StyledNode = StyledNode {node: &body.clone(), specified_values: PropertyMap::new(), children: vec![expected_styled_target_node]};
        let expected_styled_html: StyledNode = StyledNode {node: &html.clone(), specified_values: PropertyMap::new(), children: vec![expected_styled_body]};


        assert_eq!(styled_html, expected_styled_html);

    }

}