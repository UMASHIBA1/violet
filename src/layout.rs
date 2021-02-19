// 参考: https://limpet.net/mbrubeck/2014/09/08/toy-layout-engine-5-boxes.html

use crate::style::StyledNode;
use crate::layout::BoxType::{BlockNode, InlineNode, AnonymousBlock};

struct Dimensions {
    // document originに対するコンテンツエリアのポジション
    content: Rect,
    padding: EdgeSize,
    border: EdgeSize,
    margin: EdgeSize
}

struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32
}

struct EdgeSize {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32
}

struct LayoutBox<'a> {
    dimensions: Dimensions,
    box_type: BoxType<'a>,
    children: Vec<LayoutBox<'a>>,
}

enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock
}

fn build_layout_tree<'a>(style_node: &'a StyledNode<'a>) -> LayoutBox<'a> {
    let mut root = LayoutBox::new(match style_node.display() {
        Block => BlockNode(style_node),
        Inline => InlineNode(style_node),
        DisplayNone => panic!("Root node has display: none.")
    });

    for child in &style_node.children {
        match child.display() {
            Block => root.children.push(build_layout_tree(child)),
            Inline => root.get_inline_container().children.push(build_layout_tree(child)),
            DisplayNone => {}
        }
    }
    root
}

impl LayoutBox {
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type,
            dimensions: Default::default(),
            children: Vec::new(),
        }
    }

    fn get_inline_container(&mut self) -> &mut LayoutBox {
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

