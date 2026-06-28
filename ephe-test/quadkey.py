#!/usr/bin/env python3
"""Compute zoom-15 quadkey city_id (base-4 quadkey read as decimal) from lat/lng.

Mirrors decode_city_id() in astro-wasm/src/city_data.rs (inverse direction).
"""
import math

ZOOM = 15
TILES = 1 << ZOOM  # 32768


def encode_city_id(lat: float, lng: float) -> int:
    # standard slippy-map / Bing tile coords
    tile_x = int((lng + 180.0) / 360.0 * TILES)
    sin_lat = math.sin(lat * math.pi / 180.0)
    tile_y = int((0.5 - math.log((1 + sin_lat) / (1 - sin_lat)) / (4 * math.pi)) * TILES)
    tile_x = max(0, min(TILES - 1, tile_x))
    tile_y = max(0, min(TILES - 1, tile_y))
    digits = []
    for i in range(ZOOM, 0, -1):
        digit = 0
        mask = 1 << (i - 1)
        if tile_x & mask:
            digit += 1
        if tile_y & mask:
            digit += 2
        digits.append(str(digit))
    return int("".join(digits), 4)


def decode_city_id(city_id: int):
    """Port of decode_city_id() in city_data.rs — for round-trip verification."""
    tile_x = 0
    tile_y = 0
    q = city_id
    for k in range(ZOOM):
        digit = q % 4
        q //= 4
        tile_x |= (digit & 1) << k
        tile_y |= ((digit >> 1) & 1) << k
    lng = (tile_x + 0.5) / TILES * 360.0 - 180.0
    n = math.pi * (1.0 - 2.0 * (tile_y + 0.5) / TILES)
    lat = math.atan(math.sinh(n)) * 180.0 / math.pi
    return lat, lng


if __name__ == "__main__":
    import sys
    # Round-trip self-test: decode a known id, re-encode, expect same id.
    for cid in (466083478, 466083476, 467157992, 466862975):
        lat, lng = decode_city_id(cid)
        back = encode_city_id(lat, lng)
        print(f"{cid} -> ({lat:.4f},{lng:.4f}) -> {back}  {'OK' if back == cid else 'MISMATCH'}")
