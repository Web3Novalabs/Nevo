'use client';

import React from 'react';
import Link from 'next/link';
import { CopyButton } from '@/components/CopyButton';

interface ImpactMetric {
  value: string;
  label: string;
}

interface Testimonial {
  quote: string;
  name: string;
  role: string;
  avatarColor: string;
  poolName: string;
}

interface Story {
  id: string;
  title: string;
  poolName: string;
  description: string;
  impact: string;
  imageColor: string;
  metrics: { label: string; value: string }[];
  testimonial?: Testimonial;
  videoUrl?: string;
  beforeAfter?: { before: string; after: string };
}

const IMPACT_METRICS: ImpactMetric[] = [
  { value: '$480K+', label: 'Total Donated' },
  { value: '1,200+', label: 'Pools Created' },
  { value: '8,500+', label: 'Contributors' },
  { value: '156', label: 'Completed Pools' },
  { value: '12K+', label: 'Lives Impacted' },
  { value: '23', label: 'Countries Reached' },
];

const STORIES: Story[] = [
  {
    id: '1',
    title: 'Clean water for 500 families',
    poolName: 'Clean Water Initiative',
    description:
      'This campaign brought clean drinking water to three rural villages by installing solar-powered water purification systems. Each system serves over 150 families and is maintained by the local community.',
    impact:
      '500 families now have access to clean drinking water. Childhood waterborne diseases have dropped by 78% in the region.',
    imageColor: '#27926e',
    metrics: [
      { label: 'Raised', value: '6,800 XLM' },
      { label: 'Families Served', value: '500' },
      { label: 'Systems Installed', value: '3' },
    ],
    testimonial: {
      quote:
        'Before the well, we walked 3 hours every day to fetch water. Now my children can go to school instead.',
      name: 'Maria N.',
      role: 'Community Leader',
      avatarColor: '#27926e',
      poolName: 'Clean Water Initiative',
    },
  },
  {
    id: '2',
    title: 'Open source developers thrive',
    poolName: 'Open Source Dev Fund',
    description:
      'This fund supported 12 open-source developers building critical infrastructure on the Stellar network. Contributions covered development tools, server costs, and living stipends.',
    impact:
      '12 developers received funding. 8 new Stellar ecosystem tools were released. 3 developers were able to work full-time on open source.',
    imageColor: '#1c7459',
    metrics: [
      { label: 'Raised', value: '5,000 XLM' },
      { label: 'Developers Funded', value: '12' },
      { label: 'Tools Built', value: '8' },
    ],
    testimonial: {
      quote:
        'This funding allowed me to quit my side gig and focus entirely on building Stellar tooling. The ecosystem is better for it.',
      name: 'Alex K.',
      role: 'Open Source Developer',
      avatarColor: '#1c7459',
      poolName: 'Open Source Dev Fund',
    },
  },
  {
    id: '3',
    title: 'Community gardens bloom',
    poolName: 'Community Garden Project',
    description:
      'Transformed 5 unused urban lots into thriving community gardens. Local residents manage the gardens, and the produce is distributed to food-insecure households.',
    impact:
      '5 gardens established. 2,000 lbs of fresh produce grown annually. 200+ households receive weekly vegetables.',
    imageColor: '#47ae88',
    metrics: [
      { label: 'Raised', value: '3,200 XLM' },
      { label: 'Gardens', value: '5' },
      { label: 'Annual Produce', value: '2,000 lbs' },
    ],
    beforeAfter: {
      before: 'Abandoned lots with trash and overgrowth',
      after: 'Thriving gardens with raised beds and irrigation',
    },
  },
];

function QuoteIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="size-5 opacity-50"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M7.5 8.25h9m-9 3H12m-9.75 1.51c0 1.6 1.123 2.994 2.707 3.227 1.129.166 2.27.293 3.423.379.35.026.67.21.865.501L12 21l2.755-4.133a1.14 1.14 0 0 1 .865-.501 48.172 48.172 0 0 0 3.423-.379c1.584-.233 2.707-1.626 2.707-3.228V6.741c0-1.602-1.123-2.995-2.707-3.228A48.394 48.394 0 0 0 12 3c-2.392 0-4.744.175-7.043.513C3.373 3.746 2.25 5.14 2.25 6.741v6.018Z"
      />
    </svg>
  );
}

function GlobeIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="size-5"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M12 21a9.004 9.004 0 0 0 8.716-6.747M12 21a9.004 9.004 0 0 1-8.716-6.747M12 21c2.485 0 4.5-4.03 4.5-9S14.485 3 12 3m0 18c-2.485 0-4.5-4.03-4.5-9S9.515 3 12 3m0 0a8.997 8.997 0 0 1 7.843 4.582M12 3a8.997 8.997 0 0 0-7.843 4.582m15.686 0A11.953 11.953 0 0 1 12 10.5c-2.998 0-5.74-1.1-7.843-2.918m15.686 0A8.959 8.959 0 0 1 21 12c0 .778-.099 1.533-.284 2.253m0 0A17.919 17.919 0 0 1 12 16.5c-3.162 0-6.133-.815-8.716-2.247m0 0A9.015 9.015 0 0 1 3 12c0-1.605.42-3.113 1.157-4.418"
      />
    </svg>
  );
}

function HeartIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="size-5"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12Z"
      />
    </svg>
  );
}

function PlayIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="size-6"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.347a1.125 1.125 0 0 1 0 1.972l-11.54 6.347a1.125 1.125 0 0 1-1.667-.986V5.653Z"
      />
    </svg>
  );
}

function StoryCard({ story }: { story: Story }) {
  return (
    <article className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] overflow-hidden">
      <div
        className="flex h-48 items-center justify-center sm:h-56"
        style={{ backgroundColor: story.imageColor }}
        aria-hidden="true"
      >
        <HeartIcon />
      </div>

      <div className="p-6">
        <div className="flex flex-wrap items-center gap-2 mb-2">
          <span className="rounded-full bg-brand-100 px-2.5 py-0.5 text-xs font-medium text-brand-700">
            {story.poolName}
          </span>
        </div>
        <h3 className="text-xl font-bold tracking-tight mb-2">{story.title}</h3>
        <p className="text-sm text-[var(--color-text-muted)] leading-relaxed mb-4">
          {story.description}
        </p>

        <div className="rounded-xl bg-[var(--color-surface-raised)] p-4 mb-4">
          <p className="text-sm font-semibold mb-3">Impact</p>
          <p className="text-sm text-[var(--color-text-muted)] mb-3">
            {story.impact}
          </p>
          <div className="grid grid-cols-3 gap-3">
            {story.metrics.map((m) => (
              <div key={m.label} className="text-center">
                <p className="text-sm font-bold text-brand-600">{m.value}</p>
                <p className="text-xs text-[var(--color-text-muted)]">
                  {m.label}
                </p>
              </div>
            ))}
          </div>
        </div>

        {story.beforeAfter && (
          <div className="rounded-xl bg-[var(--color-surface-raised)] p-4 mb-4">
            <p className="text-sm font-semibold mb-2">Before & After</p>
            <div className="grid grid-cols-2 gap-3">
              <div className="rounded-lg bg-[var(--color-border)] p-3 text-center">
                <p className="text-xs font-medium text-[var(--color-text-muted)]">
                  Before
                </p>
                <p className="text-xs mt-1">{story.beforeAfter.before}</p>
              </div>
              <div className="rounded-lg bg-brand-100 p-3 text-center">
                <p className="text-xs font-medium text-brand-700">After</p>
                <p className="text-xs mt-1">{story.beforeAfter.after}</p>
              </div>
            </div>
          </div>
        )}

        {story.testimonial && (
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-4">
            <QuoteIcon />
            <blockquote className="mt-2 text-sm italic text-[var(--color-text-muted)]">
              &ldquo;{story.testimonial.quote}&rdquo;
            </blockquote>
            <div className="mt-3 flex items-center gap-3">
              <div
                className="flex size-9 items-center justify-center rounded-full text-xs font-bold text-white"
                style={{ backgroundColor: story.testimonial.avatarColor }}
              >
                {story.testimonial.name.charAt(0)}
              </div>
              <div>
                <p className="text-sm font-medium">{story.testimonial.name}</p>
                <p className="text-xs text-[var(--color-text-muted)]">
                  {story.testimonial.role}
                </p>
              </div>
            </div>
          </div>
        )}

        {story.videoUrl && (
          <a
            href={story.videoUrl}
            target="_blank"
            rel="noreferrer"
            className="mt-4 flex items-center justify-center gap-2 rounded-xl border border-[var(--color-border)] px-4 py-3 text-sm font-medium hover:bg-[var(--color-surface-raised)] transition-colors"
          >
            <PlayIcon />
            Watch Video Testimonial
          </a>
        )}
      </div>
    </article>
  );
}

export default function StoriesPage() {
  const shareUrl =
    typeof window !== 'undefined'
      ? window.location.origin + '/stories'
      : 'https://nevo.app/stories';

  return (
    <main className="mx-auto max-w-5xl px-6 py-10">
      {/* Header */}
      <div className="mb-12 text-center">
        <p className="mb-4 inline-block rounded-full border border-brand-200 bg-brand-50 px-4 py-1 text-sm font-medium text-brand-700">
          Real Impact, Real Stories
        </p>
        <h1 className="text-3xl font-bold tracking-tight sm:text-4xl">
          Success Stories
        </h1>
        <p className="mt-3 max-w-2xl mx-auto text-[var(--color-text-muted)]">
          See how Nevo pools are making a difference around the world. Every
          story is powered by transparent, on-chain donations.
        </p>
      </div>

      {/* Impact Metrics */}
      <section aria-labelledby="impact-heading" className="mb-16">
        <h2 id="impact-heading" className="sr-only">
          Platform Impact Metrics
        </h2>
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 sm:gap-6">
          {IMPACT_METRICS.map((metric) => (
            <div
              key={metric.label}
              className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-5 text-center"
            >
              <p className="text-2xl font-bold text-brand-600 sm:text-3xl">
                {metric.value}
              </p>
              <p className="mt-1 text-sm text-[var(--color-text-muted)]">
                {metric.label}
              </p>
            </div>
          ))}
        </div>
      </section>

      {/* Stories Grid */}
      <section aria-labelledby="stories-heading" className="mb-16">
        <h2
          id="stories-heading"
          className="text-2xl font-bold tracking-tight mb-8"
        >
          Featured Stories
        </h2>
        <div className="grid gap-8 md:grid-cols-2 lg:grid-cols-3">
          {STORIES.map((story) => (
            <StoryCard key={story.id} story={story} />
          ))}
        </div>
      </section>

      {/* Share Your Story */}
      <section
        aria-labelledby="share-heading"
        className="rounded-2xl border border-[var(--color-border)] bg-gradient-to-br from-brand-50 to-brand-100/50 p-8 text-center sm:p-12"
      >
        <GlobeIcon />
        <h2
          id="share-heading"
          className="mt-4 text-2xl font-bold tracking-tight"
        >
          Share Your Story
        </h2>
        <p className="mt-3 max-w-lg mx-auto text-sm text-[var(--color-text-muted)]">
          Have you created or contributed to a Nevo pool that made a difference?
          We want to hear from you. Share your story and inspire others.
        </p>
        <div className="mt-6 flex flex-col items-center gap-3 sm:flex-row sm:justify-center">
          <Link
            href="/pools/new"
            className="rounded-full bg-brand-600 px-6 py-2.5 text-sm font-semibold text-white hover:bg-brand-700 transition-colors"
          >
            Create a Pool
          </Link>
          <CopyButton
            text={shareUrl}
            label="Share This Page"
            copiedLabel="Link Copied!"
            className="rounded-full border border-[var(--color-border)] px-6 py-2.5 text-sm font-semibold"
          />
        </div>
      </section>
    </main>
  );
}
