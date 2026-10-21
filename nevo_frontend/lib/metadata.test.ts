import { describe, it, expect } from 'vitest';
import { generatePageMetadata, buildBreadcrumbJsonLd } from './metadata';

const BASE_URL = 'https://nevo.app';

describe('generatePageMetadata', () => {
  it('builds the correct canonical URL by concatenating BASE_URL and path', () => {
    const metadata = generatePageMetadata({
      title: 'Test Page',
      description: 'A test page',
      path: '/test-page',
    });

    expect(metadata.alternates?.canonical).toBe(`${BASE_URL}/test-page`);
    expect(metadata.openGraph?.url).toBe(`${BASE_URL}/test-page`);
    expect(metadata.twitter).toBeDefined();
  });

  it('uses the default image when image is omitted', () => {
    const metadata = generatePageMetadata({
      title: 'Test Page',
      description: 'A test page',
      path: '/test-page',
    });

    const openGraphImages = metadata.openGraph?.images;
    expect(Array.isArray(openGraphImages)).toBe(true);
    const firstOpenGraphImage = Array.isArray(openGraphImages)
      ? openGraphImages[0]
      : openGraphImages;
    const openGraphImageUrl =
      typeof firstOpenGraphImage === 'string'
        ? firstOpenGraphImage
        : firstOpenGraphImage?.url;
    expect(openGraphImageUrl).toBe('/opengraph-image.png');

    const twitterImages = metadata.twitter?.images;
    expect(Array.isArray(twitterImages)).toBe(true);
    const firstTwitterImage = Array.isArray(twitterImages)
      ? twitterImages[0]
      : twitterImages;
    const twitterImageUrl =
      typeof firstTwitterImage === 'string'
        ? firstTwitterImage
        : firstTwitterImage?.url;
    expect(twitterImageUrl).toBe('/opengraph-image.png');
  });

  it('uses the provided image when image is given', () => {
    const customImage = '/custom-image.png';
    const metadata = generatePageMetadata({
      title: 'Test Page',
      description: 'A test page',
      path: '/test-page',
      image: customImage,
    });

    const openGraphImages = metadata.openGraph?.images;
    const firstOpenGraphImage = Array.isArray(openGraphImages)
      ? openGraphImages[0]
      : openGraphImages;
    const openGraphImageUrl =
      typeof firstOpenGraphImage === 'string'
        ? firstOpenGraphImage
        : firstOpenGraphImage?.url;
    expect(openGraphImageUrl).toBe(customImage);

    const twitterImages = metadata.twitter?.images;
    const firstTwitterImage = Array.isArray(twitterImages)
      ? twitterImages[0]
      : twitterImages;
    const twitterImageUrl =
      typeof firstTwitterImage === 'string'
        ? firstTwitterImage
        : firstTwitterImage?.url;
    expect(twitterImageUrl).toBe(customImage);
  });
});

describe('buildBreadcrumbJsonLd', () => {
  it('produces the correct shape for a multi-item breadcrumb list', () => {
    const items = [
      { name: 'Home', path: '/' },
      { name: 'Products', path: '/products' },
      { name: 'Details', path: '/products/details' },
    ];

    const jsonLd = buildBreadcrumbJsonLd(items);

    expect(jsonLd['@context']).toBe('https://schema.org');
    expect(jsonLd['@type']).toBe('BreadcrumbList');
    expect(Array.isArray(jsonLd.itemListElement)).toBe(true);
    expect(jsonLd.itemListElement).toHaveLength(items.length);

    jsonLd.itemListElement.forEach((element: any, index: number) => {
      expect(element['@type']).toBe('ListItem');
      expect(element.position).toBe(index + 1);
      expect(element.name).toBe(items[index].name);
      expect(element.item).toBe(`${BASE_URL}${items[index].path}`);
    });
  });
});