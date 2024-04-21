use core::num;
use std::path::PathBuf;

use crate::keyboard::LayoutPosition;
use crate::text_processor::keycode::Keycode;
use crate::text_processor::ngram::Ngram;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum AlcError {
	#[error(transparent)]
	ParseError(#[from] strum::ParseError),
	#[error(transparent)]
	ParseIntError(#[from] num::ParseIntError),
	#[error(transparent)]
	Array2DError(#[from] array2d::Error),
	#[error(transparent)]
	RegexError(#[from] regex::Error),

	#[error("{0} cannot be parsed into a KeycodeKey, {1}")]
	InvalidKeycodeKeyFromString(String, String), // second param tries to describe what is invalid

	#[error("trying to add an ngram of length {0} to a holder with ngrams of length {1}, the ngram lengths must match")]
	NgramMatchError(usize, usize),

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
	
	#[error("layer switches are disjointed, they should be above / below each other in the corresponding layers: {0:?}")]
	LayoutLayerSwitchError(Vec<(LayoutPosition, LayoutPosition)>),
	#[error("symmetric keys are disjointed: {0:?}")]
	LayoutSymmetryError(Vec<(LayoutPosition, LayoutPosition)>),

	#[error("expected {0} to be a directory")]
	ExpectedDirectoryError(PathBuf),

	#[error("ngram {0} cannot be typed on the layout")]
	UntypeableNgramError(Ngram),
	#[error("the number of dataset weights {0} must match the number of datasets {1}")]
	DatasetWeightsMismatchError(usize, usize),

	#[error("could not find valid swap after {0} tries, {1}")]
	SwapFallbackError(u32, String),

	#[error("pathmap says {0} should be at {1}, but found {2} instead")]
	IncorrectPathmapError(Keycode, LayoutPosition, Keycode),
	#[error("found {0} at {1} in the keymap, but path to the location is not present in pathmap")]
	IncompletePathmapError(Keycode, LayoutPosition),
}