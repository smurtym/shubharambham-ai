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
                <AccordionSummary expandIcon={<ExpandMoreIcon />}>
                  <Box
                    sx={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      width: '100%',
                      flexWrap: 'wrap',
                      gap: 0.5,
                    }}
                  >
                    <Typography
                      fontWeight={mahaActive ? 700 : 400}
                      color={mahaActive ? 'primary.main' : 'inherit'}
                    >
                      {maha.label}
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      {maha.startDate} – {maha.endDate}
                    </Typography>
                  </Box>
                </AccordionSummary>

                {/* T008/US2 — Antardasa rows */}
                <AccordionDetails sx={{ p: 0 }}>
                  {maha.antardasas.map(antar => {
                    const antarActive = antar.isCurrent;
                    return (
                      <Box
                        key={antar.lord}
                        sx={{
                          display: 'flex',
                          justifyContent: 'space-between',
                          alignItems: 'center',
                          px: 2,
                          py: 0.75,
                          flexWrap: 'wrap',
                          gap: 0.5,
                          ...(antarActive && {
                            bgcolor: 'action.selected',
                            borderLeft: '3px solid',
                            borderLeftColor: 'primary.main',
                          }),
                        }}
                      >
                        <Typography
                          variant="body2"
                          fontWeight={antarActive ? 600 : 400}
                        >
                          {antar.label}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          {antar.startDate} – {antar.endDate}
                        </Typography>
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
