use array2d::{Array2D, Error};
use delegate::delegate;

use crate::text_processor::keycode::Keycode::{self, *};
use super::key::KeycodeKey;

/// Layers are grids. For non-grid keyboard layouts, create the largest grid that fits and block unused cells with dummy keys.
#[derive(Debug, PartialEq)]
pub struct KeycodeLayer<const R: usize, const C: usize> {
	layer: Array2D<KeycodeKey>
}
impl<const R: usize, const C: usize> KeycodeLayer<R, C> {
	pub fn from_rows(elements: &[Vec<KeycodeKey>]) -> Result<Self, Error> {
		let layer_array2d = Array2D::from_rows(elements)?;
		Ok(KeycodeLayer::<R, C> { layer: layer_array2d })
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_from_rows() {
		let key1 = KeycodeKey::from_keycode(_A);
		let key2 = KeycodeKey::from_keycode(_B);
		let key3 = KeycodeKey::from_keycode(_C);
		let key4 = KeycodeKey::from_keycode(_D);
		let key5 = KeycodeKey::from_keycode(_E);
		let key1again = key1.clone();
		let vec_vec_layer = &vec![vec![key1, key2, key3], vec![key5, key4, key1again]];
		let expected_layer = KeycodeLayer::<2, 3> { layer: Array2D::from_rows(&vec_vec_layer).unwrap() };
		assert_eq!(KeycodeLayer::<2, 3>::from_rows(&vec_vec_layer).unwrap(), expected_layer);
	}
}