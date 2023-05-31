import mapboxgl from "mapbox-gl"

export function CityPill ({ name, onClick }) {
    const classes = "bg-white p-2 px-3 mx-1 rounded-full shadow-lg font-medium text-gray-800 font-sans "
  const hover = "hover:bg-gray-100 "
  const active = "active:bg-gray-300 "

  return <button onClick={onClick} className={classes + hover + active}>{name}</button>
}



export function CityPillContainer ({ cities, setLocation }) {
    const cityOnClick = (city: string) => {
        setLocation(city)
  }
    const pills = cities.map(city => <CityPill key={city} name={city} onClick={() => {
        cityOnClick(city)
    }} />)

  return <div className="z-10 absolute top-0 left-0 mt-6 ml-4">{pills}</div>
}
