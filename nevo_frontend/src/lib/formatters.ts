import i18n from '@/src/lib/i18n';

export function formatDate(
  date: Date | string,
  options?: Intl.DateTimeFormatOptions
) {
  const d = typeof date === 'string' ? new Date(date) : date;
  const locale = i18n.language || 'en';
  return new Intl.DateTimeFormat(locale, options).format(d);
}

export function formatCurrency(amount: number, currency = 'USD') {
  const locale = i18n.language || 'en';
  return new Intl.NumberFormat(locale, { style: 'currency', currency }).format(
    amount
  );
}
