import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import en from '@/src/locales/en/common.json';
import es from '@/src/locales/es/common.json';

const resources = {
  en: { common: en },
  es: { common: es },
};

if (!i18n.isInitialized) {
  i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init({
      resources,
      fallbackLng: 'en',
      ns: ['common'],
      defaultNS: 'common',
      interpolation: { escapeValue: false },
      detection: {
        order: ['localStorage', 'navigator'],
        caches: ['localStorage'],
        lookupLocalStorage: 'nevo-lang',
      },
    });
}

export default i18n;
