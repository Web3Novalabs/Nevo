'use client';

import React, { useState } from 'react';

const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export default function ContactPage() {
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [subject, setSubject] = useState('');
  const [message, setMessage] = useState('');
  const [autoReply, setAutoReply] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState({ email: '', subject: '', message: '' });

  function validateEmail(value: string) {
    return EMAIL_REGEX.test(value);
  }

  function validateForm() {
    const errors = { email: '', subject: '', message: '' };

    if (!validateEmail(email)) {
      errors.email = 'Please enter a valid email address.';
    }

    if (!subject || subject.trim().length < 3) {
      errors.subject = 'Please enter a subject with at least 3 characters.';
    }

    if (!message || message.trim().length < 10) {
      errors.message = 'Please enter a message with at least 10 characters.';
    }

    setFieldErrors(errors);
    return !errors.email && !errors.subject && !errors.message;
  }

  async function handleSubmit(ev: React.FormEvent<HTMLFormElement>) {
    ev.preventDefault();
    setError(null);
    setSuccess(null);

    if (!validateForm()) {
      setError('Please fix the highlighted fields before sending.');
      return;
    }

    setLoading(true);

    try {
      const res = await fetch('/api/contact', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, email, subject, message, autoReply }),
      });

      const data = await res.json();
      if (!res.ok) {
        throw new Error(data?.error || 'Submission failed');
      }

      setSuccess('Thanks — your message was sent. We will respond shortly.');
      setName('');
      setEmail('');
      setSubject('');
      setMessage('');
      setAutoReply(true);
      setFieldErrors({ email: '', subject: '', message: '' });
    } catch (err: any) {
      setError(err.message || 'Submission failed');
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="mx-auto max-w-3xl px-6 py-10">
      <h1 className="text-2xl font-bold mb-2">Contact Support</h1>
      <p className="text-sm text-[var(--color-text-muted)] mb-6">
        Have a question, found an issue, or want to give feedback? Send us a message and we&apos;ll get back to you.
      </p>

      <form onSubmit={handleSubmit} noValidate className="space-y-6">
        <div aria-live="polite" className="min-h-[1.5rem]">
          {error && <p className="text-sm text-error">{error}</p>}
          {success && <p className="text-sm text-success">{success}</p>}
        </div>

        <div>
          <label htmlFor="name" className="block text-xs text-[var(--color-text-muted)] mb-1">
            Name (optional)
          </label>
          <input
            id="name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full rounded-xl border px-3 py-2 bg-[var(--color-surface)]"
            placeholder="Your name"
          />
        </div>

        <div>
          <label htmlFor="email" className="block text-xs text-[var(--color-text-muted)] mb-1">
            Email
          </label>
          <input
            id="email"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            required
            aria-invalid={!!fieldErrors.email}
            aria-describedby="email-error"
            className="w-full rounded-xl border px-3 py-2 bg-[var(--color-surface)]"
            placeholder="you@example.com"
          />
          {fieldErrors.email && (
            <p id="email-error" className="mt-1 text-xs text-error">
              {fieldErrors.email}
            </p>
          )}
        </div>

        <div>
          <label htmlFor="subject" className="block text-xs text-[var(--color-text-muted)] mb-1">
            Subject
          </label>
          <input
            id="subject"
            value={subject}
            onChange={(e) => setSubject(e.target.value)}
            required
            minLength={3}
            aria-invalid={!!fieldErrors.subject}
            aria-describedby="subject-error"
            className="w-full rounded-xl border px-3 py-2 bg-[var(--color-surface)]"
            placeholder="What can we help you with?"
          />
          {fieldErrors.subject && (
            <p id="subject-error" className="mt-1 text-xs text-error">
              {fieldErrors.subject}
            </p>
          )}
        </div>

        <div>
          <label htmlFor="message" className="block text-xs text-[var(--color-text-muted)] mb-1">
            Message
          </label>
          <textarea
            id="message"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            required
            minLength={10}
            rows={6}
            aria-invalid={!!fieldErrors.message}
            aria-describedby="message-error"
            className="w-full rounded-xl border px-3 py-2 bg-[var(--color-surface)]"
            placeholder="Tell us what happened, include any relevant details or links."
          />
          {fieldErrors.message && (
            <p id="message-error" className="mt-1 text-xs text-error">
              {fieldErrors.message}
            </p>
          )}
        </div>

        <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
          <label className="inline-flex items-center gap-3 text-sm text-[var(--color-text-muted)]">
            <input
              id="autoReply"
              type="checkbox"
              checked={autoReply}
              onChange={(e) => setAutoReply(e.target.checked)}
              className="h-4 w-4 rounded border"
            />
            Send me an auto-reply confirmation
          </label>
          <p className="text-xs text-[var(--color-text-muted)]">This helps us verify we can follow up to your inquiry.</p>
        </div>

        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <button
            type="submit"
            disabled={loading}
            className="rounded-xl bg-brand-600 text-white px-4 py-2 disabled:opacity-50"
          >
            {loading ? 'Sending…' : 'Send Message'}
          </button>
          <p className="text-xs text-[var(--color-text-muted)]">We aim to reply within 48 hours.</p>
        </div>
      </form>
    </main>
  );
}
