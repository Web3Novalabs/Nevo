/**
 * Centralized image paths and settings for Nevo.
 * Use OptimizedImage (next/image) for all images: AVIF/WebP, responsive sizes, lazy load.
 * Prefer JPG for photos and large graphics for faster loading; use SVG for icons.
 * For fastest logo load: add public/logo.jpg and set NEXT_PUBLIC_LOGO_URL=/logo.jpg.
 */
export const LOGO_SRC =
  process.env.NEXT_PUBLIC_LOGO_URL ?? "https://nevo.app/logo.jpeg";
export const LOGO_WIDTH = 32;
export const LOGO_HEIGHT = 32;
