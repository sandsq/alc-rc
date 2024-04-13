use std::str::FromStr;
use super::keycode::{string_to_keycode, Keycode, Keycode::*};

#[derive(Debug, PartialEq, Clone)]
pub struct Ngram {
	sequence: Vec<Keycode>,
}
impl Ngram {
    fn new(v: Vec<Keycode>) -> Ngram {
        Ngram {sequence: v}
    }

    fn append(&mut self, mut ngram: Ngram) -> () {
        self.sequence.append(&mut ngram.sequence)
    }
}
#[derive(Debug)]
pub struct ParseNgramError;
impl From<&str> for Ngram {
    fn from(s: &str) -> Ngram {
        Ngram { sequence: string_to_keycode(s) }
    }
}


#[cfg(test)]
mod tests {
	use super::*;

	const DUMMY_LONG_STR: &str = 
            "Aaaaabbbb ccc
			dd e";
    const DUMMY_SHORT_STR: &str = "Ab";

    #[test]
    fn short_to_ngram() {
        let ngram = Ngram { sequence: vec![_SFT, _A, _B] };
        assert_eq!(Ngram::from(DUMMY_SHORT_STR), ngram);
    }

    #[test]
    fn append_test() {
        let mut ngram = Ngram { sequence: vec![_SFT, _A, _B] };
        let ngram_appended = Ngram { sequence: vec![_SFT, _A, _B, _SFT, _A, _B] };
        ngram.append(ngram.clone());
        assert_eq!(ngram, ngram_appended);
    }

}