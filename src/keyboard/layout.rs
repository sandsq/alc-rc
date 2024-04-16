use array2d::{Array2D, Error as Array2DError};
use std::ops::Index;
use rand::prelude::*;
use std::error::Error;
use std::fmt::{self, Formatter};
use std::collections::HashMap;
use thiserror;

use crate::text_processor::keycode::Keycode::{self, *};
use super::key::{KeyValue, KeycodeKey, PhysicalKey};
use super::layer::{Layer, LayerError};
use super::{LayoutPosition, LayoutPositionSequence};

type KeycodePositionMap = HashMap<Keycode, Vec<LayoutPositionSequence>>;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum LayoutError {
	#[error("layer {0} is not reachable")]
	LayerAccessError(usize),
}

/// A keyboard layout is a collection of layers of KeycodeKeys, plus additional info specifying how to navigate the layout, etc. (fill in later)
/// Layouts with multiple layers must have a way to access every layer.
/// For now, the only way to change layers is via a layer switch key. _LS(2) means that key switches to layer 2
#[derive(Debug, PartialEq)]
pub struct Layout<const R: usize, const C: usize> {
	layers: Vec<Layer<R, C, KeycodeKey>>,
	keycodes_to_positions: KeycodePositionMap,
}
impl<const R: usize, const C: usize> Layout<R, C> {
	pub fn get(&self, layer_index: usize, row_index: usize, col_index: usize) -> Result<KeycodeKey, Array2DError> {
		// the first get isn't an Array2DError since it's on a vector, but deal with that later.
		self.layers.get(layer_index).unwrap().get(row_index, col_index)
	}
	pub fn get_mut(&mut self, layer_index: usize, row_index: usize, col_index: usize) -> Result<&mut KeycodeKey, Array2DError> {
		// the first get_mut isn't an Array2DError since it's on a vector, but deal with that later.
		self.layers.get_mut(layer_index).unwrap().get_mut(row_index, col_index)
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

	pub fn randomize(&mut self, rng: &mut impl Rng, valid_keycodes: Vec<Keycode>) -> Result<(), LayoutError> {
		for layer_num in 0..self.layers.len() {
			let mut layer = self.layers.get_mut(layer_num).unwrap();
			layer.randomize(rng, valid_keycodes.clone());
		}
		let keycodes_to_positions = keycode_position_mapping_from_layout::<R, C>(self.layers.clone())?;
		self.keycodes_to_positions = keycodes_to_positions;
		Ok(())
	}
}

fn keycode_position_mapping_from_layout<const R: usize, const C: usize>(layers: Vec<Layer<R, C, KeycodeKey>>) -> Result<KeycodePositionMap, LayoutError> {
	let mut keycodes_to_positions: KeycodePositionMap = Default::default();
	for (layer_num, layer) in layers.iter().enumerate() {
		for r in 0..R {
			for c in 0..C {
				let key = layer.get(r, c).unwrap();
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
						None => return Err(LayoutError::LayerAccessError(layer_num)),
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
	type Error = Box<dyn Error>;
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		
		todo!()
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
	fn test_randomize() {
		let mut rng = StdRng::seed_from_u64(0);
		let mut layout = Layout::<2, 3>::init_blank(5);
		layout.get_mut(0, 1, 2).unwrap().set_value(_D);
		layout.get_mut(0, 1, 2).unwrap().set_is_moveable(false);
		layout.randomize(&mut rng, vec![_A, _E]);
		let expected_key = KeycodeKey::try_from("D_00").unwrap();
		assert_eq!(layout.get(0, 1, 2).unwrap(), expected_key);
		println!("{:b}", layout);
	}

	
}