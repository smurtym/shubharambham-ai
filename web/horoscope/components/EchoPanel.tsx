import React from 'react';
import { Paper, Box, Typography } from '@mui/material';
import { getT } from '../i18n';
import type { EchoPanelProps } from '../types';

export default function EchoPanel({ response, dateOfBirth, timeOfBirth, lang }: EchoPanelProps) {
  const t = getT(lang);
  return (
    <Paper variant="outlined" sx={{ mt: 2, p: 2 }}>
      <Typography variant="subtitle2" fontWeight={700} gutterBottom>
        {t.calculationDetails}
      </Typography>
      <Box component="dl" sx={{ m: 0, display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '2px 12px' }}>
        <Typography component="dt" variant="body2" color="text.secondary">{t.dateOfBirth}</Typography>
        <Typography component="dd" variant="body2" sx={{ m: 0 }}>{dateOfBirth}</Typography>

        <Typography component="dt" variant="body2" color="text.secondary">{t.timeOfBirth}</Typography>
        <Typography component="dd" variant="body2" sx={{ m: 0 }}>{timeOfBirth}</Typography>

        <Typography component="dt" variant="body2" color="text.secondary">{t.city}</Typography>
        <Typography component="dd" variant="body2" sx={{ m: 0 }}>
          {response.cityName}, {response.region1}
        </Typography>

        <Typography component="dt" variant="body2" color="text.secondary">{t.latitude}</Typography>
        <Typography component="dd" variant="body2" sx={{ m: 0 }}>{response.lat.toFixed(3)}°</Typography>

        <Typography component="dt" variant="body2" color="text.secondary">{t.longitude}</Typography>
        <Typography component="dd" variant="body2" sx={{ m: 0 }}>{response.lng.toFixed(3)}°</Typography>

        <Typography component="dt" variant="body2" color="text.secondary">{t.timezone}</Typography>
        <Typography component="dd" variant="body2" sx={{ m: 0 }}>{response.timezone}</Typography>
      </Box>
    </Paper>
  );
}
