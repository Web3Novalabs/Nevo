"use client";

import Image, { ImageProps } from "next/image";

/**
 * Use for all content images across the site. Wraps next/image with sensible
 * defaults: lazy loading, responsive sizes. Prefer JPG for photos.
 */
export function OptimizedImage(props: ImageProps) {
  return (
    <Image
      loading="lazy"
      quality={85}
      {...props}
    />
  );
}
