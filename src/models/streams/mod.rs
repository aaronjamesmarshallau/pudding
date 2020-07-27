use rocket::http::Header;
use rocket::Request;
use rocket::request::FromRequest;
use rocket::http::Status;
use rocket::request::Outcome;

/// A trait that indicates that something can be converted into a Chunk.
pub trait Chunkable : Sized {

	/// Convert the given iterator in a chunking iterator.
    fn chunks(self, size: usize) -> Chunk<Self> {
        Chunk { iter: self, size }
    }
}

/// Implementation of Chunkable for all existing Iterators
impl<T: Iterator> Chunkable for T {}

/// A struct that represents a generic 'chunk'
pub struct Chunk<I> {
    iter: I,
    size: usize,
}

/// Implements iterator for the generic Chunk concept, which allows converting
/// from the Chunk type to the individual chunks.
impl<I> Iterator for Chunk<I>
where
    I: Iterator,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::with_capacity(self.size);

        while let Some(v) = self.iter.next() {
			chunk.push(v);

            if chunk.len() >= self.size {
                break;
			}
        }

        if chunk.len() > 0 {
            Some(chunk)
        } else {
            None
        }
    }
}

/// Represents a ContentLength header, implementing a FromRequest trait.
pub struct ContentLength(pub i64);

/// Represents an error regarding ContentLength
#[derive(Debug)]
pub enum ContentLengthError {
    BadCount,
    Missing,
    Invalid,
}

/// Converts ContentLength values from the incoming request
impl<'a, 'r> FromRequest<'a, 'r> for ContentLength {
    type Error = ContentLengthError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("Content-Length").collect();

        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, ContentLengthError::Missing)),
            1 if keys[0].parse().unwrap_or(-1) >= 0 => Outcome::Success(ContentLength(keys[0].parse().unwrap_or(0))),
            1 => Outcome::Failure((Status::BadRequest, ContentLengthError::Invalid)),
            _ => Outcome::Failure((Status::BadRequest, ContentLengthError::BadCount)),
        }
    }
}

/// Converts a ContentLength value into a Header.
impl From<ContentLength> for Header<'_> {
    fn from(content_length: ContentLength) -> Self {
        Header::new("Content-Length", content_length.0.to_string())
    }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn chunks_yields_correct_count() {
		let len = 32;
		let chunk_size = 8;
		let test_vec = vec![0; len];

		let chunks = test_vec.into_iter().chunks(chunk_size);
		let count = chunks.count();
		let expected_count = len / chunk_size;

		assert_eq!(count, expected_count);
	}

	#[test]
	fn chunk_is_correct_size() {
		let len = 32;
		let chunk_size = 8;
		let test_vec = vec![0; len];

		let chunks = test_vec.into_iter().chunks(chunk_size);

		for chunk in chunks {
			let actual_chunk_size = chunk.len();

			assert_eq!(actual_chunk_size, chunk_size);
		}
	}

	#[test]
	fn chunk_is_correct_size_odd_length() {
		let len = 33;
		let chunk_size = 8;
		let test_vec = vec![0; len];

		let chunks = test_vec.into_iter().chunks(chunk_size);

		for (idx, chunk) in chunks.enumerate() {
			let mut expected_chunk_size = chunk_size;
			let actual_chunk_size = chunk.len();

			let starting_offset = idx * chunk_size;
			let end_offset = starting_offset + chunk_size;

			if end_offset > len {
				expected_chunk_size = len - starting_offset;
			}

			assert_eq!(actual_chunk_size, expected_chunk_size);
		}
	}

	#[test]
	fn chunk_has_correct_content() {
		let start_value = 1;
		let end_value = 36;

		// Bail if someone changes above values to something dumb
		assert_eq!(true, start_value > -1);
		assert_eq!(true, start_value < end_value);

		let test_vec = (start_value..end_value).collect::<Vec<i8>>();

		let chunks = test_vec.into_iter().chunks(6);

		for chunk in chunks {
			let mut last_val: i8 = start_value - 1;

			for value in chunk {
				assert_eq!(true, value > last_val);
				last_val = value;
			}
		}
	}
}
