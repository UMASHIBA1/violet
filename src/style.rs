// NOTE: https://limpet.net/mbrubeck/2014/08/23/toy-layout-engine-4-style.html

use std::collections::{HashMap, HashSet};
use crate::css::{Value, Selector, SimpleSelector, Specificity, Rule, Stylesheet};
use crate::dom::{Node, ElementData, NodeType};

pub type PropertyMap = HashMap<String, Value>;

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

