#![deny(clippy::all)]

use napi::iterator::Generator;

use crate::structs::{ChunkIterator, ChunkResult, SplitOptions};

mod structs;
mod utils;

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn get_chunk(text: Vec<String>, start: Option<u32>, end: Option<u32>) -> Vec<String> {
  let mut current_length = 0;
  let mut start_index = Option::None;
  let mut start_offset = 0;
  let mut end_index = Option::None;
  let mut end_offset = 0;

  let start_value = start.unwrap_or(0);
  let end_value = end.unwrap_or(u32::MAX);

  for (i, line) in text.iter().enumerate() {
    let row_length = line.chars().count() as u32;
    current_length += row_length;

    if current_length >= start_value && start_index.is_none() {
      start_index = Some(i);
      start_offset = row_length - (current_length - start_value);
    }

    if current_length > end_value {
      end_index = Some(i);
      end_offset = row_length - (current_length - end_value);
      break;
    }
  }

  if start_index.is_none() {
    return vec![];
  }

  if end_index.is_none() {
    let last_index = text.len() - 1;
    end_index = Some(last_index);
    end_offset = text.get(last_index).unwrap().chars().count() as u32;
  }

  let start_index = start_index.unwrap();
  let end_index = end_index.unwrap();

  if start_index == end_index {
    let line = text.get(start_index).unwrap();
    return vec![line[start_offset as usize..end_offset as usize].to_string()];
  }

  if (start_index + 1) == end_index {
    let start_line = text.get(start_index).unwrap();
    let end_line = text.get(end_index).unwrap();
    return vec![
      start_line[start_offset as usize..].to_string(),
      end_line[..end_offset as usize].to_string(),
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect();
  }

  let mut result = Vec::from([text.get(start_index).unwrap()[start_offset as usize..].to_string()]);

  result.extend_from_slice(&text[start_index + 1..end_index]);

  result.push(text.get(end_index).unwrap()[..end_offset as usize].to_string());

  result.into_iter().filter(|s| !s.is_empty()).collect()
}

#[napi]
pub fn iterate_chunks(text: Vec<String>, split_options: Option<SplitOptions>) -> ChunkIterator {
  ChunkIterator::new(text, split_options)
}

#[napi]
pub fn split(text: Vec<String>, split_options: Option<SplitOptions>) -> Vec<ChunkResult> {
  let mut chunk_iterator = iterate_chunks(text, split_options);
  let mut chunk_result = Vec::new();

  while let Some(chunk) = chunk_iterator.next(None) {
    chunk_result.push(chunk);
  }

  chunk_result
}

#[cfg(test)]
mod tests_get_chunk {
  use super::*;

  #[test]
  fn test_should_return_the_full_string_if_no_start_end_provided() {
    assert_eq!(
      get_chunk(vec![String::from("abcdefgh")], Option::None, Option::None),
      vec![String::from("abcdefgh")]
    );
    assert_eq!(
      get_chunk(
        vec![String::from("abc"), String::from("def"), String::from("gh")],
        Option::None,
        Option::None
      ),
      vec![String::from("abc"), String::from("def"), String::from("gh")]
    )
  }

  #[test]
  fn test_should_return_substring_for_start_only() {
    assert_eq!(
      get_chunk(vec![String::from("abcdefgh")], Some(2), Option::None),
      vec![String::from("cdefgh")]
    );
    assert_eq!(
      get_chunk(
        vec![String::from("abc"), String::from("def"), String::from("gh")],
        Some(3),
        Option::None
      ),
      vec![String::from("def"), String::from("gh")]
    );
  }
}
