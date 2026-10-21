import { STELLAR_PUBLIC_KEY } from './stellar-public-key.js';

describe('STELLAR_PUBLIC_KEY', () => {
  it('matches a 55-char G-prefixed string ({54,55} after G)', () => {
    const key55 = 'G' + 'A'.repeat(54);
    expect(key55).toHaveLength(55);
    expect(STELLAR_PUBLIC_KEY.test(key55)).toBe(true);
  });

  it('matches a standard 56-char G-prefixed string', () => {
    const key56 = 'G' + 'A'.repeat(55);
    expect(key56).toHaveLength(56);
    expect(STELLAR_PUBLIC_KEY.test(key56)).toBe(true);
  });

  it('does not match a 57-char string', () => {
    const key57 = 'G' + 'A'.repeat(56);
    expect(key57).toHaveLength(57);
    expect(STELLAR_PUBLIC_KEY.test(key57)).toBe(false);
  });

  it('does not match a lowercase-letter-containing string (base32 is uppercase+digits only)', () => {
    const keyWithLower = 'G' + 'A'.repeat(53) + 'a';
    expect(keyWithLower).toHaveLength(55);
    expect(STELLAR_PUBLIC_KEY.test(keyWithLower)).toBe(false);

    const keyWithLowerG = 'g' + 'A'.repeat(54);
    expect(STELLAR_PUBLIC_KEY.test(keyWithLowerG)).toBe(false);
  });
});
