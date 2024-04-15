use std::fmt;
use std::error::Error;

use crate::text_processor::keycode::Keycode::{self, *};


#[derive(Debug, PartialEq, thiserror::Error)]
pub enum KeyError {
	#[error("{0} cannot be parsed into a KeycodeKey")]
	InvalidKeyFromString(String), // add another param to describe what exactly is invalid
}

pub trait Randomizeable {
	fn is_randomizeable(&self) -> bool;
}

pub trait KeyValue {
	type Item;
	fn value(&self) -> Self::Item;
}
#[derive(Debug, PartialEq, Clone)]
pub struct KeycodeKey {
	value: Keycode,
	is_moveable: bool,
	is_symmetric: bool,
}
impl KeycodeKey {
	pub fn from_keycode(k: Keycode) -> Self {
		Self { value: k, is_moveable: true, is_symmetric: false }
	}
	pub fn set_value(&mut self, new_value: Keycode) -> () {
		self.value = new_value
	}
	pub fn is_moveable(&self) -> bool {
		self.is_moveable
	}
	pub fn set_is_moveable(&mut self, new_moveability: bool) -> () {
		self.is_moveable = new_moveability
	}
	pub fn is_symmetric(&self) -> bool {
		self.is_symmetric
	}
	pub fn set_is_symmetric(&mut self, new_symmetric: bool) -> () {
		self.is_symmetric = new_symmetric
	}
}
impl TryFrom<&str> for KeycodeKey {
	type Error = Box<dyn Error>;
	fn try_from(key_string: &str) -> Result<Self, Self::Error> {
		let mut key = KeycodeKey::from_keycode(_NO);
		let mut key_details = key_string.split("_");
		// should check to make sure the string can actually be sliced
		// _NO displays as _ for less clutter, so account for that
		if &key_string[0..1] == "_" {
			key_details.next();
			key_details.next();
		} else if &key_string[0..2] == "LS" {
			let layer_target = &key_string[2..3].parse::<usize>()?;
			key.set_value(_LS(*layer_target));
			key_details.next();
		} else {
			if let Some(key_value_string) = key_details.next() {
				let key_value = Keycode::try_from(format!("_{key_value_string}").as_str())?;
				key.set_value(key_value);
			} else {
				return Err(Box::new(KeyError::InvalidKeyFromString(String::from(key_string))));
			}
		}
		if let Some(flags) = key_details.next() {
			// is_moveable flag and is_symmetric flag
			if flags.len() != 2 {
				return Err(Box::new(KeyError::InvalidKeyFromString(String::from(key_string))));	
			}
			let mut flags_iter = flags.chars();
			// should handle errors if they aren't 0 or 1, but lazy so skipping for now
			let move_flag: bool = flags_iter.next().unwrap().to_digit(10).unwrap() != 0;
			key.set_is_moveable(move_flag);
			let symm_flag: bool = flags_iter.next().unwrap().to_digit(10).unwrap() != 0;
			key.set_is_symmetric(symm_flag);
		} else {
			return Err(Box::new(KeyError::InvalidKeyFromString(String::from(key_string))));
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
impl fmt::Display for KeycodeKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let str_to_display = format!("{}", str::replace(&self.value.to_string(), "_", ""));
		let value_to_display = match self.value {
			_NO => format!("_"),
			_LS(i) => format!("LS{}", i),
			_ => str_to_display,
		};
		write!(f, "{:>3}", value_to_display)
    }
}
impl fmt::Binary for KeycodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m: u8 = self.is_moveable.into();
		let s: u8 = self.is_symmetric.into();

		let str_to_display = format!("{}", str::replace(&self.value.to_string(), "_", ""));
		let value_to_display = match self.value {
			_NO => format!("_"),
			_LS(i) => format!("LS{}", i),
			_ => str_to_display,
		};
        write!(f, "{:>3}_{}{}", value_to_display, m, s)
    }
}
impl Randomizeable for KeycodeKey {
	fn is_randomizeable(&self) -> bool {
		match self.value {
			_LS(i) => return false,
			_ => (),
		}
		match self.is_moveable {
			true => return true,
			false => return false,
		}
		match self.is_symmetric {
			true => return true,
			false => return false,
		}
	}
}


impl KeyValue for f32 {
	type Item = f32;
	fn value(&self) -> Self::Item {
		*self
	}
}

pub struct PhysicalKey {
	text: String,
	x: f32,
	y: f32,
}
impl KeyValue for PhysicalKey {
	type Item = String;
	fn value(&self) -> Self::Item {
		String::from(self.text.clone())
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

		let b = KeycodeKey::from_keycode(_B);
		assert_eq!(b.value, _B);
		assert_eq!(b.is_moveable, true);
		assert_eq!(b.is_symmetric, false);
	}
}