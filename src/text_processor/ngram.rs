use super::keycode::{string_to_keycode, Keycode, Keycode::*};

/// Holds a collection of keycodes corresponding to a string
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Ngram<const N: usize> {
	sequence: [Keycode; N],
}
impl<const N: usize> Ngram<N> {
    pub fn new(v: [Keycode; N]) -> Ngram<N> {
        Ngram { sequence: v }
    }

    pub fn len(self: Ngram<N>) -> usize {
        N
    }
}
impl<const N: usize> TryFrom<&[Keycode]> for Ngram<N> {
    type Error = &'static str;
    fn try_from(s: &[Keycode]) -> Result<Ngram<N>, Self::Error> {
        if s.len() != N {
            Err("Trying to convert a slice of size {s.len()} to an ngram of size {N}")
        } else {
            let mut sequence: [Keycode; N] = [_A; N];
            for i in 0..sequence.len() {
                sequence[i] = s[i];
            }
            Ok(Ngram { sequence: sequence })
        }
        
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
        let ngram = Ngram { sequence: [_SFT, _A, _B] };
        assert_eq!(ngram.len(), 3);
    }

}