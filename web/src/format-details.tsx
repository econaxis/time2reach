import { type FC } from "preact/compat";

interface TripDetailsInner {
  time: number
  line: number
  stop: string
}

export interface TripDetailsTransit {
  background_color: string
  text_color: string
  mode: string
  boarding: TripDetailsInner
  exit: TripDetailsInner
  method: string
}

export interface TripDetailsWalking {
  method: string
  time: number
  length: number
}

export type TripDetails = TripDetailsTransit | TripDetailsWalking

function toTitleCase (str: string): string{
  return str
    .toLowerCase()
    .split(/[\s()-/]/)
    .map(function (word) {
      return word.charAt(0).toUpperCase() + word.slice(1)
    })
    .join(' ')
}

function formatPearsonAirport (str: string): string {
  const regex = /(PEARSON AIRPORT TERMINAL \d+)/

  const match = str.match(regex)
  let station = str
  if (match) {
    station = match[1]
  }
  return station
}

function formatStop (str: string): string {
  const regex =
        /([\w.\s]+) Station - (\bNorth|\bEast|\bSouth|\bWest)bound Platform/g

  const match = str.match(regex)
  let station = str
  if (match) {
    const stationName = match[0]
    station = `${stationName} Station`
  }

  station = formatPearsonAirport(station)

  return toTitleCase(station)
}

export function formatTime (secs: number): string {
  return new Date(secs * 1000).toISOString().substring(11, 19)
}

function ModeIcon ({
  mode,
  boarding,
  background_color,
  text_color
}
): FC {
  const icon = {
    bus: <i className="fa-solid fa-bus-simple"></i>,
    tram: <i className="fa-solid fa-train-tram"></i>,
    subway: <i className="fa-solid fa-train-subway"></i>,
    rail: <i className="fa-solid fa-train"></i>
  }[mode]

  console.log('Setting ', background_color, text_color)
  const styleString = { 'background-color': background_color, color: text_color }

  return <span className="rounded p-0.5 px-1" style={styleString}>
        {icon} {boarding.line}
    </span>
}

function format_walking_duration (secs: number) {
  const minutes = Math.floor(secs / 60)
  const seconds = Math.round(secs % 60)

  if (minutes === 0) {
    return `${seconds}s`
  } else {
    return `${minutes}m${seconds}s`
  }
}

function format_walking_distance (length: number) {
  return `${Math.round(length / 10) * 10} meters`
}

export function DetailEntryTransit ({ detail }) {
  console.assert(detail.method === 'Transit')

  return (
        <div className="px-2 py-1 my-3 border-l-red-200 border-l-4 rounded font-medium">
            <div>
                <span>
                    <ModeIcon{...detail}></ModeIcon>&nbsp;
                    {formatStop(detail.boarding.stop)}
                </span>&nbsp;
                <span className="text-xs text-gray-500">
                    {formatTime(detail.boarding.time)}
                </span>
            </div>
            <div>
                <span>Exit at {formatStop(detail.exit.stop)}</span>
                &nbsp;
                <span className="text-xs text-gray-500">
                    {formatTime(detail.exit.time)}
                </span>
            </div>
        </div>
  )
}

export function DetailEntryWalking ({ detail }) {
  console.assert(detail.method === 'Walking')
  return (
        <div className="px-2 py-1 my-1 border-l-gray-200 border-l-4 rounded font-medium">
            <div>
                <span>
                    Walk {format_walking_duration(detail.time)} ({format_walking_distance(detail.length)})
                </span>
            </div>
        </div>
  )
}

export function DetailPopup ({ details, arrival_time }) {
  const detailEntries = details.map((d) => {
    console.log('detailpopup', d)
    if (d.method === 'Walking') { return <DetailEntryWalking detail={d}></DetailEntryWalking> } else return <DetailEntryTransit detail={d}></DetailEntryTransit>
  })

  return (
        <div className="border max-w-xs rounded-lg bg-slate-50 p-2 pb-1 font-sans">
            {detailEntries}
            <p>
                Arrival time <strong>{formatTime(arrival_time)}</strong>
            </p>
        </div>
  )
}
