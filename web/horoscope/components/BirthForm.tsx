import React, { useState } from 'react';
import { Box, Button, TextField, Typography } from '@mui/material';
import AirDateInput from './AirDateInput';
import TimeInput from './TimeInput';
import { getT } from '../i18n';
import type { Lang } from '../types';

interface BirthFormProps {
  wasmReady: boolean;
  loading: boolean;
  lang: Lang;
  cityId: number | null;
  cityDisplayName: string;
  dateStr: string;
  timeStr: string;
  onLocationClick: () => void;
  onDateChange: (v: string) => void;
  onTimeChange: (v: string) => void;
  onCalculate: () => void;
}

export default function BirthForm({
  wasmReady,
  loading,
  lang,
  cityId,
  cityDisplayName,
  dateStr,
  timeStr,
  onLocationClick,
  onDateChange,
  onTimeChange,
  onCalculate,
}: BirthFormProps) {
  const t = getT(lang);
  const [dateError, setDateError] = useState(false);
  const [timeError, setTimeError] = useState(false);
  const [cityError, setCityError] = useState(false);

  function handleCalculate() {
    const d = !dateStr;
    const t = !timeStr;
    const c = cityId === null;
    setDateError(d);
    setTimeError(t);
    setCityError(c);
    if (!d && !t && !c) {
      onCalculate();
    }
  }

  const disabled = !wasmReady;

  return (
    <Box
      component="form"
      onSubmit={(e) => { e.preventDefault(); handleCalculate(); }}
      sx={{ display: 'flex', flexDirection: 'column', gap: 1.5, mt: 1 }}
      noValidate
    >
      <Box sx={{ display: 'flex', gap: 1.5, flexDirection: 'row' }}>
        <AirDateInput
          label={t.dateOfBirth}
          value={dateStr}
          onChange={(v) => { setDateError(false); onDateChange(v); }}
          disabled={disabled}
          error={dateError}
          helperText={dateError ? t.dateRequired : undefined}
          lang={lang}
        />
        <TimeInput
          label={t.timeOfBirth}
          value={timeStr}
          onChange={(v) => { setTimeError(false); onTimeChange(v); }}
          disabled={disabled}
          error={timeError}
          helperText={timeError ? t.timeRequired : undefined}
          lang={lang}
        />
      </Box>
      <Box>
        <TextField
          label={t.location}
          value={cityDisplayName || ''}
          placeholder={t.locationPlaceholder}
          onClick={disabled ? undefined : onLocationClick}
          inputProps={{ readOnly: true, style: { cursor: disabled ? 'default' : 'pointer' } }}
          disabled={disabled}
          error={cityError}
          helperText={cityError ? t.locationRequired : undefined}
          InputLabelProps={{ shrink: cityDisplayName ? true : undefined }}
          size="small"
          fullWidth
        />
        {cityError && !cityDisplayName && (
          <Typography variant="caption" color="error" sx={{ ml: 1.5 }} />
        )}
      </Box>
      <Button
        type="submit"
        variant="contained"
        disabled={disabled || loading}
        onClick={handleCalculate}
        sx={{ fontWeight: 600 }}
        fullWidth
      >
        {loading ? t.calculating : t.calculate}
      </Button>
    </Box>
  );
}
