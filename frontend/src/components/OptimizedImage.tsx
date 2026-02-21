"use client";

import Image, { ImageProps } from "next/image";

/**
 * Use for all content images across the site. Wraps next/image with:
 * - Automatic AVIF/WebP via Next.js image optimization
 * - Lazy loading (override with priority for above-the-fold images like logos)
 * - Quality 85 for smaller files with minimal visual loss
 * - Sensible sizes for responsive srcset
 * Prefer JPG for photos and large graphics for faster loading.
 */
export function OptimizedImage({
  quality = 85,
  sizes,
  loading = "lazy",
  ...props
}: ImageProps) {
  return (
    <Image
      quality={quality}
      sizes={sizes ?? "(max-width: 768px) 48px, 32px"}
      loading={loading}
      {...props}
    />
  );
}
