import { createTheme } from '@mui/material/styles';

// Constitution II: Noto Sans Telugu as primary, Noto Sans for Latin glyphs.
const FONT = '"Noto Sans Telugu", "Noto Sans", sans-serif';

export const theme = createTheme({
  typography: {
    fontFamily: FONT,
  },
  components: {
    MuiCssBaseline: {
      styleOverrides: {
        body: { fontFamily: FONT },
      },
    },
  },
});
