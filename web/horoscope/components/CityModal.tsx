import React, { useState, useEffect, useCallback } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  TextField,
  List,
  ListItemButton,
  ListItemText,
  Typography,
  CircularProgress,
  Box,
} from '@mui/material';
import { listCities } from '../astro-glue';
import { getT } from '../i18n';
import type { CityRecord, CityModalProps } from '../types';

export default function CityModal({ open, lang, onSelect, onClose }: CityModalProps) {
  const t = getT(lang);
  const [allCities, setAllCities] = useState<CityRecord[]>([]);
  const [query, setQuery] = useState('');
  const [loadingCities, setLoadingCities] = useState(false);

  const loadCities = useCallback(async () => {
    setLoadingCities(true);
    try {
      const res = await listCities(lang);
      setAllCities(res.cities);
    } finally {
      setLoadingCities(false);
    }
  }, [lang]);

  useEffect(() => {
    if (open && allCities.length === 0) {
      loadCities();
    }
  }, [open, allCities.length, loadCities]);

  const q = query.toLowerCase();
  const filtered = q
    ? allCities.filter(
        (c) =>
          c.cityName.toLowerCase().includes(q) ||
          c.canonicalName.toLowerCase().includes(q) ||
          c.region1.toLowerCase().includes(q) ||
          c.region2.toLowerCase().includes(q)
      )
    : allCities;

  const noResults = q.length > 0 && filtered.length === 0;

  return (
    <Dialog
      open={open}
      onClose={onClose}
      fullScreen
      PaperProps={{ sx: { maxWidth: 600, mx: 'auto', my: { sm: 4 }, height: { sm: 'calc(100% - 64px)' } } }}
    >
      <DialogTitle>{t.selectCity}</DialogTitle>
      <DialogContent sx={{ display: 'flex', flexDirection: 'column', gap: 1, pb: 0 }}>
        <TextField
          autoFocus
          fullWidth
          placeholder={t.searchPlaceholder}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          size="small"
          sx={{ minHeight: 44 }}
          inputProps={{ style: { fontSize: 16 } }}
        />
        {loadingCities && (
          <Box sx={{ display: 'flex', justifyContent: 'center', py: 4 }}>
            <CircularProgress size={28} />
          </Box>
        )}
        {noResults && (
          <Typography sx={{ py: 2, textAlign: 'center', color: 'text.secondary' }}>
            {t.noCitiesFound}
          </Typography>
        )}
        {!loadingCities && !noResults && (
          <List disablePadding sx={{ overflowY: 'auto', flex: 1 }}>
            {filtered.map((city) => (
              <ListItemButton
                key={city.cityId}
                onClick={() => onSelect(city)}
                sx={{ minHeight: 44 }}
              >
                <ListItemText
                  primary={`${city.cityName}, ${city.region1}`}
                  secondary={city.region2}
                />
              </ListItemButton>
            ))}
          </List>
        )}
      </DialogContent>
    </Dialog>
  );
}
