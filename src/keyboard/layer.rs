use array2d::{Array2D, Error as Array2DError};
use std::fmt;
use std::ops::Index;
use std::str;
use std::collections::{HashSet, VecDeque};

use crate::alc_error::AlcError;
use crate::text_processor::keycode::Keycode::{self, *};
use super::key::{Finger, Hand, KeyValue, KeycodeKey, PhalanxKey, Randomizeable};
use super::LayoutPosition;

/// Layers are grids. For non-grid keyboard layouts, create the largest grid that fits and block unused cells with dummy keys. Works for anything implementing [KeyValue]
#[derive(Debug, PartialEq, Clone)]
pub struct Layer<const R: usize, const C: usize, K: KeyValue> {
	layer: Array2D<K>
}
impl<const R: usize, const C: usize, K: KeyValue + std::clone::Clone> Layer<R, C, K> {
	// pub fn from_rows(elements: &[Vec<K>]) -> Result<Self, Array2DError> {
	// 	let layer_array2d = Array2D::from_rows(elements)?;
	// 	Ok(Layer::<R, C, K> { layer: layer_array2d })
	// }
	// maybe just return Option like Array2D?
	// pub fn get(&self, r: usize, c: usize) -> Option<&K> {
	// 	self.layer.get(r, c)
	// }
	pub fn get_mut(&mut self, r: usize, c: usize) -> Option<&mut K> {
		self.layer.get_mut(r, c)
	}
	pub fn get_row_major(&self, index: usize) -> Option<&K> {
		self.layer.get_row_major(index)
	}
	pub fn get_mut_row_major(&mut self, index: usize) -> Option<&mut K> {
		self.layer.get_mut_row_major(index)
	}
	pub fn set(&mut self, row: usize, col: usize, element: K) -> Result<(), Array2DError> {
		self.layer.set(row, col, element)
	}
	// pub fn get_from_layout_position(&self, l: LayoutPosition) -> 
	// 		Option<&K> {
	// 	self.get(l.row_index, l.col_index)
	// }
	pub fn num_rows(&self) -> usize {
		R
	}
	pub fn num_columns(&self) -> usize {
		C
	}
	/// Specifically, mirrored left-right
	pub fn symmetric_position(&self, l: LayoutPosition) -> LayoutPosition {
		let num_cols = self.num_columns();
		let orig_row = l.row_index;
		let orig_col = l.col_index;
		let symm_col = (num_cols - 1) - orig_col;
		LayoutPosition { layer_index: l.layer_index, row_index: orig_row, col_index: symm_col }
	}

}

impl<const R: usize, const C: usize, T> Index<(usize, usize)> for Layer<R, C, T> where T: KeyValue{
	type Output = T;
	fn index(&self, index: (usize, usize)) -> &Self::Output {
		&self.layer[index]
	}
}
impl<const R: usize, const C: usize, T> Index<LayoutPosition> for Layer<R, C, T> where T: KeyValue{
	type Output = T;
	fn index(&self, index: LayoutPosition) -> &Self::Output {
		&self.layer[(index.row_index, index.col_index)]
	}
}


impl<const R: usize, const C: usize> Layer<R, C, KeycodeKey> {
	pub fn init_blank() -> Self {
		let default_key = KeycodeKey::default_from_keycode(_NO);
		let layer_array2d = Array2D::filled_with(default_key, R, C);
		Layer::<R, C, KeycodeKey> { layer: layer_array2d }
	}

	pub fn get_keycode_set(&self) -> HashSet<Keycode> {
		let mut existing_keycodes: HashSet<Keycode> = Default::default();
		for i in 0..R {
			for j in 0..C {
				existing_keycodes.insert(self[(i, j)].value());
			}
		}
		existing_keycodes
	}
	/// give layout access to this but not anything else to ensure valid_keycodes is already randomized
	pub (in super) fn randomize(&mut self, valid_keycodes_all: &VecDeque<Keycode>, valid_keycodes: &VecDeque<Keycode>, keycode_set: &HashSet<Keycode>) -> (VecDeque<Keycode>, bool) {
		
		let mut used_all_keycodes_flag = false;
		let mut valid_keycodes_to_draw_from = valid_keycodes.clone();
		// println!("keycodes to draw from {:?}", valid_keycodes_to_draw_from);
		for i in 0..R {
			for j in 0..C {
				let key = &self[(i, j)];
				if !key.is_randomizeable() || key.value() != _NO {
					continue;
				}
				if valid_keycodes_to_draw_from.is_empty() {
					valid_keycodes_to_draw_from = valid_keycodes_all.clone();
					used_all_keycodes_flag = true;
				}
				
				let mut random_keycode = valid_keycodes_to_draw_from.pop_front();
				match random_keycode {
					Some(_) => (),
					None => continue,
				};
				while keycode_set.contains(&random_keycode.unwrap()) {
					random_keycode = valid_keycodes_to_draw_from.pop_front();
					match random_keycode {
						Some(_) => (),
						None => break,
					};
				}
				match random_keycode {
					Some(v) => {
						let replacement_key = KeycodeKey::default_from_keycode(v);
						self.set(i, j, replacement_key).unwrap(); // should always work
					},
					None => ()
				}
			}
		}
		(valid_keycodes_to_draw_from, used_all_keycodes_flag)
	}
}




impl<const R: usize, const C: usize> TryFrom<&str> for Layer<R, C, KeycodeKey> {
	type Error = AlcError;
	fn try_from(layer_string: &str) -> Result<Self, Self::Error> {
		let mut layer = Self::init_blank();
		let rows = rows_from_string(layer_string, R)?;
		// yes it's silly to collect an iterator and then re-iter it
		for (i, row) in rows.iter().enumerate() {
			let cols = cols_from_string(row, C)?;
			for (j, col) in cols.iter().enumerate() {
				let key = KeycodeKey::try_from(*col)?;
				// println!("reminder: check for symmetry here");
				layer.set(i, j, key).unwrap();
			}
		}
		Ok(layer)
	}
}
impl<const R: usize, const C: usize> TryFrom<&str> for Layer<R, C, f64> {
	type Error = AlcError;
	fn try_from(layer_string: &str) -> Result<Self, Self::Error> {
		let mut effort_layer = Array2D::filled_with(0.0, R, C);
		let rows = rows_from_string(layer_string, R)?;
		for (i, row) in rows.iter().enumerate() {
			let cols = cols_from_string(row, C)?;
			for (j, col) in cols.iter().enumerate() {
				let effort_value = col.parse::<f64>()?;
				effort_layer.set(i, j, effort_value).unwrap();
			}
		}
		Ok(Layer{ layer: effort_layer })
	}
}

fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

impl<const R: usize, const C: usize> TryFrom<&str> for Layer<R, C, PhalanxKey> {
	type Error = AlcError;
	fn try_from(layer_string: &str) -> Result<Self, Self::Error> {
		let mut phalanx_layer = Array2D::filled_with(PhalanxKey::default(), R, C);
		let rows = rows_from_string(layer_string, R)?;
		for (i, row) in rows.iter().enumerate() {
			let cols = cols_from_string(row, C)?;
			for (j, col) in cols.iter().enumerate() {
				let mut phalanx = col.split(':');
				let hand_str = match phalanx.next() {
					Some(v) => match v {
						"l" | "L" => "Left",
						"r" | "R" => "Right",
						_ => v,
					},
					None => return Err(AlcError::InvalidPhalanxError(String::from(*col))),
				};
				let hand_str = uppercase_first_letter(hand_str);
				let finger_str = match phalanx.next() {
					Some(v) => match v {
						"t" | "T" => "Thumb",
						"i" | "I" => "Index",
						"m" | "M" => "Middle",
						"r" | "R" => "Ring",
						"p" | "P" => "Pinkie",
						"j" | "J" => "Joint",
						_ => v,
 					},
					None => return Err(AlcError::InvalidPhalanxError(String::from(*col))),
				};
				let finger_str = uppercase_first_letter(finger_str);
				let hand = Hand::try_from(&hand_str[..])?;
				let finger = Finger::try_from(&finger_str[..])?;
				let phalanx_key = PhalanxKey::new(hand, finger);
				phalanx_layer[(i, j)] = phalanx_key;
			}
		}
		Ok(Layer{ layer: phalanx_layer })
	}
}

fn rows_from_string(input_s: &str, r: usize) -> Result<Vec<&str>, AlcError> {
	let mut rows = input_s.split('\n').filter(|s| !s.trim().is_empty());
	let rows_vec: Vec<&str> = rows.clone().collect();
	let mut rows_vec_len = rows_vec.len();
	if rows_vec_len == r + 1 {
		// this is for convenience: layers are outputted with row and column indexes and it would be nice if we could just copy past those outputs as valid strings to create layers from
		// if first row is a series of numbers, then it is a column index row
		let first_row = rows.next().unwrap();
		rows_vec_len -= 1;
		let mut first_row_chars = first_row.chars();
		let previous_char = first_row_chars.next().unwrap();
		for _c_ind in 0..first_row.len() - 1 {
			let current_char = first_row_chars.next().unwrap();
			if (previous_char.is_ascii_digit() && current_char.is_ascii_digit()) && (previous_char.to_digit(10).unwrap() + 1 != current_char.to_digit(10).unwrap()) {
				return Err(AlcError::FromStringHeaderError(String::from(first_row)));
			}
			
		}
	}
	if rows_vec_len != r {
		return Err(AlcError::RowMismatchError(r, rows_vec_len));
	}
	else {
		Ok(rows.collect())
	}
}
fn cols_from_string(input_s: &str, c: usize) -> Result<Vec<&str>, AlcError> {
	// see note for rows_from_string
	// | is used as a separator between the row index and the row
	let cols = if input_s.contains('|') {
		let cols_no_index: Vec<&str> = input_s.split('|').collect();
		cols_no_index[1].split_whitespace()
	} else {
		input_s.split_whitespace()
	};
	// let mut cols = input_s.split_whitespace();
	let cols_vec: Vec<&str> = cols.clone().collect();
	if cols_vec.len() != c {
		return Err(AlcError::ColMismatchError(c, cols_vec.len(), input_s.to_string()));
	} else { 
		Ok(cols_vec)
	}
}


impl<const R: usize, const C: usize> fmt::Display for Layer<R, C, KeycodeKey> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write_col_indexes(f, C, false)?;
		for (i, row) in self.layer.rows_iter().enumerate() {
			write!(f, "{}|", i)?;
			for element in row {
				write!(f, "{}", element)?;
				write!(f, " ")?;
			}
			writeln!(f)?;
		}
		Ok(())
    }
}
impl<const R: usize, const C: usize> fmt::Binary for Layer<R, C, KeycodeKey> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write_col_indexes(f, C, true)?;
		for (i, row) in self.layer.rows_iter().enumerate() {
			write!(f, "{}|", i)?;
			for element in row {
				write!(f, "{:b}", element)?;
				write!(f, " ")?;
			}
			writeln!(f)?;
		}
		Ok(())
    }
}
// there should be a smarter way to do this
impl<const R: usize, const C: usize> fmt::Display for Layer<R, C, f64> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write_col_indexes(f, C, false)?;
		for (i, row) in self.layer.rows_iter().enumerate() {
			write!(f, "{}|", i)?;
			for element in row {
				write!(f, "{:>4.1}", element)?;
				write!(f, " ")?;
			}
			writeln!(f)?;
		}
		Ok(())
    }
}
impl<const R: usize, const C: usize> fmt::Display for Layer<R, C, PhalanxKey> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write_col_indexes(f, C, false)?;
		for (i, row) in self.layer.rows_iter().enumerate() {
			write!(f, "{}|", i)?;
			for element in row {
				write!(f, "{:>4}", element)?;
				write!(f, " ")?;
			}
			writeln!(f)?;
		}
		Ok(())
    }
}


/// Remember 4 is a magic number for keycodes. The moveability and symmetric flags add 3 characters (_00)
fn write_col_indexes(f: &mut fmt::Formatter, c: usize, is_binary: bool) -> fmt::Result {
	write!(f, "  ")?;
	for k in 0..c {
		if is_binary {
			write!(f, "{:>7}", k)?;
		} else {
			write!(f, "{:>4}", k)?;
		}
		
		write!(f, " ")?;
	}
	// writeln!(f);
	// write!(f, "  ");
	// for k in 0..c {
	// 	write!(f, "{:>3}", "-");
	// 	write!(f, " ");
	// }
	writeln!(f)?;
	Ok(())
}


#[cfg(test)]
mod tests {
	use super::*;
	use Hand::*;
	use Finger::*;

	// don't test things with square dimensions as doing so makes it easier for incorrect logic to still give the expected outcome
	#[test]
	fn test_keycode_key_layer() {
		let l = LayoutPosition::new(0, 0, 1);
		let key1: KeycodeKey = KeycodeKey::default_from_keycode(_A);
		let key2: KeycodeKey = KeycodeKey::default_from_keycode(_B);
		let key3: KeycodeKey = KeycodeKey::default_from_keycode(_C);
		let key4: KeycodeKey = KeycodeKey::default_from_keycode(_D);
		let key5: KeycodeKey = KeycodeKey::default_from_keycode(_E);
		let key1again = key1.clone();
		let vec_vec_layer: Vec<Vec<KeycodeKey>> = vec![vec![key1, key2, key3], vec![key5, key4, key1again]];
		let expected_layer: Layer::<2, 3, KeycodeKey> = Layer::<2, 3, KeycodeKey> { layer: Array2D::from_rows(&vec_vec_layer).unwrap() };
		let expected_layer_again = expected_layer.clone();
		// fn from_rows_test(l: Vec<Vec<KeycodeKey>>, e: Layer<2, 3, KeycodeKey>) {
		// 	assert_eq!(Layer::<2, 3, KeycodeKey>::from_rows(&l).unwrap(), e);
		// }
		// from_rows_test(vec_vec_layer, expected_layer);
		fn access_test(e: Layer<2, 3, KeycodeKey>, l: LayoutPosition, k: KeycodeKey) {
			assert_eq!(e[l], k);
		}
		access_test(expected_layer_again, l, KeycodeKey::default_from_keycode(_B));
	}

	#[test]
	fn test_float_layer() {
		let expected_layer = Layer::<1, 2, f64> { layer: Array2D::from_rows(&vec![vec![0.4, 0.5]]).unwrap() };
		assert_eq!(expected_layer[LayoutPosition::new(0, 0, 0)], 0.4);
	}

	#[test]
	fn test_init_random() {
		let random_layer = Layer::<2, 3, KeycodeKey>::init_blank();
		assert_eq!(random_layer[(0, 0)].value(), _NO);
	}

	#[test]
	fn test_symmetry() {
		let layer = Layer::<4, 6, KeycodeKey>::init_blank();
		let query_layout_pos = LayoutPosition { layer_index: 0, row_index: 2, col_index: 5 };
		let expected_layout_pos = LayoutPosition { layer_index: 0, row_index: 2, col_index: 0 };
		assert_eq!(layer.symmetric_position(query_layout_pos), expected_layout_pos.clone());
		assert_eq!(layer.symmetric_position(expected_layout_pos), query_layout_pos.clone());
	}

	#[test]
	fn test_randomize() {
		let mut layer = Layer::<3, 2, KeycodeKey>::init_blank();
		layer.get_mut(0, 0).unwrap().set_is_symmetric(true);
		layer.get_mut(0, 1).unwrap().set_is_symmetric(true); // set the corresponding slot to be symmetric to continue

		layer.get_mut(1, 1).unwrap().set_is_moveable(false);
		layer.get_mut(2, 0).unwrap().set_value(_LS(1)); // there is no layer switch to be had but use it to test that _LS does not get randomized
		layer.randomize(&VecDeque::from(vec![_E, _E, _E]), &VecDeque::from(vec![_E, _E, _E]), &HashSet::default());
		// println!("layer\n{}", layer);
		assert_eq!(layer[(0, 0)].value(), _NO);
		assert_eq!(layer[(0, 1)].value(), _NO);
		assert_eq!(layer[(1, 1)].value(), _NO);
		assert_eq!(layer[(1, 0)].value(), _E);
		assert_eq!(layer[(2, 0)].value(), _LS(1));

		let layer_string = "
			A_11 B_10 C_11
			D_00 __10 LS1_10
		";
		let mut layer = Layer::<2, 3, KeycodeKey>::try_from(layer_string).unwrap();
		layer.randomize(&VecDeque::from(vec![_H]), &VecDeque::from(vec![_H]), &HashSet::default());
		assert_eq!(layer[(0, 1)].value(), _B);
		assert_eq!(layer[(1, 1)].value(), _H);
	}

	#[test]
	fn test_displaying_things() {
		let mut layer = Layer::<5, 6, KeycodeKey>::init_blank();
		layer.get_mut(0, 0).unwrap().set_value(_ENT);
		layer.get_mut(0, 0).unwrap().set_is_moveable(false);
		layer.randomize(&VecDeque::from(vec![_E]), &VecDeque::from(vec![_A, _B, _C, _D, _E]), &HashSet::default());
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
		assert_eq!(layer[(1, 2)], KeycodeKey::default_from_keycode(_LS(1)));

		let layer_string_with_indexes = "
			0       1       2 
			0| LS1_10  LS2_10  LS3_10 
			1| LS4_10    E_10    D_00 
		";
		let layer_from_string_with_indexes = Layer::<2, 3, KeycodeKey>::try_from(layer_string_with_indexes).unwrap();
		assert_eq!(layer_from_string_with_indexes[(1, 2)].value(), _D);
		println!("layer from string that had indexes\n{}", layer_from_string_with_indexes);

		let effort_string = "
			0.5 1.0 1.5
			0.25 2.0 3.0
		";
		let effort_layer = Layer::<2, 3, f64>::try_from(effort_string).unwrap();
		println!("effort layer\n{}", effort_layer);
	}

	#[test]
	fn test_phalanx_from_string() {
		let test_str = "
			left:middle left:index right:index right:ring right:joint
		";
		let phalanx_layer = Layer::<1, 5, PhalanxKey>::try_from(test_str).unwrap();
		assert_eq!(phalanx_layer[(0, 0)], PhalanxKey::new(Left, Middle));
		assert_eq!(phalanx_layer[(0, 1)], PhalanxKey::new(Left, Index));
		assert_eq!(phalanx_layer[(0, 2)], PhalanxKey::new(Right, Index));
		assert_eq!(phalanx_layer[(0, 3)], PhalanxKey::new(Right, Ring));
		assert_eq!(phalanx_layer[(0, 4)], PhalanxKey::new(Right, Joint));

		let test_str = "
			L:M  L:I  R:I  R:R R:J
		";
		let phalanx_layer = Layer::<1, 5, PhalanxKey>::try_from(test_str).unwrap();
		assert_eq!(phalanx_layer[(0, 0)], PhalanxKey::new(Left, Middle));
		assert_eq!(phalanx_layer[(0, 1)], PhalanxKey::new(Left, Index));
		assert_eq!(phalanx_layer[(0, 2)], PhalanxKey::new(Right, Index));
		assert_eq!(phalanx_layer[(0, 3)], PhalanxKey::new(Right, Ring));
		assert_eq!(phalanx_layer[(0, 4)], PhalanxKey::new(Right, Joint));
		println!("{}", phalanx_layer);
	}

}