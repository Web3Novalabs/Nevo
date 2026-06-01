'use client';

import React, { FC, useState } from 'react';
import { trackingService } from '@/lib/tracking';

interface SocialShareButtonsProps {
  poolId: string;
  poolTitle: string;
  poolUrl?: string;
  customMessage?: string;
}

export const SocialShareButtons: FC<SocialShareButtonsProps> = ({
  poolId,
  poolTitle,
  poolUrl,
  customMessage,
}) => {
  const [copied, setCopied] = useState(false);

  // Get the full share URL
  const shareUrl =
    poolUrl ||
    (typeof window !== 'undefined'
      ? `${window.location.origin}/pools/${poolId}`
      : '');

  // Generate the share message with pool details
  const shareMessage =
    customMessage ||
    `Check out this amazing pool: "${poolTitle}" - Support the cause and make a difference!`;

  const handleTwitterShare = () => {
    trackingService.trackShareClick(poolId, 'twitter');
    const twitterUrl = `https://twitter.com/intent/tweet?text=${encodeURIComponent(shareMessage)}&url=${encodeURIComponent(shareUrl)}`;
    window.open(twitterUrl, 'twitter-share', 'width=550,height=420');
  };

  const handleFacebookShare = () => {
    trackingService.trackShareClick(poolId, 'facebook');
    const facebookUrl = `https://www.facebook.com/sharer/sharer.php?u=${encodeURIComponent(shareUrl)}&quote=${encodeURIComponent(shareMessage)}`;
    window.open(facebookUrl, 'facebook-share', 'width=550,height=420');
  };

  const handleLinkedInShare = () => {
    trackingService.trackShareClick(poolId, 'linkedin');
    const linkedInUrl = `https://www.linkedin.com/sharing/share-offsite/?url=${encodeURIComponent(shareUrl)}`;
    window.open(linkedInUrl, 'linkedin-share', 'width=550,height=420');
  };

  const handleCopyLink = async () => {
    trackingService.trackShareClick(poolId, 'copy');
    try {
      await navigator.clipboard.writeText(shareUrl);
      setCopied(true);
      // Reset the copied state after 2 seconds
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy link:', err);
    }
  };

  return (
    <section
      className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-6"
      aria-labelledby="share-heading"
    >
      <h2 id="share-heading" className="mb-4 text-sm font-semibold">
        Share this pool
      </h2>

      <div className="flex flex-wrap gap-3">
        {/* Twitter/X Button */}
        <button
          type="button"
          onClick={handleTwitterShare}
          className="inline-flex items-center justify-center gap-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-2.5 text-sm font-medium text-[var(--color-text)] hover:bg-[var(--color-border)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
          aria-label="Share on Twitter/X"
          title="Share on Twitter/X"
        >
          <TwitterIcon className="size-4" />
          <span className="hidden sm:inline">Twitter</span>
        </button>

        {/* Facebook Button */}
        <button
          type="button"
          onClick={handleFacebookShare}
          className="inline-flex items-center justify-center gap-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-2.5 text-sm font-medium text-[var(--color-text)] hover:bg-[var(--color-border)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
          aria-label="Share on Facebook"
          title="Share on Facebook"
        >
          <FacebookIcon className="size-4" />
          <span className="hidden sm:inline">Facebook</span>
        </button>

        {/* LinkedIn Button */}
        <button
          type="button"
          onClick={handleLinkedInShare}
          className="inline-flex items-center justify-center gap-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-2.5 text-sm font-medium text-[var(--color-text)] hover:bg-[var(--color-border)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
          aria-label="Share on LinkedIn"
          title="Share on LinkedIn"
        >
          <LinkedInIcon className="size-4" />
          <span className="hidden sm:inline">LinkedIn</span>
        </button>

        {/* Copy Link Button */}
        <button
          type="button"
          onClick={handleCopyLink}
          className="inline-flex items-center justify-center gap-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-2.5 text-sm font-medium text-[var(--color-text)] hover:bg-[var(--color-border)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
          aria-label={copied ? 'Link copied!' : 'Copy share link'}
          title={copied ? 'Link copied!' : 'Copy share link'}
        >
          {copied ? (
            <>
              <CheckIcon className="size-4" />
              <span className="hidden sm:inline">Copied</span>
            </>
          ) : (
            <>
              <LinkIcon className="size-4" />
              <span className="hidden sm:inline">Copy</span>
            </>
          )}
        </button>
      </div>
    </section>
  );
};

// Social media icons
function TwitterIcon({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden="true"
    >
      <path d="M23 3a10.9 10.9 0 01-3.14 1.53 4.48 4.48 0 00-7.86 3v1A10.66 10.66 0 013 4s-4 9 5 13a11.64 11.64 0 01-7 2s9 5 20 5a9.5 9.5 0 00-9-5.5c4.75 2.25 7-7 7-7a10.6 10.6 0 01-10-10.5z" />
    </svg>
  );
}

function FacebookIcon({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden="true"
    >
      <path d="M18 2h-3a6 6 0 00-6 6v3H7v4h2v8h4v-8h3l1-4h-4V8a1 1 0 011-1h3z" />
    </svg>
  );
}

function LinkedInIcon({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden="true"
    >
      <path d="M16 8a6 6 0 016 6v7h-4v-7a2 2 0 00-2-2 2 2 0 00-2 2v7h-4v-7a6 6 0 016-6zM2 9h4v12H2z" />
      <circle cx="4" cy="4" r="2" />
    </svg>
  );
}

function LinkIcon({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M13.19 8.688a4.5 4.5 0 011.242 7.244l-4.5 4.5a4.5 4.5 0 01-6.364-6.364l1.757-1.757m2.121-2.121l4.5-4.5a4.5 4.5 0 016.364 6.364l-1.757 1.757m2.121 2.121l-4.5 4.5a4.5 4.5 0 01-6.364-6.364l1.757-1.757"
      />
    </svg>
  );
}

function CheckIcon({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M4.5 12.75l6 6 9-13.5"
      />
    </svg>
  );
}
