export type Lang = 'en' | 'te';

export interface CityRecord {
  lang: string;
  cityId: number;
  timeZone: string;
  canonicalName: string;
  cityName: string;
  region1: string;
  region2: string;
  lat: number;
  lng: number;
}

export interface PlanetaryPosition {
  name: string;
  abbrev: string;
  longitude: number;
  zodiacNumber: number;
  zodiacSign: string;
  zodiacAbbrev: string;
  degreesInSign: number;
  minutes: number;
  seconds: number;
  nakshatra: number;
  nakshatraName: string;
  pada: number;
  navamsaZodiacNumber: number;
  navamsaZodiacSign: string;
  navamsaZodiacAbbrev: string;
}

export interface HoroscopeResponse {
  lang: string;
  cityId: number;
  cityName: string;
  region1: string;
  region2: string;
  lat: number;
  lng: number;
  timezone: string;
  planets: Record<string, PlanetaryPosition>;
}

export interface WasmError {
  code: number;
  message: string;
}

// ─── UI state models ─────────────────────────────────────────────────────────

export interface AppState {
  lang: Lang;
  wasmReady: boolean;
  result: HoroscopeResponse | null;
  error: string | null;
}

export interface BirthFormState {
  cityId: number | null;
  cityDisplayName: string;
  localTime: string;
  modalOpen: boolean;
  loading: boolean;
}

export interface CityModalState {
  query: string;
  cities: CityRecord[];
  loading: boolean;
  noResults: boolean;
}

// ─── Component props ──────────────────────────────────────────────────────────

export interface ChakraGridProps {
  planets: Record<string, PlanetaryPosition>;
  title: string;
  navamsa?: boolean;
}

export interface LongitudeTableProps {
  planets: Record<string, PlanetaryPosition>;
  lang: Lang;
}

export interface EchoPanelProps {
  response: HoroscopeResponse;
  dateOfBirth: string;
  timeOfBirth: string;
  lang: Lang;
}

export interface CityModalProps {
  open: boolean;
  lang: Lang;
  onSelect: (city: CityRecord) => void;
  onClose: () => void;
}

// ─── Vimsottari Dasa types ────────────────────────────────────────────────────

export interface AntardasaEntry {
  lord:      string;   // canonical English planet key (e.g. "Venus")
  label:     string;   // translated: "{planet_name} {dasa.antar}"
  startDate: string;   // "YYYY MonthName DD" — month in requested lang
  endDate:   string;   // "YYYY MonthName DD"
  isCurrent: boolean;  // true if today falls within startDate..endDate; set by Rust
}

export interface MahadasaEntry {
  lord:       string;
  label:      string;           // translated: "{planet_name} {dasa.maha}"
  startDate:  string;
  endDate:    string;
  isCurrent:  boolean;          // true if any child AntardasaEntry.isCurrent; set by Rust
  antardasas: AntardasaEntry[];
}

export interface VimsottariResponse {
  lang:     string;
  cityId:   number;
  cityName: string;
  region1:  string;
  region2:  string;
  lat:      number;
  lng:      number;
  timezone: string;
  periods:  MahadasaEntry[];
}

export interface VimsottariPanelProps {
  dasa:    VimsottariResponse | null;
  loading: boolean;
  error:   string | null;
  lang:    Lang;
}

export interface LanguageSelectorProps {
  current: Lang;
}

// ─── SIGN_CELL — South Indian 4×4 chakra layout ───────────────────────────────
// Maps zodiacNumber (1–12, Aries=1) → { row, col } (0-indexed).
// Pisces (12) is top-left {0,0}; centre 2×2 (rows 1–2, cols 1–2) = label area.
export const SIGN_CELL: Record<number, { row: number; col: number }> = {
  1:  { row: 0, col: 1 },  // Aries
  2:  { row: 0, col: 2 },  // Taurus
  3:  { row: 0, col: 3 },  // Gemini
  4:  { row: 1, col: 3 },  // Cancer
  5:  { row: 2, col: 3 },  // Leo
  6:  { row: 3, col: 3 },  // Virgo
  7:  { row: 3, col: 2 },  // Libra
  8:  { row: 3, col: 1 },  // Scorpio
  9:  { row: 3, col: 0 },  // Sagittarius
  10: { row: 2, col: 0 },  // Capricorn
  11: { row: 1, col: 0 },  // Aquarius
  12: { row: 0, col: 0 },  // Pisces
};

export const PLANET_ORDER = [
  'Ascendant', 'Sun', 'Moon', 'Mars', 'Mercury',
  'Jupiter', 'Venus', 'Saturn', 'Rahu', 'Ketu',
] as const;
