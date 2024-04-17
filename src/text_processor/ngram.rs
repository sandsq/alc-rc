use super::keycode::{string_to_keycode, Keycode, Keycode::*};

/// Holds a collection of keycodes corresponding to a string
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Ngram {
	sequence: Vec<Keycode>,
}
impl Ngram {
    pub fn new(v: Vec<Keycode>) -> Ngram {
        Ngram { sequence: v }
    }

    pub fn len(self: Ngram) -> usize {
        self.sequence.len()
    }
}

// impl From<&str> for Ngram {
//     fn from(s: &str) -> Ngram {
//         Ngram { sequence: string_to_keycode(s) }
//     }
// }



#[cfg(test)]
mod tests {
	use super::*;

	const DUMMY_LONG_STR: &str = 
            "Aaaaabbbb ccc
			dd e";
    const DUMMY_SHORT_STR: &str = "Ab";

    #[test]
    fn length_test() {
        let ngram = Ngram { sequence: vec![_SFT, _A, _B] };
        assert_eq!(ngram.len(), 3);
    }

}