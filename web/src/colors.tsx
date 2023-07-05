import createColorMap from 'colormap'
import type mapboxgl from 'mapbox-gl'
import { baseUrl } from "./dev-api"
import { BG_WHITE_COLOR } from "./app";
import { formatDuration } from "./format-details";
import { Header } from "./control-sidebar";
import type { ComponentChildren } from "preact";

const NSHADES = 80
export const cmap = createColorMap({
  alpha: 0.4,
  colormap: 'temperature',
  format: 'hex',
  nshades: NSHADES
})

// function mapper (value: number): number {
//   value = 1.1 / (1 + Math.exp(-3 * (2 * value - 1.2))) - 0.03
//   return value
// }

export function getColor0To1 (value: number): string {
  if (value < 0 || value > 1) {
    console.log('invalid value', value)
  }

  value = 1 - value

  let index = Math.trunc(value * NSHADES)

  if (index >= cmap.length) index = cmap.length - 1
  else if (index < 0) index = 0

  return cmap[index]
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

      if (normalized <= 1.0) {
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

export interface ColorLegendProps {
  tcm: TimeColorMapper
  currentHover?: number
}

export interface TickTriangleProps {
  lpercentage: number
}

export interface TickProps extends TickTriangleProps {
  children: ComponentChildren
  noRotate?: boolean
}
function Tick ({ lpercentage, noRotate, children }: TickProps) {
  const color = "rgb(38,38,38)"
  return <div className="absolute left-0 inline-block" style={{ left: `${lpercentage}%` }}>
    <span className="inline-block text-xs font-extralight" style={{ transform: noRotate ? "" : "translate(-80%, -30%) rotate(45deg)" }}>{children}</span>
    <svg width="0.5" height="4">
      <rect x="0" y="0" width="0.5" height="4" fill={color} />
    </svg>
  </div>
}

function TickTriangle ({ lpercentage }: TickTriangleProps) {
  return <div className="absolute left-0 inline-block" style={{ left: `${lpercentage}%`, transform: "translateY(9px)", transition: "transform 1s" }}>
    <span className="inline-block text-md font-extralight">▼</span>
  </div>
}

export function ColorLegend ({ tcm, currentHover }: ColorLegendProps) {
  const numSteps = 12
  const cssGradient: string[] = []

  for (let i = 0; i <= numSteps; i++) {
    const fraction = i / numSteps
    const color = getColor0To1(fraction)
    cssGradient.push(`${color} ${(fraction * 100).toFixed(1)}%`)
  }

  const spread = tcm.max - tcm.min;

  const ticks: any[] = []
  for (let i = 0; i <= spread + 1; i += 3600 * 0.5) {
    const percentage = Math.round(i / spread * 100)
    const duration = formatDuration(i);
    const cleaned = duration.substring(1, 5);
    ticks.push(<Tick key={i.toFixed(0)} lpercentage={percentage}>{cleaned}</Tick>)
  }

  if (currentHover) ticks.push(<TickTriangle key={"hover"} lpercentage={(currentHover - tcm.min) / spread * 100}/>)

  const cssStyle = "linear-gradient(to right," + cssGradient.join(',') + ")"
  return <div className={`${BG_WHITE_COLOR} absolute bottom-0 l-0 m-4 z-50 pb-5 pt-2 px-9 rounded-lg`} style={{ width: 400 }}>
      <Header>Legend (Duration of Trip)</Header>
      <div className="w-full m-auto mt-7 relative left-0 top-0" style={{ height: '1.7rem' }}>
        {ticks}
      </div>
      <div className="rounded-md w-full m-auto" style={{ background: cssStyle, height: "1.5rem" }}></div>
  </div>
}
