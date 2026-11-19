/** Stellar Ed25519 public keys are base32, start with `G`, and are 56 chars long. */
export const STELLAR_PUBLIC_KEY = /^G[A-Z0-9]{54,55}$/;
