import { BG_WHITE_COLOR } from "./app";
import track from "./analytics";

export function CityPill({ name, onClick, isCurrent }) {
    const classes = `${BG_WHITE_COLOR} p-1 px-3 mx-1 mt-2 rounded-full drop-shadow-xl shadow-inner font-medium text-gray-900 font-sans text-sm md:text-base`;
    const hover = "hover:bg-gray-200 ";
    const active = "active:bg-gray-400 ";

    let current = "";

    if (isCurrent) {
        current += "bg-blue-400";
    }

    return (
        <button onClick={onClick} className={classes + hover + active + current}>
            {name}
        </button>
    );
}

export function CityPillContainer({ cities, setLocation, currentCity }) {
    const cityOnClick = (city: string) => {
        track("city-change", { city });
        setLocation(city);
    };
    const pills = cities.map((city) => (
        <CityPill
            key={city}
            name={city}
            isCurrent={city === currentCity}
            onClick={() => {
                cityOnClick(city);
            }}
        />
    ));

    return <div className="z-10 absolute top-0 left-0 mt-3 ml-3">{pills}</div>;
}
