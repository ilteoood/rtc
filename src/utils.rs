use regex::Regex;
use std::cmp::{max, min};
use std::sync::LazyLock;

use crate::structs::{ChunkUnit, LengthFunction};

static PARAGRAPH_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{2,}").unwrap());

pub fn get_length(text: &str, length_function: &Option<LengthFunction>) -> f64 {
  match length_function {
    Some(func) => func.call(text).unwrap(),
    None => text.chars().count() as f64,
  }
}

pub fn get_units(text: &str) -> Vec<ChunkUnit> {
  let mut units: Vec<ChunkUnit> = Vec::new();

  let mut last_index: usize = 0;
  for mat in PARAGRAPH_REGEX.find_iter(text) {
    let end = mat.start();
    let unit = text[last_index..end].trim().to_string();
    if !unit.is_empty() {
      units.push(ChunkUnit {
        unit,
        start: last_index,
        end,
      });
    }
    last_index = mat.end();
  }

  let last_unit = text[last_index..].trim();
  if !last_unit.is_empty() {
    units.push(ChunkUnit {
      unit: last_unit.to_string(),
      start: last_index,
      end: text.len(),
    });
  }

  units
}

pub fn can_fit_all_units(
  chunk_units: &[ChunkUnit],
  length_function: &Option<LengthFunction>,
  chunk_size: usize,
  joiner_len: f64,
) -> bool {
  let chunk_size_f64 = chunk_size as f64;
  let all_fit = chunk_units
    .iter()
    .all(|u| get_length(&u.unit, length_function) <= chunk_size_f64);

  let total_length = chunk_units.iter().enumerate().fold(0_f64, |acc, (i, u)| {
    acc + get_length(&u.unit, length_function) + if i > 0 { joiner_len } else { 0_f64 }
  });

  all_fit && total_length <= chunk_size_f64
}

pub fn chunk_by_character(
  current_text: &str,
  chunk_size: usize,
  length_function: &Option<LengthFunction>,
  chunk_overlap: usize,
  start_offset: usize,
) -> Vec<ChunkUnit> {
  let mut start = 0;
  let text_len = current_text.chars().count();
  let mut chunks: Vec<ChunkUnit> = Vec::new();

  while start < text_len {
    // Binary search for the largest end such that get_length(current_text.slice(start, end)) <= chunk_size
    let mut low = start + 1;
    let mut high = text_len;
    let mut best_end = start + 1;

    while low <= high {
      let mid = (low + high) / 2;
      let len = get_length(&current_text[start..mid], length_function);
      if len <= chunk_size as f64 {
        best_end = mid;
        low = mid + 1;
      } else {
        high = mid - 1;
      }
    }

    // Ensure at least one character per chunk
    if best_end == start {
      best_end = min(start + 1, text_len);
    }

    chunks.push(ChunkUnit {
      unit: current_text[start..best_end].to_string(),
      start: start_offset + start,
      end: start_offset + best_end,
    });

    if best_end >= text_len {
      break;
    }

    if chunk_overlap > 0 && best_end > start {
      start = max(best_end - chunk_overlap, start + 1);
    } else {
      start = best_end;
    }
  }

  chunks
}

pub fn chunk_by_greedy_sliding_window(
  chunk_units: &[ChunkUnit],
  length_function: &Option<LengthFunction>,
  joiner_len: f64,
  chunk_size: usize,
  joiner: &str,
  chunk_overlap: usize,
) -> Vec<ChunkUnit> {
  let mut i = 0;
  let n = chunk_units.len();
  let mut chunks: Vec<ChunkUnit> = Vec::new();
  let floating_chunk_size = chunk_size as f64;

  while i < n {
    let mut current_len = 0_f64;
    let mut first = true;
    let mut j = i;

    // Find the maximal window [i, j) that fits
    while j < n {
      let unit_len = get_length(&chunk_units[j].unit, length_function);
      let simulated_len = current_len + if first { 0_f64 } else { joiner_len } + unit_len;

      if simulated_len > floating_chunk_size && j > i {
        if j > i {
          break;
        } else if j == i {
          j += 1;
          break;
        }
        break;
      }

      current_len = simulated_len;
      first = false;
      j += 1;
    }

    if j > i {
      let chunk_str: String = chunk_units[i..j]
        .iter()
        .map(|u| u.unit.clone())
        .collect::<Vec<String>>()
        .join(joiner);

      chunks.push(ChunkUnit {
        unit: chunk_str,
        start: chunk_units[i].start,
        end: chunk_units[j - 1].end,
      });
    }

    // Advance window
    if chunk_overlap > 0 && j - i > 0 {
      i += std::cmp::max(1, j - i - chunk_overlap);
    } else {
      i = j;
    }
  }

  chunks
}
