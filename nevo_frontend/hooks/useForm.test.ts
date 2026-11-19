import { act, renderHook } from '@testing-library/react';
import { isRequired } from '../lib/validation';
import { useForm } from './useForm';

describe('useForm', () => {
  const formConfig = {
    email: {
      initialValue: '',
      rules: [isRequired('Email is required')],
    },
    password: {
      initialValue: '',
      rules: [isRequired('Password is required')],
    },
  } as const;

  it('validates fields on blur and tracks touched state', async () => {
    const { result } = renderHook(() => useForm(formConfig));

    expect(result.current.touched.email).toBe(false);

    act(() => {
      result.current.handleBlur('email');
    });

    await act(async () => {
      await Promise.resolve();
    });

    expect(result.current.touched.email).toBe(true);
    expect(result.current.errors.email).toBe('Email is required');
  });

  it('revalidates a field on change when it has been touched', async () => {
    const { result } = renderHook(() => useForm(formConfig));

    act(() => {
      result.current.handleBlur('email');
    });

    await act(async () => {
      await Promise.resolve();
    });

    act(() => {
      result.current.handleChange('email', 'valid@example.com');
    });

    await act(async () => {
      await Promise.resolve();
    });

    expect(result.current.errors.email).toBeNull();
    expect(result.current.values.email).toBe('valid@example.com');
  });

  it('submits successfully when validation passes', async () => {
    const onSubmit = jest.fn();
    const { result } = renderHook(() => useForm(formConfig));

    act(() => {
      result.current.handleChange('email', 'valid@example.com');
      result.current.handleChange('password', 'secret123');
    });

    await act(async () => {
      await result.current.handleSubmit(onSubmit)();
    });

    expect(onSubmit).toHaveBeenCalledWith({
      email: 'valid@example.com',
      password: 'secret123',
    });
    expect(result.current.isSubmitting).toBe(false);
  });

  it('does not submit when validation fails', async () => {
    const onSubmit = jest.fn();
    const { result } = renderHook(() => useForm(formConfig));

    await act(async () => {
      await result.current.handleSubmit(onSubmit)();
    });

    expect(onSubmit).not.toHaveBeenCalled();
    expect(result.current.errors.email).toBe('Email is required');
    expect(result.current.errors.password).toBe('Password is required');
    expect(result.current.touched.email).toBe(true);
    expect(result.current.touched.password).toBe(true);
    expect(result.current.isSubmitting).toBe(false);
  });

  it('updates values on change without validating an untouched field', async () => {
    const { result } = renderHook(() => useForm(formConfig));

    act(() => {
      result.current.handleChange('email', '');
    });

    await act(async () => {
      await Promise.resolve();
    });

    expect(result.current.values.email).toBe('');
    expect(result.current.touched.email).toBe(false);
    expect(result.current.errors.email).toBeNull();
  });

  it('marks a field touched and validates it on blur', async () => {
    const { result } = renderHook(() => useForm(formConfig));

    act(() => {
      result.current.handleChange('email', 'invalid');
    });

    await act(async () => {
      await Promise.resolve();
    });

    expect(result.current.errors.email).toBeNull();

    act(() => {
      result.current.handleBlur('email');
    });

    await act(async () => {
      await Promise.resolve();
    });

    expect(result.current.touched.email).toBe(true);
    expect(result.current.errors.email).toBe('Email is required');
  });

  it('marks all fields touched and submits only when valid', async () => {
    const onSubmit = jest.fn();
    const { result } = renderHook(() => useForm(formConfig));

    await act(async () => {
      await result.current.handleSubmit(onSubmit)();
    });

    expect(onSubmit).not.toHaveBeenCalled();
    expect(result.current.touched.email).toBe(true);
    expect(result.current.touched.password).toBe(true);
    expect(result.current.errors.email).toBe('Email is required');
    expect(result.current.errors.password).toBe('Password is required');

    act(() => {
      result.current.handleChange('email', 'valid@example.com');
      result.current.handleChange('password', 'secret123');
    });

    await act(async () => {
      await result.current.handleSubmit(onSubmit)();
    });

    expect(onSubmit).toHaveBeenCalledTimes(1);
    expect(onSubmit).toHaveBeenCalledWith({
      email: 'valid@example.com',
      password: 'secret123',
    });
    expect(result.current.errors.email).toBeNull();
    expect(result.current.errors.password).toBeNull();
    expect(result.current.isSubmitting).toBe(false);
  });

  it('resets values, errors, and touched state', async () => {
    const onSubmit = jest.fn();
    const { result } = renderHook(() => useForm(formConfig));

    act(() => {
      result.current.handleChange('email', 'valid@example.com');
      result.current.handleChange('password', 'secret123');
    });

    await act(async () => {
      await result.current.handleSubmit(onSubmit)();
    });

    act(() => {
      result.current.handleBlur('email');
    });

    await act(async () => {
      await Promise.resolve();
    });

    expect(result.current.values.email).toBe('valid@example.com');
    expect(result.current.touched.email).toBe(true);

    act(() => {
      result.current.reset();
    });

    await act(async () => {
      await Promise.resolve();
    });

    expect(result.current.values).toEqual({ email: '', password: '' });
    expect(result.current.errors.email).toBeNull();
    expect(result.current.errors.password).toBeNull();
    expect(result.current.touched.email).toBe(false);
    expect(result.current.touched.password).toBe(false);
  });
});
