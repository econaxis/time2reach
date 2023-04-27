import mapboxgl from "mapbox-gl"

export function CityPill ({ name, onClick }) {
    const classes = "bg-white p-2 px-3 mx-1 rounded-full shadow-lg font-medium text-gray-800 font-sans "
  const hover = "hover:bg-gray-100 "
  const active = "active:bg-gray-300 "

  return <button onClick={onClick} className={classes + hover + active}>{name}</button>
}

const CITY_LOCATION = {
    Toronto: new mapboxgl.LngLat(-79.3832, 43.6532),
    "New York City": new mapboxgl.LngLat(-74.0060, 40.7128),
    Montreal: new mapboxgl.LngLat(-73.5674, 45.5019),
    Vancouver: new mapboxgl.LngLat(-123.1207, 49.2827)
}

export function CityPillContainer ({ cities, setLocation }) {
    const cityOnClick = (city: string) => {
        const cityLocation = CITY_LOCATION[city]
        setLocation(cityLocation)
  }
    const pills = cities.map(city => <CityPill key={city} name={city} onClick={() => {
        cityOnClick(city)
    }} />)

  return <div className="z-10 absolute top-0 left-0 mt-6 ml-4">{pills}</div>
}
