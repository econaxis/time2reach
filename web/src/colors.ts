import createColorMap from 'colormap'
import type mapboxgl from 'mapbox-gl'
import { baseUrl } from "./dev-api";

const NSHADES = 300
export const cmap = createColorMap({
  alpha: 0.4,
  colormap: 'portland',
  format: 'hex',
  nshades: NSHADES
})

function mapper (value: number): number {
  value = 1.1 / (1 + Math.exp(-3 * (2 * value - 1.2))) - 0.03
  return value
}

export function getColor0To1 (value: number): string {
  if (value < 0 || value > 1) {
    console.log('invalid value', value)
  }

  value = Math.sqrt(value)
  value = mapper(value)

  value = 1 - value
  return cmap[Math.trunc(value * NSHADES)]
}

function objectToTrueValues (obj: Record<string, boolean>): string[] {
  console.log(obj)
  return Object.entries(obj).filter(([_, include]) => include).map(([key, _include]) => key)
}
export class TimeColorMapper {
  m: Record<number, any>
  raw: Record<number, any>
  min: number
  max: number
  request_id: any

  constructor (requestId: object, edgeTimes: Record<string, number>, durationRange: number) {
    this.m = {}
    this.min = 9999999999999
    this.max = -this.min
    this.raw = {}
    this.request_id = 0

    for (const nodeid in edgeTimes) {
      this.raw[nodeid.toString()] = edgeTimes[nodeid]
      const time = edgeTimes[nodeid]
      this.min = Math.min(this.min, time)
    }
    this.request_id = requestId

    this.max = this.min + durationRange
    this.calculate_colors()
  }

  static async fetch (location: mapboxgl.LngLat, startTime: number, durationRange: number, agencies: Record<string, boolean>, modes: Record<string, boolean>): Promise<TimeColorMapper> {
    const body = {
      latitude: location.lat,
      longitude: location.lng,
      agencies: objectToTrueValues(agencies),
      modes: objectToTrueValues(modes),
      startTime
    }

    const data = await fetch(`${baseUrl}/hello/`, {
      method: 'POST',
      mode: 'cors',
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(body)
    })
    const js = await data.json()

    const { request_id: requestId, edge_times: edgeTimes } = js

    return new TimeColorMapper(requestId, edgeTimes, durationRange)
  }

  calculate_colors (): void {
    const spread = this.max - this.min

    for (const id in this.raw) {
      let normalized = this.raw[id] - this.min
      normalized /= spread

      if (normalized > 1.0) {
      } else {
        const color = getColor0To1(normalized)
        if (color) {
          this.m[id] = color
        } else {
          console.log('err color', color, normalized)
        }
      }
    }
  }
}
