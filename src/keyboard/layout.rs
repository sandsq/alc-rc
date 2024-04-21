use std::ops::Index;
use rand::prelude::*;
use std::fmt;
use std::collections::{HashMap, VecDeque};
use regex;
use std::mem::discriminant;

use crate::alc_error::AlcError;
use crate::text_processor::keycode::Keycode::{self, *};
use crate::text_processor::ngram::Ngram;
use super::key::{KeyValue, KeycodeKey};
use super::layer::Layer;
use super::{LayoutPosition, LayoutPositionSequence};

type KeycodePathMap = HashMap<Keycode, Vec<LayoutPositionSequence>>;


/// A keyboard layout is a collection of layers of KeycodeKeys, plus additional info specifying how to navigate the layout, etc. (fill in later)
/// Layouts with multiple layers must have a way to access every layer.
/// For now, the only way to change layers is via a layer switch key. _LS(2) means that key switches to layer 2
#[derive(Debug, PartialEq, Clone)]
pub struct Layout<const R: usize, const C: usize> {
	layers: Vec<Layer<R, C, KeycodeKey>>,
	keycode_path_map: KeycodePathMap,
}
impl<const R: usize, const C: usize> Layout<R, C> {
	pub fn get(&self, layer_index: usize, row_index: usize, col_index: usize) -> Option<&KeycodeKey> {
		// the first get isn't an Array2DError since it's on a vector, but deal with that later.
		self.layers.get(layer_index)?.get(row_index, col_index)
	}
	pub fn get_from_layout_position(&self, lp: &LayoutPosition) -> Option<&KeycodeKey> {
		// the first get isn't an Array2DError since it's on a vector, but deal with that later.
		self.layers.get(lp.layer_index)?.get(lp.row_index, lp.col_index)
	}
	pub fn get_mut(&mut self, layer_index: usize, row_index: usize, col_index: usize) -> Option<&mut KeycodeKey> {
		// the first get_mut isn't an Array2DError since it's on a vector, but deal with that later.
		self.layers.get_mut(layer_index)?.get_mut(row_index, col_index)
	}
	pub fn get_mut_from_layout_position(&mut self, lp: &LayoutPosition) -> Option<&mut KeycodeKey> {
		// the first get_mut isn't an Array2DError since it's on a vector, but deal with that later.
		self.layers.get_mut(lp.layer_index)?.get_mut(lp.row_index, lp.col_index)
	}
	pub fn get_position_sequences_to_keycode(&self, k: Keycode) -> Option<&Vec<LayoutPositionSequence>> {
		self.keycode_path_map.get(&k)
	}
	pub fn symmetric_position(&self, lp: &LayoutPosition) -> LayoutPosition {
		self.layers.get(0).unwrap().symmetric_position(&lp) // would panic if layout is empty but that shouldn't normally be possible
	}
	pub fn init_blank(num_layers: usize) -> Self {
		let mut layers: Vec<Layer<R, C, KeycodeKey>> = vec![];
		for _i in 0..num_layers {
			let layer = Layer::<R, C, KeycodeKey>::init_blank();
			layers.push(layer);
		}
		for j in 0..num_layers - 1 {
			layers[0].get_mut_row_major(j).unwrap().set_value(_LS(j + 1));
			layers[j + 1].get_mut_row_major(j).unwrap().set_value(_LST(j + 1, 0));
		}
		let keycodes_to_positions = keycode_path_map_from_layout::<R, C>(layers.clone()).unwrap();
		Layout { layers: layers, keycode_path_map: keycodes_to_positions }
	}


	/// Randomly places [Keycode]s from `valid_keycodes` into the layout. Keys can be blocked off with __00 (_NO keycode, not moveable, not symmetric) to account for (currently) unsupported sizes and non-standard form factors. Prefilled keys are not randomized so that layouts can be "seeded" with "good" initial layouts.
	pub fn randomize(&mut self, rng: &mut impl Rng, valid_keycodes: &Vec<Keycode>) -> Result<(), AlcError> {
		let mut used_all_keycodes_flag = false;
		let mut valid_keycodes_all = VecDeque::from(valid_keycodes.clone());
		valid_keycodes_all.make_contiguous().shuffle(rng);
		let mut valid_keycodes_to_draw_from = VecDeque::from(valid_keycodes.clone());
		valid_keycodes_to_draw_from.make_contiguous().shuffle(rng);
		for layer_num in 0..self.layers.len() {
			let layer = self.layers.get_mut(layer_num).unwrap();
			// we want to fill out all valid keycodes over the entire layout, not just layer by layer
			(valid_keycodes_to_draw_from, used_all_keycodes_flag) = layer.randomize(&valid_keycodes_all, &valid_keycodes_to_draw_from);
			if used_all_keycodes_flag {
				break;
			}
		}
		if !used_all_keycodes_flag {
			println!("Warning: the keycodes {:?} may not have made it into the layout since they were left over. This could happen if the layout is too small or if you prefilled a lot of immovable spots.", valid_keycodes_to_draw_from)
		}
		let keycodes_to_positions = keycode_path_map_from_layout::<R, C>(self.layers.clone())?;
		self.keycode_path_map = keycodes_to_positions;
		Ok(())
	}

	/// Within a layout there can be multiple ways to type a keycode, so there can be multiple ways to type an ngram. Keep track of all of these
	pub fn ngram_to_sequences(&self, ngram: &Ngram) -> Option<Vec<LayoutPositionSequence>> {
		let mut output_sequences_to_ngram: Vec<LayoutPositionSequence> = vec![];

		let ngram_iter = ngram.clone().into_iter();
		for keycode in ngram_iter {
			// println!("output sequences {:?} at start,  keycode {}", output_sequences_to_ngram, keycode);
			let sequences_to_keycode = match self.get_position_sequences_to_keycode(keycode) {
				Some(p) => p,
				None => {
					// println!("Warning: keycode {} is not typeable by the layout:\n{:#}\nIf this is unexpected, there is a bug somewhere.", keycode, self);
					return None;
				},
			};
			if output_sequences_to_ngram.len() == 0 {
				output_sequences_to_ngram = sequences_to_keycode.clone();
			} else {
				let mut temp_sequences_to_ngram: Vec<LayoutPositionSequence> = vec![];
				for sequence in sequences_to_keycode {
					for mut current_sequence in output_sequences_to_ngram.clone() {
						current_sequence.append(&mut sequence.clone());
						temp_sequences_to_ngram.push(current_sequence);
					}
				}
				output_sequences_to_ngram = temp_sequences_to_ngram;
			}
			// println!("output sequences {:?} at end,  keycode {}", output_sequences_to_ngram, keycode);
		}
		Some(output_sequences_to_ngram)
	}


	/// returns true if a swap happened
	pub fn swap(&mut self, p1: &LayoutPosition, p2: &LayoutPosition) -> bool {
		// todo: make use of optimized keycode to position remapping computation where only the affected keycodes get are remapped
		#[allow(unused_assignments)]
		let mut swap_happened = false;

		// Bunch of checks for issues that should be easier to resolve in whatever calls swap rather than within swap.
		if p1 == p2 {
			// panic!("Error for the developer! Don't try to swap the same positions {} and {}.", p1, p2)
			return false;
		}
		let k1 = self.get_from_layout_position(&p1).unwrap();
		let k2 = self.get_from_layout_position(&p2).unwrap();
		if !k1.is_moveable() || !k2.is_moveable() {
			panic!("Error for the developer! Don't try to swap unmoveable positions.")
		}
		if let _LS(_i) = k2.value() {
			panic!("Error for the developer! Place layer switches in the first position of the swap and disallow swaps where both keys are layer switches.");
		}
		if let _LST(_i, _j) = k1.value() {
			panic!("Error for the developer! Only allow the source of the layer switch to be chosen for swapping");
		}
		if let _LST(_i, _j) = k2.value() {
			panic!("Error for the developer! Only allow the source of the layer switch to be chosen for swapping");
		}
		if !k1.is_symmetric() && k2.is_symmetric() {
			panic!("Error for the developer! Place symmetric keys in the first position of the swap.")
		}
		if let _LS(_i) = k1.value() {
			if p1.layer_index != p2.layer_index {
				panic!("Error for the developer! Swaps involving layer switches must occur within the same layer otherwise layers could become unreachable.")
			}
			if k2.is_symmetric() {
				panic!("Error for the developer! Can't swap a layer switch key with a symmetric key.")
			}
			if k1.is_symmetric() {
				panic!("Error for the developer! Can't have a layer switch that is also symmetric due to additionaly complexity. This should be caught when reading in a Key from a string.")
			}
		}
		// cursed things
		let self_clone = self.clone();
		let k1 = self.get_mut_from_layout_position(p1).unwrap();
		let k1_clone = self_clone.get_from_layout_position(p1).unwrap();
		let k2_clone = self_clone.get_from_layout_position(p2).unwrap();
		if let _LS(target_layer) = k1.value() {
			// Layer switches need to be in the same layer position in the starting layer and the target layer. So, if the first position is a layer switch, its counterpart must be in:
			let p1_counterpart = &LayoutPosition::new(target_layer, p1.row_index, p1.col_index);
			let p2_counterpart = &LayoutPosition::new(target_layer, p2.row_index, p2.col_index);
			let k2_counterpart_clone = self_clone.get_from_layout_position(&p2_counterpart).unwrap();
			// I think these are harder to handle in the calling function, so just have nothing happen here
			if !k2_counterpart_clone.is_moveable() {
				println!("Warning: attempted to swap a layer switch with position x: {} and found that x's corresponding position {} was not moveable. Doing nothing instead.", p2, p2_counterpart);
				return false;
			}
			if k2_counterpart_clone.is_symmetric() {
				println!("Warning: attempted to swap a layer switch with position x: {} and found that x's corresponding position {} was symmetric, making the swap not valid. Doing nothing instead.", p2, p2_counterpart);
				return false;
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
			swap_happened = true;
		} else if k1_clone.is_symmetric() {
			let p1_counterpart = self_clone.symmetric_position(&p1);
			if p2.col_index as f32 == (C as f32 - 1.0) / 2.0 {
				println!("Warning: symmetric p1 {} is being swapped into the center column {}, meaning p1's counterpart {} has no where to go, doing nothing instead.", p1, p2, p1_counterpart);
				return false;
			}
			let p2_counterpart = self_clone.symmetric_position(&p2);
			let k2_counterpart_clone = self_clone.get_from_layout_position(&p2_counterpart).unwrap();
			if !k2_counterpart_clone.is_moveable() {
				println!("Warning: attempted to swap a symmetric key with position x: {} and found that x's corresponding position {} was not moveable. Doing nothing instead.", p2, p2_counterpart);
				return false;
			}
			if let _LS(_target_layer) = k2_counterpart_clone.value() {
				println!("Warning: attempted symmetric swap but p2 {}'s counterpart {} is a layer switch. Doing nothing instead.", p2, p2_counterpart);
				return false;
			}
			k1.replace_with(&k2_clone);
			let k2 = self.get_mut_from_layout_position(p2).unwrap();
			k2.replace_with(&k1_clone);

			let k1_counterpart = self.get_mut_from_layout_position(&p1_counterpart).unwrap();
			k1_counterpart.replace_with(&k2_counterpart_clone);
			let k2_counterpart = self.get_mut_from_layout_position(&p2_counterpart).unwrap();
			let k1_counterpart_clone = self_clone.get_from_layout_position(&p1_counterpart).unwrap();
			k2_counterpart.replace_with(&k1_counterpart_clone);
			swap_happened = true;
		} else {
			k1.replace_with(&k2_clone);
			let k2 = self.get_mut_from_layout_position(p2).unwrap();
			k2.replace_with(&k1_clone);
			swap_happened = true;
		}
		self.keycode_path_map = keycode_path_map_from_layout(self.layers.clone()).unwrap();
		swap_happened
	}

	pub fn replace(&mut self, p: &LayoutPosition, value: Keycode) -> bool {
		// make use of optimized keycode to position remapping computation where only the affected keycodes get are remapped
		#[allow(unused_assignments)]
		let mut replace_happened = false;
		let k = self.get_from_layout_position(&p).unwrap();
		if self.keycode_path_map.get(&k.value()).unwrap().len() == 1 {
			panic!("Error for the developer! There is only one way to reach {}, not allowed to replace.", k)
		}
		if let _LST(_i, _j) = k.value() {
			panic!("Error for the developer! Not allowed to replace LST");
		}
		if let _LS(_target_layer) = k.value() {
			panic!("Error for the developer! Not allowed to replace the layer switch ({}).", p)
		}
		if !k.is_moveable() {
			panic!("Error for the developer! Not allowed to replace a non-moveable key.")
		}
		self.get_mut_from_layout_position(&p).unwrap().set_value(value);
		replace_happened = true;
		self.keycode_path_map = keycode_path_map_from_layout(self.layers.clone()).unwrap();
		
		replace_happened
	}
	pub fn gen_random_position(&self, rng: &mut impl Rng) -> LayoutPosition {
		let layer_limit = self.layers.len();
		LayoutPosition::new(rng.gen_range(0..layer_limit), rng.gen_range(0..R), rng.gen_range(0..C))
	}

	fn generate_random_moveable_position(&self, rng: &mut impl Rng) -> Option<LayoutPosition> {
		let fallback_count = 100;
		let mut p = self.gen_random_position(rng);
		let mut k = self.get_from_layout_position(&p).unwrap();
		let mut count = 0;
		while !k.is_moveable() {
			p = self.gen_random_position(rng);
			k = self.get_from_layout_position(&p).unwrap();
			count += 1;
			if count >= fallback_count {
				return None
				// return Err(AlcError::SwapFallbackError(fallback_count, String::from("could not find moveable first key")));
			}
		}
		Some(p)
	}

	pub fn generate_random_valid_swap(&self, rng: &mut impl Rng) -> Option<(LayoutPosition, LayoutPosition)> {
		let mut p1 = self.generate_random_moveable_position(rng)?;
		let mut k1 = self.get_from_layout_position(&p1)?;
		let mut p2 = self.generate_random_moveable_position(rng)?;
		let mut k2 = self.get_from_layout_position(&p2)?;
		let mut count = 0;
		let fallback_count = 100;

		while discriminant(&k1.value()) == discriminant(&_LST(1, 2)) {
			p1 = self.generate_random_moveable_position(rng)?;
			k1 = self.get_from_layout_position(&p1)?;
		}

		if let _LS(_i) = k1.value() {
			if k1.is_symmetric() {
				// return panic!("Error for the developer! Can't have a layer switch that is also symmetric due to additionaly complexity. This should be caught when reading in a Key from a string.");
			}
			while k2.is_symmetric() || (p1.layer_index != p2.layer_index) || std::mem::discriminant(&k2.value()) == std::mem::discriminant(&_LS(1)) || std::mem::discriminant(&k2.value()) == std::mem::discriminant(&_LST(1, 2)) {
				p2 = self.generate_random_moveable_position(rng)?;
				k2 = self.get_from_layout_position(&p2)?;
				count += 1;
				if count >= fallback_count {
					// return None;
					// return Err(AlcError::SwapFallbackError(fallback_count, String::from("key 1 was a layer switch and either i) no non-symmetric key 2s could be found or ii) no key 2s could be found in the same layer or iii) not non-layer switch key 2s could be found")));
					panic!("key 1 was a layer switch and either i) no non-symmetric key 2s could be found or ii) no key 2s could be found in the same layer or iii) not non-layer switch key 2s could be found");
				}
			}
			return Some((p1, p2));
		} else {
			while std::mem::discriminant(&k2.value()) == std::mem::discriminant(&_LS(1)) || (!k1.is_symmetric() && k2.is_symmetric()) || std::mem::discriminant(&k2.value()) == std::mem::discriminant(&_LST(1, 2)) {
				p2 = self.generate_random_moveable_position(rng)?;
				k2 = self.get_from_layout_position(&p2)?;
				count += 1;
				if count >= fallback_count {
					// return Err(AlcError::SwapFallbackError(fallback_count, String::from("key 1 was not a layer switch but proper k2 could not be found")));
					// return None;
					panic!("key 1 was not a layer switch but proper k2 could not be found");
				}
			}
			return Some((p1, p2))
		}
	}

	pub fn gen_valid_replace(&self, rng: &mut impl Rng) -> Option<LayoutPosition> {
		let mut p = self.generate_random_moveable_position(rng).unwrap();
		let mut k = self.get_from_layout_position(&p).unwrap();
		let fallback_count = 100;
		let mut count = 0;
		let paths = match self.keycode_path_map.get(&k.value()) {
			Some(v) => v,
			None => return None,
		};
		while paths.len() <= 1 || std::mem::discriminant(&k.value()) == std::mem::discriminant(&_LS(1)) || std::mem::discriminant(&k.value()) == std::mem::discriminant(&_LST(1, 2)) || !k.is_moveable() {
			p = self.generate_random_moveable_position(rng).unwrap();
			k = self.get_from_layout_position(&p).unwrap();
			count += 1;
			if count >= fallback_count {
				return None;
			}
		}
		Some(p)
	}

	/// layer switches, symmetry
	pub fn verify_layout_correctness(&self) -> (Vec<(LayoutPosition, LayoutPosition)>, Vec<(LayoutPosition, LayoutPosition)>) {
		let mut incorrect_layer_switch_locations: Vec<(LayoutPosition, LayoutPosition)> = vec![];
		let mut incorrect_symmetry_locations: Vec<(LayoutPosition, LayoutPosition)> = vec![];
		for layer_index in 0..self.layers.len() {
			for row_index in 0..R {
				for col_index in 0..C {
					let lp = LayoutPosition::new(layer_index, row_index, col_index);
					let key = self.get_from_layout_position(&lp).unwrap();
					if let _LS(target_layer) = key.value() {
						// println!("position {}", lp);
						let lp_corresponding = LayoutPosition::new(target_layer, row_index, col_index);
						let key_corresponding = self.get_from_layout_position(&lp_corresponding).unwrap();
						// if key_corresponding.value() != _NO {
						// 	panic!("For layer switch at {}, it's corresponding position {} should be blank, not {}", lp, lp_corresponding, key_corresponding);
						
						if let _LST(new_target_layer, _source_layer) = key_corresponding.value() {
							if new_target_layer != target_layer {
								panic!("LS's linked to each other should have the same layer number. For example, LS1 in layer 0 should be under LST(1, 0) in layer 1.")
							}
							// if source_layer != layer_index {
							// 	panic!("LS in the higher layer should point back down to its source layer. For example, LS1 in layer 0 should be under LS0 in layer 1.")
						
						} else {
							incorrect_layer_switch_locations.push((lp.clone(), lp_corresponding));
						}
					} else if let _LST(target_layer, source_layer) = key.value() {
						let lp_corresponding = LayoutPosition::new(source_layer, row_index, col_index);
						let key_corresponding = self.get_from_layout_position(&lp_corresponding).unwrap();
						if let _LS(new_target_layer) = key_corresponding.value() {
							if target_layer != new_target_layer {
								incorrect_layer_switch_locations.push((lp.clone(), lp_corresponding));		
							}
						}
					}
					if key.is_symmetric() {
						let lp_corresponding = self.symmetric_position(&lp);
						let key_corresponding = self.get_from_layout_position(&lp_corresponding).unwrap();
						if !key_corresponding.is_symmetric() {
							incorrect_symmetry_locations.push((lp, lp_corresponding));
						}
					}

				}
			}
		}
		(incorrect_layer_switch_locations, incorrect_symmetry_locations)
	}

}

fn keycode_path_map_from_layout<const R: usize, const C: usize>(layers: Vec<Layer<R, C, KeycodeKey>>) -> Result<KeycodePathMap, AlcError> {
	let mut keycode_path_map: KeycodePathMap = Default::default();
	for (layer_num, layer) in layers.iter().enumerate() {
		for r in 0..R {
			for c in 0..C {
				let key = &layer[(r, c)];
				let key_value = key.value();
				let layout_position = LayoutPosition::new(layer_num, r, c);
				let layout_position_sequence = LayoutPositionSequence::from_vector(vec![layout_position.clone()]);
				if layer_num == 0 {
					keycode_path_map.entry(key_value).or_insert(vec![]).push(layout_position_sequence);
				} else {
					match key_value {
						_LS(_i) => {
							// keycode_path_map.entry(key_value).or_insert(vec![]).push(layout_position_sequence);
							// // if _LS(i) is already in
							// if keycode_path_map.contains_key(&key_value) {
							// 	let path_end = keycode_path_map.get(&key_value).unwrap().last().unwrap().last().unwrap();
							// 	// but the previous _LS(i) is the same layer, then it is another _LS(i) that should be added
							// 	if layer_num == path_end.layer_index {
							// 		keycode_path_map.entry(key_value).or_insert(vec![]).push(layout_position_sequence);
							// 	}
							// } else {
							// 	// if _LS(i) is not already in
							// 	keycode_path_map.entry(key_value).or_insert(vec![]).push(layout_position_sequence);
							// }
						},
						_LST(_i, _j) => continue,
						_ => (),
						// keycode_path_map.entry(key_value).or_insert(vec![]).push(layout_position_sequence),
					}
					let map_clone = keycode_path_map.clone();
					// check that layer_num is reachable. If layer is currently not reachable, could pass until after the rest of the layout is processed in case there is a downward layer move, but not going to implement that now since QMK does not recommend having layer switches like that
					let sequences_to_reach_layer = match map_clone.get(&_LS(layer_num)) {
						Some(v) => v,
						None => return Err(AlcError::LayerAccessError(layer_num)),
					};
					// loop through all sequences that can reach _LS(i)
					for s_index in 0..sequences_to_reach_layer.len() {
						let mut seq_clone = sequences_to_reach_layer.clone();
						let new_seq = seq_clone.get_mut(s_index).unwrap();
						// add the position of the current key we are on at the end
						new_seq.push(layout_position.clone());
						keycode_path_map.entry(key_value).or_insert(vec![]).push(new_seq.clone());
					}
				}
			}
		}	
	}
	Ok(keycode_path_map)
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
		// don't show LST when printing out layout, so manually add LST back if LS is detected
		for layer_index in 0..layers.len() {
			for row_index in 0..R {
				for col_index in 0..C {
					let k = &layers[layer_index][(row_index, col_index)];
					if let _LS(target_layer) = k.value() {
						let k_counterpart = layers.get_mut(target_layer).unwrap().get_mut(row_index, col_index).unwrap();
						k_counterpart.set_value(_LST(target_layer, layer_index));
					}
				}
			}
		}

		let keycodes_to_positions = keycode_path_map_from_layout::<R, C>(layers.clone())?;
		let layout = Layout { layers, keycode_path_map: keycodes_to_positions};
		
		let (v1, v2) = layout.verify_layout_correctness();
		if v1.len() > 0 {
			return Err(AlcError::LayoutLayerSwitchError(v1));
		}
		if v2.len() > 0 {
			return Err(AlcError::LayoutSymmetryError(v2));
		}
		
		Ok(layout)
	}
}

impl<const R: usize, const C: usize> Index<(usize, usize, usize)> for Layout<R, C> {
	type Output = KeycodeKey;
	fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
		&self.layers[index.0][(index.1, index.2)]
	}
}

/// {:b} shows `is_moveable` and `is_symmetric` flags
/// {:#} shows keycode to position mapping
impl<const R: usize, const C: usize> fmt::Display for Layout<R, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (i, layer) in self.layers.iter().enumerate() {
			writeln!(f, "___Layer {}___", i)?;
			writeln!(f, "{}", layer)?;
		}
		if f.alternate() {
			for k in self.keycode_path_map.keys() {
				let key_text = match k {
					_LS(i) => format!("_LS{}", i),
					_ => k.to_string(),
				};
				write!(f, "{}: ", key_text)?;
				for seq in self.keycode_path_map.get(k).unwrap().iter() {
					write!(f, "{}, ", seq)?;
				}
				writeln!(f, "")?;
			}
		}
		Ok(())
    }
}
impl<const R: usize, const C: usize> fmt::Binary for Layout<R, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (i, layer) in self.layers.iter().enumerate() {
			writeln!(f, "___Layer {}___", i)?;
			writeln!(f, "{:b}", layer)?;
		}
		if f.alternate() {
			for k in self.keycode_path_map.keys() {
				let key_text = match k {
					_LS(i) => format!("_LS{}", i),
					_ => k.to_string(),
				};
				write!(f, "{}: ", key_text)?;
				for seq in self.keycode_path_map.get(k).unwrap().iter() {
					write!(f, "{}, ", seq)?;
				}
				writeln!(f, "")?;
			}
		}
		Ok(())
    }
}

impl Default for Layout<4, 12> {
	fn default() -> Self {
		let layout_string = "
		___Layer 0___
				0       1       2       3       4       5       6       7       8       9      10      11 
		0| LS1_10  LS2_10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10 
		1| SFT_11    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10  SFT_11 
		2|   __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10 
		3|   __10    __10    __10    __10    __10    SPC_00    __00    __10    __10    __10    __10    __10 

		___Layer 1___
				0       1       2       3       4       5       6       7       8       9      10      11 
		0|__10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10 
		1|   __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10 
		2|   __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10 
		3|   __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10 

		___Layer 2___
				0       1       2       3       4       5       6       7       8       9      10      11 
		0|   __10 __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10 
		1|   __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10 
		2|   __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10 
		3|   __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10    __10 
		
		";


		let layout = Layout::try_from(layout_string).unwrap();
		layout
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
		layout.randomize(&mut rng, &vec![_A, _E]).unwrap();
		fn test_randomize<const R: usize, const C: usize>(layout: Layout<R, C>) {
			let expected_key = KeycodeKey::try_from("D_00").unwrap();
			assert_eq!(*layout.get(0, 1, 2).unwrap(), expected_key);
			println!("{:b}", layout);
		}
		test_randomize::<2, 3>(layout.clone());

		fn test_string_construction<const R: usize, const C: usize>(layout: Layout<R, C>) {
			let layout_string = "
			___Layer 0___
					0       1       2 
			0| LS1_10  LS2_10  LS3_10 
			1| LS4_10    A_10    D_00 
			
			___Layer 1___
					0       1       2 
			0| LST1_0_10    E_10    A_10 
			1|   E_10    A_10    E_10 
			
			___Layer 2___
					0       1       2 
			0|   __10  LST2_0_10    __10 
			1|   __10    __10    __10 
			
			___Layer 3___
					0       1       2 
			0|   __10    __10  LST3_0_10 
			1|   __10    __10    __10 
			
			___Layer 4___
					0       1       2 
			0|   __10    __10    __10 
			1| LST4_0_10    __10    __10 
			
			";
			let layout_from_string = Layout::try_from(layout_string).unwrap();
			// println!("layout from string\n{:b}", layout_from_string.clone());
			assert_eq!(layout_from_string, layout);
		}
		test_string_construction::<2, 3>(layout);
	}
	
	#[test]
	fn test_incorrect_ls() {
		let test_str = "
		___Layer 0___
		A_10 B_10 LS1_10
		___Layer 1___
		C_10 LS2_10 LST1_0_10 
		___Layer 2___
		LST2_1_10 E_10 F_10
		";
		let test_layout = Layout::<1, 3>::try_from(test_str);
		match test_layout {
			Ok(_v) => (),
			Err(e) => assert_eq!(e, AlcError::LayoutLayerSwitchError(vec![
				(LayoutPosition::new(1, 0, 1), LayoutPosition::new(2, 0, 1))]))
		};
	}

	#[test]
	fn test_incorrect_symm() {
		let test_str = "
		___Layer 0___
		A_11 B_10 C_10
		";
		let test_layout = Layout::<1, 3>::try_from(test_str);
		match test_layout {
			Ok(_v) => (),
			Err(e) => assert_eq!(e, AlcError::LayoutSymmetryError(vec![(LayoutPosition::from_tuple((0, 0, 0)), LayoutPosition::from_tuple((0, 0, 2)))])),
		};
	}

	#[test]
	fn test_keycode_path_map () {
		let layout = Layout::<1, 4>::try_from("
			___Layer 0___
			LS1_10 B_10 C_10 LS1_10
			___Layer 1___
			LST1_0_10 LS2_10 H_10 LST1_0_10
			___Layer 2___
			A_10 LST2_1_10 H_10 C_10
		").unwrap();
		println!("{:#}", layout);	
	}

	// #[test]
	// fn test_swap_two() {
	// 	let mut layout = Layout::<1, 4>::try_from("
	// 		___Layer 0___
	// 		A_10 B_10 C_10 LS1_10
	// 		___Layer 1___
	// 		D_10 E_10 H_10 LS1_10
	// 	").unwrap();
	// 	println!("{}", layout);
	// 	unsafe { 
	// 		let k1 = layout.get_mut_from_layout_position(&LayoutPosition::for_layout(0, 0, 0)).unwrap() as *mut KeycodeKey;
	// 		let k2 = layout.get_mut_from_layout_position(&LayoutPosition::for_layout(0, 0, 2)).unwrap() as *mut KeycodeKey;
	// 		swap_two(k1, k2) 
	// 	};
	// 	assert_eq!(layout.get(0, 0, 0).unwrap().value(), _C);
	// 	assert_eq!(layout.get(0, 0, 2).unwrap().value(), _A);
		
	// 	// unsafe {layout.swap_two(&LayoutPosition::for_layout(0, 0, 1), &LayoutPosition::for_layout(1, 0, 2)) };
	// 	// assert_eq!(layout.get(0, 0, 1).unwrap().value(), _H);
	// 	// assert_eq!(layout.get(1, 0, 2).unwrap().value(), _B);
	// 	println!("{}", layout);
	// }

	#[test]
	fn test_swap() {
		let mut layout = Layout::<1, 4>::try_from("
			___Layer 0___
			A_10 B_10 C_10 LS1_10
			___Layer 1___
			D_10 E_10 H_10 LST1_0_10
		").unwrap();
		println!("{}", layout);
		layout.swap(&LayoutPosition::new(0, 0, 0), &LayoutPosition::new(0, 0, 2));
		assert_eq!(layout.get(0, 0, 0).unwrap().value(), _C);
		assert_eq!(layout.get(0, 0, 2).unwrap().value(), _A);
		
		layout.swap(&LayoutPosition::new(0, 0, 1), &LayoutPosition::new(1, 0, 2));
		assert_eq!(layout.get(0, 0, 1).unwrap().value(), _H);
		assert_eq!(layout.get(1, 0, 2).unwrap().value(), _B);

		layout.swap(&LayoutPosition::new(0, 0, 3), &LayoutPosition::new(0, 0, 2));
		assert_eq!(layout.get(0, 0, 3).unwrap().value(), _A);
		assert_eq!(layout.get(0, 0, 2).unwrap().value(), _LS(1));
		assert_eq!(layout.get(1, 0, 3).unwrap().value(), _B);
		assert_eq!(layout.get(1, 0, 2).unwrap().value(), _LST(1, 0));
		println!("{}", layout);

		let mut layout = Layout::<1, 4>::try_from("
			___Layer 0___
			A_10 B_11 C_11 LS1_10
			___Layer 1___
			D_10 E_10 H_10 LST1_0_10
		").unwrap();
		layout.swap(&LayoutPosition::new(0, 0, 1), &LayoutPosition::new(0, 0, 2));
		assert_eq!(layout.get(0, 0, 1).unwrap().value(), _C);
		assert_eq!(layout.get(0, 0, 2).unwrap().value(), _B);

		layout.swap(&LayoutPosition::new(0, 0, 1), &LayoutPosition::new(1, 0, 2));
		assert_eq!(layout.get(0, 0, 1).unwrap().value(), _H);
		assert_eq!(layout.get(0, 0, 2).unwrap().value(), _E);
		assert_eq!(layout.get(1, 0, 1).unwrap().value(), _B);
		assert_eq!(layout.get(1, 0, 2).unwrap().value(), _C);
		println!("{}", layout);
	}

	#[test]
	fn test_replace() {
		let mut layout = Layout::<1, 4>::try_from("
			___Layer 0___
			A_10 B_11 C_11 LS1_10
			___Layer 1___
			D_10 E_10 E_10 LST1_0_10
		").unwrap();
		// layout.replace(&LayoutPosition::for_layout(0, 0, 3), _E);
		// layout.replace(&LayoutPosition::for_layout(0, 0, 0), _E);
		layout.replace(&LayoutPosition::new(1, 0, 1), _C);
		assert_eq!(layout.get(1, 0, 1).unwrap().value(), _C);
	}

	#[test]
	fn test_ngram_to_sequences() {
		let layout = Layout::<1, 4>::try_from("
			___Layer 0___
			A_10 E_11 C_11 LS1_10
			___Layer 1___
			D_10 E_10 A_10 LST1_0_10
		").unwrap();
		let seqs = layout.ngram_to_sequences(&Ngram::new(vec![_A, _E])).unwrap();
		// println!("{:?}", seqs);
		assert_eq!(seqs.len(), 4);
		let seq1 = LayoutPositionSequence::from_tuples(vec![(0, 0, 0), (0, 0, 1)]);
		assert!(seqs.contains(&seq1));
		let seq2 = LayoutPositionSequence::from_tuples(vec![(0, 0, 3), (1, 0, 2), (0, 0, 1)]);
		assert!(seqs.contains(&seq2));
		let seq3 = LayoutPositionSequence::from_tuples(vec![(0, 0, 0), (0, 0, 3), (1, 0, 1)]);
		assert!(seqs.contains(&seq3));
		let seq4 = LayoutPositionSequence::from_tuples(vec![(0, 0, 3), (1, 0, 2), (0, 0, 3), (1, 0, 1)]);
		assert!(seqs.contains(&seq4));
		let seqs2 = layout.ngram_to_sequences(&Ngram::new(vec![_A, _E, _A, _E])).unwrap();
		assert_eq!(seqs2.len(), 16);

		let layout2 = Layout::<1, 4>::try_from("
			___Layer 0___
			A_10 B_11 C_11 LS1_10
			___Layer 1___
			D_10 E_10 A_10 LST1_0_10
		").unwrap();
		let seqs2 = layout2.ngram_to_sequences(&Ngram::new(vec![_A, _B, _C, _D, _E])).unwrap();
		let seq2_1 = LayoutPositionSequence::from_tuples(vec![(0, 0, 0), (0, 0, 1), (0, 0, 2), (0, 0, 3), (1, 0, 0), (0, 0, 3), (1, 0, 1)]);
		assert!(seqs2.contains(&seq2_1));
		let seq2_2 = LayoutPositionSequence::from_tuples(vec![(0, 0, 3), (1, 0, 2), (0, 0, 1), (0, 0, 2), (0, 0, 3), (1, 0, 0), (0, 0, 3), (1, 0, 1)]);
		assert!(seqs2.contains(&seq2_2));
	}
	
}