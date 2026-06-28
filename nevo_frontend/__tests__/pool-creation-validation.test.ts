import {
  validateFormData,
  validateImageFile,
  MAX_IMAGE_SIZE_BYTES,
  SUPPORTED_IMAGE_TYPES,
} from '@/lib/pool-creation-validation';
import type { FormData, FormErrors } from '@/lib/pool-creation-validation';

function validForm(): FormData {
  return {
    title: 'Clean Water Fund',
    description:
      'Providing clean drinking water to communities in rural areas across Africa.',
    category: 'Humanitarian',
    goalAmount: '5000',
    duration: 30,
    imageUrl: '',
    tags: '',
  };
}

describe('validateFormData', () => {
  // ── Title ────────────────────────────────────────────────────────

  it('requires a title', () => {
    const errs = validateFormData({ ...validForm(), title: '' });
    expect(errs.title).toBe('Title is required.');
  });

  it('requires title to be at least 5 characters', () => {
    const errs = validateFormData({ ...validForm(), title: 'abc' });
    expect(errs.title).toBe('Title must be at least 5 characters.');
  });

  it('accepts a title exactly 5 characters long', () => {
    const errs = validateFormData({ ...validForm(), title: 'abcde' });
    expect(errs.title).toBeUndefined();
  });

  it('accepts a valid title', () => {
    const errs = validateFormData(validForm());
    expect(errs.title).toBeUndefined();
  });

  it('rejects a whitespace-only title', () => {
    const errs = validateFormData({ ...validForm(), title: '   ' });
    expect(errs.title).toBe('Title is required.');
  });

  // ── Description ─────────────────────────────────────────────────

  it('requires a description', () => {
    const errs = validateFormData({ ...validForm(), description: '' });
    expect(errs.description).toBe('Description is required.');
  });

  it('requires description to be at least 20 characters', () => {
    const errs = validateFormData({ ...validForm(), description: 'Too short' });
    expect(errs.description).toBe(
      'Description must be at least 20 characters.'
    );
  });

  it('accepts a description exactly 20 characters long', () => {
    const errs = validateFormData({
      ...validForm(),
      description: 'a'.repeat(20),
    });
    expect(errs.description).toBeUndefined();
  });

  it('accepts a valid description', () => {
    const errs = validateFormData(validForm());
    expect(errs.description).toBeUndefined();
  });

  it('rejects a whitespace-only description', () => {
    const errs = validateFormData({ ...validForm(), description: '     ' });
    expect(errs.description).toBe('Description is required.');
  });

  // ── Category ─────────────────────────────────────────────────────

  it('requires a category', () => {
    const errs = validateFormData({ ...validForm(), category: '' });
    expect(errs.category).toBe('Please select a category.');
  });

  it('accepts a valid category', () => {
    const errs = validateFormData(validForm());
    expect(errs.category).toBeUndefined();
  });

  // ── Goal Amount ─────────────────────────────────────────────────

  it('requires a goal amount', () => {
    const errs = validateFormData({ ...validForm(), goalAmount: '' });
    expect(errs.goalAmount).toBe('Goal amount is required.');
  });

  it('rejects a goal amount of zero', () => {
    const errs = validateFormData({ ...validForm(), goalAmount: '0' });
    expect(errs.goalAmount).toBe('Enter a valid amount greater than 0.');
  });

  it('rejects a negative goal amount', () => {
    const errs = validateFormData({ ...validForm(), goalAmount: '-100' });
    expect(errs.goalAmount).toBe('Enter a valid amount greater than 0.');
  });

  it('rejects a non-numeric goal amount', () => {
    const errs = validateFormData({ ...validForm(), goalAmount: 'abc' });
    expect(errs.goalAmount).toBe('Enter a valid amount greater than 0.');
  });

  it('accepts a valid goal amount', () => {
    const errs = validateFormData(validForm());
    expect(errs.goalAmount).toBeUndefined();
  });

  it('accepts a decimal goal amount', () => {
    const errs = validateFormData({ ...validForm(), goalAmount: '1234.56' });
    expect(errs.goalAmount).toBeUndefined();
  });

  // ── Duration ─────────────────────────────────────────────────────

  it('requires a duration', () => {
    const errs = validateFormData({ ...validForm(), duration: 0 });
    expect(errs.duration).toBe('Please select a duration.');
  });

  it('accepts a valid duration', () => {
    const errs = validateFormData(validForm());
    expect(errs.duration).toBeUndefined();
  });

  // ── Banner Image URL ─────────────────────────────────────────────

  it('accepts an empty image URL', () => {
    const errs = validateFormData({ ...validForm(), imageUrl: '' });
    expect(errs.imageUrl).toBeUndefined();
  });

  it('accepts a valid http URL', () => {
    const errs = validateFormData({
      ...validForm(),
      imageUrl: 'http://example.com/banner.jpg',
    });
    expect(errs.imageUrl).toBeUndefined();
  });

  it('accepts a valid https URL', () => {
    const errs = validateFormData({
      ...validForm(),
      imageUrl: 'https://example.com/banner.jpg',
    });
    expect(errs.imageUrl).toBeUndefined();
  });

  it('rejects a URL without a protocol', () => {
    const errs = validateFormData({
      ...validForm(),
      imageUrl: 'example.com/banner.jpg',
    });
    expect(errs.imageUrl).toBe(
      'Enter a valid URL starting with http:// or https://'
    );
  });

  it('rejects an ftp URL', () => {
    const errs = validateFormData({
      ...validForm(),
      imageUrl: 'ftp://files.example.com/banner.jpg',
    });
    expect(errs.imageUrl).toBe(
      'Enter a valid URL starting with http:// or https://'
    );
  });

  it('rejects a URL longer than 2048 characters', () => {
    const longUrl = 'https://x.com/' + 'a'.repeat(2040); // total > 2048
    const errs = validateFormData({ ...validForm(), imageUrl: longUrl });
    expect(errs.imageUrl).toBe('URL is too long (max 2048 characters).');
  });

  it('formats the URL check before the length check', () => {
    // A malformed URL that's also very long should show the format error
    const longBadUrl = 'x'.repeat(3000);
    const errs = validateFormData({ ...validForm(), imageUrl: longBadUrl });
    expect(errs.imageUrl).toBe(
      'Enter a valid URL starting with http:// or https://'
    );
  });

  // ── Tags ─────────────────────────────────────────────────────────

  it('accepts empty tags', () => {
    const errs = validateFormData({ ...validForm(), tags: '' });
    expect(errs.tags).toBeUndefined();
  });

  it('accepts a single valid tag', () => {
    const errs = validateFormData({ ...validForm(), tags: 'water' });
    expect(errs.tags).toBeUndefined();
  });

  it('accepts multiple valid tags', () => {
    const errs = validateFormData({
      ...validForm(),
      tags: 'water, africa, community',
    });
    expect(errs.tags).toBeUndefined();
  });

  it('rejects more than 10 tags', () => {
    const tags = Array.from({ length: 11 }, (_, i) => `tag${i}`).join(',');
    const errs = validateFormData({ ...validForm(), tags });
    expect(errs.tags).toBe('Maximum of 10 tags allowed.');
  });

  it('accepts exactly 10 tags', () => {
    const tags = Array.from({ length: 10 }, (_, i) => `tag${i}`).join(',');
    const errs = validateFormData({ ...validForm(), tags });
    expect(errs.tags).toBeUndefined();
  });

  it('rejects a tag longer than 30 characters', () => {
    const errs = validateFormData({
      ...validForm(),
      tags: 'this-tag-is-way-too-long-and-exceeds-the-maximum-length',
    });
    expect(errs.tags).toBe('Each tag must be 30 characters or fewer.');
  });

  it('accepts a tag exactly 30 characters long', () => {
    const errs = validateFormData({
      ...validForm(),
      tags: 'a'.repeat(30),
    });
    expect(errs.tags).toBeUndefined();
  });

  it('rejects tags that are only whitespace/comma junk', () => {
    const errs = validateFormData({
      ...validForm(),
      tags: '  ,  ,  ,  ',
    });
    expect(errs.tags).toBe('Enter at least one tag or leave empty.');
  });

  it('trims whitespace around tags and deduplicates empty entries', () => {
    const errs = validateFormData({
      ...validForm(),
      tags: '  water ,  africa ,  ',
    });
    expect(errs.tags).toBeUndefined(); // "water" and "africa" are valid
  });

  it('flags the first too-long tag when multiple tags are provided', () => {
    const errs = validateFormData({
      ...validForm(),
      tags: 'good-tag, this-one-is-way-too-longgggggggggggggggggg, another-good',
    });
    expect(errs.tags).toBe('Each tag must be 30 characters or fewer.');
  });

  // ── Multiple errors ──────────────────────────────────────────────

  it('returns multiple errors for an empty form', () => {
    const errs = validateFormData({
      title: '',
      description: '',
      category: '',
      goalAmount: '',
      duration: 0,
      imageUrl: '',
      tags: '',
    });
    expect(errs.title).toBeDefined();
    expect(errs.description).toBeDefined();
    expect(errs.category).toBeDefined();
    expect(errs.goalAmount).toBeDefined();
    expect(errs.duration).toBeDefined();
    // imageUrl and tags are optional — they should NOT produce errors when empty
    expect(errs.imageUrl).toBeUndefined();
    expect(errs.tags).toBeUndefined();
  });

  it('returns no errors for a completely valid form', () => {
    const errs = validateFormData(validForm());
    expect(Object.keys(errs)).toHaveLength(0);
  });

  // ── validateImageFile ──────────────────────────────────────────────

  describe('validateImageFile', () => {
    function validFile(overrides?: Partial<File>): File {
      return {
        name: 'banner.jpg',
        type: 'image/jpeg',
        size: 1024 * 1024, // 1MB — well under 5MB limit
        ...overrides,
      } as File;
    }

    it('returns undefined for a valid JPEG', () => {
      expect(validateImageFile(validFile())).toBeUndefined();
    });

    it('returns undefined for a valid PNG', () => {
      expect(
        validateImageFile(validFile({ type: 'image/png' }))
      ).toBeUndefined();
    });

    it('returns undefined for a valid WebP', () => {
      expect(
        validateImageFile(validFile({ type: 'image/webp' }))
      ).toBeUndefined();
    });

    it('rejects unsupported image formats', () => {
      const result = validateImageFile(validFile({ type: 'image/gif' }));
      expect(result).toBe('Unsupported format. Use JPG, PNG, or WebP.');
    });

    it('rejects non-image files', () => {
      const result = validateImageFile(validFile({ type: 'application/pdf' }));
      expect(result).toBe('Unsupported format. Use JPG, PNG, or WebP.');
    });

    it('rejects files exceeding MAX_IMAGE_SIZE_BYTES', () => {
      const result = validateImageFile(
        validFile({ size: MAX_IMAGE_SIZE_BYTES + 1 })
      );
      expect(result).toBe('Image is too large. Max size is 5MB.');
    });

    it('accepts a file exactly at MAX_IMAGE_SIZE_BYTES', () => {
      expect(
        validateImageFile(validFile({ size: MAX_IMAGE_SIZE_BYTES }))
      ).toBeUndefined();
    });

    it('uses SUPPORTED_IMAGE_TYPES to determine valid types', () => {
      // Sanity: the constant matches what the function checks
      expect(SUPPORTED_IMAGE_TYPES).toEqual([
        'image/jpeg',
        'image/png',
        'image/webp',
      ]);
    });
  });
});
