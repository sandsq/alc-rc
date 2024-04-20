use strum::IntoEnumIterator;
use strum_macros;
use std::fmt;



#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, strum_macros::Display, strum_macros::EnumString, strum_macros::EnumIter)]
pub enum Keycode {
	_A,
	_B,
	_C,
	_D,
	_E,
	_F,
	_G,
	_H,
	_I,
	_J,
	_K,
	_L,
	_M,
	_N,
	_O,
	_P,
	_Q,
	_R,
	_S,
	_T,
	_U,
	_V,
	_W,
	_X,
	_Y,
	_Z,
	_1,
	_2,
	_3,
	_4,
	_5,
	_6,
	_7,
	_8,
	_9,
	_ZER,
	_SPC,
	_SFT,
	_ENT,
	_COM,
	_DOT,
	_EXL,
	_HSH,
	_PER,
	_AMP,
	_AST,
	_LPR,
	_RPR,
	_MIN,
	_UND,
	_TCK,
	_QUO,
	_DQT,
	_SCN,
	_CLN,
	_LT,
	_RT,
	_EQL,
	_SLS,
	_QUE,
	_BSL,
	_LCB,
	_RCB,
	_LBR,
	_RBR,
	_LS(usize),
	_LST(usize, usize),
	_PLACEHOLDER,
	_NO,
}
use Keycode::*;

pub fn get_default_keycode_set() -> Vec<Keycode> {
	let mut keycodes: Vec<Keycode> = vec![];
	for keycode in Keycode::iter() {
		// there should be more valid keycodes than invalid ones when it comes to what is allowed to be randomized into a layout
		match keycode {
			_LS(i) => (),
			_LST(i, j) => (),
			_PLACEHOLDER => (),
			_NO => (),
			_ => keycodes.push(keycode),
		}
	}
	keycodes
}
// impl fmt::Display for Keycode {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 		write!(f, "{}", self) // str::replace(self, "_", ""))
//     }
// }


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
			'0' => keycodes.push(_ZER),
			' ' => keycodes.push(_SPC),
			',' => keycodes.push(_COM),
			'.' => keycodes.push(_DOT),
			'`' => keycodes.push(_TCK),
			'-' => keycodes.push(_MIN),
			'—' => keycodes.push(_MIN), // this is technically different
			'_' => keycodes.push(_UND),
			'\'' => keycodes.push(_QUO), 
			'’' => keycodes.push(_QUO),
			'"' => keycodes.push(_DQT),
			'“' => keycodes.push(_DQT),
			'”' => keycodes.push(_DQT),
			'\n' => keycodes.push(_ENT),
			'!' => keycodes.push(_EXL),
			'#' => keycodes.push(_HSH),
			'%' => keycodes.push(_PER),
			'&' => keycodes.push(_AMP),
			'*' => keycodes.push(_AST),
			'(' => keycodes.push(_LPR),
			')' => keycodes.push(_RPR),
			';' => keycodes.push(_SCN),
			':' => keycodes.push(_CLN),
			'<' => keycodes.push(_LT),
			'>' => keycodes.push(_RT),
			'=' => keycodes.push(_EQL),
			'/' => keycodes.push(_SLS),
			'?' => keycodes.push(_QUE),
			'\\' => keycodes.push(_BSL),
			'{' => keycodes.push(_LCB),
			'}' => keycodes.push(_RCB),
			'[' => keycodes.push(_LBR),
			']' => keycodes.push(_RBR),
			_ => panic!("keycode for {} doesn't exist", c), //keycodes.push(_PLACEHOLDER),
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
