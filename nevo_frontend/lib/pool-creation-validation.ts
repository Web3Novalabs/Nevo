export interface FormData {
  title: string;
  description: string;
  category: string;
  goalAmount: string;
  duration: number;
  imageUrl: string;
  tags: string;
}

export interface FormErrors {
  title?: string;
  description?: string;
  category?: string;
  goalAmount?: string;
  duration?: string;
  imageUrl?: string;
  tags?: string;
  submit?: string;
}

export const MAX_IMAGE_SIZE_BYTES = 5 * 1024 * 1024;
export const SUPPORTED_IMAGE_TYPES = ['image/jpeg', 'image/png', 'image/webp'];

/**
 * Validate an image file for banner upload.
 * Returns an error message string if invalid, or undefined if valid.
 */
export function validateImageFile(file: File): string | undefined {
  if (!SUPPORTED_IMAGE_TYPES.includes(file.type)) {
    return 'Unsupported format. Use JPG, PNG, or WebP.';
  }
  if (file.size > MAX_IMAGE_SIZE_BYTES) {
    return 'Image is too large. Max size is 5MB.';
  }
  return undefined;
}

/** Shared validation — single source of truth for all form field rules. */
export function validateFormData(form: FormData): FormErrors {
  const errs: FormErrors = {};
  if (!form.title.trim()) errs.title = 'Title is required.';
  else if (form.title.trim().length < 5)
    errs.title = 'Title must be at least 5 characters.';
  if (!form.description.trim()) errs.description = 'Description is required.';
  else if (form.description.trim().length < 20)
    errs.description = 'Description must be at least 20 characters.';
  if (!form.category) errs.category = 'Please select a category.';
  const goal = parseFloat(form.goalAmount);
  if (!form.goalAmount) errs.goalAmount = 'Goal amount is required.';
  else if (isNaN(goal) || goal <= 0)
    errs.goalAmount = 'Enter a valid amount greater than 0.';
  if (!form.duration) errs.duration = 'Please select a duration.';

  // Optional banner image URL — validate format if provided
  const url = form.imageUrl.trim();
  if (url && !/^https?:\/\//i.test(url)) {
    errs.imageUrl = 'Enter a valid URL starting with http:// or https://';
  } else if (url.length > 2048) {
    errs.imageUrl = 'URL is too long (max 2048 characters).';
  }

  // Optional tags — validate format if provided
  const rawTags = form.tags.trim();
  if (rawTags) {
    const tags = rawTags
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean);
    if (tags.length === 0) {
      errs.tags = 'Enter at least one tag or leave empty.';
    } else if (tags.length > 10) {
      errs.tags = 'Maximum of 10 tags allowed.';
    } else {
      const tooLong = tags.find((t) => t.length > 30);
      if (tooLong) errs.tags = 'Each tag must be 30 characters or fewer.';
    }
  }

  return errs;
}
