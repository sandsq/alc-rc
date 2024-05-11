use strum_macros;
use std::collections::HashSet;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct KeycodeOptions {
	pub include_alphas: bool,
	pub include_numbers: bool,
	pub include_number_symbols: bool, // !@#$%^&*()
	pub include_brackets: bool, // ()[]{}
	pub include_misc_symbols: bool, // -/ etc.
	pub include_misc_symbols_shifted: bool, // _? etc.
	pub explicit_inclusions: Vec<Keycode>,
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
			explicit_inclusions: vec![_SPC, _SFT, _ENT, _TAB],
		}
	}
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, strum_macros::Display, strum_macros::EnumString, strum_macros::EnumIter, Serialize, Deserialize)]
pub enum Keycode {
	_NO,
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
	_EXLM, _AT, _HASH, _DLR, _PERC,
	_CIRC, _AMPR, _ASTR, _LPRN, _RPRN,
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
	_TAB,
	_PLACEHOLDER,
}
use Keycode::*;

pub fn generate_default_keycode_set(options: &KeycodeOptions) -> HashSet<Keycode> {
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
			_MINS, _EQL, _BSLS, _SCLN, _QUOT, _GRV, _SLSH, _LBRC, _RBRC,
		]));
	}
	if options.include_misc_symbols_shifted {
		keycodes.extend(&HashSet::from([
			_UNDS, _PLUS, _PIPE, _COLN, _DQUO, _TILD, _QUES, _LCBR, _RCBR,
		]));
	}
	if !options.explicit_inclusions.is_empty() {
		let ei = options.explicit_inclusions.iter().filter(|x| **x != _NO);
		keycodes.extend(ei);
	}
	
	keycodes
}


impl Keycode {
	fn to_char(self) -> Option<char> {
		let c = match self {
			_SPC => ' ',
			_ENT => '\n',
			_EXLM => '!',
			_AT => '@',
			_HASH => '#',
			_DLR => '$',
			_PERC => '%',
			_CIRC => '^',
			_AMPR => '&',
			_ASTR => '*',
			_LPRN => '(',
			_RPRN => ')',
			_COMM => ',',
			_DOT => '.',
			_MINS => '-',
			_UNDS => '_',
			_GRV => '`',
			_TILD => '~',
			_QUOT => '\'',
			_DQUO => '\"',
			_SCLN => ';',
			_COLN => ':',
			_LT => '<',
			_GT => '>',
			_EQL => '=',
			_PLUS => '+',
			_SLSH => '/',
			_QUES => '?',
			_BSLS => '\\',
			_PIPE => '|',
			_LCBR => '{',
			_RCBR => '}',
			_LBRC => '[',
			_RBRC => ']',
			_SFT => return None,
			_ => panic!("{} is not a valid explicit inclusion yet", self), //return None,
		};
		Some(c)
	}

	fn from_char(c: char, options: &KeycodeOptions) -> Vec<Keycode> {
		let mut keycodes: Vec<Keycode> = vec![];
		for inclusion in options.explicit_inclusions.clone() {
			if let Some(k) = inclusion.to_char() {
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
			Err(_e) => match c_to_test {
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
					if options.include_brackets || options.include_misc_symbols_shifted {
						keycodes.push(_LCBR);
					} else {
						keycodes.push(_SFT); keycodes.push(_LBRC);
					}
				}, 
				'}' => {
					if options.include_brackets || options.include_misc_symbols_shifted {
						keycodes.push(_RCBR);
					} else {
						keycodes.push(_SFT); keycodes.push(_RBRC);
					}
				}, 
				'[' => keycodes.push(_LBRC),
				']' => keycodes.push(_RBRC),
				'…' => {
					keycodes.push(_DOT);
					keycodes.push(_DOT);
					keycodes.push(_DOT);
				},
				_ => {
					if !c_to_test.is_ascii() {
						println!("non-ascii character {} found", c_to_test);
					} else {
						panic!("keycode for {} doesn't exist. This will not be an error in the future and just use some placeholder keycode.", c);
					}
				},
			}
		}
		keycodes
	}

	pub fn from_string(s: &str, options: &KeycodeOptions) -> Vec<Keycode> {
		let mut keycodes: Vec<Keycode> = vec![];
		for c in s.chars() {
			keycodes.append(&mut Keycode::from_char(c, options));
		}
		keycodes
	}
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
		assert_eq!(Keycode::from_char('a', &KeycodeOptions::default()), res);
	}

	#[test]
	fn cap_e_to_keycode() {
		let res: Vec<Keycode> = vec![_SFT, _E];
		assert_eq!(Keycode::from_char('E', &KeycodeOptions::default()), res);
	}

	#[test]
	fn newline_to_keycode() {
		let res: Vec<Keycode> = vec![_ENT];
		assert_eq!(Keycode::from_char('\n', &KeycodeOptions::default()), res)
	}

	#[test]
	fn acb_to_keycodes() {
		let res: Vec<Keycode> = vec![_A, _SFT, _C, _B];
		assert_eq!(Keycode::from_string("aCb", &KeycodeOptions::default()), res);
	}

	#[test]
	fn test_default_keycodes() {
		let s = generate_default_keycode_set(&KeycodeOptions::default());
		println!("default keycodes {:?}", s);
		assert!(s.contains(&_SPC));
	}
	
}
