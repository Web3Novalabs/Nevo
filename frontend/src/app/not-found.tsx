'use client'

import Link from 'next/link'
import { ArrowLeft } from 'lucide-react'

export default function NotFound() {
  return (
    <div
      className="min-h-screen w-full flex items-center justify-center px-4 sm:px-6 lg:px-8"
      style={{ backgroundColor: '#0a1428' }}
    >
      <div className="w-full max-w-lg text-center">
        {/* Error Code */}
        <div className="mb-8 sm:mb-12">
          <h1
            className="text-7xl sm:text-8xl lg:text-9xl font-bold mb-4 tracking-tight"
            style={{ color: '#1fe4ff' }}
          >
            404
          </h1>
          <div
            className="h-1 w-20 mx-auto rounded-full"
            style={{ backgroundColor: '#10b981' }}
          ></div>
        </div>

        {/* Heading */}
        <h2
          className="text-3xl sm:text-4xl lg:text-5xl font-bold mb-4 tracking-tight"
          style={{ color: '#ffffff' }}
        >
          Page Not Found
        </h2>

        {/* Description */}
        <p
          className="text-lg sm:text-xl mb-8 leading-relaxed"
          style={{ color: '#a0aec0' }}
        >
          Oops! It looks like you&apos;ve ventured into uncharted territory. The page you&apos;re looking
          for doesn&apos;t exist or has been moved.
        </p>

        {/* Decorative Element */}
        <div className="mb-8 flex justify-center gap-2">
          <div
            className="h-2 w-2 rounded-full"
            style={{ backgroundColor: '#1fe4ff' }}
          ></div>
          <div
            className="h-2 w-2 rounded-full"
            style={{ backgroundColor: '#10b981' }}
          ></div>
          <div
            className="h-2 w-2 rounded-full"
            style={{ backgroundColor: '#1fe4ff' }}
          ></div>
        </div>

        {/* Back to Home Button */}
        <Link
          href="/"
          className="inline-flex items-center gap-2 px-6 sm:px-8 py-3 sm:py-4 rounded-lg font-semibold text-base sm:text-lg transition-all duration-300 hover:shadow-lg hover:scale-105 active:scale-95"
          style={{
            backgroundColor: '#1fe4ff',
            color: '#0a1428',
          }}
        >
          <ArrowLeft size={20} />
          <span>Back to Home</span>
        </Link>

        {/* Optional: Helpful Links */}
        <div className="mt-12 sm:mt-16 pt-8 sm:pt-12 border-t border-opacity-20" style={{ borderColor: '#1fe4ff' }}>
          <p
            className="text-sm mb-6"
            style={{ color: '#a0aec0' }}
          >
            Need help? Try these:
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link
              href="/"
              className="text-sm sm:text-base font-medium transition-colors duration-200 hover:underline"
              style={{ color: '#1fe4ff' }}
            >
              Go to Homepage
            </Link>
            <span style={{ color: '#a0aec0' }}>•</span>
            <Link
              href="/"
              className="text-sm sm:text-base font-medium transition-colors duration-200 hover:underline"
              style={{ color: '#1fe4ff' }}
            >
              Contact Support
            </Link>
          </div>
        </div>
      </div>
    </div>
  )
}
