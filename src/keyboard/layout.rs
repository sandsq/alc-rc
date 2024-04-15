use array2d::{Array2D, Error as Array2DError};
use std::ops::Index;
use rand::prelude::*;
use std::error::Error;
use std::fmt;
use thiserror;

use crate::text_processor::keycode::Keycode::{self, *};
use super::key::{KeyValue, KeycodeKey, PhysicalKey};
use super::layer::Layer;
use super::LayoutPosition;

/// A keyboard layout is a collection of layers of KeycodeKeys, plus additional info specifying how to navigate the layout, etc. (fill in later)
/// Layouts with multiple layers must have a way to access every layer.
/// For now, the only way to change layers is via a layer switch key. _LS(2) means that key switches to layer 2
#[derive(Debug, PartialEq)]
pub struct Layout<const R: usize, const C: usize> {
	layers: Vec<Layer<R, C, KeycodeKey>>,
}
// impl<const R: usize, const C: usize> Layout<R, C> {
// 	pub fn init_blank(num_layers: usize) {
// 		let layers: Vec<Layer<R, C, KeycodeKey>> = vec![];
// 		for i in 0..num_layers {
// 			let layer = Layer::<R, C, KeycodeKey>::init_blank();
// 			layer
// 		}
// 	}
// }

#[cfg(test)]
mod tests {
	use super::*;


}