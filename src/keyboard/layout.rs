use array2d::{Array2D, Error as Array2DError};
use std::ops::Index;
use rand::prelude::*;
use std::error::Error;
use std::fmt::{self, Formatter};
use std::collections::HashMap;
use thiserror;
use regex;
use std::ptr;

use crate::alc_error::AlcError;
use crate::text_processor::keycode::Keycode::{self, *};
use super::key::{KeyValue, KeycodeKey, PhysicalKey};
use super::layer::Layer;
use super::{LayoutPosition, LayoutPositionSequence};

type KeycodePositionMap = HashMap<Keycode, Vec<LayoutPositionSequence>>;


/// A keyboard layout is a collection of layers of KeycodeKeys, plus additional info specifying how to navigate the layout, etc. (fill in later)
/// Layouts with multiple layers must have a way to access every layer.
/// For now, the only way to change layers is via a layer switch key. _LS(2) means that key switches to layer 2
#[derive(Debug, PartialEq, Clone)]
pub struct Layout<const R: usize, const C: usize> {
	layers: Vec<Layer<R, C, KeycodeKey>>,
	keycodes_to_positions: KeycodePositionMap,
}
impl<const R: usize, const C: usize> Layout<R, C> {
	pub fn get(&self, layer_index: usize, row_index: usize, col_index: usize) -> Result<KeycodeKey, Array2DError> {
		// the first get isn't an Array2DError since it's on a vector, but deal with that later.
		self.layers.get(layer_index).unwrap().get(row_index, col_index)
	}
	pub fn get_from_layout_position(&self, lp: &LayoutPosition) -> Result<KeycodeKey, Array2DError> {
		// the first get isn't an Array2DError since it's on a vector, but deal with that later.
		self.layers.get(lp.layer_index).unwrap().get(lp.row_index, lp.col_index)
	}
	pub fn get_mut(&mut self, layer_index: usize, row_index: usize, col_index: usize) -> Result<&mut KeycodeKey, Array2DError> {
		// the first get_mut isn't an Array2DError since it's on a vector, but deal with that later.
		self.layers.get_mut(layer_index).unwrap().get_mut(row_index, col_index)
	}
	pub fn get_mut_from_layout_position(&mut self, lp: &LayoutPosition) -> Result<&mut KeycodeKey, Array2DError> {
		// the first get_mut isn't an Array2DError since it's on a vector, but deal with that later.
		self.layers.get_mut(lp.layer_index).unwrap().get_mut(lp.row_index, lp.col_index)
	}
	pub fn symmetric_position(&self, lp: &LayoutPosition) -> LayoutPosition {
		self.layers.get(0).unwrap().symmetric_position(&lp)
	}
	pub fn init_blank(num_layers: usize) -> Self {
		let mut layers: Vec<Layer<R, C, KeycodeKey>> = vec![];
		for i in 0..num_layers {
			let mut layer = Layer::<R, C, KeycodeKey>::init_blank();
			layers.push(layer);
		}
		for j in 0..num_layers - 1 {
			layers[0].get_mut_row_major(j).unwrap().set_value(_LS(j + 1));
			layers[j + 1].get_mut_row_major(j).unwrap().set_value(_LS(j + 1));
		}
		let keycodes_to_positions = keycode_position_mapping_from_layout::<R, C>(layers.clone()).unwrap();
		Layout { layers: layers, keycodes_to_positions: keycodes_to_positions }
	}

	pub fn randomize(&mut self, rng: &mut impl Rng, valid_keycodes: Vec<Keycode>) -> Result<(), AlcError> {
		for layer_num in 0..self.layers.len() {
			let mut layer = self.layers.get_mut(layer_num).unwrap();
			layer.randomize(rng, valid_keycodes.clone());
		}
		let keycodes_to_positions = keycode_position_mapping_from_layout::<R, C>(self.layers.clone())?;
		self.keycodes_to_positions = keycodes_to_positions;
		Ok(())
	}

	/// Swaps two keys. Ignores symmetry, layer switching, etc., as those should be taken care of by the calling function.
	unsafe fn swap_two(&mut self, p1: &LayoutPosition, p2: &LayoutPosition) -> () {
		// this is ~probably~ definitely not the most efficient, but it is easy
		// let k1_clone = self.get_from_layout_position(p1).unwrap().clone();
		// let k2_clone = self.get_from_layout_position(p2).unwrap().clone();

		// let k1 = self.get_mut_from_layout_position(p1).unwrap();
		// *k1 = k2_clone;
		// let k2 = self.get_mut_from_layout_position(p2).unwrap();
		// *k2 = k1_clone;
		let mut k1 = self.get_mut_from_layout_position(p1).unwrap() as *mut KeycodeKey;
		let mut k2 = self.get_mut_from_layout_position(p2).unwrap() as *mut KeycodeKey;
		ptr::swap(k1, k2)
	}

	pub fn swap(&mut self, p1: &LayoutPosition, p2: &LayoutPosition) -> Option<()> {
		// todo: make use of optimized keycode to position remapping computation where only the affected keycodes get are remapped

		// Bunch of checks for things that should be set up in whatever calls swap. I think these are easy to do in the calling function so the panic is mainly as a reminder when I'm implementing things.
		if p1 == p2 {
			panic!("Don't try to swap the same positions {} and {}, fix in calling function.", p1, p2)
		}
		let k1 = self.get_from_layout_position(&p1).unwrap();
		let k2 = self.get_from_layout_position(&p2).unwrap();
		if !k1.is_moveable() || !k2.is_moveable() {
			panic!("I think it would be better to handle moveability in whatever calls this swap function rather than swapping nothing here.")
		}
		if let _LS(i) = k2.value() {
			panic!("For convenience, place layer switches in the first position of the swap and disallow swaps where both keys are layer switches. Fix this in the calling function.");
		}
		if !k1.is_symmetric() && k2.is_symmetric() {
			panic!("For convenience, place symmetric keys in the first position of the swap. Fix this in the calling function.")
		}
		if let _LS(i) = k1.value() {
			if p1.layer_index != p2.layer_index {
				panic!("Swaps involving layer switches must occur within the same layer otherwise layers could become unreachable, fix this in the calling function. {} vs {}, k1 is {}, layout is {}", p1, p2, k1, self)
			}
			if k2.is_symmetric() {
				panic!("(The first position should already be the layer switch for convenience.) Can't swap a layer switch key with a symmetric key, fix in the callling function.")
			}
			if k1.is_symmetric() {
				panic!("Can't have a layer switch that is also symmetric due to additionaly complexity. Maybe later.")
			}
		}
		// cursed things
		let self_clone = self.clone();
		let k1 = self.get_mut_from_layout_position(p1).unwrap();
		let k1_clone = self_clone.get_from_layout_position(p1).unwrap();
		let k2_clone = self_clone.get_from_layout_position(p2).unwrap();
		if let _LS(target_layer) = k1.value() {
			// Layer switches need to be in the same layer position in the starting layer and the target layer. So, if the first position is a layer switch, its counterpart must be in:
			let p1_counterpart = &LayoutPosition::for_layout(target_layer, p1.row_index, p1.col_index);
			let p2_counterpart = &LayoutPosition::for_layout(target_layer, p2.row_index, p2.col_index);
			let k2_counterpart_clone = self_clone.get_from_layout_position(&p2_counterpart).unwrap();
			// I think these are harder to handle in the calling function, so just have nothing happen here
			if !k2_counterpart_clone.is_moveable() {
				println!("Warning: attempted to swap a layer switch with position x: {} and found that x's corresponding position {} was not moveable. Doing nothing instead.", p2, p2_counterpart);
				return None;
			}
			if k2_counterpart_clone.is_symmetric() {
				println!("Warning: attempted to swap a layer switch with position x: {} and found that x's corresponding position {} was symmetric, making the swap not valid. Doing nothing instead.", p2, p2_counterpart);
				return None;
			}
			// yeah gonna want to redo this section once I understand more
			k1.replace_with(&k2_clone);
			let k2 = self.get_mut_from_layout_position(p2).unwrap();
			k2.replace_with(&k1_clone);

			let k1_counterpart = self.get_mut_from_layout_position(p1_counterpart).unwrap();
			k1_counterpart.replace_with(&k2_counterpart_clone);
			let k2_counterpart = self.get_mut_from_layout_position(p2_counterpart).unwrap();
			let k1_counterpart_clone = self_clone.get_from_layout_position(p1_counterpart).unwrap();
			k2_counterpart.replace_with(&k1_counterpart_clone);
		} else if k1_clone.is_symmetric() {
			let p1_counterpart = self_clone.symmetric_position(&p1);
			if p2.col_index as f32 == (C as f32 - 1.0) / 2.0 {
				println!("Warning: symmetric p1 {} is being swapped into the center column {}, meaning p1's counterpart {} has no where to go, doing nothing instead.", p1, p2, p1_counterpart);
				return None;
			}
			let p2_counterpart = self_clone.symmetric_position(&p2);
			let k2_counterpart_clone = self_clone.get_from_layout_position(&p2_counterpart).unwrap();
			if !k2_counterpart_clone.is_moveable() {
				println!("Warning: attempted to swap a symmetric key with position x: {} and found that x's corresponding position {} was not moveable. Doing nothing instead.", p2, p2_counterpart);
				return None;
			}
			if let _LS(target_layer) = k2_counterpart_clone.value() {
				println!("Warning: attempted symmetric swap but p2 {}'s counterpart {} is a layer switch. Doing nothing instead.", p2, p2_counterpart);
				return None;
			}
			k1.replace_with(&k2_clone);
			let k2 = self.get_mut_from_layout_position(p2).unwrap();
			k2.replace_with(&k1_clone);

			let k1_counterpart = self.get_mut_from_layout_position(&p1_counterpart).unwrap();
			k1_counterpart.replace_with(&k2_counterpart_clone);
			let k2_counterpart = self.get_mut_from_layout_position(&p2_counterpart).unwrap();
			let k1_counterpart_clone = self_clone.get_from_layout_position(&p1_counterpart).unwrap();
			k2_counterpart.replace_with(&k1_counterpart_clone);
		} else {
			k1.replace_with(&k2_clone);
			let k2 = self.get_mut_from_layout_position(p2).unwrap();
			k2.replace_with(&k1_clone);
		}
		
		Some(())
	}

	pub fn replace(&mut self, p: &LayoutPosition, value: Keycode) -> Option<()> {
		// make use of optimized keycode to position remapping computation where only the affected keycodes get are remapped

		let k = self.get_from_layout_position(&p).unwrap();
		if self.keycodes_to_positions.get(&k.value()).unwrap().len() == 1 {
			panic!("There is only one way to reach {}, not allowed to replace. Fix in the calling function.", k)
		}
		if let _LS(target_layer) = k.value() {
			panic!("Not allowed to replace the layer switch at {}, fix in calling function.", p)
		}
		let k = self.get_mut_from_layout_position(&p).unwrap().set_value(value);

		Some(())

	}
}

fn keycode_position_mapping_from_layout<const R: usize, const C: usize>(layers: Vec<Layer<R, C, KeycodeKey>>) -> Result<KeycodePositionMap, AlcError> {
	let mut keycodes_to_positions: KeycodePositionMap = Default::default();
	for (layer_num, layer) in layers.iter().enumerate() {
		for r in 0..R {
			for c in 0..C {
				let key = layer.get(r, c)?;
				let key_value = key.value();
				let layout_position = LayoutPosition::for_layout(layer_num, r, c);
				let layout_position_sequence = LayoutPositionSequence::from(vec![layout_position.clone()]);
				if layer_num == 0 {
					keycodes_to_positions.entry(key_value).or_insert(vec![]).push(layout_position_sequence);
				} else {
					match key_value {
						_LS(i) => continue,
						_ => (),
					}
					let mut map_clone = keycodes_to_positions.clone();
					// check that layer_num is reachable. If layer is currently not reachable, could pass until after the rest of the layout is processed in case there is a downward layer move, but not going to implement that now since QMK does not recommend having layer switches like that
					let mut sequences_to_reach_layer = match map_clone.get(&_LS(layer_num)) {
						Some(v) => v,
						None => return Err(AlcError::LayerAccessError(layer_num)),
					};
					// loop through all sequences that can reach _LS(i)
					for s_index in 0..sequences_to_reach_layer.len() {
						let mut seq_clone = sequences_to_reach_layer.clone();
						let new_seq = seq_clone.get_mut(s_index).unwrap();
						// add the position of the current key we are on at the end
						new_seq.push(layout_position.clone());
						keycodes_to_positions.entry(key_value).or_insert(vec![]).push(new_seq.clone());
					}
				}
			}
		}	
	}
	Ok(keycodes_to_positions)
}
impl<const R: usize, const C: usize> TryFrom<&str> for Layout<R, C> {
	type Error = AlcError; //Box<dyn Error>;

	fn try_from(layout_string: &str) -> Result<Self, Self::Error> {
		let mut layers: Vec<Layer<R, C, KeycodeKey>> = vec![];

		let re = regex::Regex::new(r"(___)(.*)(___)")?; //.unwrap();
		for layer_string in re.split(layout_string).collect::<Vec<&str>>() {
			if layer_string.trim().is_empty() {
				continue;
			}
			layers.push(Layer::try_from(layer_string)?);
		}
		let keycodes_to_positions = keycode_position_mapping_from_layout::<R, C>(layers.clone())?;
		Ok(Layout { layers, keycodes_to_positions: keycodes_to_positions})
	}
}


impl<const R: usize, const C: usize> fmt::Display for Layout<R, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (i, layer) in self.layers.iter().enumerate() {
			writeln!(f, "___Layer {}___", i);
			writeln!(f, "{}", layer);
		}
		for k in self.keycodes_to_positions.keys() {
			let key_text = match k {
				_LS(i) => format!("_LS{}", i),
				_ => k.to_string(),
			};
			write!(f, "{}: ", key_text);
			for seq in self.keycodes_to_positions.get(k).unwrap().iter() {
				write!(f, "{}, ", seq);
			}
			writeln!(f, "");
		}
		
		write!(f, "")
    }
}
impl<const R: usize, const C: usize> fmt::Binary for Layout<R, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (i, layer) in self.layers.iter().enumerate() {
			writeln!(f, "___Layer {}___", i);
			writeln!(f, "{:b}", layer);
		}
		for k in self.keycodes_to_positions.keys() {
			let key_text = match k {
				_LS(i) => format!("_LS{}", i),
				_ => k.to_string(),
			};
			write!(f, "{}: ", key_text);
			for seq in self.keycodes_to_positions.get(k).unwrap().iter() {
				write!(f, "{}, ", seq);
			}
			writeln!(f, "");
		}
		write!(f, "")
    }
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test() {
		let mut rng = StdRng::seed_from_u64(0);
		let mut layout = Layout::<2, 3>::init_blank(5);
		layout.get_mut(0, 1, 2).unwrap().set_value(_D);
		layout.get_mut(0, 1, 2).unwrap().set_is_moveable(false);
		layout.randomize(&mut rng, vec![_A, _E]);
		fn test_randomize<const R: usize, const C: usize>(layout: Layout<R, C>) {
			let expected_key = KeycodeKey::try_from("D_00").unwrap();
			assert_eq!(layout.get(0, 1, 2).unwrap(), expected_key);
			println!("{:b}", layout);
		}
		test_randomize::<2, 3>(layout.clone());

		fn test_string_construction<const R: usize, const C: usize>(layout: Layout<R, C>) {
			let layout_string = "
			___Layer 0___
					0       1       2 
			0| LS1_10  LS2_10  LS3_10 
			1| LS4_10    E_10    D_00 
			
			___Layer 1___
					0       1       2 
			0| LS1_10    E_10    A_10 
			1|   E_10    A_10    A_10 
			
			___Layer 2___
					0       1       2 
			0|   A_10  LS2_10    E_10 
			1|   E_10    A_10    A_10 
			
			___Layer 3___
					0       1       2 
			0|   A_10    E_10  LS3_10 
			1|   A_10    E_10    E_10 
			
			___Layer 4___
					0       1       2 
			0|   A_10    E_10    E_10 
			1| LS4_10    A_10    E_10 
			";
			let layout_from_string = Layout::try_from(layout_string).unwrap();
			println!("layout from string\n{:b}", layout_from_string.clone());
			assert_eq!(layout_from_string, layout);
		}
		test_string_construction::<2, 3>(layout);
	}

	#[test]
	fn test_keycode_position_map () {
		// assert!(false);
	}

	#[test]
	fn test_swap_two() {
		let mut layout = match Layout::<1, 4>::try_from("
			___Layer 0___
			A_10 B_10 C_10 LS1_10
			___Layer 1___
			D_10 E_10 H_10 LS1_10
		") {
			Ok(v) => v,
			Err(e) => panic!("{}", e),
		};
		println!("{}", layout);
		unsafe { layout.swap_two(&LayoutPosition::for_layout(0, 0, 0), &LayoutPosition::for_layout(0, 0, 2)) };
		assert_eq!(layout.get(0, 0, 0).unwrap().value(), _C);
		assert_eq!(layout.get(0, 0, 2).unwrap().value(), _A);
		
		unsafe {layout.swap_two(&LayoutPosition::for_layout(0, 0, 1), &LayoutPosition::for_layout(1, 0, 2)) };
		assert_eq!(layout.get(0, 0, 1).unwrap().value(), _H);
		assert_eq!(layout.get(1, 0, 2).unwrap().value(), _B);
		println!("{}", layout);
	}

	#[test]
	fn test_swap_for_ls() {
		let mut layout = match Layout::<1, 4>::try_from("
			___Layer 0___
			A_10 B_10 C_10 LS1_10
			___Layer 1___
			D_10 E_10 H_10 LS1_10
		") {
			Ok(v) => v,
			Err(e) => panic!("{}", e),
		};
		println!("{}", layout);
		layout.swap(&LayoutPosition::for_layout(0, 0, 0), &LayoutPosition::for_layout(0, 0, 2));
		assert_eq!(layout.get(0, 0, 0).unwrap().value(), _C);
		assert_eq!(layout.get(0, 0, 2).unwrap().value(), _A);
		
		layout.swap(&LayoutPosition::for_layout(0, 0, 1), &LayoutPosition::for_layout(1, 0, 2));
		assert_eq!(layout.get(0, 0, 1).unwrap().value(), _H);
		assert_eq!(layout.get(1, 0, 2).unwrap().value(), _B);

		// layout.swap(&LayoutPosition::for_layout(0, 0, 3), &LayoutPosition::for_layout(1, 0, 1)); // correctly panics

		layout.swap(&LayoutPosition::for_layout(0, 0, 3), &LayoutPosition::for_layout(0, 0, 2));
		assert_eq!(layout.get(0, 0, 3).unwrap().value(), _A);
		assert_eq!(layout.get(0, 0, 2).unwrap().value(), _LS(1));
		assert_eq!(layout.get(1, 0, 3).unwrap().value(), _B);
		assert_eq!(layout.get(1, 0, 2).unwrap().value(), _LS(1));
		println!("{}", layout);
	}

	#[test]
	fn test_swap_for_symm() {
		let mut layout = match Layout::<1, 4>::try_from("
			___Layer 0___
			A_10 B_11 C_11 LS1_10
			___Layer 1___
			D_10 E_10 H_10 LS1_10
		") {
			Ok(v) => v,
			Err(e) => panic!("{}", e),
		};
		layout.swap(&LayoutPosition::for_layout(0, 0, 1), &LayoutPosition::for_layout(0, 0, 2));
		assert_eq!(layout.get(0, 0, 1).unwrap().value(), _C);
		assert_eq!(layout.get(0, 0, 2).unwrap().value(), _B);

		layout.swap(&LayoutPosition::for_layout(0, 0, 1), &LayoutPosition::for_layout(1, 0, 2));
		assert_eq!(layout.get(0, 0, 1).unwrap().value(), _H);
		assert_eq!(layout.get(0, 0, 2).unwrap().value(), _E);
		assert_eq!(layout.get(1, 0, 1).unwrap().value(), _B);
		assert_eq!(layout.get(1, 0, 2).unwrap().value(), _C);
		println!("{}", layout);
	}

	#[test]
	fn test_replace() {
		let mut layout = match Layout::<1, 4>::try_from("
			___Layer 0___
			A_10 B_11 C_11 LS1_10
			___Layer 1___
			D_10 E_10 E_10 LS1_10
		") {
			Ok(v) => v,
			Err(e) => panic!("{}", e),
		};
		// layout.replace(&LayoutPosition::for_layout(0, 0, 3), _E);
		// layout.replace(&LayoutPosition::for_layout(0, 0, 0), _E);
		layout.replace(&LayoutPosition::for_layout(1, 0, 1), _C);
		assert_eq!(layout.get(1, 0, 1).unwrap().value(), _C);
	}
	
}