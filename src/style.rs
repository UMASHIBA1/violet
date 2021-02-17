// NOTE: https://limpet.net/mbrubeck/2014/08/23/toy-layout-engine-4-style.html

use std::collections::{HashMap};
use crate::css::{Value, Selector, SimpleSelector, Specificity, Rule, Stylesheet, Unit};
use crate::dom::{Node, ElementData, NodeType};

pub type PropertyMap = HashMap<String, Value>;

#[derive(Clone, Debug, PartialEq)]
pub struct StyledNode<'a> {
    node: &'a Node,
    specified_values: PropertyMap,
    children: Vec<StyledNode<'a>>,
}

const INHERIT_PROPS: [&str; 4] = ["color", "font-size", "font-weight", "line-height"];

pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    let default_prop_map = create_default_props();

    style_tree_rec(root, stylesheet, &default_prop_map)
}

fn create_default_props() -> PropertyMap {
    let mut default_prop_map = PropertyMap::new();
    default_prop_map.insert("color".to_string(), Value::Keyword("#000000".to_string()));
    default_prop_map.insert("font-size".to_string(), Value::Length(16.0, Unit::Px));
    default_prop_map.insert("font-weight".to_string(), Value::Keyword("normal".to_string()));
    default_prop_map.insert("line-height".to_string(), Value::Keyword("normal".to_string()));
    default_prop_map
}



fn style_tree_rec<'a>(root: &'a Node, stylesheet: &'a Stylesheet, parent_prop_map: &PropertyMap) -> StyledNode<'a> {
    let specified_values = match root.node_type {
        NodeType::Element(ref elem) => specified_values(elem, stylesheet, parent_prop_map),
        NodeType::Text(_) => HashMap::new()
    };
    StyledNode {
        node: root,
        specified_values: specified_values.clone(),
        children: root.children.iter().map(|child| style_tree_rec(child, stylesheet, &specified_values)).collect(),
    }
}

// その要素に渡すDeclarationのプロパティ名と値のマップを返す
fn specified_values(elem: &ElementData, stylesheet: &Stylesheet, parent_prop_map: &PropertyMap) -> PropertyMap {
    let mut values: PropertyMap = HashMap::new();

        // 継承するのがデフォルトの値に対して全部親から値をとる
        for prop_name in INHERIT_PROPS.iter() {
            match parent_prop_map.get(&prop_name.to_string()) {
                Some(x) => {values.insert(prop_name.to_string(), x.clone());},
                None => ()
            };
        }

    let mut rules = matching_rules(elem, stylesheet);

    rules.sort_by(|&(a, _), &(b,_)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            if declaration.value == Value::Keyword("inherit".to_string()) {
                let parent_value_opt = parent_prop_map.get(declaration.name.as_str());
                match parent_value_opt {
                    Some(x) => {values.insert(declaration.name.clone(), x.clone());},
                    None => ()
                };
            }else {
                values.insert(declaration.name.clone(), declaration.value.clone());
            }
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

    fn create_text_node(text: &str) -> Node {
        let this_element = NodeType::Text(text.to_string());
        Node {node_type: this_element, children: vec![]}
    }

    fn create_styled_node<'a>(node: &'a Node, specified_values: PropertyMap, children: Vec<StyledNode<'a>>) -> StyledNode<'a> {
        StyledNode {node, specified_values, children}
    }

    fn create_simple_selector_rule(selector_data: Vec<(Option<&str>, Option<&str>, Vec<&str>)>, declaration_data: Vec<(&str, Value)>) -> Rule {
        let mut selectors: Vec<Selector> = vec![];
        let mut declarations: Vec<Declaration> = vec![];

        for data in selector_data {
            let selector = Selector::Simple(SimpleSelector {
                tag_name: data.0.and_then(|x| Some(x.to_string())), id: data.1.and_then(|x| Some(x.to_string())), class: data.2.iter().map(|x|x.to_string()).collect()
            });
            selectors.push(selector);
        }

        for data in declaration_data {
            let declaration = Declaration {
                name: data.0.to_string(),
                value: data.1
            };
            declarations.push(declaration);
        }

        Rule {
            selectors,
            declarations
        }
    }

    fn create_inherit_props_map_for_test() -> PropertyMap {
        let mut expected_prop_map = PropertyMap::new();
        expected_prop_map.insert("color".to_string(), Value::Keyword("#000000".to_string()));
        expected_prop_map.insert("font-size".to_string(), Value::Length(16.0, Unit::Px));
        expected_prop_map.insert("font-weight".to_string(), Value::Keyword("normal".to_string()));
        expected_prop_map.insert("line-height".to_string(), Value::Keyword("normal".to_string()));
        expected_prop_map
    }

    #[test]
    fn test_merge_one_div_and_one_rule() {

        let target_element = create_element_node("div".to_string(), AttrMap::new(), vec![]);
        let body = create_element_node("body".to_string(), AttrMap::new(), vec![target_element.clone()]);
        let html = create_element_node("html".to_string(), AttrMap::new(), vec![body.clone()]);

        let target_stylesheet = Stylesheet {rules: vec![
            create_simple_selector_rule(vec![(Some("div"), None, vec![])], vec![
            ("margin", Value::Keyword("auto".to_string())), ("padding", Value::Length(4.0, Unit::Px))
            ])
        ]};
        let styled_html = style_tree(&html, &target_stylesheet);

        let mut expected_property_map = create_inherit_props_map_for_test();
        expected_property_map.insert("margin".to_string(), Value::Keyword("auto".to_string()));
        expected_property_map.insert("padding".to_string(), Value::Length(4.0, Unit::Px));

        let expected_styled_target_node = create_styled_node(&target_element, expected_property_map, vec![]);
        let expected_styled_body = create_styled_node(&body, create_inherit_props_map_for_test(), vec![expected_styled_target_node]);
        let expected_styled_html = create_styled_node(&html, create_inherit_props_map_for_test(), vec![expected_styled_body]);

        assert_eq!(styled_html, expected_styled_html);
    }

    #[test]
    fn test_merge_style_rule_by_id() {
        let id = "id1".to_string();
        let mut target_attr = AttrMap::new();
        target_attr.insert("id".to_string(), id.clone());
        let target_element = create_element_node("div".to_string(), target_attr, vec![]);
        let body = create_element_node("body".to_string(), AttrMap::new(), vec![target_element.clone()]);
        let html = create_element_node("html".to_string(), AttrMap::new(), vec![body.clone()]);

        let target_stylesheet = Stylesheet {rules: vec![
            create_simple_selector_rule(vec![(None, Some(id.clone().as_str()), vec![])], vec![
                ("margin", Value::Keyword("auto".to_string())), ("padding", Value::Length(4.0, Unit::Px))
            ])
        ]};

        let styled_html = style_tree(&html, &target_stylesheet);

        let mut expected_property_map = create_inherit_props_map_for_test();
        expected_property_map.insert("margin".to_string(), Value::Keyword("auto".to_string()));
        expected_property_map.insert("padding".to_string(), Value::Length(4.0, Unit::Px));

        let expected_styled_target_node = create_styled_node(&target_element, expected_property_map, vec![]);
        let expected_styled_body = create_styled_node(&body, create_inherit_props_map_for_test(), vec![expected_styled_target_node]);
        let expected_styled_html = create_styled_node(&html, create_inherit_props_map_for_test(), vec![expected_styled_body]);

        assert_eq!(styled_html, expected_styled_html);

    }

    #[test]
    fn test_merge_style_rule_by_class() {
        let class = "class1".to_string();
        let mut target_attr = AttrMap::new();
        target_attr.insert("class".to_string(), class.clone());
        let target_element = create_element_node("div".to_string(), target_attr, vec![]);
        let body = create_element_node("body".to_string(), AttrMap::new(), vec![target_element.clone()]);
        let html = create_element_node("html".to_string(), AttrMap::new(), vec![body.clone()]);

        let target_stylesheet = Stylesheet {rules: vec![
            create_simple_selector_rule(vec![(None, None, vec![class.as_str()])], vec![
                ("margin", Value::Keyword("auto".to_string())), ("padding", Value::Length(4.0, Unit::Px))
            ])
        ]};

        let styled_html = style_tree(&html, &target_stylesheet);

        let mut expected_property_map = create_inherit_props_map_for_test();
        expected_property_map.insert("margin".to_string(), Value::Keyword("auto".to_string()));
        expected_property_map.insert("padding".to_string(), Value::Length(4.0, Unit::Px));

        let expected_styled_target_node = create_styled_node(&target_element, expected_property_map, vec![]);
        let expected_styled_body = create_styled_node(&body, create_inherit_props_map_for_test(), vec![expected_styled_target_node]);
        let expected_styled_html = create_styled_node(&html, create_inherit_props_map_for_test(), vec![expected_styled_body]);

        assert_eq!(styled_html, expected_styled_html);

    }

    #[test]
    fn test_merge_nodes_including_text_node_and_style() {
        let text_node = create_text_node("sample");
        let target_element = create_element_node("div".to_string(), AttrMap::new(), vec![text_node.clone()]);
        let body = create_element_node("body".to_string(), AttrMap::new(), vec![target_element.clone()]);
        let html = create_element_node("html".to_string(), AttrMap::new(), vec![body.clone()]);

        let target_stylesheet = Stylesheet {rules: vec![
            create_simple_selector_rule(vec![(Some("div"), None, vec![])], vec![
                ("margin", Value::Keyword("auto".to_string())), ("padding", Value::Length(4.0, Unit::Px))
            ])
        ]};
        let styled_html = style_tree(&html, &target_stylesheet);

        let mut expected_property_map = create_inherit_props_map_for_test();
        expected_property_map.insert("margin".to_string(), Value::Keyword("auto".to_string()));
        expected_property_map.insert("padding".to_string(), Value::Length(4.0, Unit::Px));

        let expected_styled_text_node = create_styled_node(&text_node, PropertyMap::new(), vec![]);
        let expected_styled_target_node = create_styled_node(&target_element, expected_property_map, vec![expected_styled_text_node]);
        let expected_styled_body = create_styled_node(&body, create_inherit_props_map_for_test(), vec![expected_styled_target_node]);
        let expected_styled_html = create_styled_node(&html, create_inherit_props_map_for_test(), vec![expected_styled_body]);

        assert_eq!(styled_html, expected_styled_html);
    }

    #[test]
    fn test_merge_a_element_and_multi_rules() {
        let id = "id1".to_string();
        let mut attr = AttrMap::new();
        attr.insert("id".to_string(), id.clone());
        let target_element = create_element_node("div".to_string(), attr, vec![]);
        let body = create_element_node("body".to_string(), AttrMap::new(), vec![target_element.clone()]);
        let html = create_element_node("html".to_string(), AttrMap::new(), vec![body.clone()]);

        let target_stylesheet = Stylesheet {rules: vec![
            create_simple_selector_rule(vec![(Some("div"), None, vec![])], vec![
                ("margin", Value::Keyword("auto".to_string()))
            ]),
            create_simple_selector_rule(vec![(None, Some(id.as_str()),vec![])], vec![("padding", Value::Length(4.0, Unit::Px))])
        ]};
        let styled_html = style_tree(&html, &target_stylesheet);

        let mut expected_property_map = create_inherit_props_map_for_test();
        expected_property_map.insert("margin".to_string(), Value::Keyword("auto".to_string()));
        expected_property_map.insert("padding".to_string(), Value::Length(4.0, Unit::Px));

        let expected_styled_target_node = create_styled_node(&target_element, expected_property_map, vec![]);
        let expected_styled_body = create_styled_node(&body, create_inherit_props_map_for_test(), vec![expected_styled_target_node]);
        let expected_styled_html = create_styled_node(&html, create_inherit_props_map_for_test(), vec![expected_styled_body]);

        assert_eq!(styled_html, expected_styled_html);
    }

    #[test]
    fn test_merge_multi_elements_and_a_rule() {

        let target_element1 = create_element_node("div".to_string(), AttrMap::new(), vec![]);
        let target_element2 = create_element_node("div".to_string(), AttrMap::new(), vec![]);
        let body = create_element_node("body".to_string(), AttrMap::new(), vec![target_element1.clone(), target_element2.clone()]);
        let html = create_element_node("html".to_string(), AttrMap::new(), vec![body.clone()]);

        let target_stylesheet = Stylesheet {rules: vec![
            create_simple_selector_rule(vec![(Some("div"), None, vec![])], vec![
                ("margin", Value::Keyword("auto".to_string()))
            ])
        ]};
        let styled_html = style_tree(&html, &target_stylesheet);

        let mut expected_property_map = create_inherit_props_map_for_test();
        expected_property_map.insert("margin".to_string(), Value::Keyword("auto".to_string()));

        let expected_styled_target_node1 = create_styled_node(&target_element1, expected_property_map.clone(), vec![]);
        let expected_styled_target_node2 = create_styled_node(&target_element1, expected_property_map, vec![]);
        let expected_styled_body = create_styled_node(&body, create_inherit_props_map_for_test(), vec![expected_styled_target_node1, expected_styled_target_node2]);
        let expected_styled_html = create_styled_node(&html, create_inherit_props_map_for_test(), vec![expected_styled_body]);

        assert_eq!(styled_html, expected_styled_html);
    }


}