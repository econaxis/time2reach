import { BG_WHITE_COLOR } from "./app";

export function CityPill ({ name, onClick, isCurrent }) {
    const classes = `${BG_WHITE_COLOR} p-1 px-3 mx-1 rounded-full drop-shadow-xl shadow-inner font-medium text-gray-900 bg-gray-100 font-sans `
    const hover = "hover:bg-gray-200 "
    const active = "active:bg-gray-400 "

    let current = ""

    if (isCurrent) {
        current += "bg-blue-400"
    }

    return <button onClick={onClick} className={classes + hover + active + current}>{name}</button>
}

export function CityPillContainer ({ cities, setLocation, currentCity }) {
    const cityOnClick = (city: string) => {
        setLocation(city)
    }
    const pills = cities.map(city => <CityPill key={city} name={city} isCurrent={city === currentCity} onClick={() => {
        cityOnClick(city)
    }} />)

    return <div className="z-10 absolute top-0 left-0 mt-5 ml-3">{pills}</div>
}
