interface TripDetailsInner {
    time: number,
    line: number,
    stop: string
}

export interface TripDetails {
    boarding: TripDetailsInner,
    exit: TripDetailsInner
}

function toTitleCase(str) {
    return str.toLowerCase().split(/[\s()]/).map(function(word) {
        return (word.charAt(0).toUpperCase() + word.slice(1));
    }).join(" ");
}

function formatPearsonAirport(str) {
    let regex = /(PEARSON AIRPORT TERMINAL \d+)/;

    let match = str.match(regex);
    let station = str;
    if (match) {
        const airport = str.match(regex)[1];
        station = airport;
    }
    return station;
}

function formatStop(str) {
    let regex = /([\w.\s]+) Station - (\bNorth|\bEast|\bSouth|\bWest)bound Platform/g;

    let match = str.match(regex);
    let station = str;
    if (match) {
        const station_name = str.match(regex)[1];
        station = `${station_name} Station`;
    }

    station = formatPearsonAirport(station);

    return toTitleCase(station);
}

function format_time(secs: number): string {
    return new Date(secs * 1000).toISOString().substring(11, 19);
}

export function format_popup_html(arrival_time: number, details: Array<TripDetails>) {
    let detail_string = "";
    for (const detail of details) {
        const detail_template = `
    <div class="px-2 py-1 my-2 border-l-red-200 border-l-4 rounded font-medium leading-tight">
        <div>
            <span class="">Get on #${detail.boarding.line} at ${formatStop(detail.boarding.stop)}</span>
            <span class="text-xs text-gray-500">${format_time(detail.boarding.time)}</span>
        </div>
        <div>
            <span class="">Get off at ${formatStop(detail.exit.stop)}</span>
            <span class="text-xs text-gray-500">${format_time(detail.exit.time)}</span>
        </div>
    </div>`;
        detail_string += detail_template;
    }

    return `<div class="border max-w-xs rounded-lg bg-slate-50 p-2 pb-1 font-sans"> <p class="text-center">Arrived at <strong>${format_time(arrival_time)}</strong></p>${detail_string}</div> `;
}