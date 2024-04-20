#[allow(unused)]

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
	un_to_shifted.insert(m.0, m.1);
	shifted_to_un.insert(m.1, m.0);

	(un_to_shifted, shifted_to_un)
} 