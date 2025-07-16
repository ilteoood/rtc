use std::collections::VecDeque;

use napi::{bindgen_prelude::Function, iterator::Generator};

use crate::utils::{
  can_fit_all_units, chunk_by_character, chunk_by_greedy_sliding_window, get_length, get_units,
};

pub type LengthFunction<'a> = Function<'a, &'a str, f64>;

#[derive(PartialEq)]
#[napi]
pub enum ChunkStrategy {
  Paragraph,
}

#[napi(object)]
pub struct SplitOptions {
  pub chunk_size: Option<u32>,
  pub chunk_overlap: Option<u32>,
  pub length_function: Option<LengthFunction<'static>>,
  pub chunk_strategy: Option<ChunkStrategy>,
}

impl SplitOptions {
  pub fn default() -> Self {
    SplitOptions {
      chunk_size: Option::Some(512_u32),
      chunk_overlap: Option::Some(0_u32),
      length_function: Option::None,
      chunk_strategy: Option::None,
    }
  }
}

#[derive(PartialEq)]
pub struct ChunkUnit {
  pub unit: String,
  pub start: usize,
  pub end: usize,
}

#[napi(object)]
pub struct ChunkResult {
  pub text: Vec<String>,
  pub start: u32,
  pub end: u32,
}

#[napi(iterator)]
pub struct ChunkIterator {
  text: VecDeque<String>,
  split_options: SplitOptions,
  chunk_units: VecDeque<ChunkUnit>,
  chunk_unit_global_offset: u32,
  global_offset: u32,
  chunk_size: usize,
  chunk_overlap: usize,
}

impl ChunkIterator {
  pub fn new(text: Vec<String>, split_options: Option<SplitOptions>) -> Self {
    let split_options = split_options.unwrap_or(SplitOptions::default());
    let chunk_size = split_options.chunk_size.unwrap_or(512) as usize;
    let chunk_overlap = split_options.chunk_overlap.unwrap_or(0) as usize;

    ChunkIterator {
      text: VecDeque::from(text),
      split_options,
      chunk_units: VecDeque::new(),
      chunk_unit_global_offset: 0,
      global_offset: 0,
      chunk_size,
      chunk_overlap,
    }
  }

  fn generate_chunk_result(&mut self, chunk_unit: ChunkUnit) -> Option<ChunkResult> {
    let chunk_result = ChunkResult {
      text: vec![chunk_unit.unit],
      start: self.chunk_unit_global_offset + chunk_unit.start as u32,
      end: self.chunk_unit_global_offset + chunk_unit.end as u32,
    };
    Some(chunk_result)
  }
}

const JOINER: &str = "\n\n";

#[napi]
impl Generator for ChunkIterator {
  type Yield = ChunkResult;
  type Next = ();
  type Return = ();

  fn next(&mut self, _: Option<Self::Next>) -> Option<Self::Yield> {
    match self.chunk_units.pop_front() {
      Some(chunk_unit) => self.generate_chunk_result(chunk_unit),
      None => match self.text.pop_front() {
        Some(current_text) => {
          if current_text.is_empty() {
            self.chunk_unit_global_offset = self.global_offset;
            return self.generate_chunk_result(ChunkUnit {
              unit: String::new(),
              start: 0,
              end: 0,
            });
          }

          if self
            .split_options
            .chunk_strategy
            .is_some_and(|chunk_strategy| chunk_strategy == ChunkStrategy::Paragraph)
          {
            let chunk_units = get_units(&current_text);
            let joiner_len = get_length(JOINER, &self.split_options.length_function);

            if can_fit_all_units(
              &chunk_units,
              &self.split_options.length_function,
              self.chunk_size,
              joiner_len,
            ) {
              self.chunk_unit_global_offset = self.global_offset;
              self.chunk_units.extend(VecDeque::from(chunk_units));
            } else {
              let chunks = chunk_by_greedy_sliding_window(
                &chunk_units,
                &self.split_options.length_function,
                joiner_len,
                self.chunk_size,
                JOINER,
                self.chunk_overlap,
              );
              self.chunk_unit_global_offset = self.global_offset;
              self.chunk_units.extend(VecDeque::from(chunks));
            }
          } else {
            let chunks = chunk_by_character(
              &current_text,
              self.chunk_size,
              &self.split_options.length_function,
              self.chunk_overlap,
              self.global_offset as usize,
            );
            self.chunk_unit_global_offset = 0;
            self.chunk_units.extend(VecDeque::from(chunks));
          }

          self.global_offset += current_text.chars().count() as u32;
          let chunk_unit = self.chunk_units.pop_front().unwrap();
          self.generate_chunk_result(chunk_unit)
        }
        None => None,
      },
    }
  }
}
