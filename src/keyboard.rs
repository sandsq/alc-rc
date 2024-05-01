use std::ops::Index;
use std::fmt;

pub mod key;
pub mod layer;
pub mod layout;
pub mod layout_presets;


/// Describes position in a keyboard layout (i.e., a collection of layers). For a single layer, layer_index is ignored.
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash, PartialOrd, Ord)]
pub struct LayoutPosition {
	pub layer_index: usize,
	pub row_index: usize,
	pub col_index: usize,
}
impl LayoutPosition {
	pub fn new(layer_index: usize, row_index: usize, col_index: usize) -> Self {
		LayoutPosition { layer_index, row_index, col_index }
	}
	pub fn from_tuple(t: (usize, usize, usize)) -> Self {
		LayoutPosition::new(t.0, t.1, t.2)
	}
}
impl fmt::Display for LayoutPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "(L{} R{} C{})", self.layer_index, self.row_index, self.col_index)
    }
}


#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct LayoutPositionSequence {
	sequence: Vec<LayoutPosition>
}
impl LayoutPositionSequence {
	pub fn push(&mut self, lp: LayoutPosition) {
		self.sequence.push(lp)
	}
	pub fn from_vector(lps: Vec<LayoutPosition>) -> Self {
		LayoutPositionSequence { sequence: lps }
	}
	pub fn from_tuples(lps: Vec<(usize, usize, usize)>) -> Self {
		let lps_vec = lps.into_iter().map(LayoutPosition::from_tuple).collect();
		LayoutPositionSequence { sequence: lps_vec }
	}
	pub fn append(&mut self, other: &mut Self) {
		self.sequence.append(&mut other.sequence)
	}
	pub fn last(&self) -> Option<&LayoutPosition> {
		self.sequence.last()
	}
	pub fn len(&self) -> usize {
		self.sequence.len()
	}

}
impl Index<usize> for LayoutPositionSequence {
	type Output = LayoutPosition;
	fn index(&self, index: usize) -> &Self::Output {
		self.sequence.index(index)
	}
}
impl IntoIterator for LayoutPositionSequence {
	type Item = LayoutPosition;
	type IntoIter = std::vec::IntoIter<Self::Item>;
	fn into_iter(self) -> Self::IntoIter {
		self.sequence.into_iter()
	}
}
impl fmt::Display for LayoutPositionSequence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "[")?;
		for (i, lp) in self.sequence.clone().iter().enumerate() {
			if i == self.sequence.len() - 1 {
				write!(f, "{}", lp)?;
			} else {
				write!(f, "{} -> ", lp)?;
			}
			
		}
		write!(f, "]")
    }
}

