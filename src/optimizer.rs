use std::fmt;

use crate::keyboard::{key::*, layout::*, layer::*};
use crate::text_processor::*;
use crate::objective::scoring::*;


pub struct LayoutOptimizer<const R: usize, const C: usize, S> where S: Score<R, C> {
	base_layout: Layout<R, C>,
	effort_layer: Layer<R, C, f32>,
	score_function: S,
}