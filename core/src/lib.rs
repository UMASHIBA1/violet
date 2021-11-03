pub mod dom;
pub mod html;
pub mod css;
pub mod style;

use style::StyledNode;

pub fn render_style_from_source<'a>(html: String, css: String) -> StyledNode<'a> {
    let root_node = html::parse(html);
    let stylesheet = css::parse(css);
    // FIXME
    let style_root = style::style_tree(&root_node.clone(), &stylesheet.clone());
    style_root.clone()
}
