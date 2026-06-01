import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import en from '../locales/en/common.json';
import es from '../locales/es/common.json';

const resources = {
  en: { common: en },
  es: { common: es },
};

if (!i18n.isInitialized) {
  i18n
    // detect user language
    .use(LanguageDetector)
    // pass the i18n instance to react-i18next
    .use(initReactI18next)
    .init({
      resources,
      fallbackLng: 'en',
      ns: ['common'],
      defaultNS: 'common',
      interpolation: { escapeValue: false },
      detection: {
        // look for language in localStorage first, then navigator
        order: ['localStorage', 'navigator'],
        caches: ['localStorage'],
        lookupLocalStorage: 'nevo-lang',
      },
    });
}

export default i18n;
