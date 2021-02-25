// 参考: https://limpet.net/mbrubeck/2014/09/08/toy-layout-engine-5-boxes.html

use crate::style::StyledNode;
use crate::layout::BoxType::{BlockNode, InlineNode, AnonymousBlock};
use crate::css::Value::{Keyword, Length};
use crate::css::Unit::Px;

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

enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock
}

struct LayoutBox<'a> {
    dimensions: Dimensions,
    box_type: BoxType<'a>,
    children: Vec<LayoutBox<'a>>,
}


impl LayoutBox {

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

// NOTE: StyledNodeをとりあえず全部LayoutBoxに変換する処理
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

    fn layout(&mut self, containing_block: &Dimensions) {
        match self.box_type {
            BlockNode(_) => self.layout_block(containing_block),
            InlineNode(_) => {},
            AnonymousBlock => {}
        }
    }

    fn layout_block(&mut self, containing_block: &Dimensions) {
        // widthを計算
        self.calculate_block_width(containing_block);

        // x,yを計算
        self.calculate_block_position(containing_block);

        // 子要素を再帰的に計算、加えてそこから現在の要素のheightを計算
        self.layout_block_children();

        // ユーザーがheightプロパティを指定していた場合のheightの値を計算
        self.calculate_block_height();
    }

    // NOTE: 対象の要素の横幅(width, border-right, padding-left, margin-left等を含んだもの)を決める
    fn calculate_block_width(&mut self, containing_block: &Dimensions) {
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

        let total = [&margin_left, &margin_right, &border_left, &border_right,
        &padding_left, &padding_right, &width].iter().map(|v| v.to_px()).sum();

        // NOTE: もし横幅が親要素よりデカかったらmargin-leftとmargin-rightでautoになってるものの値を0にする
        if width != auto && total > containing_block.content.width {
            if margin_left == auto {
                margin_left = Length(0.0, Px);
            }
            if margin_right == auto {
                margin_right = Length(0.0, Px);
            }
        }

        // 親要素とこの要素の横幅の違い(この値がマイナスだったらこの要素がoverflowしてる)
        let underflow = containing_block.content.width - total;

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

        let this_dimension = &mut self.dimensions;
        this_dimension.content.width = width.to_px();

        this_dimension.padding.left = padding_left.to_px();
        this_dimension.padding.right = padding_right.to_px();

        this_dimension.border.left = border_left.to_px();
        this_dimension.border.right = border_right.to_px();

        this_dimension.margin.left = margin_left.to_px();
        this_dimension.margin.right = margin_right.to_px();
    }

    // NOTE: 対象のページ上の位置を計算、つまりxとyを計算
    fn calculate_block_position(&mut self, containing_block: &Dimensions) {
        let style = self.get_style_node();
        let this_dimensions = &mut self.dimensions;

        let zero = Length(0.0, Px);

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
        let this_dimensions = &mut self.dimensions;
        for child in &mut self.children {
            child.layout(this_dimensions);
            // loopでこの要素のheightに子要素のmargin含めたheightを足していって最終的に正しいheightを算出する
            this_dimensions.content.height = this_dimensions.content.height + child.dimensions.margin_box().height;
        }
    }

    // NOTE: デフォルトでは子要素のheightの合計から対象要素のheightを算出するけど明示的にheightプロパティで指定されていた場合はその値を使う
    fn calculate_block_height(&mut self) {
        if let Some(Length(h, Px)) = self.get_style_node().value("height") {
            self.dimensions.content.height = h;
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

impl Dimensions {
    fn padding_box(self) -> Rect {
        self.content.expanded_by(self.padding)
    }

    fn border_box(self) -> Rect {
        self.padding_box().expanded_by(self.border)
    }

    // marginまで含めたx,y,width,heightの値を返す
    fn margin_box(self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
}

impl Rect {
    // 現在のRectのそれぞれのプロパティに対してedgeの値を足す
    fn expanded_by(self, edge: EdgeSize) -> Rect {
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
    
}