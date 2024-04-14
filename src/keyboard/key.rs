use crate::text_processor::keycode::Keycode::{self, *};

pub trait Value {
	type Item;
	fn value(&self) -> Self::Item;
}
pub struct Key {
	value: Keycode,
	is_moveable: bool,
	is_symmetric: bool,
}
impl Key {
	fn from_keycode(k: Keycode) -> Self {
		Self { value: k, is_moveable: true, is_symmetric: false }
	}
}
impl Default for Key {
	fn default() -> Self {
		Self {
			value: _E,
			is_moveable: true,
			is_symmetric: false,
		}
	}
}
impl Value for Key {
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
		let k = Key::default();
		assert_eq!(k.value(), _E);
		assert_eq!(k.value, _E);
		assert_eq!(k.is_moveable, true);
		assert_eq!(k.is_symmetric, false);

		let b = Key::from_keycode(_B);
		assert_eq!(b.value, _B);
		assert_eq!(b.is_moveable, true);
		assert_eq!(b.is_symmetric, false);
	}
}