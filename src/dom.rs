use std::collections::HashMap;

    #[derive(Debug, PartialEq)]
    pub struct Node {
        children: Vec<Node>,
        node_type: NodeType
    }

    #[derive(Debug, PartialEq)]
    pub enum NodeType {
        Text(String),
        Element(ElementData)
    }

    #[derive(Debug, PartialEq)]
    pub struct ElementData {
        tag_name: String,
        attributes: AttrMap
    }

    pub type AttrMap = HashMap<String, String>;

    pub fn text(data: String) -> Node {
        Node {children: Vec::new(), node_type: NodeType::Text(data)}
    }

    pub fn elem(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
        Node {
            children,
            node_type: NodeType::Element(ElementData {
                tag_name: name,
                attributes: attrs
            })
        }
    }


