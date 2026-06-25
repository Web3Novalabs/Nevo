'use client';

import React from 'react';
import Link from 'next/link';
import { Avatar } from '@/components/Avatar';

interface TeamMember {
  id: string;
  name: string;
  role: string;
  bio: string;
  avatarSrc?: string;
  social?: {
    twitter?: string;
    github?: string;
    linkedin?: string;
  };
}

interface CompanyValue {
  id: string;
  title: string;
  description: string;
  icon: React.ReactNode;
}

const TEAM_MEMBERS: TeamMember[] = [
  {
    id: '1',
    name: 'Alex Morgan',
    role: 'Founder & CEO',
    bio: 'Blockchain enthusiast and philanthropy advocate with 8+ years in Web3 development.',
    social: {
      twitter: 'https://twitter.com/alexmorgan',
      github: 'https://github.com/alexmorgan',
    },
  },
  {
    id: '2',
    name: 'Sam Rivera',
    role: 'CTO',
    bio: 'Stellar ecosystem veteran focused on scalable smart contract architecture.',
    social: {
      github: 'https://github.com/samrivera',
      linkedin: 'https://linkedin.com/in/samrivera',
    },
  },
  {
    id: '3',
    name: 'Jordan Lee',
    role: 'Head of Product',
    bio: 'Product strategist passionate about making blockchain accessible to everyone.',
    social: {
      twitter: 'https://twitter.com/jordanlee',
    },
  },
];

const COMPANY_VALUES: CompanyValue[] = [
  {
    id: 'transparency',
    title: 'Transparency',
    description:
      'All donations are recorded on-chain with full visibility into every transaction.',
    icon: (
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
          d="M2.036 12.322a48.928 48.928 0 0 1 18.928-4.882c-.826.286-1.553.415-2.26.533a48.928 48.928 0 0 0-7.867 3.048 48.928 48.928 0 0 0-7.867-3.048c-.707-.119-1.433-.248-2.259-.533Zm0 0a48.928 48.928 0 0 1 18.928 4.882c-.826-.286-1.553-.415-2.26-.533a48.928 48.928 0 0 0-7.867-3.048 48.928 48.928 0 0 0-7.867 3.048c-.707.119-1.433.248-2.259.533Z"
        />
      </svg>
    ),
  },
  {
    id: 'security',
    title: 'Security',
    description:
      'Built on Stellar smart contracts with battle-tested security patterns.',
    icon: (
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
          d="M9 12.75 11.25 15 15 9.75m-3-7.036A11.959 11.959 0 0 1 3.598 6 11.99 11.99 0 0 0 3 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285Z"
        />
      </svg>
    ),
  },
  {
    id: 'community',
    title: 'Community',
    description:
      'Empowering local communities through decentralized fundraising.',
    icon: (
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
          d="M18 18.72a9.04 9.04 0 0 0-3.69-.82c-1.232 0-2.442.23-3.593.64-.675-.18-1.31-.4-1.928-.64-3.241.57-5.196 2.49-5.196 4.64v.58c0 .28.22.5.5.5h15c.28 0 .5-.22.5-.5v-.58c0-1.79-.956-3.41-2.697-4.36Z"
        />
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M6.75 6.75a4.5 4.5 0 1 1 9 0 4.5 4.5 0 0 1-9 0Z"
        />
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M12 12.75a4.5 4.5 0 1 0 0-9 4.5 4.5 0 0 0 0 9Z"
        />
      </svg>
    ),
  },
  {
    id: 'accessibility',
    title: 'Accessibility',
    description:
      'Low fees under $0.01 per transaction for truly inclusive donations.',
    icon: (
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
          d="M12 18.75a3.75 3.75 0 1 0 0-7.5 3.75 3.75 0 0 0 0 7.5Zm0 0V8.25m0 0c0-1.232-.738-2.423-1.917-2.943l-.117-.044a4.5 4.5 0 0 0-6.633 3.917 4.499 4.499 0 0 0 7.694 2.394l.44-.248c.81-.46 1.392-1.276 1.392-2.2V6.75Z"
        />
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z"
        />
      </svg>
    ),
  },
];

const TESTIMONIALS = [
  {
    quote:
      'Nevo has revolutionized how we collect donations. The transparency and low fees make it perfect for our mission.',
    author: 'Maria Santos',
    role: 'Clean Water Initiative',
  },
  {
    quote:
      'Finally, a platform that makes crypto donations accessible to everyone without hidden fees.',
    author: 'Dr. James Chen',
    role: 'Healthcare for All',
  },
];

export default function AboutPage() {
  return (
    <main className="mx-auto max-w-5xl px-6 py-12">
      <section aria-labelledby="mission-heading" className="mb-16 text-center">
        <h1
          id="mission-heading"
          className="text-3xl font-bold tracking-tight sm:text-4xl"
        >
          Our Mission
        </h1>
        <p className="mt-4 max-w-2xl text-lg text-[var(--color-text-muted)] mx-auto">
          To empower anyone, anywhere to create transparent, secure, and
          efficient fundraising pools on the Stellar blockchain — without
          intermediaries, hidden fees, or complex technology barriers.
        </p>
      </section>

      <section aria-labelledby="about-heading" className="mb-16">
        <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-8">
          <h2 id="about-heading" className="text-2xl font-semibold mb-4">
            About Nevo
          </h2>
          <p className="text-[var(--color-text-muted)] leading-relaxed mb-4">
            Nevo was founded in 2024 with a simple goal: make charitable giving
            transparent, secure, and accessible on Web3. We believe that trust
            comes from openness, not gatekeepers.
          </p>
          <p className="text-[var(--color-text-muted)] leading-relaxed mb-4">
            Built on the Stellar blockchain, our platform leverages smart
            contracts to create donation pools that are auditable, trustless,
            and efficient. Every contribution is recorded on-chain, allowing
            anyone to verify that funds reach their intended destination.
          </p>
          <p className="text-[var(--color-text-muted)] leading-relaxed">
            Our platform is open-source and community-driven. We welcome
            contributors who share our vision of a more transparent future for
            charitable giving.
          </p>
        </div>
      </section>

      <section aria-labelledby="values-heading" className="mb-16">
        <h2 id="values-heading" className="text-2xl font-bold text-center mb-8">
          Our Values
        </h2>
        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
          {COMPANY_VALUES.map((value) => (
            <article
              key={value.id}
              className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 flex flex-col gap-3"
            >
              <div className="flex size-12 items-center justify-center rounded-xl bg-brand-100 text-brand-600">
                {value.icon}
              </div>
              <h3 className="font-semibold">{value.title}</h3>
              <p className="text-sm text-[var(--color-text-muted)]">
                {value.description}
              </p>
            </article>
          ))}
        </div>
      </section>

      <section aria-labelledby="team-heading" className="mb-16">
        <h2 id="team-heading" className="text-2xl font-bold text-center mb-8">
          Meet the Team
        </h2>
        <div className="grid gap-8 sm:grid-cols-2 lg:grid-cols-3">
          {TEAM_MEMBERS.map((member) => (
            <article
              key={member.id}
              className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 flex flex-col items-center text-center"
            >
              <Avatar
                name={member.name}
                src={member.avatarSrc}
                size="lg"
                className="mb-4"
              />
              <h3 className="font-semibold text-lg">{member.name}</h3>
              <p className="text-sm text-brand-600 mb-2">{member.role}</p>
              <p className="text-sm text-[var(--color-text-muted)]">
                {member.bio}
              </p>
              {member.social && (
                <div className="mt-4 flex gap-3">
                  {member.social.twitter && (
                    <a
                      href={member.social.twitter}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-[var(--color-text-muted)] hover:text-brand-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600 rounded"
                      aria-label={`${member.name}'s Twitter`}
                    >
                      <svg
                        className="h-5 w-5"
                        fill="currentColor"
                        viewBox="0 0 24 24"
                        aria-hidden="true"
                      >
                        <path d="M8.29 20.251c7.547 0 11.675-6.253 11.675-11.675 0-.178 0-.355-.012-.53A8.348 8.348 0 0022 5.92a8.19 8.19 0 01-2.357.646 4.118 4.118 0 001.804-2.27 8.224 8.224 0 01-2.605.996 8.224 8.224 0 01-2.605-.996 4.118 4.118 0 001.804 2.27A8.348 8.348 0 0012 0C5.373 0 0 5.373 0 12c0 5.523 4.373 10.098 9.999 11.675a11.65 11.65 0 002.6-.416 4.107 4.107 0 01-2.27.646Z" />
                      </svg>
                    </a>
                  )}
                  {member.social.github && (
                    <a
                      href={member.social.github}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-[var(--color-text-muted)] hover:text-brand-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600 rounded"
                      aria-label={`${member.name}'s GitHub`}
                    >
                      <svg
                        className="h-5 w-5"
                        fill="currentColor"
                        viewBox="0 0 24 24"
                        aria-hidden="true"
                      >
                        <path
                          fillRule="evenodd"
                          d="M12 2C6.477 2 2 6.484 2 12.017c0 4.502 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.851.004 1.706.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 12z"
                        />
                      </svg>
                    </a>
                  )}
                  {member.social.linkedin && (
                    <a
                      href={member.social.linkedin}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-[var(--color-text-muted)] hover:text-brand-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600 rounded"
                      aria-label={`${member.name}'s LinkedIn`}
                    >
                      <svg
                        className="h-5 w-5"
                        fill="currentColor"
                        viewBox="0 0 24 24"
                        aria-hidden="true"
                      >
                        <path d="M20.447 20.452h-3.554v-5.525c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.566H9.375V9h3.414v1.561h.046c.477-.9 1.637-1.848 3.368-1.848 3.6 0 4.267 2.37 4.267 5.455v6.226zM5.337 7.433c-1.144 0-2.063-.925-2.063-2.068 0-1.144.92-2.067 2.063-2.067 1.143 0 2.063.923 2.063 2.067 0 1.143-.92 2.068-2.063 2.068zm1.777 13.019H3.558V9h3.554v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.203 24 24 23.227 24 22.271V1.729C24 .774 23.203 0 22.222 0h.003z" />
                      </svg>
                    </a>
                  )}
                </div>
              )}
            </article>
          ))}
        </div>
      </section>

      <section aria-labelledby="testimonials-heading" className="mb-16">
        <h2
          id="testimonials-heading"
          className="text-2xl font-bold text-center mb-8"
        >
          What They Say
        </h2>
        <div className="space-y-6">
          {TESTIMONIALS.map((testimonial, index) => (
            <blockquote
              key={index}
              className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6"
            >
              <p className="text-[var(--color-text)] italic mb-3">
                &quot;{testimonial.quote}&quot;
              </p>
              <footer className="text-sm">
                <span className="font-semibold">{testimonial.author}</span>
                <span className="text-[var(--color-text-muted)]">
                  {' '}
                  — {testimonial.role}
                </span>
              </footer>
            </blockquote>
          ))}
        </div>
      </section>

      <section aria-labelledby="contact-heading">
        <h2
          id="contact-heading"
          className="text-2xl font-bold text-center mb-8"
        >
          Get in Touch
        </h2>
        <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-8 text-center">
          <p className="text-[var(--color-text-muted)] mb-4">
            Have questions or want to learn more about Nevo?
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <a
              href="mailto:hello@nevo.app"
              className="inline-flex items-center justify-center rounded-xl bg-brand-600 px-6 py-3 text-sm font-semibold text-white hover:bg-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
            >
              Email Us
            </a>
            <Link
              href="/contact"
              className="inline-flex items-center justify-center rounded-xl border border-[var(--color-border)] px-6 py-3 text-sm font-semibold hover:bg-[var(--color-surface)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
            >
              Contact Form
            </Link>
          </div>
        </div>
      </section>
    </main>
  );
}
