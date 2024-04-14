use crate::text_processor::keycode::Keycode::{self, *};

pub trait Value {
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
impl Value for KeycodeKey {
	type Item = Keycode;
	fn value(&self) -> Self::Item {
		self.value
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn basic_key() {
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