import { Fragment } from 'preact'
import { Sidebar } from "./control-sidebar"

import { Bus, Train, Subway, Tram } from './svg-icons'

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

function toTitleCase (str: string): string {
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

export function formatDuration (secs: number): string {
    return new Date(secs * 1000).toISOString().substring(11, 19)
}

export function formatTime (secs: number): string {
    const t = new Date((secs % (12 * 3600)) * 1000).toISOString().substring(11, 16)
    const ampm = secs >= 12 * 3600
    const nextday = secs >= 24 * 3600

    if (nextday) {
        return t + " am (next day)"
    } else if (ampm) {
        return t + " pm"
    } else {
        return t + " am"
    }
}

// const SVG_ICONS = {
//     bus: <svg width="50px" height="50px" viewBox="0 0 24 24" fill="none" ><g id="SVGRepo_bgCarrier" strokeWidth="0"></g><g id="SVGRepo_tracerCarrier" strokeLineCap="round" stroke-linejoin="round"></g><g id="SVGRepo_iconCarrier"> <path fill-rule="evenodd" clip-rule="evenodd" d="M3 7.29412V8V11C3 10.4477 2.55228 10 2 10C1.44772 10 1 10.4477 1 11V13C1 13.5523 1.44772 14 2 14C2.55228 14 3 13.5523 3 13V14V16V17.8824C3 18.7868 3.37964 19.6274 4 20.2073V22C4 22.5523 4.44772 23 5 23H6C6.55228 23 7 22.5523 7 22V21H17V22C17 22.5523 17.4477 23 18 23H19C19.5523 23 20 22.5523 20 22V20.2073C20.6204 19.6274 21 18.7868 21 17.8824V16V14V13C21 13.5523 21.4477 14 22 14C22.5523 14 23 13.5523 23 13V11C23 10.4477 22.5523 10 22 10C21.4477 10 21 10.4477 21 11V8V7.29412C21 5.99868 20.7518 4.90271 20.2417 4.00093C19.727 3.091 18.9841 2.44733 18.1114 2.00201C16.4263 1.14214 14.2016 1 12 1C9.79836 1 7.57368 1.14214 5.88861 2.00201C5.01594 2.44733 4.27305 3.091 3.7583 4.00093C3.24816 4.90271 3 5.99868 3 7.29412ZM18 19C18.4992 19 19 18.5542 19 17.8824V16V14C19 13.4477 18.5523 13 18 13H6C5.44772 13 5 13.4477 5 14V16V17.8824C5 18.5542 5.5008 19 6 19H18ZM18 7C18.5523 7 19 7.44772 19 8V11.1707C18.6872 11.0602 18.3506 11 18 11H6C5.64936 11 5.31278 11.0602 5 11.1707V8C5 7.44772 5.44772 7 6 7H18ZM18.5009 4.98568C18.5124 5.006 18.5238 5.02663 18.535 5.04757C18.3614 5.01632 18.1826 5 18 5H6C5.8174 5 5.63862 5.01631 5.46502 5.04756C5.47622 5.02663 5.48757 5.006 5.49906 4.98568C5.79396 4.46439 6.22263 4.07691 6.79768 3.78347C8.00804 3.16583 9.78336 3 12 3C14.2166 3 15.992 3.16583 17.2023 3.78347C17.7774 4.07691 18.206 4.46439 18.5009 4.98568ZM6 15.5C6 14.6716 6.67157 14 7.5 14C8.32843 14 9 14.6716 9 15.5C9 16.3284 8.32843 17 7.5 17C6.67157 17 6 16.3284 6 15.5ZM16.5 14C15.6716 14 15 14.6716 15 15.5C15 16.3284 15.6716 17 16.5 17C17.3284 17 18 16.3284 18 15.5C18 14.6716 17.3284 14 16.5 14Z" fill="#000000"></path> </g></svg>,
//     tram: <svg width="50px" height="50px" fill="#000000" viewBox="0 0 50 50"  ><g id="SVGRepo_bgCarrier" strokeWidth="0"></g><g id="SVGRepo_tracerCarrier" strokeLineCap="round" stroke-linejoin="round"></g><g id="SVGRepo_iconCarrier"><path d="M25 0C21.820313 0 19.257813 0.234375 17.375 0.78125C16.433594 1.054688 15.644531 1.394531 15.03125 1.90625C14.417969 2.417969 14 3.1875 14 4C13.996094 4.359375 14.183594 4.695313 14.496094 4.878906C14.808594 5.058594 15.191406 5.058594 15.503906 4.878906C15.816406 4.695313 16.003906 4.359375 16 4C16 3.8125 16.027344 3.707031 16.3125 3.46875C16.597656 3.230469 17.160156 2.945313 17.9375 2.71875C18.523438 2.546875 19.269531 2.394531 20.09375 2.28125L20.9375 7.21875C17.660156 7.578125 14.917969 8.367188 12.8125 9.40625C9.886719 10.847656 8 12.789063 8 15L8 38.28125C8 41.371094 10.191406 43.960938 13.125 44.75L8.9375 48.25C8.632813 48.46875 8.476563 48.839844 8.535156 49.207031C8.59375 49.578125 8.851563 49.882813 9.210938 50C9.566406 50.113281 9.957031 50.015625 10.21875 49.75L15.90625 45L34.09375 45L39.78125 49.75C40.042969 50.015625 40.433594 50.113281 40.789063 50C41.148438 49.882813 41.40625 49.578125 41.464844 49.207031C41.523438 48.839844 41.367188 48.46875 41.0625 48.25L36.875 44.75C39.808594 43.960938 42 41.371094 42 38.28125L42 15C42 12.789063 40.113281 10.847656 37.1875 9.40625C35.082031 8.367188 32.339844 7.578125 29.0625 7.21875L29.90625 2.28125C30.730469 2.394531 31.476563 2.546875 32.0625 2.71875C32.839844 2.945313 33.402344 3.230469 33.6875 3.46875C33.972656 3.707031 34 3.8125 34 4C33.996094 4.359375 34.183594 4.695313 34.496094 4.878906C34.808594 5.058594 35.191406 5.058594 35.503906 4.878906C35.816406 4.695313 36.003906 4.359375 36 4C36 3.1875 35.582031 2.417969 34.96875 1.90625C34.355469 1.394531 33.566406 1.054688 32.625 0.78125C30.742188 0.234375 28.179688 0 25 0 Z M 25 2C26.042969 2 26.988281 2.035156 27.875 2.09375L27.0625 7.03125C26.394531 6.996094 25.707031 7 25 7C24.292969 7 23.605469 6.996094 22.9375 7.03125L22.125 2.09375C23.011719 2.035156 23.957031 2 25 2 Z M 25 9C29.871094 9 33.738281 9.949219 36.3125 11.21875C38.886719 12.488281 40 14.058594 40 15L40 38.28125C40 40.867188 37.796875 43 35 43L15 43C12.203125 43 10 40.867188 10 38.28125L10 15C10 14.058594 11.113281 12.488281 13.6875 11.21875C16.261719 9.949219 20.128906 9 25 9 Z M 19 12C17.914063 12 17 12.914063 17 14L17 16L15 16C13.398438 16 12 17.242188 12 18.84375L12 27.15625C12 28.757813 13.398438 30 15 30L35 30C36.601563 30 38 28.757813 38 27.15625L38 18.84375C38 17.242188 36.601563 16 35 16L33 16L33 14C33 12.914063 32.085938 12 31 12 Z M 19 14L31 14L31 16L19 16 Z M 15 18L35 18C35.609375 18 36 18.421875 36 18.84375L36 27.15625C36 27.578125 35.609375 28 35 28L15 28C14.390625 28 14 27.578125 14 27.15625L14 18.84375C14 18.421875 14.390625 18 15 18 Z M 16 33.0625C13.832031 33.0625 12.0625 34.832031 12.0625 37C12.0625 39.167969 13.832031 40.9375 16 40.9375C18.167969 40.9375 19.9375 39.167969 19.9375 37C19.9375 34.832031 18.167969 33.0625 16 33.0625 Z M 34 33.0625C31.832031 33.0625 30.0625 34.832031 30.0625 37C30.0625 39.167969 31.832031 40.9375 34 40.9375C36.167969 40.9375 37.9375 39.167969 37.9375 37C37.9375 34.832031 36.167969 33.0625 34 33.0625 Z M 16 34.9375C17.144531 34.9375 18.0625 35.855469 18.0625 37C18.0625 38.144531 17.144531 39.0625 16 39.0625C14.855469 39.0625 13.9375 38.144531 13.9375 37C13.9375 35.855469 14.855469 34.9375 16 34.9375 Z M 34 34.9375C35.144531 34.9375 36.0625 35.855469 36.0625 37C36.0625 38.144531 35.144531 39.0625 34 39.0625C32.855469 39.0625 31.9375 38.144531 31.9375 37C31.9375 35.855469 32.855469 34.9375 34 34.9375Z"></path></g></svg>,
//     subway: <svg width="50px" height="50px" version="1.1" id="_x32_" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" xml:space="preserve" fill="#000000"><g id="SVGRepo_bgCarrier" strokeWidth="0"></g><g id="SVGRepo_tracerCarrier" strokeLineCap="round" stroke-linejoin="round"></g><g id="SVGRepo_iconCarrier"><g> <path className="st0" d="M411.61,512h57.07l-95.817-114.102c37.022-7.162,65.006-39.719,65.006-78.826V80.335 C437.869,28.535,387.563,0,255.437,0C123.302,0,73.003,28.535,73.003,80.335v238.738c0,39.451,28.49,72.207,66.01,78.972L43.32,512 h57.07l64.279-76.574H347.3L411.61,512z M357.535,337.833c-15.336,0-27.776-12.44-27.776-27.777s12.44-27.776,27.776-27.776 c15.344,0,27.777,12.44,27.777,27.776S372.879,337.833,357.535,337.833z M182.617,35.368c0-2.13,1.716-3.83,3.83-3.83h137.98 c2.114,0,3.83,1.7,3.83,3.83V59.23c0,2.122-1.716,3.83-3.83,3.83h-137.98c-2.115,0-3.83-1.708-3.83-3.83V35.368z M112.799,207.346 V106.465c0-9.874,7.998-17.872,17.88-17.872h249.523c9.867,0,17.872,7.997,17.872,17.872v100.881 c0,9.874-8.005,17.878-17.872,17.878H130.679C120.797,225.224,112.799,217.219,112.799,207.346z M125.561,310.056 c0-15.336,12.433-27.776,27.77-27.776c15.343,0,27.776,12.44,27.776,27.776s-12.433,27.777-27.776,27.777 C137.995,337.833,125.561,325.392,125.561,310.056z"></path> </g> </g></svg>,
//     rail: <i className="fa-solid fa-train"></i>
// }

const SVG_ICONS = {
    bus: <Bus style={{ display: 'inline', width: '12px', height: '12px' }}/>,
    tram: <Tram style={{ display: 'inline', width: '12px', height: '12px' }}/>,
    train: <Train style={{ display: 'inline', width: '12px', height: '12px' }}/>,
    subway: <Subway style={{ display: 'inline', width: '12px', height: '12px' }}/>
}

function ModeIcon ({
        mode,
        boarding,
        background_color,
        text_color
    }
) {
    const icon = SVG_ICONS[mode]

    const styleString = {
        'background-color': background_color,
        color: text_color
    }

    let inner
    if (boarding.line.trim() === '') {
        inner = <Fragment>{icon}</Fragment>
    } else {
        inner = <Fragment>
            {icon} {boarding.line.trim()}
        </Fragment>
    }

    return <span className="rounded p-0.5 px-1" style={styleString}>
        {inner}
    </span>
}

function formatWalkingDuration (secs: number) {
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

function SmallSpan ({
    children,
    light
}) {
    let className
    if (light) {
        className = 'text-xs text-gray-500'
    } else {
        className = 'text-xs text-gray-800'
    }
    return <span className={className}>{children}</span>
}

export function DetailEntryTransit ({ detail }) {
    console.assert(detail.method === 'Transit')

    return (
        <div className="px-1 my-2 border-l-red-200 border-l-4 rounded font-medium">
            <div>
                <SmallSpan>
                    <ModeIcon{...detail}></ModeIcon>&nbsp;
                    {formatStop(detail.boarding.stop)}
                </SmallSpan>&nbsp;
                <SmallSpan light>
                    {formatDuration(detail.boarding.time)}
                </SmallSpan>
            </div>
            <div>
                <SmallSpan>Exit at {formatStop(detail.exit.stop)}</SmallSpan>
                &nbsp;
                <SmallSpan light>
                    {formatDuration(detail.exit.time)}
                </SmallSpan>
            </div>
        </div>
    )
}

export function DetailEntryWalking ({ detail }) {
    console.assert(detail.method === 'Walking')
    return (
        <div className="px-1 my-2 border-l-gray-200 border-l-4 rounded font-medium">
            <div>
                <SmallSpan>
                    Walk {formatWalkingDuration(detail.time)}&nbsp;
                    <SmallSpan light>({format_walking_distance(detail.length)})</SmallSpan>
                </SmallSpan>
            </div>
        </div>
    )
}

export function DetailPopup ({
    details,
    arrival_time: arrivalTime
}) {
    const detailEntries = details.map((d) => {
        if (d.method === 'Walking') {
            return <DetailEntryWalking detail={d}></DetailEntryWalking>
        } else {
            return <DetailEntryTransit detail={d}></DetailEntryTransit>
        }
    })

    return (
        <Sidebar
            positioning="bottom-0 right-0 absolute z-50"
        zi={100}>
            {detailEntries}
            <p className="mt-2 ml-1 text-xs font-bold">
                Arrival time: {formatDuration(arrivalTime)}
            </p>
        </Sidebar>
    )
}
