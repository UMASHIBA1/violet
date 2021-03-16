// 参考: https://limpet.net/mbrubeck/2014/11/05/toy-layout-engine-7-painting.html
use crate::css::{Color, Value};
use crate::layout::{Rect, BoxType, LayoutBox};
use std::io::{repeat, Read};

type DisplayList = Vec<DisplayCommand>;

enum DisplayCommand {
    SolidColor(Color, Rect)
}

pub struct Canvas {
    pub pixels: Vec<Color>,
    pub width: usize,
    pub height: usize
}

pub fn paint(layout_root: &LayoutBox, bounds: Rect) -> Canvas {
    let display_list = build_display_list(layout_root);
    println!("paint: {:?}", bounds.height);
    let mut canvas = Canvas::new(bounds.width as usize, bounds.height as usize);
    for item in display_list {
        canvas.paint_item(&item);
    }
    canvas
}

fn build_display_list(layout_root: &LayoutBox) -> DisplayList {
    let mut list = DisplayList::new();
    render_layout_box(&mut list, layout_root);
    list
}

fn render_layout_box(list: &mut DisplayList, layout_box: &LayoutBox) {
    // NOTE:
    render_background(list, layout_box);
    render_borders(list, layout_box);
    // TODO: render text

    for child in &layout_box.children {
        render_layout_box(list, child);
    }
}

fn render_background(list: &mut DisplayList, layout_box: &LayoutBox) {
    get_color(layout_box, "background").map(|color|
        list.push(DisplayCommand::SolidColor(color, layout_box.dimensions.borrow().border_box()))
    );
}

// NOTE: LayoutBoxが持っている色のプロパティを取得
fn get_color(layout_box: &LayoutBox, name: &str) -> Option<Color> {
    match layout_box.box_type {
        BoxType::BlockNode(style) | BoxType::InlineNode(style) => match style.value(name) {
            Some(Value::ColorValue(color)) => Some(color),
            _ => None
        },
        BoxType::AnonymousBlock => None
    }
}

fn render_borders(list: &mut DisplayList, layout_box: &LayoutBox) {
    let color = match get_color(layout_box, "border-color") {
        Some(color) => color,
        _ => return
    };

    let this_dimension = &layout_box.dimensions;
    let border_box = &this_dimension.clone().borrow().border_box();
    let border = &this_dimension.borrow().border;

    // left border
    list.push(DisplayCommand::SolidColor(color.clone(), Rect {
        x: border_box.x,
        y: border_box.y,
        width: border.left.clone(),
        height: border_box.height
    }));

    // right border
    list.push(DisplayCommand::SolidColor(color.clone(), Rect {
        x: border_box.x + border_box.width - border.right.clone(),
        y: border_box.y,
        width: border.right.clone(),
        height: border_box.height
    }));

    // top border
    list.push(DisplayCommand::SolidColor(color, Rect {
        x: border_box.x,
        y: border_box.y + border_box.height - border.bottom.clone(),
        width: border_box.width,
        height: border.bottom.clone()
    }));

}

impl Canvas {
    fn new(width: usize, height: usize) -> Canvas {
        let white = Color {r: 255, g: 255, b: 255, a: 255};
        Canvas {
            pixels: vec![white; width * height],
            width,
            height
        }
    }

    fn paint_item(&mut self, item: &DisplayCommand) {

        fn clamp(target: f32, min: f32, max: f32) -> f32 {
            if target < min {
                min
            } else if target > max {
                max
            } else {
                target
            }
        }

        match item {
            DisplayCommand::SolidColor(color, rect) => {
                let x0 = clamp(rect.x, 0.0, self.width as f32) as usize;
                let y0 = clamp(rect.y, 0.0, self.height as f32) as usize;
                let x1 = clamp(rect.x + rect.width, 0.0, self.width as f32) as usize;
                let y1 = clamp(rect.y + rect.height, 0.0, self.height as f32) as usize;

                for y in y0..y1 {
                    for x in x0 .. x1 {
                        self.pixels[x + y * self.width] = color.clone();
                    }
                }
            }
        }
    }
}


// NOTE: 自分なりに処理の手順メモ
// 目標: 最終的にCanvasオブジェクト内にwindowのwidthとheightの大きさ
// に対応した1pixelごとの色のリストを作成したい
// 手順
// 1. DisplayCommandを作成することでx,y,width,height,colorを一つのオブジェクトにまとめDisplayListに追加する
// 2. 1.をLayoutBoxのchildrenに対して繰り返す(childrenでもparentの入ってるDisplayListにpushする)
// 3. DisplayListの各要素に対してループを回してwidth,height,x,yから各ピクセルのcolorを計算しCanvasのpixelsに追加

