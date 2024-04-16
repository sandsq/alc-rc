use array2d::{Array2D, Error as Array2DError};
use rand::prelude::*;
use std::error::Error;
use std::fmt;
use thiserror;

use crate::text_processor::keycode::Keycode::{self, *};
use super::key::{KeyValue, KeycodeKey, Randomizeable};
use super::LayoutPosition;


#[derive(Debug, PartialEq, thiserror::Error)]
pub enum KeyboardError {
	#[error("position ({0}, {1}) is marked as symmetric but its corresponding symmetric position ({2}, {3}) is not")]
	SymmetryError(usize, usize, usize, usize),
	#[error("expected {0} rows but tried to create {1} rows instead")]
	RowMismatchError(usize, usize),
	#[error("expected {0} cols but tried to create {1} cols instead")]
	ColMismatchError(usize, usize),
}
// impl fmt::Display for KeyboardError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 		match self {
// 			KeyboardError::SymmetryError(r1, c1, r2, c2) =>
// 					write!(f, "Position ({r1}, {c1}) is marked as symmetric but its corresponding symmetric position ({r2}, {c2}) is not."),
// 			KeyboardError::RowMismatchError(r1, r2) =>
// 					write!(f, "Expected {r1} rows but found {:?} rows.", r1),
// 			KeyboardError::ColMismatchError(c1, c2) =>
// 					write!(f, "Expected {c1} rows but found {:?} rows.", c2),
// 			KeyboardError::InvalidKeyFromString(s) =>
// 					write!(f, "{} cannot be parsed into a KeycodeKey.", s),
// 			_ => write!(f, "Oops, don't have this error yet.")
// 		}
//     }
// }


/// Layers are grids. For non-grid keyboard layouts, create the largest grid that fits and block unused cells with dummy keys. Works for anything implementing [KeyValue]
#[derive(Debug, PartialEq, Clone)]
pub struct Layer<const R: usize, const C: usize, K: KeyValue> {
	layer: Array2D<K>
}
impl<const R: usize, const C: usize, K: KeyValue + std::clone::Clone> Layer<R, C, K> {
	pub fn from_rows(elements: &[Vec<K>]) -> Result<Self, Array2DError> {
		let layer_array2d = Array2D::from_rows(elements)?;
		Ok(Layer::<R, C, K> { layer: layer_array2d })
	}
	// maybe just return Option like Array2D?
	pub fn get(&self, r: usize, c: usize) -> Result<K, Array2DError> {
		match self.layer.get(r, c) {
			Some(v) => Ok(v.clone()),
			None => Err(Array2DError::IndicesOutOfBounds(r, c)),
		}
	}
	pub fn get_mut(&mut self, r: usize, c: usize) -> Result<&mut K, Array2DError> {
		match self.layer.get_mut(r, c) {
			Some(v) => Ok(v),
			None => Err(Array2DError::IndicesOutOfBounds(r, c)),
		}
	}
	pub fn get_row_major(&self, index: usize) -> Result<K, Array2DError> {
		match self.layer.get_row_major(index) {
			Some(v) => Ok(v.clone()),
			None => Err(Array2DError::IndexOutOfBounds(index)),
		}
	}
	pub fn get_mut_row_major(&mut self, index: usize) -> Result<&mut K, Array2DError> {
		match self.layer.get_mut_row_major(index) {
			Some(v) => Ok(v),
			None => Err(Array2DError::IndexOutOfBounds(index)),
		}
	}
	pub fn set(&mut self, row: usize, col: usize, element: K) -> Result<(), Array2DError> {
		self.layer.set(row, col, element)
	}
	pub fn get_from_layout_position(&self, l: &LayoutPosition) -> 
			Result<K, Array2DError> {
		self.get(l.row_index, l.col_index)
	}
	pub fn num_rows(&self) -> usize {
		R
	}
	pub fn num_columns(&self) -> usize {
		C
	}
	/// Specifically, mirrored left-right
	pub fn symmetric_position(&self, l: LayoutPosition) -> LayoutPosition {
		let num_rows = self.num_rows();
		let num_cols = self.num_columns();
		let orig_row = l.row_index;
		let orig_col = l.col_index;
		let symm_col = (num_cols - 1) - orig_col;
		LayoutPosition { layer_index: l.layer_index, row_index: orig_row, col_index: symm_col }
	}
	
}
impl<const R: usize, const C: usize> Layer<R, C, KeycodeKey> {
	pub fn init_blank() -> Self {
		let default_key = KeycodeKey::from_keycode(_NO);
		let mut layer_array2d = Array2D::filled_with(default_key.clone(), R, C);
		Layer::<R, C, KeycodeKey> { layer: layer_array2d }
	}
	pub fn randomize(&mut self, rng: &mut impl Rng, valid_keycodes: Vec<Keycode>) -> Result<(), KeyboardError> {
		for i in 0..R {
			for j in 0..C {
				let key = self.get(i, j).unwrap();
				let lp = LayoutPosition::for_layer(i, j);
				if key.is_symmetric() {
					let symm_lp = self.symmetric_position(lp);
					let symm_key = self.get_from_layout_position(&symm_lp).unwrap();
					if !symm_key.is_symmetric() {
						return Err(KeyboardError::SymmetryError(i, j, symm_lp.row_index, symm_lp.col_index));
					} else {
						continue;
					}
				}
				if  !key.is_randomizeable() {
					continue;
				}
				if let Some(random_keycode) = valid_keycodes.choose(rng) {
					let replacement_key = KeycodeKey::from_keycode(*random_keycode);
					self.set(i, j, replacement_key);
				}
			}
		}
		Ok(())
	}
}
impl<const R: usize, const C: usize> TryFrom<&str> for Layer<R, C, KeycodeKey> {
	type Error = Box<dyn Error>;
	fn try_from(layer_string: &str) -> Result<Self, Self::Error> {
		let mut layer = Self::init_blank();
		let rows = rows_from_string(layer_string, R)?;
		// yes it's silly to collect an iterator and then re-iter it
		for (i, row) in rows.iter().enumerate() {
			let cols = cols_from_string(row, C)?;
			for (j, col) in cols.iter().enumerate() {
				let mut key = KeycodeKey::try_from(*col)?;
				layer.set(i, j, key);
			}
		}
		Ok(layer)
	}
}
impl<const R: usize, const C: usize> TryFrom<&str> for Layer<R, C, f32> {
	type Error = Box<dyn Error>;
	fn try_from(layer_string: &str) -> Result<Self, Self::Error> {
		let mut effort_layer = Array2D::filled_with(0.0, R, C);
		let rows = rows_from_string(layer_string, R)?;
		for (i, row) in rows.iter().enumerate() {
			let cols = cols_from_string(row, C)?;
			for (j, col) in cols.iter().enumerate() {
				let effort_value = col.parse::<f32>()?;
				effort_layer.set(i, j, effort_value);
			}
		}
		Ok(Layer{ layer: effort_layer })
	}
}


fn rows_from_string(input_s: &str, r: usize) -> Result<Vec<&str>, KeyboardError> {
	let rows: Vec<&str> = input_s.split("\n").filter(|s| s.trim().len() > 0).collect();
	if rows.len() != r {
		Err(KeyboardError::RowMismatchError(r, rows.len()))
	} else {
		Ok(rows)
	}
}
fn cols_from_string(input_s: &str, c: usize) -> Result<Vec<&str>, KeyboardError> {
	let cols: Vec<&str> = input_s.split_whitespace().collect();
	if cols.len() != c {
		return Err(KeyboardError::ColMismatchError(c, cols.len()));
	} else { 
		Ok(cols)
	}
}


impl<const R: usize, const C: usize> fmt::Display for Layer<R, C, KeycodeKey> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write_col_indexes(f, C);
		for (i, row) in self.layer.rows_iter().enumerate() {
			write!(f, "{}|", i);
			for element in row {
				write!(f, "{}", element);
				write!(f, " ");
			}
			writeln!(f);
		}
		write!(f, "")
    }
}
impl<const R: usize, const C: usize> fmt::Binary for Layer<R, C, KeycodeKey> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write_col_indexes(f, C);
		for (i, row) in self.layer.rows_iter().enumerate() {
			write!(f, "{}|", i);
			for element in row {
				write!(f, "{:b}", element);
				write!(f, " ");
			}
			writeln!(f);
		}
		write!(f, "")
    }
}
// there should be a smarter way to do this
impl<const R: usize, const C: usize> fmt::Display for Layer<R, C, f32> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write_col_indexes(f, C);
		for (i, row) in self.layer.rows_iter().enumerate() {
			write!(f, "{}|", i);
			for element in row {
				write!(f, "{:>4.2}", element);
				write!(f, " ");
			}
			writeln!(f);
		}
		write!(f, "")
    }
}

fn write_col_indexes(f: &mut fmt::Formatter, c: usize) -> () {
	write!(f, "  ");
	for k in 0..c {
		write!(f, "{:>3}", k);
		write!(f, " ");
	}
	// writeln!(f);
	// write!(f, "  ");
	// for k in 0..c {
	// 	write!(f, "{:>3}", "-");
	// 	write!(f, " ");
	// }
	writeln!(f);
}


#[cfg(test)]
mod tests {
	use super::*;

	// don't test things with square dimensions as doing so makes it easier for incorrect logic to still give the expected outcome
	#[test]
	fn test_keycode_key_layer() {
		let l = LayoutPosition::for_layer(0, 1);
		let key1: KeycodeKey = KeycodeKey::from_keycode(_A);
		let key2: KeycodeKey = KeycodeKey::from_keycode(_B);
		let key3: KeycodeKey = KeycodeKey::from_keycode(_C);
		let key4: KeycodeKey = KeycodeKey::from_keycode(_D);
		let key5: KeycodeKey = KeycodeKey::from_keycode(_E);
		let key1again = key1.clone();
		let vec_vec_layer: Vec<Vec<KeycodeKey>> = vec![vec![key1, key2, key3], vec![key5, key4, key1again]];
		let expected_layer: Layer::<2, 3, KeycodeKey> = Layer::<2, 3, KeycodeKey> { layer: Array2D::from_rows(&vec_vec_layer).unwrap() };
		let expected_layer_again = expected_layer.clone();
		fn from_rows_test(l: Vec<Vec<KeycodeKey>>, e: Layer<2, 3, KeycodeKey>) {
			assert_eq!(Layer::<2, 3, KeycodeKey>::from_rows(&l).unwrap(), e);
		}
		from_rows_test(vec_vec_layer, expected_layer);
		fn access_test(e: Layer<2, 3, KeycodeKey>, l: LayoutPosition, k: KeycodeKey) {
			assert_eq!(e.get_from_layout_position(&l).unwrap(), k);
		}
		access_test(expected_layer_again, l, KeycodeKey::from_keycode(_B));
	}

	#[test]
	fn test_float_layer() {
		let expected_layer = Layer::<1, 2, f32> { layer: Array2D::from_rows(&vec![vec![0.4, 0.5]]).unwrap() };
		assert_eq!(expected_layer.get_from_layout_position(&LayoutPosition::for_layer(0, 0)).unwrap(), 0.4);
	}

	#[test]
	fn test_init_random() {
		let mut rng = StdRng::seed_from_u64(0);
		let random_layer = Layer::<2, 3, KeycodeKey>::init_blank();
		assert_eq!(random_layer.get(0, 0).unwrap().value(), _NO);
	}

	#[test]
	fn test_symmetry() {
		let layer = Layer::<4, 6, KeycodeKey>::init_blank();
		let query_layout_pos = LayoutPosition { layer_index: 0, row_index: 2, col_index: 5 };
		let expected_layout_pos = LayoutPosition { layer_index: 0, row_index: 2, col_index: 0 };
		assert_eq!(layer.symmetric_position(query_layout_pos.clone()), expected_layout_pos.clone());
		assert_eq!(layer.symmetric_position(expected_layout_pos.clone()), query_layout_pos.clone());
	}

	#[test]
	fn test_randomize() {
		let mut rng = StdRng::seed_from_u64(0);
		let mut layer = Layer::<3, 2, KeycodeKey>::init_blank();
		layer.get_mut(0, 0).unwrap().set_is_symmetric(true);
		assert_eq!(layer.randomize(&mut rng, vec![_E]).unwrap_err(), KeyboardError::SymmetryError(0, 0, 0, 1));
		layer.get_mut(0, 1).unwrap().set_is_symmetric(true); // set the corresponding slot to be symmetric to continue

		layer.get_mut(1, 1).unwrap().set_is_moveable(false);
		layer.get_mut(2, 0).unwrap().set_value(_LS(1)); // there is no layer switch to be had but use it to test that _LS does not get randomized
		layer.randomize(&mut rng, vec![_E]);
		assert_eq!(layer.get(0, 0).unwrap().value(), _NO);
		assert_eq!(layer.get(0, 1).unwrap().value(), _NO);
		assert_eq!(layer.get(1, 1).unwrap().value(), _NO);
		assert_eq!(layer.get(1, 0).unwrap().value(), _E);
		assert_eq!(layer.get(2, 0).unwrap().value(), _LS(1));
	}

	#[test]
	fn test_displaying_things() {
		let mut rng = StdRng::seed_from_u64(0);
		let mut layer = Layer::<5, 6, KeycodeKey>::init_blank();
		layer.get_mut(0, 0).unwrap().set_value(_ENT);
		layer.get_mut(0, 0).unwrap().set_is_moveable(false);
		layer.randomize(&mut rng, vec![_A, _B, _C, _D, _E]);
		layer.get_mut(3, 5).unwrap().set_is_moveable(false);
		println!("{}", layer);
		println!("{:b}", layer);
	}

	#[test]
	fn test_from_string() {
		let layer_string = "
			A_11 B_10 C_11
			D_00 __01 LS1_10
		";
		let layer = Layer::<2, 3, KeycodeKey>::try_from(layer_string).unwrap();
		println!("layer from string\n{:b}", layer);
		println!("layer from string\n{}", layer);
		assert_eq!(layer.get(1, 2).unwrap(), KeycodeKey::from_keycode(_LS(1)));

		let effort_string = "
			0.5 1.0 1.5
			0.25 2.0 3.0
		";
		let effort_layer = Layer::<2, 3, f32>::try_from(effort_string).unwrap();
		println!("{}", effort_layer);
	}
}