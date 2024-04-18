use core::num;
use std::error::Error;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum AlcError {
	#[error(transparent)]
	ParseError(#[from] strum::ParseError),
	#[error(transparent)]
	ParseIntError(#[from] num::ParseIntError),
	#[error(transparent)]
	Array2DError(#[from] array2d::Error),
	#[error{transparent}]
	Regex(#[from] regex::Error),

	#[error("{0} cannot be parsed into a KeycodeKey, {1}")]
	InvalidKeycodeKeyFromString(String, String), // second param tries to describe what is invalid

	#[error("position ({0}, {1}) is marked as symmetric but its corresponding symmetric position ({2}, {3}) is not")]
	SymmetryError(usize, usize, usize, usize),
	#[error("expected {0} rows but tried to create {1} rows instead")]
	RowMismatchError(usize, usize),
	#[error("expected {0} cols but tried to create {1} cols instead")]
	ColMismatchError(usize, usize),
	#[error("layer string contains one more row than expected suggesting a column index header row its format is invalid {0}")]
	FromStringHeaderError(String),

	#[error("layer {0} is not reachable, check to make sure LS{0} exists in your layout and does not require first accessing a higher layer")]
	LayerAccessError(usize),

}
