use super::Batch;
use crate::{CKKSEvaluator, Context, Error, Evaluator, GaloisKeys, RelinearizationKeys, Result};

/// An evaluator that evaluates batches of data.
pub struct BatchEvaluator<E> {
	evaluator: E,
}

impl<E> BatchEvaluator<E>
where
	E: Evaluator,
{
	/// Creates a new batch evaluator.
	pub fn new(evaluator: E) -> Self {
		Self {
			evaluator,
		}
	}
}

impl BatchEvaluator<CKKSEvaluator> {
	/// Creates a new batch evaluator.
	pub fn ckks(ctx: &Context) -> Result<Self> {
		Ok(Self {
			evaluator: CKKSEvaluator::new(ctx)?,
		})
	}
}

impl<E> Evaluator for BatchEvaluator<E>
where
	E: Evaluator,
	E::Ciphertext: Clone,
	E::Plaintext: Clone,
{
	type Plaintext = Batch<E::Plaintext>;

	type Ciphertext = Batch<E::Ciphertext>;

	fn negate_inplace(&self, a: &mut Self::Ciphertext) -> Result<()> {
		for value in a.iter_mut() {
			self.evaluator.negate_inplace(value)?;
		}

		Ok(())
	}

	fn negate(&self, a: &Self::Ciphertext) -> Result<Self::Ciphertext> {
		a.map(|value| self.evaluator.negate(value)).collect()
	}

	fn add_inplace(&self, a: &mut Self::Ciphertext, b: &Self::Ciphertext) -> Result<()> {
		for (a, b) in a.iter_mut().zip(b.iter()) {
			self.evaluator.add_inplace(a, b)?;
		}

		Ok(())
	}

	fn add(&self, a: &Self::Ciphertext, b: &Self::Ciphertext) -> Result<Self::Ciphertext> {
		if a.len() != b.len() {
			return Err(Error::InvalidArgument);
		}

		a.zip(b, |a, b| self.evaluator.add(a, b)).collect()
	}

	fn add_many(&self, a: &[Self::Ciphertext]) -> Result<Self::Ciphertext> {
		let mut result = Vec::with_capacity(a.len());
		let length = a.first().ok_or_else(|| Error::InvalidArgument)?.len();

		for i in 0..length {
			let mut values = Vec::with_capacity(a.len());

			for batch in a.iter() {
				let value = batch.get_cloned(i).ok_or_else(|| Error::InvalidArgument)?;
				values.push(value);
			}

			result.push(self.evaluator.add_many(values.as_slice())?);
		}

		Ok(Batch(result))
	}

	fn multiply_many(
		&self, a: &[Self::Ciphertext], relin_keys: &RelinearizationKeys,
	) -> Result<Self::Ciphertext> {
		let mut result = Vec::with_capacity(a.len());
		let length = a.first().ok_or_else(|| Error::InvalidArgument)?.len();

		for i in 0..length {
			let mut values = Vec::with_capacity(a.len());

			for batch in a.iter() {
				let value = batch.get_cloned(i).ok_or_else(|| Error::InvalidArgument)?;
				values.push(value);
			}

			result.push(
				self.evaluator
					.multiply_many(values.as_slice(), relin_keys)?,
			);
		}

		Ok(Batch(result))
	}

	fn sub_inplace(&self, a: &mut Self::Ciphertext, b: &Self::Ciphertext) -> Result<()> {
		for (a, b) in a.iter_mut().zip(b.iter()) {
			self.evaluator.sub_inplace(a, b)?;
		}

		Ok(())
	}

	fn sub(&self, a: &Self::Ciphertext, b: &Self::Ciphertext) -> Result<Self::Ciphertext> {
		a.zip(b, |a, b| self.evaluator.sub(a, b)).collect()
	}

	fn multiply_inplace(&self, a: &mut Self::Ciphertext, b: &Self::Ciphertext) -> Result<()> {
		for (a, b) in a.iter_mut().zip(b.iter()) {
			self.evaluator.multiply_inplace(a, b)?;
		}

		Ok(())
	}

	fn multiply(&self, a: &Self::Ciphertext, b: &Self::Ciphertext) -> Result<Self::Ciphertext> {
		a.zip(b, |a, b| self.evaluator.multiply(a, b)).collect()
	}

	fn square_inplace(&self, a: &mut Self::Ciphertext) -> Result<()> {
		for value in a.iter_mut() {
			self.evaluator.square_inplace(value)?;
		}

		Ok(())
	}

	fn square(&self, a: &Self::Ciphertext) -> Result<Self::Ciphertext> {
		a.map(|value| self.evaluator.square(value)).collect()
	}

	fn mod_switch_to_next(&self, a: &Self::Ciphertext) -> Result<Self::Ciphertext> {
		a.map(|value| self.evaluator.mod_switch_to_next(value))
			.collect()
	}

	fn mod_switch_to_next_inplace(&self, a: &Self::Ciphertext) -> Result<()> {
		for value in a.iter() {
			self.evaluator.mod_switch_to_next_inplace(value)?;
		}

		Ok(())
	}

	fn mod_switch_to_next_plaintext(&self, a: &Self::Plaintext) -> Result<Self::Plaintext> {
		a.map(|value| self.evaluator.mod_switch_to_next_plaintext(value))
			.collect()
	}

	fn mod_switch_to_next_inplace_plaintext(&self, a: &Self::Plaintext) -> Result<()> {
		for value in a.iter() {
			self.evaluator.mod_switch_to_next_inplace_plaintext(value)?;
		}

		Ok(())
	}

	fn exponentiate(
		&self, a: &Self::Ciphertext, exponent: u64, relin_keys: &RelinearizationKeys,
	) -> Result<Self::Ciphertext> {
		a.map(|value| self.evaluator.exponentiate(value, exponent, relin_keys))
			.collect()
	}

	fn exponentiate_inplace(
		&self, a: &Self::Ciphertext, exponent: u64, relin_keys: &RelinearizationKeys,
	) -> Result<()> {
		for value in a.iter() {
			self.evaluator
				.exponentiate_inplace(value, exponent, relin_keys)?;
		}

		Ok(())
	}

	fn add_plain(&self, a: &Self::Ciphertext, b: &Self::Plaintext) -> Result<Self::Ciphertext> {
		a.zip(b, |a, b| self.evaluator.add_plain(a, b)).collect()
	}

	fn add_plain_inplace(&self, a: &mut Self::Ciphertext, b: &Self::Plaintext) -> Result<()> {
		for (a, b) in a.iter_mut().zip(b.iter()) {
			self.evaluator.add_plain_inplace(a, b)?;
		}

		Ok(())
	}

	fn sub_plain(&self, a: &Self::Ciphertext, b: &Self::Plaintext) -> Result<Self::Ciphertext> {
		a.zip(b, |a, b| self.evaluator.sub_plain(a, b)).collect()
	}

	fn sub_plain_inplace(&self, a: &mut Self::Ciphertext, b: &Self::Plaintext) -> Result<()> {
		for (a, b) in a.iter_mut().zip(b.iter()) {
			self.evaluator.sub_plain_inplace(a, b)?;
		}

		Ok(())
	}

	fn multiply_plain(
		&self, a: &Self::Ciphertext, b: &Self::Plaintext,
	) -> Result<Self::Ciphertext> {
		a.zip(b, |a, b| self.evaluator.multiply_plain(a, b))
			.collect()
	}

	fn multiply_plain_inplace(&self, a: &mut Self::Ciphertext, b: &Self::Plaintext) -> Result<()> {
		for (a, b) in a.iter_mut().zip(b.iter()) {
			self.evaluator.multiply_plain_inplace(a, b)?;
		}

		Ok(())
	}

	fn relinearize_inplace(
		&self, a: &mut Self::Ciphertext, relin_keys: &RelinearizationKeys,
	) -> Result<()> {
		for value in a.iter_mut() {
			self.evaluator.relinearize_inplace(value, relin_keys)?;
		}

		Ok(())
	}

	fn relinearize(
		&self, a: &Self::Ciphertext, relin_keys: &RelinearizationKeys,
	) -> Result<Self::Ciphertext> {
		a.map(|value| self.evaluator.relinearize(value, relin_keys))
			.collect()
	}

	fn rotate_rows(
		&self, a: &Self::Ciphertext, steps: i32, galois_keys: &GaloisKeys,
	) -> Result<Self::Ciphertext> {
		a.map(|value| self.evaluator.rotate_rows(value, steps, galois_keys))
			.collect()
	}

	fn rotate_rows_inplace(
		&self, a: &Self::Ciphertext, steps: i32, galois_keys: &GaloisKeys,
	) -> Result<()> {
		for value in a.iter() {
			self.evaluator
				.rotate_rows_inplace(value, steps, galois_keys)?;
		}

		Ok(())
	}

	fn rotate_columns(
		&self, a: &Self::Ciphertext, galois_keys: &GaloisKeys,
	) -> Result<Self::Ciphertext> {
		a.map(|value| self.evaluator.rotate_columns(value, galois_keys))
			.collect()
	}

	fn rotate_columns_inplace(&self, a: &Self::Ciphertext, galois_keys: &GaloisKeys) -> Result<()> {
		for value in a.iter() {
			self.evaluator.rotate_columns_inplace(value, galois_keys)?;
		}

		Ok(())
	}
}
