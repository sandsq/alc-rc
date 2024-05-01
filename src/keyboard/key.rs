use std::fmt;
use std::cmp::Ordering::*;


use crate::alc_error::AlcError;
use crate::text_processor::keycode::Keycode::{self, *};


pub trait Randomizeable {
	fn is_randomizeable(&self) -> bool;
}

pub trait KeyValue {
	type Item;
	fn value(&self) -> Self::Item;
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct KeycodeKey {
	value: Keycode,
	is_moveable: bool,
	is_symmetric: bool,
}
impl KeycodeKey {
	pub fn default_from_keycode(k: Keycode) -> Self {
		Self { value: k, is_moveable: true, is_symmetric: false }
	}
	pub fn set_value(&mut self, new_value: Keycode) {
		self.value = new_value
	}
	pub fn is_moveable(&self) -> bool {
		self.is_moveable
	}
	pub fn set_is_moveable(&mut self, new_moveability: bool) {
		self.is_moveable = new_moveability
	}
	pub fn is_symmetric(&self) -> bool {
		self.is_symmetric
	}
	pub fn set_is_symmetric(&mut self, new_symmetric: bool) {
		self.is_symmetric = new_symmetric
	}
	pub fn replace_with(&mut self, new_key: &KeycodeKey) {
		self.set_value(new_key.value());
		self.set_is_moveable(new_key.is_moveable());
		self.set_is_symmetric(new_key.is_symmetric());

	}
}
impl TryFrom<&str> for KeycodeKey {
	type Error = AlcError;
	fn try_from(key_string: &str) -> Result<Self, Self::Error> {
		let mut key = KeycodeKey::default_from_keycode(_NO);
		let mut key_details = key_string.split('_');
		// should check to make sure the string can actually be sliced
		// _NO displays as _ for less clutter, so account for that
		if &key_string[0..1] == "_" {
			key_details.next();
			key_details.next();
		} else if &key_string[0..3] == "LST" {
			let layer_target = &key_string[3..4].parse::<usize>()?;
			let layer_source = &key_string[5..6].parse::<usize>()?;
			key.set_value(_LST(*layer_target, *layer_source));
			key_details.next();
			key_details.next();
		} else if &key_string[0..2] == "LS" {
			let layer_target = &key_string[2..3].parse::<usize>()?;
			key.set_value(_LS(*layer_target));
			key_details.next();
		} else if let Some(key_value_string) = key_details.next() {
			let key_value = Keycode::try_from(format!("_{key_value_string}").as_str())?;
			key.set_value(key_value);
		} else {
			return Err(AlcError::InvalidKeycodeKeyFromString(String::from(key_string), String::from("keycode not found")));
		}
		
		if let Some(flags) = key_details.next() {
			// is_moveable flag and is_symmetric flag
			if flags.len() != 2 {
				return Err(AlcError::InvalidKeycodeKeyFromString(String::from(key_string), String::from("expected two bit flags")));	
			}
			let mut flags_iter = flags.chars();
			// .next() is guaranteed(?) to work here since we check that there are 2 flags
			// to_digit returns an option, should handle that somehow
			let move_flag: bool = flags_iter.next().unwrap().to_digit(10).unwrap() != 0;
			key.set_is_moveable(move_flag);
			let symm_flag: bool = flags_iter.next().unwrap().to_digit(10).unwrap() != 0;
			key.set_is_symmetric(symm_flag);

			if symm_flag {
				if let _LS(_layer_num) = key.value() {
					return Err(AlcError::InvalidKeycodeKeyFromString(String::from(key_string), String::from("don't set a layer switch key to be symmetric due to the additional complexity; this may change in the future")));
				}
				if let _LST(_l1, _l2) = key.value() {
					return Err(AlcError::InvalidKeycodeKeyFromString(String::from(key_string), String::from("don't set a layer switch key to be symmetric due to the additional complexity; this may change in the future")));
				}
			}

		} else {
			return Err(AlcError::InvalidKeycodeKeyFromString(String::from(key_string), String::from("no bit flags found")));
		}
		Ok(key)
	}
}
impl Default for KeycodeKey {
	fn default() -> Self {
		Self {
			value: _E,
			is_moveable: true,
			is_symmetric: false,
		}
	}
}
impl KeyValue for KeycodeKey {
	type Item = Keycode;
	fn value(&self) -> Self::Item {
		self.value
	}
}
/// {Keycode}_{Moveability}{Symmetry}
/// Keycodes have an up to 4 character representation in QMK, so {:>4} pads that (we ignore the KC_). Fix magic number later
impl fmt::Display for KeycodeKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let str_to_display = str::replace(&self.value.to_string(), "_", "");
		let value_to_display = match self.value {
			_NO => "_".to_string(),
			_LS(i) => format!("LS{}", i),
			_LST(_i, _j) => "_".to_string(), //format!("LST{}_{}", i, j),
			_ => str_to_display,
		};
		write!(f, "{:>4}", value_to_display)
    }
}
impl fmt::Binary for KeycodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m: u8 = self.is_moveable.into();
		let s: u8 = self.is_symmetric.into();

		let str_to_display = str::replace(&self.value.to_string(), "_", "");
		let value_to_display = match self.value {
			_NO => "_".to_string(),
			_LS(i) => format!("LS{}", i),
			_LST(_i, _j) => "_".to_string(), //format!("LST{}_{}", i, j),
			_ => str_to_display,
		};
        write!(f, "{:>4}_{}{}", value_to_display, m, s)
    }
}
impl Randomizeable for KeycodeKey {
	fn is_randomizeable(&self) -> bool {
		match self.value {
			_LS(_i) => return false,
			_LST(_i, _j) => return false,
			_ => (),
		}
		match self.is_moveable {
			true => (),
			false => return false,
		}
		match self.is_symmetric {
			true => false,
			false => true,
		}
	}
}

impl KeyValue for f64 {
	type Item = f64;
	fn value(&self) -> Self::Item {
		*self
	}
}

pub struct PhysicalKey {
	text: String,
	_x: f64,
	_y: f64,
}
impl KeyValue for PhysicalKey {
	type Item = String;
	fn value(&self) -> Self::Item {
		self.text.clone()
	}
}

#[derive(Debug, PartialEq, Clone, Copy, strum_macros::EnumString, strum_macros::Display)]
pub enum Hand {
	Left,
	Right,
	PlaceholderHand,
}
use Hand::*;

/// depending on your keyboard, you may be able to press the bottom left / bottom right corner key with the upper palm / joint of your pinkie finger, hence Joint
#[derive(Debug, PartialEq, Clone, Copy, strum_macros::EnumString, strum_macros::Display)]
pub enum Finger {
	Thumb,
	Index,
	Middle,
	Ring,
	Pinkie,
	Joint,
	PlaceholderFinger,
}
use Finger::*;
/// Thumb is the "biggest" finger, so a sequence of ascending fingers is an inner roll and a sequences of descending fingers is an outer roll. Joints are excluded from rolling since it's a bit awkward, but this may change in the future.
impl PartialOrd for Finger {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		match *self {
			Thumb => {
				match *other {
					Thumb => Some(Equal),
					PlaceholderFinger | Joint => None,
					_ =>  Some(Greater),
				}
			},
			Index => {
				match *other {
					Thumb => Some(Less), 
					Index => Some(Equal),
					PlaceholderFinger | Joint => None,
					_ => Some(Greater),
				}
			},
			Middle => {
				match *other {
					Thumb | Index => Some(Less),
					Middle => Some(Equal),
					PlaceholderFinger | Joint => None,
					_ => Some(Greater),
				}
			},
			Ring => {
				match *other {
					Thumb | Index | Middle => Some(Less),
					Ring => Some(Equal),
					PlaceholderFinger | Joint => None,
					_ => Some(Greater),
				}
			},
			Pinkie => {
				match *other {
					Thumb | Index | Middle | Ring => Some(Less),
					Pinkie => Some(Equal),
					Joint | PlaceholderFinger => None,
				}
			},
			Joint => {
				match *other {
					Joint => Some(Equal),
					_ => None,
				}
			}
			_ => None,
		}
	}
}
#[derive(Debug, PartialEq, Clone)]
pub struct PhalanxKey {
	pub hand: Hand,
	pub finger: Finger,
}
impl PhalanxKey {
	pub fn new(hand: Hand, finger: Finger) -> Self {
		PhalanxKey{ hand, finger }
	}
}
impl KeyValue for PhalanxKey {
	type Item = (Hand, Finger);
	fn value(&self) -> Self::Item {
		(self.hand, self.finger)
	}
}
impl Default for PhalanxKey {
	fn default() -> Self {
		PhalanxKey { hand: Left, finger: Index }
	}
}
impl fmt::Display for PhalanxKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let (hand, finger) = self.value();
		write!(f, "{:>2}:{}", &hand.to_string()[0..1], &finger.to_string()[0..1])
    }
}



#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn keycode_key() {
		let k = KeycodeKey::default();
		assert_eq!(k.value(), _E);
		assert_eq!(k.value, _E);
		assert_eq!(k.is_moveable, true);
		assert_eq!(k.is_symmetric, false);

		let b = KeycodeKey::default_from_keycode(_B);
		assert_eq!(b.value, _B);
		assert_eq!(b.is_moveable, true);
		assert_eq!(b.is_symmetric, false);
	}

	#[test]
	fn test_finger_comp() {
		assert!(Thumb == Thumb);
		assert!(Thumb > Index);
		assert!(Thumb > Middle);
		assert!(Thumb > Ring);
		assert!(Thumb > Pinkie);

		assert!(Index == Index);
		assert!(Index > Middle);
		assert!(Index > Ring);
		assert!(Index > Pinkie);

		assert!(Middle == Middle);
		assert!(Middle > Ring);
		assert!(Middle > Pinkie);

		assert!(Ring == Ring);
		assert!(Ring > Pinkie);

		assert!(Pinkie == Pinkie);
		assert!(Pinkie < Ring);
		assert!(Pinkie < Middle);
		assert!(Pinkie < Index);
		assert!(Pinkie < Thumb);

		// etc.

		assert!(!(Thumb > PlaceholderFinger));
		assert!(!(Thumb > Joint));
		assert!(!(Thumb < Joint));
	}
}