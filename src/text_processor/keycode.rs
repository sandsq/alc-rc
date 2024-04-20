use strum::IntoEnumIterator;
use strum_macros;
use std::{collections::HashMap, fmt};



#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, strum_macros::Display, strum_macros::EnumString, strum_macros::EnumIter)]
pub enum Keycode {
	_A, _B, _C, _D, _E,
	_F, _G, _H, _I, _J,
	_K, _L, _M, _N, _O,
	_P, _Q, _R, _S, _T,
	_U, _V, _W, _X, _Y,
	_Z,
	_1, _2, _3, _4, _5,
	_6, _7, _8, _9, _ZERO,
	_SPC,
	_SFT,
	_ENT,
	_COMM,
	_DOT,
	_EXLM,
	_HASH,
	_PERC,
	_AMPR,
	_ASTR,
	_LPRN,
	_RPRN,
	_MINS,
	_UNDS,
	_GRV,
	_QUOT,
	_DQUO,
	_SCLN,
	_COLN,
	_LT,
	_GT,
	_EQL,
	_SLSH,
	_QUES,
	_BSLS,
	_PIPE,
	_LCBR,
	_RCBR,
	_LBRC,
	_RBRC,
	_LS(usize),
	_LST(usize, usize),
	_PLACEHOLDER,
	_NO,
}
use Keycode::*;


/// (unshifted to shifted, shifted to shifted)
/// eg unshifted 1 becomes !, shifted ! becomes 1
fn shift_maps() -> (HashMap<Keycode, Keycode>, HashMap<Keycode, Keycode>) {
	let mut un_to_shifted: HashMap<Keycode, Keycode> = HashMap::new();
	let mut shifted_to_un: HashMap<Keycode, Keycode> = HashMap::new();
	let m = (_1, _EXLM);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_3, _HASH);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_5, _PERC);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_7, _AMPR);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_8, _ASTR);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_9, _LPRN);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_ZERO, _RPRN);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_MINS, _UNDS);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_QUOT, _DQUO);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_SCLN, _COLN);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_COMM, _LT);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_DOT, _GT);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_SLSH, _QUES);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_BSLS, _PIPE);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_LBRC, _LCBR);
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);
	let m = (_RBRC, _RCBR);	

	(un_to_shifted, shifted_to_un)
} 

pub fn get_default_keycode_set() -> Vec<Keycode> {
	let mut keycodes: Vec<Keycode> = vec![];
	for keycode in Keycode::iter() {
		// there should be more valid keycodes than invalid ones when it comes to what is allowed to be randomized into a layout
		match keycode {
			_LS(i) => (),
			_LST(i, j) => (),
			_PLACEHOLDER => (),
			_NO => (),
			_DQUO => (),
			_EXLM => (),
			_HASH => (),
			_PERC => (),
			_AMPR => (),
			_ASTR => (),
			_LT => (),
			_GT => (),
			_QUES => (),
			_PIPE => (),
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
			'0' => keycodes.push(_ZERO),
			' ' => keycodes.push(_SPC),
			',' => keycodes.push(_COMM),
			'.' => keycodes.push(_DOT),
			'`' => keycodes.push(_GRV),
			'-' => keycodes.push(_MINS),
			'—' => keycodes.push(_MINS), // this is technically different
			'_' => keycodes.push(_UNDS),
			'\'' => keycodes.push(_QUOT), 
			'’' => keycodes.push(_QUOT),
			'"' => {keycodes.push(_SFT); keycodes.push(_QUOT)},
			'“' => {keycodes.push(_SFT); keycodes.push(_QUOT)},
			'”' => {keycodes.push(_SFT); keycodes.push(_QUOT)},
			'\n' => keycodes.push(_ENT),
			'!' => {keycodes.push(_SFT); keycodes.push(_1)},
			'#' => {keycodes.push(_SFT); keycodes.push(_3)},
			'%' => {keycodes.push(_SFT); keycodes.push(_5)},
			'&' => {keycodes.push(_SFT); keycodes.push(_7)},
			'*' => {keycodes.push(_SFT); keycodes.push(_8)},
			'(' => keycodes.push(_LPRN),
			')' => keycodes.push(_RPRN),
			';' => keycodes.push(_SCLN),
			':' => keycodes.push(_COLN),
			'<' => {keycodes.push(_SFT); keycodes.push(_COMM)},
			'>' => {keycodes.push(_SFT); keycodes.push(_DOT)}
			'=' => keycodes.push(_EQL),
			'/' => keycodes.push(_SLSH),
			'?' => {keycodes.push(_SFT); keycodes.push(_SLSH)},
			'\\' => keycodes.push(_BSLS),
			'|' => {keycodes.push(_SFT); keycodes.push(_BSLS)},
			'{' => keycodes.push(_LCBR),
			'}' => keycodes.push(_RCBR),
			'[' => keycodes.push(_LBRC),
			']' => keycodes.push(_RBRC),
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
