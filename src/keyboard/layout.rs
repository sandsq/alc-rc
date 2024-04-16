use array2d::{Array2D, Error as Array2DError};
use std::ops::Index;
use rand::prelude::*;
use std::error::Error;
use std::fmt;
use std::collections::HashMap;
use thiserror;

use crate::text_processor::keycode::Keycode::{self, *};
use super::key::{KeyValue, KeycodeKey, PhysicalKey};
use super::layer::{Layer, KeyboardError};
use super::{LayoutPosition, LayoutPositionSequence};

/// A keyboard layout is a collection of layers of KeycodeKeys, plus additional info specifying how to navigate the layout, etc. (fill in later)
/// Layouts with multiple layers must have a way to access every layer.
/// For now, the only way to change layers is via a layer switch key. _LS(2) means that key switches to layer 2
#[derive(Debug, PartialEq)]
pub struct Layout<const R: usize, const C: usize> {
	layers: Vec<Layer<R, C, KeycodeKey>>,
	keycodes_to_positions: HashMap<Keycode, Vec<LayoutPositionSequence>>,
}
impl<const R: usize, const C: usize> Layout<R, C> {
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

		let mut keycodes_to_positions: HashMap<Keycode, Vec<LayoutPositionSequence>> = Default::default();
		for (i, layer) in layers.iter().enumerate() {
			for r in 0..R {
				for c in 0..C {
					let key = layer.get(r, c).unwrap();
					let key_value = key.value();
					let layout_position = LayoutPosition::for_layout(i, r, c);
					let layout_position_sequence = LayoutPositionSequence::from(vec![layout_position.clone()]);
					if i == 0 {
						keycodes_to_positions.entry(key_value).or_insert(vec![]).push(layout_position_sequence);
					} else {
						let mut map_clone = keycodes_to_positions.clone();
						let mut sequences_to_reach_layer = map_clone.get(&_LS(i)).unwrap(); // shouldn't need to check that this exists since we are controlling how the layout is put together, but would need to check if coming from user input
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
		
		Layout { layers: layers, keycodes_to_positions: keycodes_to_positions }
	}
	pub fn randomize(&mut self, rng: &mut impl Rng, valid_keycodes: Vec<Keycode>) -> Result<(), KeyboardError> {
		
		
		Ok(())
	}
}
impl<const R: usize, const C: usize> fmt::Display for Layout<R, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (i, layer) in self.layers.iter().enumerate() {
			writeln!(f, "@@@ L{} @@@", i);
			writeln!(f, "{}", layer);
		}
		writeln!(f, "{}", self.keycodes_to_positions);
		write!(f, "")
    }
}
impl<const R: usize, const C: usize> fmt::Binary for Layout<R, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (i, layer) in self.layers.iter().enumerate() {
			writeln!(f, "@@@ L{} @@@", i);
			writeln!(f, "{:b}", layer);
		}
		writeln!(f, "{}", self.keycodes_to_positions);
		write!(f, "")
    }
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn display_layout() {
		let layout = Layout::<3, 4>::init_blank(6);
		println!("{:b}", layout);
	}
}