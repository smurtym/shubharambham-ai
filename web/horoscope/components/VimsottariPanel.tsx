import React, { useState } from 'react';
import {
  Alert,
  Accordion,
  AccordionDetails,
  AccordionSummary,
  Box,
  CircularProgress,
  SvgIcon,
  Typography,
} from '@mui/material';

function ExpandMoreIcon() {
  return (
    <SvgIcon><path d="M16.59 8.59 12 13.17 7.41 8.59 6 10l6 6 6-6z" /></SvgIcon>
  );
}

import { getT } from '../i18n';
import type { VimsottariPanelProps } from '../types';

export default function VimsottariPanel({ dasa, loading, error, lang }: VimsottariPanelProps) {
  const t = getT(lang);

  const [expanded, setExpanded] = useState<Set<number>>(() => {
    if (!dasa) return new Set();
    const idx = dasa.periods.findIndex(p => p.isCurrent);
    return idx >= 0 ? new Set([idx]) : new Set();
  });

  function toggleAccordion(i: number) {
    setExpanded(prev => {
      const next = new Set(prev);
      next.has(i) ? next.delete(i) : next.add(i);
      return next;
    });
  }

  return (
    <Box sx={{ mt: 3 }}>
      <Typography variant="h6" fontWeight={700} sx={{ mb: 1 }}>
        {t.vimsottariDasa}
      </Typography>

      {/* T010/US3 — loading state */}
      {loading && (
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, my: 2 }}>
          <CircularProgress size={20} />
        </Box>
      )}

      {/* T011/US3 — error state */}
      {!loading && error && (
        <Alert severity="error" sx={{ mt: 1 }}>
          {error}
        </Alert>
      )}

      {/* T006/US1 — Mahadasa accordion list */}
      {!loading && !error && dasa !== null && (
        <>
          {dasa.periods.map((maha, i) => {
            const mahaActive = maha.isCurrent;
            return (
              <Accordion
                key={maha.lord}
                expanded={expanded.has(i)}
                onChange={() => toggleAccordion(i)}
                disableGutters
              >
                <AccordionSummary
                  expandIcon={<ExpandMoreIcon />}
                  sx={{ borderLeft: '4px solid', borderLeftColor: mahaActive ? 'primary.main' : 'grey.500', bgcolor: '#e0ecf9' }}
                >
                  <Box sx={{ width: '100%' }}>
                    <Typography
                      fontWeight={mahaActive ? 700 : 400}
                      color={mahaActive ? 'primary.main' : 'inherit'}
                    >
                      {maha.label}
                    </Typography>
                    <Box sx={{ display: 'flex', mt: '2px' }}>
                      <Typography sx={{ width: '50%', pl: 2, pb: '4px' }}>{maha.startDate}</Typography>
                      <Typography sx={{ width: '50%', pb: '4px' }}>{maha.endDate}</Typography>
                    </Box>
                  </Box>
                </AccordionSummary>

                {/* T008/US2 — Antardasa rows */}
                <AccordionDetails sx={{ p: 0 }}>
                  {maha.antardasas.map((antar, j) => {
                    const antarActive = antar.isCurrent;
                    const isEven = j % 2 === 0;
                    return (
                      <Box
                        key={antar.lord}
                        sx={{
                          bgcolor: isEven ? 'grey.100' : 'background.paper',
                          borderLeft: antarActive ? '4px solid' : '4px solid transparent',
                          borderLeftColor: antarActive ? 'primary.main' : 'transparent',
                          pt: '4px',
                          pb: '4px',
                        }}
                      >
                        <Typography sx={{ pl: 2, fontWeight: antarActive ? 600 : 400 }}>{antar.label}</Typography>
                        <Box sx={{ display: 'flex', mt: '2px' }}>
                          <Typography sx={{ width: '50%', pl: 2, pb: '4px' }}>{antar.startDate}</Typography>
                          <Typography sx={{ width: '50%', pb: '4px' }}>{antar.endDate}</Typography>
                        </Box>
                      </Box>
                    );
                  })}
                </AccordionDetails>
              </Accordion>
            );
          })}
        </>
      )}
    </Box>
  );
}
