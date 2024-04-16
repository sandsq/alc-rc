use std::ops::Index;
use std::fmt;

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
impl fmt::Display for LayoutPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "L{}: (R{}, C{})", self.layer_index, self.row_index, self.col_index)
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct LayoutPositionSequence {
	sequence: Vec<LayoutPosition>
}
impl LayoutPositionSequence {
	pub fn push(&mut self, lp: LayoutPosition) {
		self.sequence.push(lp)
	}
	pub fn from(lps: Vec<LayoutPosition>) -> Self {
		LayoutPositionSequence { sequence: lps }
	}
}
impl Index<usize> for LayoutPositionSequence {
	type Output = LayoutPosition;
	fn index(&self, index: usize) -> &Self::Output {
		self.sequence.index(index)
	}
}
impl fmt::Display for LayoutPositionSequence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for lp in self.sequence.clone() {
			write!(f, "{} -> ", lp);
		}
		write!(f, "")
    }
}