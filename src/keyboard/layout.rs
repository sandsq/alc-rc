use std::ops::Index;
use rand::prelude::*;
use std::fmt;
use std::collections::{HashMap, HashSet, VecDeque};
use regex;
use std::mem::discriminant;

use crate::alc_error::AlcError;
use crate::text_processor::keycode::Keycode::{self, *};
use crate::text_processor::ngram::Ngram;
use super::key::{KeyValue, KeycodeKey};
use super::layer::Layer;
use super::{LayoutPosition, LayoutPositionSequence};

type KeycodePathMap = HashMap<Keycode, Vec<LayoutPositionSequence>>;
type CorrespondingPositions = Vec<(LayoutPosition, LayoutPosition)>;

/// A keyboard layout is a collection of layers of KeycodeKeys, plus additional info specifying how to navigate the layout, etc. (fill in later)
/// Layouts with multiple layers must have a way to access every layer.
/// For now, the only way to change layers is via a layer switch key. _LS(2) means that key switches to layer 2
#[derive(Debug, PartialEq, Clone)]
pub struct Layout<const R: usize, const C: usize> {
	layers: Vec<Layer<R, C, KeycodeKey>>,
	keycode_pathmap: KeycodePathMap,
}
impl<const R: usize, const C: usize> Layout<R, C> {
	// pub fn get(&self, layer_index: usize, row_index: usize, col_index: usize) -> Option<&KeycodeKey> {
	// 	self.layers.get(layer_index)?.get(row_index, col_index)
	// }
	// pub fn get_from_layout_position(&self, lp: LayoutPosition) -> Option<&KeycodeKey> {
	// 	self.layers.get(lp.layer_index)?.get(lp.row_index, lp.col_index)
	// }
	pub fn get_mut(&mut self, layer_index: usize, row_index: usize, col_index: usize) -> Option<&mut KeycodeKey> {
		self.layers.get_mut(layer_index)?.get_mut(row_index, col_index)
	}
	pub fn get_mut_from_layout_position(&mut self, lp: LayoutPosition) -> Option<&mut KeycodeKey> {
		self.layers.get_mut(lp.layer_index)?.get_mut(lp.row_index, lp.col_index)
	}
	pub fn paths_to_keycode(&self, k: Keycode) -> Option<&Vec<LayoutPositionSequence>> {
		if self.keycode_pathmap.is_empty() {
			panic!("Error for the developer! Somehow created a layout without creating a pathmap.");
		}
		self.keycode_pathmap.get(&k) // is there a possibility that the order could be non-deterministic? this could cause randomness despite fixed rng seed
	}
	pub fn symmetric_position(&self, lp: LayoutPosition) -> LayoutPosition {
		self.layers.first().unwrap().symmetric_position(lp) // would panic if layout is empty but that shouldn't normally be possible
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
		let mut layout = Layout { 
			layers,
			keycode_pathmap: KeycodePathMap::default() 
		};
		layout.generate_pathmap().unwrap();
		layout
	}


	/// Randomly places [Keycode]s from `valid_keycodes` into the layout. Keys can be blocked off with __00 (_NO keycode, not moveable, not symmetric) to account for (currently) unsupported sizes and non-standard form factors. Prefilled keys are not randomized so that layouts can be "seeded" with "good" initial layouts.
	pub fn randomize(&mut self, rng: &mut impl Rng, valid_keycodes: &[Keycode]) -> Result<(), AlcError> {
		let keycode_set: HashSet<Keycode> = self.keycode_pathmap.keys().cloned().collect();
		// println!("keycodes already in layout {:?}", keycode_set);
		let mut used_all_keycodes_flag = false;
		let mut valid_keycodes_all = VecDeque::from(valid_keycodes.to_owned());
		valid_keycodes_all.make_contiguous().shuffle(rng);
		let mut valid_keycodes_to_draw_from = VecDeque::from(valid_keycodes.to_owned());
		valid_keycodes_to_draw_from.make_contiguous().shuffle(rng);
		for layer_num in 0..self.layers.len() {
			let layer = self.layers.get_mut(layer_num).unwrap();
			// we want to fill out all valid keycodes over the entire layout, not just layer by layer
			(valid_keycodes_to_draw_from, used_all_keycodes_flag) = layer.randomize(&valid_keycodes_all, &valid_keycodes_to_draw_from, &keycode_set);
			if used_all_keycodes_flag {
				break;
			}
		}
		if !used_all_keycodes_flag {
			println!("Warning: the keycodes {:?} may not have made it into the layout since they were left over. This could happen if the layout is too small or if you prefilled a lot of immovable spots.", valid_keycodes_to_draw_from)
		}
		self.generate_pathmap()?;
		Ok(())
	}

	/// Within a layout there can be multiple ways to type a keycode, so there can be multiple ways to type an ngram. Keep track of all of these
	pub fn ngram_to_sequences(&self, ngram: &Ngram) -> Option<Vec<LayoutPositionSequence>> {
		let mut output_sequences_to_ngram: Vec<LayoutPositionSequence> = vec![];

		let ngram_iter = ngram.clone().into_iter();
		for keycode in ngram_iter {
			// if keycode == _AMPR {
			// 	panic!("found &");
			// }
			// println!("output sequences {:?} at start,  keycode {}", output_sequences_to_ngram, keycode);
			let sequences_to_keycode = match self.paths_to_keycode(keycode) {
				Some(p) => p,
				None => {
					// println!("Warning: keycode {} is not typeable by the layout:\n{:#}\nIf this is unexpected, there is a bug somewhere.", keycode, self);
					return None;
				},
			};
			if output_sequences_to_ngram.is_empty() {
				output_sequences_to_ngram = sequences_to_keycode.to_vec();
				// output_sequences_to_ngram = sequences_to_keycode.clone();
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
	pub fn swap(&mut self, p1: LayoutPosition, p2: LayoutPosition) -> Result<bool, AlcError> {
		// todo: make use of optimized keycode to position remapping computation where only the affected keycodes get are remapped

		if cfg!(debug_assertions) {
			self.verify_pathmap_correctness().unwrap();
			let (s1, s2) = self.verify_layout_correctness();
			if !s1.is_empty() || !s2.is_empty() {
				panic!("swapping {} with {}, layer switch issues: {:?}, symmetry issues: {:?}\n{}", p1, p2, s1, s2, self)
			}
		}

		#[allow(unused_assignments)]
		let mut swap_happened = false;

		// Bunch of checks for issues that should be easier to resolve in whatever calls swap rather than within swap.
		if p1 == p2 {
			// panic!("Error for the developer! Don't try to swap the same positions {} and {}.", p1, p2)
			return Ok(false);
		}
		let k1 = &self[p1];
		let k2 = &self[p2];
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
		let k1_clone = &self_clone[p1];
		let k2_clone = &self_clone[p2];
		if let _LS(target_layer) = k1.value() {
			// Layer switches need to be in the same layer position in the starting layer and the target layer. So, if the first position is a layer switch, its counterpart must be in:
			let p1_counterpart = LayoutPosition::new(target_layer, p1.row_index, p1.col_index);
			let p2_counterpart = LayoutPosition::new(target_layer, p2.row_index, p2.col_index);
			let k2_counterpart_clone = &self_clone[p2_counterpart];
			// I think these are harder to handle in the calling function, so just have nothing happen here
			if !k2_counterpart_clone.is_moveable() {
				// println!("Warning: attempted to swap a layer switch with position x: {} and found that x's corresponding position {} was not moveable. Doing nothing instead.", p2, p2_counterpart);
				return Ok(false);
			}
			if k2_counterpart_clone.is_symmetric() {
				// println!("Warning: attempted to swap a layer switch with position x: {} and found that x's corresponding position {} was symmetric, making the swap not valid. Doing nothing instead.", p2, p2_counterpart);
				return Ok(false);
			}
			// yeah gonna want to redo this section once I understand more
			k1.replace_with(k2_clone);
			let k2 = self.get_mut_from_layout_position(p2).unwrap();
			k2.replace_with(k1_clone);

			let k1_counterpart = self.get_mut_from_layout_position(p1_counterpart).unwrap();
			k1_counterpart.replace_with(k2_counterpart_clone);
			let k2_counterpart = self.get_mut_from_layout_position(p2_counterpart).unwrap();
			let k1_counterpart_clone = &self_clone[p1_counterpart];
			k2_counterpart.replace_with(k1_counterpart_clone);
			swap_happened = true;
		} else if k1_clone.is_symmetric() {
			let p1_counterpart = self_clone.symmetric_position(p1);
			if p2.col_index as f64 == (C as f64 - 1.0) / 2.0 {
				// println!("Warning: symmetric p1 {} is being swapped into the center column {}, meaning p1's counterpart {} has no where to go, doing nothing instead.", p1, p2, p1_counterpart);
				return Ok(false);
			}
			let p2_counterpart = self_clone.symmetric_position(p2);
			let k2_counterpart_clone = &self_clone[p2_counterpart];
			if !k2_counterpart_clone.is_moveable() {
				// println!("Warning: attempted to swap a symmetric key with position x: {} and found that x's corresponding position {} was not moveable. Doing nothing instead.", p2, p2_counterpart);
				return Ok(false);
			}
			// if let _LS(_target_layer) = k2_counterpart_clone.value() {
			// 	// println!("Warning: attempted symmetric swap but p2 {}'s counterpart {} is a layer switch. Doing nothing instead.", p2, p2_counterpart);
			// 	return false;
			// }
			if discriminant(&k2_counterpart_clone.value()) == discriminant(&_LS(0)) || discriminant(&k2_counterpart_clone.value()) == discriminant(&_LST(0, 0)) {
				return Ok(false);
			}
			k1.replace_with(k2_clone);
			let k2 = self.get_mut_from_layout_position(p2).unwrap();
			k2.replace_with(k1_clone);

			let k1_counterpart = self.get_mut_from_layout_position(p1_counterpart).unwrap();
			k1_counterpart.replace_with(k2_counterpart_clone);
			let k2_counterpart = self.get_mut_from_layout_position(p2_counterpart).unwrap();
			let k1_counterpart_clone = &self_clone[p1_counterpart];
			k2_counterpart.replace_with(k1_counterpart_clone);
			swap_happened = true;
		} else {
			k1.replace_with(k2_clone);
			let k2 = self.get_mut_from_layout_position(p2).unwrap();
			k2.replace_with(k1_clone);
			swap_happened = true;
		}
		self.generate_pathmap()?;
		// self.keycode_pathmap = keycode_path_map_from_layout(self.layers.clone()).unwrap();
		
		Ok(swap_happened)
	}

	pub fn replace(&mut self, p: LayoutPosition, value: Keycode) -> Result<bool, AlcError> {
		// make use of optimized keycode to position remapping computation where only the affected keycodes get are remapped
		if cfg!(debug_assertions) {
			// println!("verifying keycode path map during debugging");
			self.verify_pathmap_correctness().unwrap();
			let (s1, s2) = self.verify_layout_correctness();
			if !s1.is_empty() || !s2.is_empty() {
				panic!("replacing {} with {}, layer switch issues: {:?}, symmetry issues: {:?}\n{}", p, value, s1, s2, self)
			}
		}
		
		#[allow(unused_assignments)]
		let mut replace_happened = false;
		let k = self[p];
		if k.value() != _NO && self.keycode_pathmap[&k.value()].len() == 1 {
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
		self.get_mut_from_layout_position(p).unwrap().set_value(value);
		replace_happened = true;
		self.generate_pathmap()?;
		
		
		Ok(replace_happened)
	}
	pub fn generate_random_position(&self, rng: &mut impl Rng) -> LayoutPosition {
		let layer_limit = self.layers.len();
		LayoutPosition::new(rng.gen_range(0..layer_limit), rng.gen_range(0..R), rng.gen_range(0..C))
	}

	fn generate_random_moveable_position(&self, rng: &mut impl Rng) -> Option<LayoutPosition> {
		let fallback_count = 100;
		let mut p = self.generate_random_position(rng);
		let mut k = &self[p];
		let mut count = 0;
		while !k.is_moveable() {
			p = self.generate_random_position(rng);
			k = &self[p];
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
		let mut k1 = &self[p1];
		let mut p2 = self.generate_random_moveable_position(rng)?;
		let mut k2 = &self[p2];
		let mut count = 0;
		let fallback_count = 100;

		while discriminant(&k1.value()) == discriminant(&_LST(1, 2)) || k1.value() == _NO {
			p1 = self.generate_random_moveable_position(rng)?;
			k1 = &self[p1];

			count += 1;
			if count >= fallback_count {
				panic!("Error for developer! Only finding _NO keycodes.")
			}
		}
		count = 0;

		if let _LS(_i) = k1.value() {
			if k1.is_symmetric() {
				// return panic!("Error for the developer! Can't have a layer switch that is also symmetric due to additionaly complexity. This should be caught when reading in a Key from a string.");
			}
			while k2.is_symmetric() || (p1.layer_index != p2.layer_index) || std::mem::discriminant(&k2.value()) == std::mem::discriminant(&_LS(1)) || std::mem::discriminant(&k2.value()) == std::mem::discriminant(&_LST(1, 2)) {
				p2 = self.generate_random_moveable_position(rng)?;
				k2 = &self[p2];
				count += 1;
				if count >= fallback_count {
					// return None;
					// return Err(AlcError::SwapFallbackError(fallback_count, String::from("key 1 was a layer switch and either i) no non-symmetric key 2s could be found or ii) no key 2s could be found in the same layer or iii) not non-layer switch key 2s could be found")));
					panic!("key 1 was a layer switch and either i) no non-symmetric key 2s could be found or ii) no key 2s could be found in the same layer or iii) not non-layer switch key 2s could be found");
				}
			}
			Some((p1, p2))
		} else {
			while std::mem::discriminant(&k2.value()) == std::mem::discriminant(&_LS(1)) || (!k1.is_symmetric() && k2.is_symmetric()) || std::mem::discriminant(&k2.value()) == std::mem::discriminant(&_LST(1, 2)) {
				p2 = self.generate_random_moveable_position(rng)?;
				k2 = &self[p2];
				count += 1;
				if count >= fallback_count {
					// return Err(AlcError::SwapFallbackError(fallback_count, String::from("key 1 was not a layer switch but proper k2 could not be found")));
					// return None;
					panic!("key 1 was not a layer switch but proper k2 could not be found");
				}
			}
			Some((p1, p2))
		}
	}

	pub fn generate_valid_replace_position(&self, rng: &mut impl Rng) -> Option<LayoutPosition> {
		let mut p = self.generate_random_moveable_position(rng).unwrap();
		let mut k = &self[p];
		let mut paths = match self.keycode_pathmap.get(&k.value()) {
			Some(v) => v.clone(),
			None => match k.value() {
				_NO => {
					vec![LayoutPositionSequence::from_vector(vec![p]), LayoutPositionSequence::from_vector(vec![p])]
				},
				_ => return None,
			}
		};
		let fallback_count = 100;
		let mut count = 0;
		while paths.len() <= 1 || std::mem::discriminant(&k.value()) == std::mem::discriminant(&_LS(1)) || std::mem::discriminant(&k.value()) == std::mem::discriminant(&_LST(1, 2)) || !k.is_moveable() || k.is_symmetric() {
			p = self.generate_random_moveable_position(rng).unwrap();
			k = &self[p];
			paths = match self.keycode_pathmap.get(&k.value()) {
				Some(v) => v.clone(),
				None => match k.value() {
					_NO => {
						vec![LayoutPositionSequence::from_vector(vec![p]), LayoutPositionSequence::from_vector(vec![p])]
					},
					_ => return None,
				}
			};
			count += 1;
			if count >= fallback_count {
				return None;
			}
		}
		Some(p)
		
	}

	pub fn verify_pathmap_correctness(&self) -> Result<bool, AlcError> {
		let pathmap = &self.keycode_pathmap;
		let mut visited_positions: Vec<LayoutPosition> = vec![];
		for (pathmap_keycode, paths) in pathmap {
			for path in paths {
				let last_position = path.last().unwrap();
				visited_positions.push(*last_position);
				let found_keycode = &self[*last_position].value();
				if *pathmap_keycode != *found_keycode {
					return Err(AlcError::IncorrectPathmapError(*pathmap_keycode, *last_position, *found_keycode));
				}
			}
		}
		for layer_index in 0..self.layers.len() {
			for row_index in 0..R {
				for col_index in 0..C {
					let current_position = LayoutPosition::new(layer_index, row_index, col_index);
					let current_key_value = self[current_position].value();
					// if visited_positions.contains(&current_position) {
					// 	continue;
					// } else if discriminant(&current_key_value) == discriminant(&_LST(0, 0)) {
					// 	continue;
					// } else 
					if visited_positions.contains(&current_position) || discriminant(&current_key_value) == discriminant(&_LST(0, 0)) || current_key_value == _NO {
						continue;
					} else {
						return Err(AlcError::IncompletePathmapError(current_key_value, current_position));
					}
				}
			}
		}

		Ok(true)
	}

	
	/// layer switches, symmetry
	pub fn verify_layout_correctness(&self) -> (CorrespondingPositions, CorrespondingPositions) {
		let mut incorrect_layer_switch_locations: Vec<(LayoutPosition, LayoutPosition)> = vec![];
		let mut incorrect_symmetry_locations: Vec<(LayoutPosition, LayoutPosition)> = vec![];
		for layer_index in 0..self.layers.len() {
			for row_index in 0..R {
				for col_index in 0..C {
					let lp = LayoutPosition::new(layer_index, row_index, col_index);
					let key = &self[lp];
					if let _LS(target_layer) = key.value() {
						// println!("position {}", lp);
						let lp_corresponding = LayoutPosition::new(target_layer, row_index, col_index);
						let key_corresponding = &self[lp_corresponding];
						// if key_corresponding.value() != _NO {
						// 	panic!("For layer switch at {}, it's corresponding position {} should be blank, not {}", lp, lp_corresponding, key_corresponding);
						
						if let _LST(new_target_layer, _source_layer) = key_corresponding.value() {
							if new_target_layer != target_layer {
								panic!("LS's linked to each other should have the same layer number. For example, LS1 in layer 0 should be under LST(1, 0) in layer 1.")
							}
							// if source_layer != layer_index {
							// 	panic!("LS in the higher layer should point back down to its source layer. For example, LS1 in layer 0 should be under LS0 in layer 1.")
						
						} else {
							incorrect_layer_switch_locations.push((lp, lp_corresponding));
						}
					} else if let _LST(target_layer, source_layer) = key.value() {
						let lp_corresponding = LayoutPosition::new(source_layer, row_index, col_index);
						let key_corresponding = &self[lp_corresponding];
						if let _LS(new_target_layer) = key_corresponding.value() {
							if target_layer != new_target_layer {
								incorrect_layer_switch_locations.push((lp, lp_corresponding));		
							}
						}
					}
					if key.is_symmetric() {
						let lp_corresponding = self.symmetric_position(lp);
						let key_corresponding = &self[lp_corresponding];
						// println!("{:b} at {} is symmetric, corresponding key {:b} at {}", key, lp, key_corresponding, lp_corresponding);
						if !key_corresponding.is_symmetric() {
							incorrect_symmetry_locations.push((lp, lp_corresponding));
						}
					}

				}
			}
		}
		(incorrect_layer_switch_locations, incorrect_symmetry_locations)
	}



	pub fn generate_pathmap(&mut self) -> Result<(), AlcError> {
		let mut pathmap = KeycodePathMap::default();
		let mut layer_switch_pathmap =  KeycodePathMap::default();
		for (layer_num, layer) in self.layers.iter().enumerate() {
			for r in 0..R {
				for c in 0..C {
					let key = &layer[(r, c)];
					let key_value = key.value();
					if key_value == _NO {
						continue;
					}
					let layout_position = LayoutPosition::new(layer_num, r, c);
					let layout_position_sequence = LayoutPositionSequence::from_vector(vec![layout_position]);
					if layer_num == 0 {
						match key_value {
							_LS(_i) => layer_switch_pathmap.entry(key_value).or_insert(vec![]).push(layout_position_sequence),
							_ => pathmap.entry(key_value).or_insert(vec![]).push(layout_position_sequence),
						}
					} else if discriminant(&key_value) == discriminant(&_LST(0, 0)) {
						continue;
					} else {
						// check that layer_num is reachable. If layer is currently not reachable, could pass until after the rest of the layout is processed in case there is a downward layer move, but not going to implement that now since QMK does not recommend having layer switches like that
						let layer_switches = layer_switch_pathmap.clone();
						let sequences_to_reach_layer = match layer_switches.get(&_LS(layer_num)) {
							Some(v) => v.clone(),
							None => return Err(AlcError::LayerAccessError(layer_num)),
						};
						// let sequences_to_reach_layer = &layer_switch_pathmap.clone()[&_LS(layer_num)];
						for sequence in sequences_to_reach_layer {
							let mut sequence_clone = sequence.clone();
							sequence_clone.push(layout_position);
							match key_value {
								_LS(_i) => layer_switch_pathmap.entry(key_value).or_insert(vec![]).push(sequence_clone),
								_ => pathmap.entry(key_value).or_insert(vec![]).push(sequence_clone),
							}
							
						}
					}
				}
			}
		}
		for (key, value) in layer_switch_pathmap {
			pathmap.insert(key, value);
		}
		self.keycode_pathmap = pathmap;
		Ok(())
	}

	pub fn len(&self) -> usize {
		self.layers.len()
	}
	pub fn is_empty(&self) -> bool {
		self.layers.is_empty()
	}

	pub fn remove_unused_keys(&mut self, visited: &HashSet<LayoutPosition>) {
		for layer_index in 0..self.len() {
			for row_index in 0..R {
				for col_index in 0..C {
					let current_pos = LayoutPosition::new(layer_index, row_index, col_index);
					let k = self[current_pos];
					// if visited.contains(&current_pos) {
					// 	continue;
					// } else 
					if visited.contains(&current_pos) || !k.is_moveable() || k.is_symmetric() || discriminant(&k.value()) == discriminant(&Keycode::_LS(0)) || discriminant(&k.value()) == discriminant(&Keycode::_LST(0, 0,)) {
						continue;
					} else {
						self.get_mut(layer_index, row_index, col_index).unwrap().set_value(Keycode::_NO);
					}
				}
			}
		}
	}
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

		let mut layout = Layout { layers, keycode_pathmap: KeycodePathMap::default() };
		layout.generate_pathmap()?;

		let (v1, v2) = layout.verify_layout_correctness();
		if !v1.is_empty() {
			return Err(AlcError::LayoutLayerSwitchError(v1));
		}
		if !v2.is_empty() {
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

impl<const R: usize, const C: usize> Index<LayoutPosition> for Layout<R, C> {
	type Output = KeycodeKey;
	fn index(&self, index: LayoutPosition) -> &Self::Output {
		self.index((index.layer_index, index.row_index, index.col_index))
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
			let mut keys: Vec<&Keycode> = self.keycode_pathmap.keys().collect();
			keys.sort();
			for k in keys {
				let key_text = match k {
					_LS(i) => format!("_LS{}", i),
					_ => k.to_string(),
				};
				write!(f, "{}: ", key_text)?;
				for seq in self.keycode_pathmap[k].iter() {
					write!(f, "{}, ", seq)?;
				}
				writeln!(f)?;
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
			let mut keys: Vec<&Keycode> = self.keycode_pathmap.keys().collect();
			keys.sort();
			for k in keys {
				let key_text = match k {
					_LS(i) => format!("_LS{}", i),
					_ => k.to_string(),
				};
				write!(f, "{}: ", key_text)?;
				for seq in self.keycode_pathmap[k].iter() {
					write!(f, "{}, ", seq)?;
				}
				writeln!(f)?;
			}
		}
		Ok(())
    }
}


#[cfg(test)]
mod tests {
	use rand_chacha::ChaCha8Rng;

use super::*;

	#[test]
	fn test() {
		let mut rng = ChaCha8Rng::seed_from_u64(1);
		let mut layout = Layout::<2, 3>::init_blank(5);
		layout.get_mut(0, 1, 2).unwrap().set_value(_D);
		layout.get_mut(0, 1, 2).unwrap().set_is_moveable(false);
		layout.randomize(&mut rng, &vec![_A, _E]).unwrap();
		fn test_randomize<const R: usize, const C: usize>(layout: Layout<R, C>) {
			let expected_key = KeycodeKey::try_from("D_00").unwrap();
			assert_eq!(layout[(0, 1, 2)], expected_key);
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
			0|   __10    A_10    A_10 
			1|   E_10    A_10    E_10 
			
			___Layer 2___
					0       1       2 
			0|   __10    __10    __10 
			1|   __10    __10    __10 
			
			___Layer 3___
					0       1       2 
			0|   __10    __10    __10 
			1|   __10    __10    __10 
			
			___Layer 4___
					0       1       2 
			0|   __10    __10    __10 
			1|   __10    __10    __10 
			
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

	#[test]
	fn test_pathmap_correctness() {
		let mut layout = Layout::<3, 4>::try_from("
			___Layer 0___
			LS1_10 A_10 B_10 C_10
			E_10 Z_10 H_10 C_10
			M_10 E_10 B_10 D_10
			___Layer 1___
			__10 __10 J_10 L_00
			SFT_11 K_10 X_10 SFT_11
			M_10 E_10 B_10 D_10
			M_10 E_10 B_10 D_10
		").unwrap();
		assert!(layout.verify_pathmap_correctness().unwrap());
		layout.keycode_pathmap.remove(&_A);
		match layout.verify_pathmap_correctness() {
			Ok(_v) => panic!("x"),
			Err(e) => assert_eq!(e, AlcError::IncompletePathmapError(_A, LayoutPosition::new(0, 0, 1))),
		};
		let seq = LayoutPositionSequence::from_tuples(vec![(0, 0, 1)]);
		layout.keycode_pathmap.insert(_A, vec![seq]);
		layout.verify_pathmap_correctness().unwrap();

		layout.keycode_pathmap.get_mut(&_B).unwrap().pop();
		match layout.verify_pathmap_correctness() {
			Ok(_v) => panic!("x"),
			Err(e) => assert_eq!(e, AlcError::IncompletePathmapError(_B, LayoutPosition::new(1, 2, 2))), // pop removes the last element
		};
		layout.keycode_pathmap.get_mut(&_B).unwrap().push(LayoutPositionSequence::from_tuples(vec![(1, 2, 2)]));
		layout.verify_pathmap_correctness().unwrap();

		layout.get_mut(1, 2, 1).unwrap().set_value(_A);
		match layout.verify_pathmap_correctness() {
			Ok(_v) => panic!("x"),
			Err(e) => assert_eq!(e, AlcError::IncorrectPathmapError(_E, LayoutPosition::new(1, 2, 1), _A)),
		}
	}

	#[test]
	fn test_swap() -> Result<(), AlcError> {
		let mut layout = Layout::<1, 4>::try_from("
			___Layer 0___
			A_10 B_10 C_10 LS1_10
			___Layer 1___
			D_10 E_10 H_10 LST1_0_10
		").unwrap();
		println!("{}", layout);
		layout.swap(LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 2))?;
		assert_eq!(layout[(0, 0, 0)].value(), _C);
		assert_eq!(layout[(0, 0, 2)].value(), _A);
		
		layout.swap(LayoutPosition::new(0, 0, 1), LayoutPosition::new(1, 0, 2))?;
		assert_eq!(layout[(0, 0, 1)].value(), _H);
		assert_eq!(layout[(1, 0, 2)].value(), _B);

		layout.swap(LayoutPosition::new(0, 0, 3), LayoutPosition::new(0, 0, 2))?;
		assert_eq!(layout[(0, 0, 3)].value(), _A);
		assert_eq!(layout[(0, 0, 2)].value(), _LS(1));
		assert_eq!(layout[(1, 0, 3)].value(), _B);
		assert_eq!(layout[(1, 0, 2)].value(), _LST(1, 0));
		println!("{}", layout);

		let mut layout = Layout::<1, 4>::try_from("
			___Layer 0___
			A_10 B_11 C_11 LS1_10
			___Layer 1___
			D_10 E_10 H_10 LST1_0_10
		").unwrap();
		layout.swap(LayoutPosition::new(0, 0, 1), LayoutPosition::new(0, 0, 2))?;
		assert_eq!(layout[(0, 0, 1)].value(), _C);
		assert_eq!(layout[(0, 0, 2)].value(), _B);

		layout.swap(LayoutPosition::new(0, 0, 1), LayoutPosition::new(1, 0, 2))?;
		assert_eq!(layout[(0, 0, 1)].value(), _H);
		assert_eq!(layout[(0, 0, 2)].value(), _E);
		assert_eq!(layout[(1, 0, 1)].value(), _B);
		assert_eq!(layout[(1, 0, 2)].value(), _C);
		println!("{}", layout);
		Ok(())
	}

	#[test]
	fn test_replace() -> Result<(), AlcError> {
		let mut layout = Layout::<1, 4>::try_from("
			___Layer 0___
			A_10 B_11 C_11 LS1_10
			___Layer 1___
			D_10 E_10 E_10 LST1_0_10
		").unwrap();
		// layout.replace(&LayoutPosition::for_layout(0, 0, 3), _E);
		// layout.replace(&LayoutPosition::for_layout(0, 0, 0), _E);
		layout.replace(LayoutPosition::new(1, 0, 1), _C)?;
		assert_eq!(layout[(1, 0, 1)].value(), _C);
		Ok(())
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

	#[test]
	fn test_randomization() {
		let mut rng = ChaCha8Rng::seed_from_u64(1);
		println!("{}", rng.gen::<f64>());
		let mut score = 0.0;
		for i in 1..100 {
			score += i as f64 * &rng.gen() / (i as f64  + 1.0);
		}
		println!("{}", score);

		let valid_keycodes = vec![_A, _B, _C, _D, _E];
		let mut valid_keycodes_all = VecDeque::from(valid_keycodes.clone());
		
		valid_keycodes_all.make_contiguous().shuffle(&mut rng);
		println!("{:?}", valid_keycodes_all);
		

		let mut initial_layout = Layout::<4, 12>::init_blank(3);
		initial_layout.randomize(&mut rng, &vec![_F, _G, _A, _C, _B, _D, _E]).unwrap();
		println!("{}", initial_layout);
	}
	
}