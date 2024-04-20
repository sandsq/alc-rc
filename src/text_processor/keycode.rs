use strum::IntoEnumIterator;
use strum_macros;
use std::{collections::{HashMap, HashSet}, fmt};

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
	_BSPC,
	_SFT,
	_CTRL,
	_ALT,
	_GUI,
	_ENT,
	_COMM,
	_DOT,
	_EXLM,
	_AT,
	_HASH,
	_DLR,
	_PERC,
	_CIRC,
	_AMPR,
	_ASTR,
	_LPRN,
	_RPRN,
	_MINS, _UNDS,
	_GRV, _TILD,
	_QUOT,
	_DQUO,
	_SCLN,
	_COLN,
	_LT,
	_GT,
	_EQL,
	_PLUS,
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
	_UP,
	_RGHT,
	_DOWN,
	_LEFT,
	_PGUP,
	_END,
	_PGDN,
	_HOME,
	_PSCR,
	_DEL,
	_PLACEHOLDER,
	_NO,
}
use Keycode::*;

use crate::alc_error::AlcError;


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

pub struct KeycodeOptions {
	include_alphas: bool,
	include_numbers: bool,
	include_number_symbols: bool, // !@#$%^&*()
	include_brackets: bool, // ()[]{}
	include_misc_symbols: bool, // -/ etc.
	include_misc_symbols_shifted: bool, // _? etc.
	explicit_inclusion: HashSet<Keycode>,
}
impl Default for KeycodeOptions {
	fn default() -> Self {
		KeycodeOptions {
			include_alphas: true,
			include_numbers: false,
			include_number_symbols: false,
			include_brackets: false,
			include_misc_symbols: true,
			include_misc_symbols_shifted: false,
			explicit_inclusion: HashSet::from([_SPC, _SFT, _ENT]),
		}
	}
}

pub fn get_default_keycode_set(options: &KeycodeOptions) -> HashSet<Keycode> {
	let mut keycodes: HashSet<Keycode> = Default::default();
	if options.include_alphas {
		keycodes.extend(&HashSet::from([
			_A, _B, _C, _D, _E,
			_F, _G, _H, _I, _J,
			_K, _L, _M, _N, _O,
			_P, _Q, _R, _S, _T,
			_U, _V, _W, _X, _Y,
			_Z, _DOT, _COMM,]));
	}
	if options.include_numbers {
		keycodes.extend(&HashSet::from([
			_1, _2, _3, _4, _5,
			_6, _7, _8, _9, _ZERO,
		]));
	}
	if options.include_number_symbols {
		keycodes.extend(&HashSet::from([
			_EXLM, _AT, _HASH, _DLR, _PERC,
			_CIRC, _AMPR, _ASTR, _LPRN, _RPRN,
		]));
	}
	if options.include_brackets {
		keycodes.extend(&HashSet::from([
			_LPRN, _RPRN, _LBRC, _RBRC, _LCBR, _RCBR, _LT, _GT,
		]));
	}
	if options.include_misc_symbols {
		keycodes.extend(&HashSet::from([
			_MINS, _EQL, _BSLS, _SCLN, _QUOT, _GRV, _SLSH,
		]));
	}
	if options.include_misc_symbols_shifted {
		keycodes.extend(&HashSet::from([
			_UNDS, _PLUS, _PIPE, _COLN, _DQUO, _TILD, _QUES,
		]));
	}
	if options.explicit_inclusion.len() > 0 {
		keycodes.extend(&options.explicit_inclusion);
	}
	
	keycodes
}

fn keycode_to_char(k: Keycode) -> Option<char> {
	let c = match k {
		_SPC => ' ',
		_ENT => '\n',
		_ => return None,
	};
	Some(c)
}

fn char_to_keycode(c: char, options: &KeycodeOptions) -> Vec<Keycode> {
	let mut keycodes: Vec<Keycode> = vec![];
	for inclusion in options.explicit_inclusion.clone() {
		if let Some(k) = keycode_to_char(inclusion) {
			if k == c {
				keycodes.push(inclusion);
				return keycodes;
			}
		}
	}
	if c.is_uppercase() {
		keycodes.push(_SFT);
	}
	let c_to_test = c.to_lowercase().next().unwrap();
	match Keycode::try_from(format!("_{}", c_to_test.to_uppercase()).as_str()) {
		Ok(k) => keycodes.push(k),
		Err(e) => match c_to_test {
			'0' => keycodes.push(_ZERO),
			' ' => keycodes.push(_SPC),
			',' => keycodes.push(_COMM),
			'.' => keycodes.push(_DOT),
			'`' => keycodes.push(_GRV),
			'-' => keycodes.push(_MINS),
			'—' => keycodes.push(_MINS), // technicaly not - 
			'!' => {
				if options.include_number_symbols {
					keycodes.push(_EXLM);
				} else {
					keycodes.push(_SFT); keycodes.push(_1);
				}
			},
			'@' => {
				if options.include_number_symbols {
					keycodes.push(_AT);
				} else {
					keycodes.push(_SFT); keycodes.push(_2);
				}
			},
			'#' => {
				if options.include_number_symbols {
					keycodes.push(_HASH);
				} else {
					keycodes.push(_SFT); keycodes.push(_3);
				}
			},
			'$' => {
				if options.include_number_symbols {
					keycodes.push(_DLR);
				} else {
					keycodes.push(_SFT); keycodes.push(_4);
				}
			},
			'%' => {
				if options.include_number_symbols {
					keycodes.push(_PERC);
				} else {
					keycodes.push(_SFT); keycodes.push(_5);
				}
			},
			'^' => {
				if options.include_number_symbols {
					keycodes.push(_CIRC);
				} else {
					keycodes.push(_SFT); keycodes.push(_6);
				}
			},
			'&' => {
				if options.include_number_symbols {
					keycodes.push(_AMPR);
				} else {
					keycodes.push(_SFT); keycodes.push(_7);
				}
			},
			'*' => {
				if options.include_number_symbols {
					keycodes.push(_ASTR);
				} else {
					keycodes.push(_SFT); keycodes.push(_8);
				}
			},
			'(' => {
				if options.include_number_symbols || options.include_brackets {
					keycodes.push(_LPRN);
				} else {
					keycodes.push(_SFT); keycodes.push(_9);
				}
			},
			')' => {
				if options.include_number_symbols || options.include_brackets {
					keycodes.push(_RPRN);
				} else {
					keycodes.push(_SFT); keycodes.push(_ZERO);
				}
			},
			// _UNDS, _PLUS, _PIPE, _COLN, _DQUO, _TILD, _QUES,
			'_' => {
				if options.include_misc_symbols_shifted {
					keycodes.push(_UNDS);
				} else {
					keycodes.push(_SFT); keycodes.push(_MINS);
				}
			},
			'+' => {
				if options.include_misc_symbols_shifted {
					keycodes.push(_PLUS);
				} else {
					keycodes.push(_SFT); keycodes.push(_EQL);
				}
			},
			'|' => {
				if options.include_misc_symbols_shifted {
					keycodes.push(_PIPE);
				} else {
					keycodes.push(_SFT); keycodes.push(_BSLS);
				}
			},
			':' => {
				if options.include_misc_symbols_shifted {
					keycodes.push(_COLN);
				} else {
					keycodes.push(_SFT); keycodes.push(_SCLN);
				}
			},
			'"' | '“' | '”' =>  {
				if options.include_misc_symbols_shifted {
					keycodes.push(_DQUO);
				} else {
					keycodes.push(_SFT); keycodes.push(_QUOT);
				}
			},
			'~' => {
				if options.include_misc_symbols_shifted {
					keycodes.push(_TILD);
				} else {
					keycodes.push(_SFT); keycodes.push(_GRV);
				}
			},
			'?' => {
				if options.include_misc_symbols_shifted {
					keycodes.push(_QUES);
				} else {
					keycodes.push(_SFT); keycodes.push(_SLSH);
				}
			}, 
			'\'' => keycodes.push(_QUOT),
			'’' => keycodes.push(_QUOT),
			'\n' => keycodes.push(_ENT),
			';' => keycodes.push(_SCLN),
			'=' => keycodes.push(_EQL),
			'/' => keycodes.push(_SLSH),
			'\\' => keycodes.push(_BSLS),
			'<' => {
				if options.include_brackets {
					keycodes.push(_LT);
				} else {
					keycodes.push(_SFT); keycodes.push(_COMM);
				}
			}, 
			'>' => {
				if options.include_brackets {
					keycodes.push(_GT);
				} else {
					keycodes.push(_SFT); keycodes.push(_DOT);
				}
			},
			'{' => {
				if options.include_brackets {
					keycodes.push(_LCBR);
				} else {
					keycodes.push(_SFT); keycodes.push(_LBRC);
				}
			}, 
			'}' => {
				if options.include_brackets {
					keycodes.push(_RCBR);
				} else {
					keycodes.push(_SFT); keycodes.push(_RBRC);
				}
			}, 
			'[' => keycodes.push(_LBRC),
			']' => keycodes.push(_RBRC),
			_ => panic!("keycode for {} doesn't exist", c), //keycodes.push(_PLACEHOLDER),
		}
	}
	keycodes
}

pub fn string_to_keycode(s: &str, options: &KeycodeOptions) -> Vec<Keycode> {
	let mut keycodes: Vec<Keycode> = vec![];
	for c in s.chars() {
		keycodes.append(&mut char_to_keycode(c, options));
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
		assert_eq!(char_to_keycode('a', &KeycodeOptions::default()), res);
	}

	#[test]
	fn cap_e_to_keycode() {
		let res: Vec<Keycode> = vec![_SFT, _E];
		assert_eq!(char_to_keycode('E', &KeycodeOptions::default()), res);
	}

	fn newline_to_keycode() {
		let res: Vec<Keycode> = vec![_ENT];
		assert_eq!(char_to_keycode('\n', &KeycodeOptions::default()), res)
	}

	#[test]
	fn acb_to_keycodes() {
		let res: Vec<Keycode> = vec![_A, _SFT, _C, _B];
		assert_eq!(string_to_keycode("aCb", &KeycodeOptions::default()), res);
	}

	#[test]
	fn test_default_keycodes() {
		let s = get_default_keycode_set(&KeycodeOptions::default());
		println!("default keycodes {:?}", s);
		assert!(s.contains(&_SPC));
	}
	
}
