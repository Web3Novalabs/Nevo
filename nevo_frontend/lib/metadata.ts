import type { Metadata } from 'next';

const BASE_URL = 'https://nevo.app';

interface PageMetaOptions {
  title: string;
  description: string;
  /** Relative path, e.g. "/dashboard" */
  path: string;
  image?: string;
}

/**
 * Generate consistent metadata for any page.
 *
 * Usage in a page or layout file:
 *
 * ```ts
 * // app/dashboard/page.tsx
 * import { generatePageMetadata } from '@/lib/metadata';
 * import type { Metadata } from 'next';
 *
 * export const metadata: Metadata = generatePageMetadata({
 *   title: 'Dashboard',
 *   description: 'Manage your fundraising pools.',
 *   path: '/dashboard',
 * });
 * ```
 *
 * For dynamic routes, use generateMetadata instead:
 *
 * ```ts
 * // app/pools/[id]/page.tsx
 * import { generatePageMetadata } from '@/lib/metadata';
 * import type { Metadata } from 'next';
 *
 * export async function generateMetadata({ params }: { params: { id: string } }): Promise<Metadata> {
 *   const pool = await fetchPool(params.id);
 *   return generatePageMetadata({
 *     title: pool.name,
 *     description: pool.description,
 *     path: `/pools/${params.id}`,
 *   });
 * }
 * ```
 *
 * To add breadcrumb JSON-LD to a page, use buildBreadcrumbJsonLd:
 *
 * ```tsx
 * import { buildBreadcrumbJsonLd } from '@/lib/metadata';
 *
 * export default function Page() {
 *   const jsonLd = buildBreadcrumbJsonLd([
 *     { name: 'Home', path: '/' },
 *     { name: 'Dashboard', path: '/dashboard' },
 *   ]);
 *   return (
 *     <>
 *       <script type="application/ld+json" dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }} />
 *       ...
 *     </>
 *   );
 * }
 * ```
 */
export function generatePageMetadata({
  title,
  description,
  path,
  image = '/opengraph-image.png',
}: PageMetaOptions): Metadata {
  const url = `${BASE_URL}${path}`;
  return {
    title,
    description,
    alternates: { canonical: url },
    openGraph: {
      title,
      description,
      url,
      images: [{ url: image, width: 1200, height: 630 }],
    },
    twitter: {
      card: 'summary_large_image',
      title,
      description,
      images: [image],
    },
  };
}

interface BreadcrumbItem {
  name: string;
  path: string;
}

export function buildBreadcrumbJsonLd(items: BreadcrumbItem[]) {
  return {
    '@context': 'https://schema.org',
    '@type': 'BreadcrumbList',
    itemListElement: items.map((item, index) => ({
      '@type': 'ListItem',
      position: index + 1,
      name: item.name,
      item: `${BASE_URL}${item.path}`,
    })),
  };
}
