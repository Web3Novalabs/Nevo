'use client';

import React from 'react';
import { useTranslation } from 'react-i18next';
import '@/src/lib/i18n';

const LANGS = [
  { code: 'en', label: 'English', flag: '🇺🇸' },
  { code: 'es', label: 'Español', flag: '🇪🇸' },
];

export default function LanguageSwitcher() {
  const { i18n, t } = useTranslation();

  const change = (lng: string) => {
    i18n.changeLanguage(lng);
    try {
      localStorage.setItem('nevo-lang', lng);
    } catch (e) {
      // ignore
    }
  };

  return (
    <div className="flex items-center gap-2">
      <label className="sr-only">{t('language.label')}</label>
      <select
        aria-label={t('language.label')}
        value={i18n.language || 'en'}
        onChange={(e) => change(e.target.value)}
        className="rounded border px-2 py-1 text-sm"
      >
        {LANGS.map((l) => (
          <option key={l.code} value={l.code}>
            {l.flag} {l.label}
          </option>
        ))}
      </select>
    </div>
  );
}
