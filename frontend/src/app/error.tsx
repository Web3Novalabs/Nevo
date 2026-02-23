'use client'

import { useEffect } from 'react'
import Link from 'next/link'
import { RotateCcw } from 'lucide-react'

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string }
  reset: () => void
}) {
  useEffect(() => {
    // Log the error to an error reporting service
    console.error(error)
  }, [error])

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
            500
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
          Server Error
        </h2>

        {/* Description */}
        <p
          className="text-lg sm:text-xl mb-4 leading-relaxed"
          style={{ color: '#a0aec0' }}
        >
          Something went wrong on our end. Our team has been notified and is working to fix the
          issue.
        </p>

        {/* Error Details (Development Only) */}
        {process.env.NODE_ENV === 'development' && error.message && (
          <div
            className="mb-8 p-4 rounded-lg text-left text-sm overflow-auto max-h-32 border"
            style={{
              backgroundColor: 'rgba(31, 228, 255, 0.05)',
              borderColor: '#1fe4ff',
              color: '#a0aec0',
            }}
          >
            <p className="font-mono">{error.message}</p>
          </div>
        )}

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

        {/* Action Buttons */}
        <div className="flex flex-col sm:flex-row gap-4 justify-center mb-8">
          {/* Try Again Button */}
          <button
            onClick={() => reset()}
            className="inline-flex items-center justify-center gap-2 px-6 sm:px-8 py-3 sm:py-4 rounded-lg font-semibold text-base sm:text-lg transition-all duration-300 hover:shadow-lg hover:scale-105 active:scale-95"
            style={{
              backgroundColor: '#1fe4ff',
              color: '#0a1428',
            }}
          >
            <RotateCcw size={20} />
            <span>Try Again</span>
          </button>

          {/* Back to Home Button */}
          <Link
            href="/"
            className="inline-flex items-center justify-center gap-2 px-6 sm:px-8 py-3 sm:py-4 rounded-lg font-semibold text-base sm:text-lg transition-all duration-300 border-2 hover:bg-opacity-10 hover:scale-105 active:scale-95"
            style={{
              backgroundColor: 'transparent',
              borderColor: '#1fe4ff',
              color: '#1fe4ff',
            }}
          >
            <span>Back to Home</span>
          </Link>
        </div>

        {/* Support Section */}
        <div className="pt-8 sm:pt-12 border-t border-opacity-20" style={{ borderColor: '#1fe4ff' }}>
          <p
            className="text-sm mb-6"
            style={{ color: '#a0aec0' }}
          >
            Still having trouble? Reach out to our support team.
          </p>
          <Link
            href="/"
            className="text-sm sm:text-base font-medium transition-colors duration-200 hover:underline"
            style={{ color: '#10b981' }}
          >
            Contact Support
          </Link>
        </div>
      </div>
    </div>
  )
}