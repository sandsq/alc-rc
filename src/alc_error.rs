use core::num;
use std::error::Error;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum AlcError {
	#[error("{0} cannot be parsed into a KeycodeKey, {1}")]
	InvalidKeycodeKeyFromString(String, String), // add another param to describe what exactly is invalid
	#[error(transparent)]
	ParseError(#[from] strum::ParseError),
	#[error(transparent)]
	ParseIntError(#[from] num::ParseIntError),

	#[error("position ({0}, {1}) is marked as symmetric but its corresponding symmetric position ({2}, {3}) is not")]
	SymmetryError(usize, usize, usize, usize),
	#[error("expected {0} rows but tried to create {1} rows instead")]
	RowMismatchError(usize, usize),
	#[error("expected {0} cols but tried to create {1} cols instead")]
	ColMismatchError(usize, usize),
	#[error("layer string contains one more row than expected suggesting a column index header row its format is invalid {0}")]
	FromStringHeaderError(String),
}
