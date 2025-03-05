use rand_chacha::rand_core::RngCore;

_implement_crypto_rng!(rand_chacha::ChaCha8Rng);
_implement_crypto_rng!(rand_chacha::ChaCha12Rng);
_implement_crypto_rng!(rand_chacha::ChaCha20Rng);
