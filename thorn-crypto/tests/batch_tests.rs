use thorn_crypto::fhe::batched::{
	decryptor::BatchDecryptor, encoder::BatchEncoder, encryptor::BatchEncryptor,
	evaluator::BatchEvaluator,
};
use thorn_seal::{
	CKKSEncoder, CkksEncryptionParametersBuilder, CoefficientModulus, Context, DegreeType, Encoder,
	EncryptionParameters, Error, Evaluator, KeyGenerator, SecurityLevel,
};

#[test]
fn test_batched_sum() -> Result<(), Error> {
	// generate keypair to encrypt and decrypt data.
	let degree = DegreeType::D8192;
	let security_level = SecurityLevel::TC128;
	let bit_sizes = [60, 40, 40, 60];

	let expand_mod_chain = false;
	let modulus_chain = CoefficientModulus::create(degree, bit_sizes.as_slice())?;
	let encryption_parameters: EncryptionParameters = CkksEncryptionParametersBuilder::new()
		.set_poly_modulus_degree(degree)
		.set_coefficient_modulus(modulus_chain.clone())
		.build()?;

	let ctx = Context::new(&encryption_parameters, expand_mod_chain, security_level)?;

	let key_gen = KeyGenerator::new(&ctx)?;

	let encoder = BatchEncoder::new(CKKSEncoder::new(&ctx, 2.0f64.powi(40))?);

	let public_key = key_gen.create_public_key();
	let private_key = key_gen.secret_key();

	let encryptor = BatchEncryptor::with_public_and_secret_key(&ctx, &public_key, &private_key)?;
	let decryptor = BatchDecryptor::new(&ctx, &private_key)?;

	let evaluator = BatchEvaluator::ckks(&ctx)?;

	let x = 5.2;
	let y = 3.3;

	let x_encoded = encoder.encode(&[x])?;
	let y_encoded = encoder.encode(&[y])?;

	let x_enc = encryptor.encrypt(&x_encoded)?;
	let y_enc = encryptor.encrypt(&y_encoded)?;

	let sum = evaluator.add(&x_enc, &y_enc)?;
	let sum_dec = decryptor.decrypt(&sum)?;

	let sum_plain = encoder.decode(&sum_dec)?;

	let truth = x + y;

	assert!((sum_plain.first().unwrap() - truth).abs() < 1e-6);

	Ok(())
}