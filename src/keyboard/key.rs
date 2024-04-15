use std::fmt;

use crate::text_processor::keycode::Keycode::{self, *};

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
		// add row / column number later maybe
		write!(f, "{:>3}", str::replace(&self.value.to_string(), "_", ""))
    }
}
impl fmt::Binary for KeycodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m: u8 = self.is_moveable.into();
		let s: u8 = self.is_symmetric.into();

        write!(f, "{:>3}_{}{}", str::replace(&self.value.to_string(), "_", ""), m, s)
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