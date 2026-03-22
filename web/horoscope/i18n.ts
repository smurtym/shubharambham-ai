// UI-level presentation strings only.
// Per constitution VI, all calculation / body / sign / nakshatra translations
// live exclusively in Rust/WASM. This file covers only chrome labels.

import type { Lang } from './types';

export interface UIStrings {
  // Page
  appTitle: string;
  loadingEngine: string;
  // Birth form
  dateOfBirth: string;
  timeOfBirth: string;
  location: string;
  locationPlaceholder: string;
  dateRequired: string;
  timeRequired: string;
  locationRequired: string;
  calculating: string;
  calculate: string;
  // City modal
  selectCity: string;
  searchPlaceholder: string;
  noCitiesFound: string;
  // Echo panel
  calculationDetails: string;
  city: string;
  latitude: string;
  longitude: string;
  timezone: string;
  // Longitude table
  bodyLongitudes: string;
  bodyCol: string;
  longitudeCol: string;
  nakshatraCol: string;
  padaCol: string;
  rasiCol: string;
  navamsaCol: string;
  // Language selector
  language: string;
  // Chakra grid titles
  rasi: string;
  navamsa: string;
  // Time picker
  selectTime: string;
  hours: string;
  minutes: string;
  ok: string;
  cancel: string;
  // Vimsottari Dasa
  vimsottariDasa: string;
}

const en: UIStrings = {
  appTitle: 'Horoscope',
  loadingEngine: 'Loading calculation engine…',
  dateOfBirth: 'Date of Birth',
  timeOfBirth: 'Time of Birth',
  location: 'Location',
  locationPlaceholder: 'Tap to select a city…',
  dateRequired: 'Date of birth is required',
  timeRequired: 'Time of birth is required',
  locationRequired: 'Location is required',
  calculating: 'Calculating…',
  calculate: 'Calculate',
  selectCity: 'Select City',
  searchPlaceholder: 'Search city, region, country…',
  noCitiesFound: 'No cities found',
  calculationDetails: 'Calculation Details',
  city: 'City',
  latitude: 'Latitude',
  longitude: 'Longitude',
  timezone: 'Timezone',
  bodyLongitudes: 'Body Longitudes',
  bodyCol: 'Body',
  longitudeCol: 'Longitude',
  nakshatraCol: 'Nakshatra',
  padaCol: 'Pada',
  rasiCol: 'Rasi',
  navamsaCol: 'Navamsa',
  language: 'Language',
  rasi: 'Rasi',
  navamsa: 'Navamsa',
  selectTime: 'Select Time',
  hours: 'Hours',
  minutes: 'Minutes',
  ok: 'OK',
  cancel: 'Cancel',
  vimsottariDasa: 'Vimsottari Dasa',
};

const te: UIStrings = {
  appTitle: 'జాతక చక్రం',
  loadingEngine: 'పేజ్ లోడ్ అవుతోంది…',
  dateOfBirth: 'పుట్టిన తేదీ',
  timeOfBirth: 'పుట్టిన సమయం',
  location: 'పుట్టిన ప్రాంతం',
  locationPlaceholder: 'నగరాన్ని ఎంచుకోండి…',
  dateRequired: 'పుట్టిన తేదీ అవసరం',
  timeRequired: 'పుట్టిన సమయం అవసరం',
  locationRequired: 'పుట్టిన ప్రాంతం అవసరం',
  calculating: 'గణిస్తోంది…',
  calculate: 'గణించు',
  selectCity: 'నగరాన్ని ఎంచుకోండి',
  searchPlaceholder: 'నగరం, ప్రాంతం, దేశం వెతకండి…',
  noCitiesFound: 'నగరాలు కనుగొనబడలేదు',
  calculationDetails: 'వివరాలు',
  city: 'నగరం',
  latitude: 'అక్షాంశం',
  longitude: 'రేఖాంశం',
  timezone: 'కాల మండలం(టైమ్ జోన్)',
  bodyLongitudes: 'గ్రహ స్ఫుటలు',
  bodyCol: 'గ్రహం',
  longitudeCol: 'స్ఫుట',
  nakshatraCol: 'నక్షత్రం',
  padaCol: 'పాదం',
  rasiCol: 'రాశి',
  navamsaCol: 'నవాంశ',
  language: 'భాష',
  rasi: 'రాశి',
  navamsa: 'నవాంశ',
  selectTime: 'సమయం ఎంచుకోండి',
  hours: 'గంటలు',
  minutes: 'నిమిషాలు',
  ok: 'సరే',
  cancel: 'రద్దు',
  vimsottariDasa: 'వింశోత్తరి దశ',
};

const TRANSLATIONS: Record<Lang, UIStrings> = { en, te };

export function getT(lang: Lang): UIStrings {
  return TRANSLATIONS[lang] ?? en;
}
