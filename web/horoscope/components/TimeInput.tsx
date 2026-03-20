import React, { useState, useRef } from 'react';
import {
  Box,
  Button,
  Popover,
  TextField,
  Typography,
  Divider,
} from '@mui/material';
import { getT } from '../i18n';
import type { Lang } from '../types';

interface TimeInputProps {
  label: string;
  value: string;       // HH:mm or ''
  onChange: (v: string) => void;
  disabled?: boolean;
  error?: boolean;
  helperText?: string;
  lang: Lang;
}

function pad(n: number) {
  return String(n).padStart(2, '0');
}

export default function TimeInput({
  label, value, onChange, disabled = false, error = false, helperText, lang,
}: TimeInputProps) {
  const t = getT(lang);
  const anchorRef = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);

  // Parse current value into pending selection
  const [pendingHour, setPendingHour] = useState<number | null>(() => {
    if (!value) return null;
    const h = parseInt(value.split(':')[0], 10);
    return isNaN(h) ? null : h;
  });
  const [pendingMin, setPendingMin] = useState<number | null>(() => {
    if (!value) return null;
    const m = parseInt(value.split(':')[1], 10);
    return isNaN(m) ? null : m;
  });

  function handleOpen() {
    if (disabled) return;
    // Reset pending to current value each time popup opens
    if (value) {
      const [hStr, mStr] = value.split(':');
      setPendingHour(parseInt(hStr, 10));
      setPendingMin(parseInt(mStr, 10));
    } else {
      setPendingHour(null);
      setPendingMin(null);
    }
    setOpen(true);
  }

  function handleOk() {
    if (pendingHour !== null && pendingMin !== null) {
      onChange(`${pad(pendingHour)}:${pad(pendingMin)}`);
    }
    setOpen(false);
  }

  function handleCancel() {
    setOpen(false);
  }

  const displayValue = value
    ? `${pad(parseInt(value.split(':')[0], 10))}:${pad(parseInt(value.split(':')[1], 10))}`
    : '';

  const cellSx = (selected: boolean) => ({
    minWidth: 0,
    width: '100%',
    height: 36,
    fontSize: '0.9rem',
    fontFamily: '"Noto Sans Telugu", "Noto Sans", sans-serif',
    borderRadius: 1,
    border: '1px solid',
    borderColor: selected ? 'primary.main' : 'divider',
    bgcolor: selected ? 'primary.main' : 'transparent',
    color: selected ? 'primary.contrastText' : 'text.primary',
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    '&:hover': {
      bgcolor: selected ? 'primary.dark' : 'action.hover',
    },
    transition: 'background-color 0.15s',
  });

  return (
    <>
      <Box ref={anchorRef} sx={{ flex: 1 }}>
        <TextField
          label={label}
          value={displayValue}
          onClick={handleOpen}
          disabled={disabled}
          error={error}
          helperText={helperText}
          InputLabelProps={{ shrink: displayValue ? true : undefined }}
          inputProps={{ readOnly: true, style: { cursor: disabled ? 'default' : 'pointer' } }}
          size="small"
          fullWidth
        />
      </Box>

      <Popover
        open={open}
        anchorEl={anchorRef.current}
        onClose={handleCancel}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'left' }}
        transformOrigin={{ vertical: 'top', horizontal: 'left' }}
        PaperProps={{ sx: { p: 2, width: 320 } }}
      >
        <Typography variant="subtitle2" fontWeight={700} mb={1.5}>
          {t.selectTime}
        </Typography>

        {/* Hours */}
        <Typography variant="caption" color="text.secondary" fontWeight={600}>
          {t.hours}
        </Typography>
        <Box
          sx={{
            display: 'grid',
            gridTemplateColumns: 'repeat(6, 1fr)',
            gap: 0.5,
            mt: 0.5,
            mb: 1.5,
          }}
        >
          {Array.from({ length: 24 }, (_, i) => (
            <Box
              key={i}
              sx={cellSx(pendingHour === i)}
              onClick={() => setPendingHour(i)}
            >
              {pad(i)}
            </Box>
          ))}
        </Box>

        <Divider sx={{ mb: 1.5 }} />

        {/* Minutes */}
        <Typography variant="caption" color="text.secondary" fontWeight={600}>
          {t.minutes}
        </Typography>
        <Box
          sx={{
            display: 'grid',
            gridTemplateColumns: 'repeat(10, 1fr)',
            gap: 0.5,
            mt: 0.5,
            mb: 2,
          }}
        >
          {Array.from({ length: 60 }, (_, i) => (
            <Box
              key={i}
              sx={cellSx(pendingMin === i)}
              onClick={() => setPendingMin(i)}
            >
              {pad(i)}
            </Box>
          ))}
        </Box>

        {/* Footer */}
        <Box sx={{ display: 'flex', justifyContent: 'flex-end', gap: 1 }}>
          <Button size="small" onClick={handleCancel} color="inherit">
            {t.cancel}
          </Button>
          <Button
            size="small"
            variant="contained"
            onClick={handleOk}
            disabled={pendingHour === null || pendingMin === null}
          >
            {t.ok}
          </Button>
        </Box>
      </Popover>
    </>
  );
}
