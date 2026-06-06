import React from 'react';
import { Box, Typography } from '@mui/material';
import { SIGN_CELL, PLANET_ORDER } from '../types';
import type { ChakraGridProps, PlanetaryPosition } from '../types';

// Fixed 3×3 sub-grid positions [row, col] (0-indexed) per planet count.
// Matches the layout diagram: 1→center, 2→diagonal, 3→top-row+bottom-mid,
// 4→corners, 5→corners+center, 6→top+bottom rows, 7–9→fill remaining cells.
const SUBGRID_POSITIONS: Array<[number, number]>[] = [
  /* 1 */ [[1, 1]],
  /* 2 */ [[0, 0], [2, 2]],
  /* 3 */ [[0, 0], [0, 2], [2, 1]],
  /* 4 */ [[0, 0], [0, 2], [2, 0], [2, 2]],
  /* 5 */ [[0, 0], [0, 2], [1, 1], [2, 0], [2, 2]],
  /* 6 */ [[0, 0], [0, 1], [0, 2], [2, 0], [2, 1], [2, 2]],
  /* 7 */ [[0, 0], [0, 1], [0, 2], [1, 1], [2, 0], [2, 1], [2, 2]],
  /* 8 */ [[0, 0], [0, 1], [0, 2], [1, 0], [1, 2], [2, 0], [2, 1], [2, 2]],
  /* 9 */ [[0, 0], [0, 1], [0, 2], [1, 0], [1, 1], [1, 2], [2, 0], [2, 1], [2, 2]],
];

function getSubgridPos(count: number, index: number): [number, number] {
  const positions = SUBGRID_POSITIONS[Math.min(count, 9) - 1] ?? SUBGRID_POSITIONS[8];
  return positions[index] ?? [1, 1];
}

export default function ChakraGrid({ planets, title, navamsa = false }: ChakraGridProps) {
  // Build a map: grid key "r,c" → planet abbreviations
  const cellMap: Record<string, string[]> = {};
  let ascendantCell: string | null = null;

  for (const key of PLANET_ORDER) {
    const planet: PlanetaryPosition | undefined = planets[key];
    if (!planet) continue;
    const signNum = navamsa ? planet.navamsaZodiacNumber : planet.zodiacNumber;
    const pos = SIGN_CELL[signNum];
    if (!pos) continue;
    const cellKey = `${pos.row},${pos.col}`;
    if (!cellMap[cellKey]) cellMap[cellKey] = [];
    cellMap[cellKey].push(planet.abbrev);
    if (key === 'Ascendant') {
      ascendantCell = cellKey;
    }
  }

  return (
    <Box
      sx={{
        display: 'grid',
        gridTemplateColumns: 'repeat(4, 1fr)',
        gridTemplateRows: 'repeat(4, 1fr)',
        border: '2px solid',
        borderColor: 'divider',
        aspectRatio: '1 / 1',
        width: '100%',
        maxWidth: 360,
        mx: 'auto',
      }}
    >
      {/* Centre 2×2 label area */}
      <Box
        sx={{
          gridRow: '2 / 4',
          gridColumn: '2 / 4',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          border: '1px solid',
          borderColor: 'divider',
          bgcolor: 'action.hover',
        }}
      >
        <Typography variant="caption" fontWeight={600} sx={{ letterSpacing: 1 }}>
          {title}
        </Typography>
      </Box>

      {/* 12 sign cells */}
      {Object.entries(SIGN_CELL).map(([signNumStr, pos]) => {
        const cellKey = `${pos.row},${pos.col}`;
        const abbrevs = cellMap[cellKey] ?? [];
        const isLagna = cellKey === ascendantCell;

        return (
          <Box
            key={signNumStr}
            sx={{
              gridRow: `${pos.row + 1} / ${pos.row + 2}`,
              gridColumn: `${pos.col + 1} / ${pos.col + 2}`,
              border: '1px solid',
              borderColor: 'divider',
              borderLeft: isLagna ? '3px solid' : undefined,
              borderLeftColor: isLagna ? 'primary.main' : undefined,
              p: 0.25,
              display: 'grid',
              gridTemplateColumns: 'repeat(3, 1fr)',
              gridTemplateRows: 'repeat(3, 1fr)',
              overflow: 'hidden',
              minWidth: 0,
            }}
          >
            {abbrevs.map((abbrev, i) => {
              const [r, c] = getSubgridPos(abbrevs.length, i);
              return (
                <Typography
                  key={i}
                  component="div"
                  sx={{
                    gridRow: r + 1,
                    gridColumn: c + 1,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    lineHeight: 1,
                    fontSize: abbrev.length > 3
                      ? { xs: '0.85rem', sm: '0.9rem' }
                      : { xs: '0.85rem', sm: '0.9rem' },
                    fontWeight: isLagna && i === 0 ? 700 : 400,
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                  }}
                >
                  {abbrev}
                </Typography>
              );
            })}
          </Box>
        );
      })}
    </Box>
  );
}
