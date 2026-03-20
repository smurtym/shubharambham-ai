import React, { useState, useEffect } from 'react';
import { Box, CircularProgress, Alert, Stack, Typography } from '@mui/material';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import dayjs from 'dayjs';
import 'dayjs/locale/te';

import LanguageSelector from './components/LanguageSelector';
import BirthForm from './components/BirthForm';
import CityModal from './components/CityModal';
import ChakraGrid from './components/ChakraGrid';
import EchoPanel from './components/EchoPanel';
import LongitudeTable from './components/LongitudeTable';
import { getHoroscopePositions } from './astro-glue';
import { getT } from './i18n';
import type { Lang, HoroscopeResponse, CityRecord, WasmError } from './types';

function readLang(): Lang {
  const p = new URLSearchParams(window.location.search).get('lang');
  return p === 'te' ? 'te' : 'en';
}

export default function App() {
  const lang = readLang();
  const t = getT(lang);

  // WASM gate (FR-011)
  const [wasmReady, setWasmReady] = useState(false);

  // Calculation result / error
  const [result, setResult] = useState<HoroscopeResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  // Birth form state (owned here, passed as props to BirthForm)
  const [cityId, setCityId] = useState<number | null>(null);
  const [cityDisplayName, setCityDisplayName] = useState('');
  const [dateStr, setDateStr] = useState('');
  const [timeStr, setTimeStr] = useState('');
  const [modalOpen, setModalOpen] = useState(false);

  useEffect(() => {
    document.documentElement.lang = lang;
  }, [lang]);

  useEffect(() => {
    const handler = () => setWasmReady(true);
    document.addEventListener('wasm-ready', handler, { once: true });
    return () => document.removeEventListener('wasm-ready', handler);
  }, []);

  function clearResult() {
    setResult(null);
    setError(null);
  }

  // FR-003: selecting a city stores its id and display name
  function handleCitySelect(city: CityRecord) {
    setCityId(city.cityId);
    setCityDisplayName(`${city.cityName}, ${city.region1}`);
    setModalOpen(false);
    clearResult(); // FR-015
  }

  // FR-015: any input change clears the chart immediately
  function handleDateChange(v: string) {
    setDateStr(v);
    clearResult();
  }

  function handleTimeChange(v: string) {
    setTimeStr(v);
    clearResult();
  }

  // FR-005, FR-012: Calculate
  async function handleCalculate() {
    if (!cityId) return;
    const localTime = dateStr + 'T' + timeStr + ':00'; // seconds default to :00
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const res = await getHoroscopePositions(cityId, localTime, lang);
      setResult(res);
    } catch (e: unknown) {
      const wasmErr = e as WasmError;
      setError(wasmErr?.message ?? String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <LocalizationProvider dateAdapter={AdapterDayjs} adapterLocale={lang}>
    <Box
      sx={{
        p: { xs: 1.5, sm: 3 },
        maxWidth: 900,
        mx: 'auto',
      }}
    >
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1 }}>
        <Typography variant="h5" fontWeight={700}>
          {t.appTitle}
        </Typography>
        <LanguageSelector current={lang} />
      </Box>

      {/* FR-011: loading indicator while WASM init */}
      {!wasmReady && (
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, my: 2 }}>
          <CircularProgress size={20} />
          <Typography variant="body2" color="text.secondary">
            {t.loadingEngine}
          </Typography>
        </Box>
      )}

      {/* US1: Birth details form */}
      <BirthForm
        wasmReady={wasmReady}
        loading={loading}
        lang={lang}
        cityId={cityId}
        cityDisplayName={cityDisplayName}
        dateStr={dateStr}
        timeStr={timeStr}
        onLocationClick={() => setModalOpen(true)}
        onDateChange={handleDateChange}
        onTimeChange={handleTimeChange}
        onCalculate={handleCalculate}
      />

      {/* City search modal (FR-002) */}
      <CityModal
        open={modalOpen}
        lang={lang}
        onSelect={handleCitySelect}
        onClose={() => setModalOpen(false)}
      />

      {/* FR-013: error display */}
      {error && (
        <Alert severity="error" sx={{ mt: 2 }}>
          {error}
        </Alert>
      )}

      {/* Result area: charts + echo + table */}
      {result && (
        <>
          {/* FR-006, FR-007: Rasi + Navamsa chakras */}
          <Stack
            direction={{ xs: 'column', sm: 'row' }}
            spacing={2}
            sx={{ mt: 3 }}
            alignItems="flex-start"
          >
            <Box sx={{ flex: 1, width: '100%' }}>
              <ChakraGrid planets={result.planets} title={t.rasi} navamsa={false} />
            </Box>
            <Box sx={{ flex: 1, width: '100%' }}>
              <ChakraGrid planets={result.planets} title={t.navamsa} navamsa={true} />
            </Box>
          </Stack>

          {/* FR-008: echo panel */}
          <EchoPanel
            response={result}
            dateOfBirth={dateStr}
            timeOfBirth={timeStr}
            lang={lang}
          />

          {/* FR-009: longitude table */}
          <LongitudeTable planets={result.planets} lang={lang} />
        </>
      )}
    </Box>
    </LocalizationProvider>
  );
}
