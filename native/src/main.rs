use core::render_style_from_source;
use crate::layout::layout_tree;
use std::fs::File;
use std::io::{Read, BufWriter};
use std::rc::Rc;
use std::cell::RefCell;

mod layout;

fn main() {
    let mut opts = getopts::Options::new();
    opts.optopt("h", "html", "HTML document", "FILENAME");
    opts.optopt("c", "css", "CSS stylesheet", "FILENAME");
    opts.optopt("o", "output", "Output file", "FILENAME");

    let matches = opts.parse(std::env::args().skip(1)).unwrap();
    let str_arg = |flag: &str, default: &str| -> String {
        matches.opt_str(flag).unwrap_or(default.to_string())
    };

    let html = read_source(str_arg("h", "examples/test.html"));
    let css = read_source(str_arg("c", "examples/test.css"));

    let mut viewport = Rc::new(RefCell::new(Dimensions::default()));
    viewport.clone().borrow_mut().content.width = 800.0;
    viewport.clone().borrow_mut().content.height = 600.0;

    let styled_node = render_style_from_source(html, css);
    let layouted_tree = layout_tree(&styled_node, viewport.clone());

    let filename = str_arg("o", "output.png");
    // let mut file = BufWriter::new(File::create(&filename).unwrap());
    viewport.clone().borrow_mut().content.height = 600.0;

    let canvas = painting::paint(&layouted_tree, viewport.borrow().content.clone());
    let (width, height) = (canvas.width as u32, canvas.height as u32);
    println!("{:?}", canvas.height);
    println!("{:?}, {:?}", width, height);
    let mut imgbuf = image::ImageBuffer::new(width, height);
    // let imgbuf2 = image::ImageBuffer::from_fn(width, height, |x,y|  {
    //     let color = canvas.pixels.get((y * width + x) as usize).unwrap();
    //     image::Rgb([color.r, color.g, color.b])
    // });
    for (x,y,pixel) in imgbuf.enumerate_pixels_mut() {
        let color = canvas.pixels.get((y * width + x) as usize).unwrap();
        // println!("{:?}", color);
        *pixel = image::Rgb([color.r, color.g, color.b]);
    }
    let ok = imgbuf.save_with_format("output.png", image::ImageFormat::Png).is_ok();
    if ok {
        println!("success");
    } else {
        println!("failed");
    }


    // imgbuf.save_with_format("output.png", image::ImageFormat::Png);
    // imgbuf.save("output.png");

    // let ok = {
    //     let content = viewport.clone().borrow().content.clone();
    //     let canvas = painting::paint(&layout_root, content);
    //     let (w,h) = (canvas.width as u32, canvas.height as u32);
    //     let img = image::ImageBuffer::from_fn(w,h, move|x,y| {
    //         let color = canvas.pixels.get((y * w + x) as usize).unwrap();
    //         image::Pixels::from_channels(color.r,color.g,color.b,color.a)
    //     });
    //     img.save(filename).is_ok()
    // };
    //
    // if ok {
    //     println!("Saved output as {}", filename);
    // } else {
    //     println!("Error saving output as {}", filename);
    // }
}

fn read_source(filename: String) -> String {
    let mut str = String::new();
    File::open(filename).unwrap().read_to_string(&mut str).unwrap();
    str
}