use core::num;
use std::num::{ParseFloatError, ParseIntError};
use std::path::PathBuf;

use crate::keyboard::LayoutPosition;
use crate::text_processor::keycode::Keycode;
use crate::text_processor::ngram::Ngram;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum AlcError {
	#[error(transparent)]
	ParseError(#[from] strum::ParseError),
	#[error("{0}, {1}")]
	ParseIntError(num::ParseIntError, String),
	#[error(transparent)]
	ParseFloatError(#[from] ParseFloatError),
	#[error(transparent)]
	Array2DError(#[from] array2d::Error),
	#[error(transparent)]
	RegexError(#[from] regex::Error),
	#[error(transparent)]
	TomlError(#[from] toml::de::Error),

	#[error("{0} cannot be parsed into a KeycodeKey, {1}")]
	InvalidKeycodeKeyFromString(String, String), // second param tries to describe what is invalid
	#[error("{0} is not a valid hand-finger combination, use \"Hand:Finger\" formatting, e.g., \"Left:Index\"")]
	InvalidPhalanxError(String),

	#[error("trying to add an ngram of length {0} to a holder with ngrams of length {1}, the ngram lengths must match")]
	NgramMatchError(usize, usize),

	#[error("position ({0}, {1}) is marked as symmetric but its corresponding symmetric position ({2}, {3}) is not")]
	SymmetryError(usize, usize, usize, usize),
	#[error("expected {0} rows but tried to create {1} rows instead")]
	RowMismatchError(usize, usize),
	#[error("expected {0} cols but tried to create {1} cols instead, the row with the error is {2}")]
	ColMismatchError(usize, usize, String),
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

	#[error("ngram {0} cannot be typed on the layout, check to make sure all keycodes are present in the layout or that the relevant options are checked")]
	UntypeableNgramError(Ngram),
	#[error("the number of dataset weights {0} must match the number of datasets {1}")]
	DatasetWeightsMismatchError(usize, usize),

	#[error("could not find valid swap after {0} tries, {1}")]
	SwapFallbackError(u32, String),

	#[error("pathmap says {0} should be at {1}, but found {2} instead")]
	IncorrectPathmapError(Keycode, LayoutPosition, Keycode),
	#[error("found {0} at {1} in the keymap, but path to the location is not present in pathmap")]
	IncompletePathmapError(Keycode, LayoutPosition),

	#[error("unsupported layout size {0:?}; valid sizes are {1:?}, use the next largest and block out positions")]
	UnsupportedSizeError((usize, usize), Vec<(usize, usize)>),

	#[error("{0}")]
	GenericError(String),
}
impl From<ParseIntError> for AlcError {
	fn from(value: ParseIntError) -> Self {
		AlcError::GenericError(format!("{}", value))
	}
}

impl serde::Serialize for AlcError {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
	  S: serde::ser::Serializer,
	{
	  serializer.serialize_str(self.to_string().as_ref())
	}
  }