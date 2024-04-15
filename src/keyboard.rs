pub mod key;
pub mod layer;
pub mod layout;

/// Describes position in a keyboard layout (i.e., a collection of layers). For a single layer, layer_index is ignored.
#[derive(Debug, PartialEq, Clone)]
pub struct LayoutPosition {
	layer_index: usize,
	row_index: usize,
	col_index: usize,
}
impl LayoutPosition {
	pub fn for_layer(r: usize, c: usize) -> LayoutPosition {
		LayoutPosition { layer_index: 0, row_index: r, col_index: c }
	}
	pub fn for_layout(l: usize, r: usize, c: usize) -> LayoutPosition {
		LayoutPosition { layer_index: l, row_index: r, col_index: c }
	}
}