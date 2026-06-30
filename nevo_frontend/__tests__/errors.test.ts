import { ApiError } from '@/lib/api-client';
import { parseApiError } from '@/lib/errors';

describe('parseApiError', () => {
  it('extracts message from axios response.data.message', () => {
    const err = {
      isAxiosError: true,
      response: { data: { message: 'Invalid credentials' } },
      message: 'Request failed with status code 401',
    };

    expect(parseApiError(err)).toBe('Invalid credentials');
  });

  it('extracts message from axios response.data.error', () => {
    const err = {
      isAxiosError: true,
      response: { data: { error: 'Pool not found' } },
      message: 'Request failed with status code 404',
    };

    expect(parseApiError(err)).toBe('Pool not found');
  });

  it('falls back to err.message for axios errors without response payload', () => {
    const err = {
      isAxiosError: true,
      message: 'Network Error',
    };

    expect(parseApiError(err)).toBe('Network Error');
  });

  it('extracts message from ApiError data.message', () => {
    const err = new ApiError(400, 'Bad Request', {
      message: 'Title is required',
    });

    expect(parseApiError(err)).toBe('Title is required');
  });

  it('extracts message from ApiError data.error', () => {
    const err = new ApiError(500, 'Internal Server Error', {
      error: 'Database unavailable',
    });

    expect(parseApiError(err)).toBe('Database unavailable');
  });

  it('falls back to ApiError.message when data has no message fields', () => {
    const err = new ApiError(403, 'Forbidden');

    expect(parseApiError(err)).toBe('Forbidden');
  });

  it('returns fallback message for unknown errors', () => {
    expect(parseApiError(null)).toBe('Something went wrong. Please try again.');
    expect(parseApiError(undefined)).toBe(
      'Something went wrong. Please try again.'
    );
    expect(parseApiError({})).toBe('Something went wrong. Please try again.');
  });
});
