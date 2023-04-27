import createColorMap from 'colormap'
import type mapboxgl from 'mapbox-gl'
import setLoading from './loading-spinner'
import { getCityFromUrl } from './ol'

const NSHADES = 300
export const cmap = createColorMap({
  alpha: 0.4,
  colormap: 'portland',
  format: 'hex',
  nshades: NSHADES
})

function mapper (value) {
  value = 1.1 / (1 + Math.exp(-3 * (2 * value - 1.2))) - 0.03
  return value
}

export function get_color_0_1 (value: number): string {
  if (value < 0 || value > 1) {
    console.log('invalid value', value)
  }

  value = Math.sqrt(value)
  value = mapper(value)
  return cmap[Math.trunc(value * NSHADES)]
}

function object_to_true_values (obj: Record<string, boolean>) {
  return Object.entries(obj).filter(([_, include]) => include).map(([key, include]) => key)
}
export class TimeColorMapper {
  m: Record<number, any>
  raw: Record<number, any>
  min: number
  max: number
  request_id: any

  constructor (request_id, edge_times, durationRange) {
    this.m = {}
    this.min = 9999999999999
    this.max = -this.min
    this.raw = {}
    this.request_id = 0

    for (const nodeid in edge_times) {
      this.raw[nodeid.toString()] = edge_times[nodeid]
      const time = edge_times[nodeid]
      this.min = Math.min(this.min, time)
    }
    this.request_id = request_id

    this.max = this.min + durationRange
    this.calculate_colors()
  }

  static async fetch (location: mapboxgl.LngLat, durationRange: number, agencies: Record<string, boolean>, modes: Record<string, boolean>) {
    const body = {
      latitude: location.lat,
      longitude: location.lng,
      agencies: object_to_true_values(agencies),
      modes: object_to_true_values(modes)
    }

    setLoading(true)
    const data = await fetch(`http://localhost:3030/hello/${getCityFromUrl()}`, {
      method: 'POST',
      mode: 'cors',
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(body)
    })
    const js = await data.json()

    const { request_id, edge_times } = js

    return new TimeColorMapper(request_id, edge_times, durationRange)
  }

  calculate_colors () {
    const spread = this.max - this.min

    for (const id in this.raw) {
      let normalized = this.raw[id] - this.min
      normalized /= spread

      if (normalized > 1.0) {
      } else {
        const color = get_color_0_1(normalized)
        if (color) {
          this.m[id] = color
        } else {
          console.log('err color', color, normalized)
        }
      }
    }
  }
}
