
use strum_macros;
use std::fmt;



#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, strum_macros::Display, strum_macros::EnumString)]
pub enum Keycode {
	_A,
	_B,
	_C,
	_D,
	_E,
	_F,
	_H,
	_T,
	_SFT,
	_ENT,
	_LS(usize),
	_PLACEHOLDER,
	_NO,
}
use Keycode::*;
// impl fmt::Display for Keycode {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 		write!(f, "{}", self) // str::replace(self, "_", ""))
//     }
// }




struct ParseKeycodeError;

fn char_to_keycode(c: char) -> Vec<Keycode> {
	let mut keycodes: Vec<Keycode> = vec![];
	if c.is_uppercase() {
		keycodes.push(_SFT);
	}
	let c_to_test = c.to_lowercase().next().unwrap();
	// Standard letters can be converted into keycodes easily thanks to strum_macros
	match Keycode::try_from(format!("_{}", c_to_test.to_uppercase()).as_str()) {
		Ok(k) => keycodes.push(k),
		Err(e) => match c_to_test {
			'\n' => keycodes.push(_ENT),
			_ => keycodes.push(_PLACEHOLDER),
		}
	}
	keycodes
}

pub fn string_to_keycode(s: &str) -> Vec<Keycode> {
	let mut keycodes: Vec<Keycode> = vec![];
	for c in s.chars() {
		keycodes.append(&mut char_to_keycode(c));
	}
	keycodes
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_creating_enum_from_string() {
		assert_eq!(Keycode::try_from("_A").unwrap(), _A);
	}

	#[test]
	fn a_to_keycode() {
		let res: Vec<Keycode> = vec![_A];
		assert_eq!(char_to_keycode('a'), res);
	}

	#[test]
	fn cap_e_to_keycode() {
		let res: Vec<Keycode> = vec![_SFT, _E];
		assert_eq!(char_to_keycode('E'), res);
	}

	fn newline_to_keycode() {
		let res: Vec<Keycode> = vec![_ENT];
		assert_eq!(char_to_keycode('\n'), res)
	}

	#[test]
	fn acb_to_keycodes() {
		let res: Vec<Keycode> = vec![_A, _SFT, _C, _B];
		assert_eq!(string_to_keycode("aCb"), res);
	}

	
}
