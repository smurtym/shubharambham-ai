declare global {
  interface Window {
    Module: {
      _malloc: (size: number) => number;
      _bridge: (
        opPtr: number,
        inPtr: number,
        outPtr: number,
        outSize: number
      ) => number;
      _free: (ptr: number) => void;
      HEAPU8: Uint8Array;
      onRuntimeInitialized?: () => void;
    };
  }
}
export {};
