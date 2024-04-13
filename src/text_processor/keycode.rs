pub fn placeholder() -> i32 {
	5
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn placeholder() {
		assert_eq!(super::placeholder(), 5)
	}
}
