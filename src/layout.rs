// 参考: https://limpet.net/mbrubeck/2014/09/08/toy-layout-engine-5-boxes.html

use crate::style::{StyledNode, Display};
use crate::layout::BoxType::{BlockNode, InlineNode, AnonymousBlock};
use crate::css::Value::{Keyword, Length};
use crate::css::Unit::Px;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default,Clone, Debug, PartialEq)]
pub struct Dimensions {
    // document originに対するコンテンツエリアのポジション
    content: Rect,
    padding: EdgeSize,
    border: EdgeSize,
    margin: EdgeSize
}
#[derive(Default,Clone, Debug, PartialEq)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32
}

#[derive(Default,Clone, Debug, PartialEq)]
struct EdgeSize {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32
}

#[derive(Clone, Debug, PartialEq)]
enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock
}

#[derive(Clone, Debug, PartialEq, )]
pub struct LayoutBox<'a> {
    dimensions: Rc<RefCell<Dimensions>>,
    box_type: BoxType<'a>,
    children: Vec<LayoutBox<'a>>,
}


impl<'a> LayoutBox<'a> {

    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type,
            dimensions: Default::default(),
            children: Vec::new(),
        }
    }

    fn get_style_node(&self) -> &StyledNode {
        match self.box_type {
            BlockNode(node) | InlineNode(node) => node,
            AnonymousBlock => panic!("Anonymous block box has no style node")
        }
    }

}

pub fn layout_tree<'a>(node: &'a StyledNode<'a>, containing_block: Rc<RefCell<Dimensions>>) -> LayoutBox<'a>{
    Rc::clone(&containing_block).borrow_mut().content.height = 0.0;
    let mut root_box = build_layout_tree(node);
    root_box.layout(containing_block);
    root_box
}

// NOTE: StyledNodeをとりあえず全部LayoutBoxに変換する処理
fn build_layout_tree<'a>(style_node: &'a StyledNode<'a>) -> LayoutBox<'a> {
    let mut root = LayoutBox::new(match style_node.display() {
        Display::Block => BlockNode(style_node),
        Display::Inline => InlineNode(style_node),
        Display::None => panic!("Root node has display: none.")
    });

    for child in &style_node.children {
        match child.display() {
            Display::Block => root.children.push(build_layout_tree(child)),
            Display::Inline => root.get_inline_container().children.push(build_layout_tree(child)),
            Display::None => {}
        }
    }
    root
}

impl<'a> LayoutBox<'a> {

    fn layout(&mut self, containing_block: Rc<RefCell<Dimensions>>) {
        match self.box_type {
            BlockNode(_) => self.layout_block(containing_block),
            InlineNode(_) => {}, // FIXME:処理追加
            AnonymousBlock => {}
        }
    }

    fn layout_block(&mut self, containing_block: Rc<RefCell<Dimensions>>) {
        // widthを計算
        self.calculate_block_width(containing_block.clone());

        // x,yを計算
        self.calculate_block_position(containing_block);

        // 子要素を再帰的に計算、加えてそこから現在の要素のheightを計算
        self.layout_block_children();

        // ユーザーがheightプロパティを指定していた場合のheightの値を計算
        self.calculate_block_height();
    }

    // NOTE: 対象の要素の横幅(width, border-right, padding-left, margin-left等を含んだもの)を決める
    fn calculate_block_width(&mut self, containing_block: Rc<RefCell<Dimensions>>) {
        let style = self.get_style_node();

        let auto = Keyword("auto".to_string());
        let mut width = style.value("width").unwrap_or(auto.clone());

        let zero = Length(0.0, Px);

        let mut margin_left = style.lookup("margin-left", "margin", &zero);
        let mut margin_right = style.lookup("margin-right", "margin", &zero);

        let border_left = style.lookup("border-left-width", "border-width", &zero);
        let border_right = style.lookup("border-right-width", "border-width", &zero);

        let padding_left = style.lookup("padding-left", "padding", &zero);
        let padding_right = style.lookup("padding-right", "padding", &zero);

        let total: f32 = [&margin_left, &margin_right, &border_left, &border_right,
        &padding_left, &padding_right, &width].iter().map(|v| v.to_px()).sum();

        // NOTE: もし横幅が親要素よりデカかったらmargin-leftとmargin-rightでautoになってるものの値を0にする
        if width != auto && total > containing_block.borrow().content.width {
            if margin_left == auto {
                margin_left = Length(0.0, Px);
            }
            if margin_right == auto {
                margin_right = Length(0.0, Px);
            }
        }

        // 親要素とこの要素の横幅の違い(この値がマイナスだったらこの要素がoverflowしてる)
        let underflow = containing_block.borrow().content.width - total;

        match (width == auto, margin_left == auto, margin_right == auto) {
            // NOTE: width,margin_left,margin_rightが全て10pxみたいに固定値の場合margin_rightを調整する
            (false, false, false) => {
                margin_right = Length(margin_right.to_px() + underflow, Px);
            },
            // NOTE: margin-right, margin-leftのどちらかの値がautoだった場合そちらの方のプロパティでunderflowを調整する
            (false, false, true) => {margin_right = Length(underflow, Px);},
            (false, true, false) => {margin_left = Length(underflow, Px);},

            (true, _, _) => {
                // NOTE: widthがautoでmargin系プロパティがautoの場合marginはゼロになる
                if margin_left == auto {margin_left = Length(0.0, Px);}
                if margin_right == auto {margin_right = Length(0.0, Px);}

                // NOTE: widthが残っている幅を全てとるようにする
                if underflow >= 0.0 {
                    width = Length(underflow, Px);
                } else {
                    // NOTE: もし要素がoverflowしていた場合はwidthをマイナス値にすることができないのでmargin_rightをマイナス値にする
                    width = Length(0.0, Px);
                    margin_right = Length(margin_right.to_px() + underflow, Px);
                }

            },
            // NOTE: margin_left,margin_rightがどちらともautoの場合仲良く半分ずつoverflowを担当する、こうすると要素が真ん中にくる
            (false, true, true) => {
                margin_left = Length(underflow / 2.0, Px);
                margin_right = Length(underflow / 2.0, Px);
            }
        }

        let this_dimension = &mut self.dimensions.borrow_mut();
        this_dimension.content.width = width.to_px();

        this_dimension.padding.left = padding_left.to_px();
        this_dimension.padding.right = padding_right.to_px();

        this_dimension.border.left = border_left.to_px();
        this_dimension.border.right = border_right.to_px();

        this_dimension.margin.left = margin_left.to_px();
        this_dimension.margin.right = margin_right.to_px();
    }

    // NOTE: 対象のページ上の位置を計算、つまりxとyを計算
    // xとyは親要素のx,yとheight(yの場合)とmargin, padding, borderの値足した値
    fn calculate_block_position(&mut self, containing_block_ref: Rc<RefCell<Dimensions>>) {
        let style = self.get_style_node();
        let this_dimensions = &mut self.dimensions.borrow_mut();

        let zero = Length(0.0, Px);

        let containing_block = containing_block_ref.borrow();

        this_dimensions.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
        this_dimensions.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_px();

        this_dimensions.border.top = style.lookup("border-top-width", "border-width", &zero).to_px();
        this_dimensions.border.bottom = style.lookup("border-bottom-width", "border-width", &zero).to_px();

        this_dimensions.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
        this_dimensions.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_px();

        this_dimensions.content.x = containing_block.content.x + this_dimensions.margin.left + this_dimensions.border.left + this_dimensions.padding.left;
        this_dimensions.content.y = containing_block.content.height + containing_block.content.y + this_dimensions.margin.top + this_dimensions.border.top + this_dimensions.padding.top;
    }

    fn layout_block_children(&mut self) {
        for child in &mut self.children {
            child.layout(self.dimensions.clone());
            let this_dimensions = &mut self.dimensions.borrow_mut();
            // loopでこの要素のheightに子要素のmargin含めたheightを足していって最終的に正しいheightを算出する
            this_dimensions.content.height = this_dimensions.content.height + child.dimensions.borrow().margin_box().height;
        }
    }

    // NOTE: デフォルトでは子要素のheightの合計から対象要素のheightを算出するけど明示的にheightプロパティで指定されていた場合はその値を使う
    fn calculate_block_height(&mut self) {
        if let Some(Length(h, Px)) = self.get_style_node().value("height") {
            self.dimensions.borrow_mut().content.height = h;
        }
    }

    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            InlineNode(_) | AnonymousBlock => self,
            BlockNode(_) => {
                match self.children.last() {
                    Some(&LayoutBox {box_type: AnonymousBlock,..}) => {},
                    _ => self.children.push(LayoutBox::new(AnonymousBlock))
                }
                self.children.last_mut().unwrap()
            }
        }
    }
}

impl Dimensions {
    fn padding_box(&self) -> Rect {
        self.content.expanded_by(&self.padding)
    }

    fn border_box(&self) -> Rect {
        self.padding_box().expanded_by(&self.border)
    }

    // marginまで含めたx,y,width,heightの値を返す
    fn margin_box(&self) -> Rect {
        self.border_box().expanded_by(&self.margin)
    }
}

impl Rect {
    // 現在のRectのそれぞれのプロパティに対してedgeの値を足す
    fn expanded_by(&self, edge: &EdgeSize) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::style::{StyledNode, PropertyMap};
    use crate::dom::{Node, AttrMap, NodeType, ElementData};
    use crate::css::{Value, Unit};
    use super::{Dimensions};
    use crate::layout::{layout_tree, LayoutBox, Rect, BoxType, EdgeSize};
    use crate::layout::BoxType::AnonymousBlock;
    use std::cell::RefCell;
    use std::rc::Rc;

    // NOTE: テストしたいもの
    // margin, border, padding, width, height, x, y

    fn create_element_node(tag_name: String, attributes: AttrMap, children: Vec<Node>) -> Node {
        let this_element = NodeType::Element(ElementData {tag_name, attributes});
        Node {node_type: this_element, children}
    }

    fn create_styled_node<'a>(node: &'a Node, specified_values: PropertyMap, children: Vec<StyledNode<'a>>) -> StyledNode<'a> {
        StyledNode {node, specified_values, children}
    }

    fn create_viewport() -> Rc<RefCell<Dimensions>> {
        let mut viewport: Dimensions = Default::default();
        viewport.content.width = 800.0;
        viewport.content.height = 600.0;
        Rc::new(RefCell::new(viewport))
    }

    fn create_edge_size(left: Option<f32>, right: Option<f32>, top: Option<f32>, bottom: Option<f32>) -> EdgeSize {
        EdgeSize {
            left: left.unwrap_or(0.0),
            right: right.unwrap_or(0.0),
            top: top.unwrap_or(0.0),
            bottom: bottom.unwrap_or(0.0)
        }
    }

    fn create_anonymous_layout_block(children: Vec<LayoutBox>) -> LayoutBox {
        let dimension = Dimensions {
            content: Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0
            },
            margin: create_edge_size(None,None,None,None),
            border: create_edge_size(None,None,None,None),
            padding: create_edge_size(None,None,None,None),
        };
        LayoutBox {
            dimensions: Rc::new(RefCell::new(dimension)),
            box_type: AnonymousBlock,
            children
        }
    }

    #[test]
    fn test_layout_only_block_node_tree() {
        // <div> block {margin: 8.0, padding: 4.0, width: auto}
        //   <div></div> block {margin-left: 2.0, width: 100, height: 200}
        // </div>
        let child_element = create_element_node("div".to_string(), AttrMap::new(), vec![]);
        let parent_element = create_element_node("div".to_string(), AttrMap::new(), vec![child_element.clone()]);

        let mut child_property_map = PropertyMap::new();
        child_property_map.insert("display".to_string(), Value::Keyword("block".to_string()));
        child_property_map.insert("margin-left".to_string(), Value::Length(2.0, Unit::Px));
        child_property_map.insert("width".to_string(), Value::Length(100.0, Unit::Px));
        child_property_map.insert("height".to_string(), Value::Length(200.0, Unit::Px));

        let mut parent_property_map = PropertyMap::new();
        parent_property_map.insert("display".to_string(), Value::Keyword("block".to_string()));
        parent_property_map.insert("margin".to_string(), Value::Length(8.0, Unit::Px));
        parent_property_map.insert("padding".to_string(), Value::Length(4.0, Unit::Px));
        parent_property_map.insert("width".to_string(), Value::Keyword("auto".to_string()));

        let styled_child_node = create_styled_node(&child_element, child_property_map, vec![]);
        let styled_parent_node = create_styled_node(&parent_element, parent_property_map, vec![styled_child_node.clone()]);

        let viewport = create_viewport();
        let layout = layout_tree(&styled_parent_node, viewport);

        let expected_child_dimension = Rc::new(RefCell::new(Dimensions {
            content: Rect {
                x: 14.0,
                y: 12.0,
                width: 100.0,
                height: 200.0
            },
            margin: create_edge_size(Some(2.0), Some(674.0),None,None), // rightは親要素のwidth776pxからmargin_left2px,width100pxを引いて計算
            border: create_edge_size(None,None,None,None),
            padding: create_edge_size(None,None,None,None),
        }));

        let expected_parent_dimension = Rc::new(RefCell::new(Dimensions {
            content: Rect {
                x: 12.0,
                y: 12.0,
                width: 776.0, // 800 - 16 - 8
                height: 200.0
            },
            margin: create_edge_size(Some(8.0),Some(8.0),Some(8.0),Some(8.0)),
            border: create_edge_size(None,None,None,None),
            padding: create_edge_size(Some(4.0),Some(4.0),Some(4.0),Some(4.0)),
        }));

        let expected_child_layout_box = LayoutBox {
            dimensions: expected_child_dimension,
            box_type: BoxType::BlockNode(&styled_child_node),
            children: vec![]
        };
        // let anonymous_container = create_anonymous_layout_block(vec![expected_child_layout_box]);
        let expected_parent_layout_box = LayoutBox {
            dimensions: expected_parent_dimension,
            box_type: BoxType::BlockNode(&styled_parent_node),
            children: vec![expected_child_layout_box]
        };

        assert_eq!(layout, expected_parent_layout_box);

    }

}