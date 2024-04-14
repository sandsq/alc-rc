use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Index;
use super::keycode::{string_to_keycode, Keycode, Keycode::*};
use super::ngram::Ngram;

trait Frequencies {}
impl Frequencies for f32 {}

#[derive(Debug, PartialEq)]
pub struct NgramFrequencyHolder<const N: usize, T> {
	frequencies: HashMap<Ngram<N>, T>
}
impl<const N: usize, T> TryFrom<&str> for NgramFrequencyHolder<N, T> where T: From<u32> + std::ops::AddAssign<u32> {
	type Error = &'static str;
	fn try_from(s: &str) -> Result<NgramFrequencyHolder<N, T>, Self::Error> {
		let mut ngram_to_counts: HashMap<Ngram<N>, T> = HashMap::new();
		let keycodes = string_to_keycode(s);
		for i in 0..(keycodes.len() - N + 1) {
			let ngram = Ngram::<N>::try_from(&keycodes[i..i + N])?;
			*ngram_to_counts.entry(ngram).or_insert(0.into()) += 1;
		}

		Ok(NgramFrequencyHolder { frequencies: ngram_to_counts })
	}
}
impl<const N: usize, T> Index<Ngram<N>> for NgramFrequencyHolder<N, T> {
	type Output = T;
	fn index(&self, ngram: Ngram<N>) -> &Self::Output {
		&self.frequencies[&ngram]
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_frequency_holder_ab() {
		let ngram = Ngram::new([_A, _B]);
		let mut expected_ngram_to_counts: HashMap<Ngram<2>, u32> = HashMap::new();
		expected_ngram_to_counts.insert(ngram, 1);
		assert_eq!(NgramFrequencyHolder::try_from("ab").unwrap(), NgramFrequencyHolder { frequencies: expected_ngram_to_counts });
	}

	#[test]
	fn test_frequency_holder_abab() {
		let ngram = Ngram::new([_A, _B]);
		// let map: HashMap<Ngram<2>, u32> = HashMap::new();
		// let holder = NgramFrequencyHolder { frequencies: map };
		let holder2 = NgramFrequencyHolder::<2, u32>::try_from("abab").unwrap();
		let holder2_ab = holder2[ngram];
		assert_eq!(holder2_ab, 2);
		let holder2_ba = holder2[Ngram::new([_B, _A])];
		assert_eq!(holder2_ba, 1);
		let holder4 = NgramFrequencyHolder::<4, u32>::try_from("abab").unwrap();
		let holder4_val = holder4[Ngram::new([_A, _B, _A, _B])];
		assert_eq!(holder4_val, 1)
	}

}