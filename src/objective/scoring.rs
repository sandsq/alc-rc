use std::fmt::{self, Arguments};

use crate::keyboard::{LayoutPositionSequence, layer::Layer, layout::Layout};
use crate:: text_processor::frequency_holder::NgramFrequencyHolder;


#[derive(Debug, PartialEq, Clone)]
pub struct LayoutCandidate<const R: usize, const C: usize> {
	layout: Layout<R, C>,
	effort_layer: Layer<R, C, f32>
}
impl<const R: usize, const C: usize> LayoutCandidate<R, C> {

}

pub trait Score<const R: usize, const C: usize> {
	fn score_layout_position_sequence(&self, layout_candidate: LayoutCandidate<R, C>, layout_position_sequence: LayoutPositionSequence) -> f32;
}

pub struct SimpleScoreFunction {
	function: fn(Arguments) -> f32,
}

