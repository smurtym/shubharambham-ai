import React from 'react';
import { FormControl, InputLabel, Select, MenuItem } from '@mui/material';
import type { SelectChangeEvent } from '@mui/material';
import { getT } from '../i18n';
import type { Lang, LanguageSelectorProps } from '../types';

const LANG_OPTIONS: { value: Lang; label: string }[] = [
  { value: 'en', label: 'English' },
  { value: 'te', label: 'తెలుగు' },
];

export default function LanguageSelector({ current }: LanguageSelectorProps) {
  const t = getT(current);
  function handleChange(e: SelectChangeEvent) {
    const lang = e.target.value as Lang;
    const url = new URL(window.location.href);
    url.searchParams.set('lang', lang);
    window.location.href = url.toString();
  }

  return (
    <FormControl size="small" sx={{ minWidth: 140 }}>
      <InputLabel id="lang-label">{t.language}</InputLabel>
      <Select
        labelId="lang-label"
        value={current}
        label={t.language}
        onChange={handleChange}
        sx={{ minHeight: 44 }}
      >
        {LANG_OPTIONS.map((opt) => (
          <MenuItem key={opt.value} value={opt.value} sx={{ minHeight: 44 }}>
            {opt.label}
          </MenuItem>
        ))}
      </Select>
    </FormControl>
  );
}
