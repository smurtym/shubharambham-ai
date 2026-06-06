import React, { useEffect, useRef } from 'react';
import { TextField } from '@mui/material';
import AirDatepicker from 'air-datepicker';
import 'air-datepicker/air-datepicker.css';
import './AirDateInput.css';
import type { AirDatepickerLocale } from 'air-datepicker';
import type { Lang } from '../types';

const TE_LOCALE: AirDatepickerLocale = {
  days: ['ఆదివారం', 'సోమవారం', 'మంగళవారం', 'బుధవారం', 'గురువారం', 'శుక్రవారం', 'శనివారం'],
  daysShort: ['ఆది', 'సోమ', 'మంగళ', 'బుధ', 'గురు', 'శుక్ర', 'శని'],
  daysMin: ['ఆ', 'సో', 'మం', 'బు', 'గు', 'శు', 'శ'],
  months: ['జనవరి', 'ఫిబ్రవరి', 'మార్చి', 'ఏప్రిల్', 'మే', 'జూన్', 'జులై', 'ఆగస్టు', 'సెప్టెంబర్', 'అక్టోబర్', 'నవంబర్', 'డిసెంబర్'],
  monthsShort: ['జనవరి', 'ఫిబ్రవరి', 'మార్చి', 'ఏప్రిల్', 'మే', 'జూన్', 'జులై', 'ఆగస్టు', 'సెప్టెంబర్', 'అక్టోబర్', 'నవంబర్', 'డిసెంబర్'],
  today: 'నేడు',
  clear: 'తొలగించు',
  dateFormat: 'dd/MM/yyyy',
  timeFormat: 'HH:mm',
  firstDay: 0,
};

const EN_LOCALE: AirDatepickerLocale = {
  days: ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'],
  daysShort: ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'],
  daysMin: ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'],
  months: ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'],
  monthsShort: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'],
  today: 'Today',
  clear: 'Clear',
  dateFormat: 'MM/dd/yyyy',
  timeFormat: 'HH:mm',
  firstDay: 0,
};

interface AirDateInputProps {
  label: string;
  value: string;           // YYYY-MM-DD
  onChange: (v: string) => void;
  disabled?: boolean;
  error?: boolean;
  helperText?: string;
  lang: Lang;
}

export default function AirDateInput({
  label, value, onChange, disabled = false, error = false, helperText, lang,
}: AirDateInputProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const dpRef = useRef<AirDatepicker | null>(null);
  const valueRef = useRef(value);
  valueRef.current = value;

  useEffect(() => {
    if (!inputRef.current) return;
    const locale = lang === 'te' ? TE_LOCALE : EN_LOCALE;

    dpRef.current = new AirDatepicker(inputRef.current, {
      locale,
      // Display format: yyyy-mmm-dd using locale's monthsShort
      dateFormat: (date: Date) => {
        const y = date.getFullYear();
        const m = locale.monthsShort[date.getMonth()];
        const d = String(date.getDate()).padStart(2, '0');
        return `${y}-${m}-${d}`;
      },
      autoClose: true,
      selectedDates: value ? [new Date(value)] : [],
      onSelect({ date }) {
        const d = Array.isArray(date) ? date[0] : date;
        if (!d) return;
        const iso = [
          d.getFullYear(),
          String(d.getMonth() + 1).padStart(2, '0'),
          String(d.getDate()).padStart(2, '0'),
        ].join('-');
        if (iso !== valueRef.current) onChange(iso);
      },
    });

    return () => {
      dpRef.current?.destroy();
      dpRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [lang]);

  // Sync external value changes into the picker without re-init
  useEffect(() => {
    if (!dpRef.current) return;
    const dp = dpRef.current;
    if (!value) {
      dp.clear();
    } else {
      const current = dp.selectedDates[0];
      const iso = current
        ? [
            current.getFullYear(),
            String(current.getMonth() + 1).padStart(2, '0'),
            String(current.getDate()).padStart(2, '0'),
          ].join('-')
        : '';
      if (iso !== value) dp.selectDate(new Date(value));
    }
  }, [value]);

  return (
    <TextField
      label={label}
      inputRef={inputRef}
      disabled={disabled}
      error={error}
      helperText={helperText}
      InputLabelProps={{ shrink: !!value }}
      size="small"
      sx={{ flex: 1 }}
      inputProps={{
        readOnly: true,
        inputMode: 'none',   // suppresses virtual keyboard on mobile
        autoComplete: 'off',
        style: { cursor: disabled ? 'default' : 'pointer' },
      }}
    />
  );
}
