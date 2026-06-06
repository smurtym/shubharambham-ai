import React from 'react';
import {
  Table,
  TableHead,
  TableBody,
  TableRow,
  TableCell,
  TableContainer,
  Paper,
  Typography,
  Box,
} from '@mui/material';
import { getT } from '../i18n';
import { PLANET_ORDER } from '../types';
import type { LongitudeTableProps } from '../types';

// Format longitude as "DD Sg MM' SS""
function formatLongitude(
  deg: number, min: number, sec: number, signAbbrev: string,
): string {
  return `${deg} ${signAbbrev} ${String(min).padStart(2, '0')}' ${String(Math.trunc(sec)).padStart(2, '0')}"`;
}

const hdrSx = { fontWeight: 700, px: 1, py: 0.75 };

export default function LongitudeTable({ planets, lang }: LongitudeTableProps) {
  const t = getT(lang);
  return (
    <TableContainer component={Paper} variant="outlined" sx={{ mt: 2 }}>
      <Typography variant="subtitle2" fontWeight={700} sx={{ px: 2, pt: 1.5 }}>
        {t.bodyLongitudes}
      </Typography>
      <Table size="small" sx={{ tableLayout: 'auto' }}>
        <TableHead>
          <TableRow>
            <TableCell sx={hdrSx}>{t.bodyCol}</TableCell>
            <TableCell sx={hdrSx}>{t.longitudeCol}</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {PLANET_ORDER.map((key) => {
            const p = planets[key];
            if (!p) return null;
            return (
              <React.Fragment key={key}>
                <TableRow>
                  <TableCell sx={{ px: 1, pb: 0, borderBottom: 'none' }}>
                    {p.name}
                  </TableCell>
                  <TableCell sx={{ px: 1, pb: 0, borderBottom: 'none', fontVariantNumeric: 'tabular-nums', whiteSpace: 'nowrap' }}>
                    {formatLongitude(p.degreesInSign, p.minutes, p.seconds, p.zodiacAbbrev)}
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell colSpan={2} sx={{ px: 1, pt: 0, pb: 0.75 }}>
                    <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: '2px 10px', color: 'text.secondary' }}>
                      <span>{p.nakshatraName}</span>
                      <span>{t.padaCol} {p.pada}</span>
                      <span>{p.zodiacAbbrev} ({t.rasiCol})</span>
                      <span>{p.navamsaZodiacAbbrev} ({t.navamsaCol})</span>
                    </Box>
                  </TableCell>
                </TableRow>
              </React.Fragment>
            );
          })}
        </TableBody>
      </Table>
    </TableContainer>
  );
}
