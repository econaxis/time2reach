import { useQuery } from "react-query";
import { type ReactNode, useEffect, useRef, useState } from "react";
import { TimeSlider } from "./time-slider";
import { baseUrl } from "./dev-api";
import { BG_WHITE_COLOR, CITY_LOCATION } from "./app";
import track from "./analytics";
import "./control-sidebar.css";
import { MapboxMap, setAndColorNewOriginLocation } from "./mapbox-map";
import { LoadingSpinner } from "./loading-spinner";
import { formatTime } from "./format-details";
import { GIF_RENDER_START_TIME, GIF_RENDER, useGifRenderNewAnimationFrame } from "./gif-generator";

interface Agency {
    agencyCode: string
    agencyLongName: string
    city: string
}

export interface AgencyEntryProps {
    setSelectValue: (value: string, status: string) => void
}

export function AgencyEntry({
    agencyCode,
    agencyLongName,
    setSelectValue,
}: Agency & AgencyEntryProps) {
    const onChange = (element: any) => {
        setSelectValue(agencyCode, element.target.checked);
    };

    const id = `agency-${agencyCode}`;
    return (
        <div>
            <input
                id={id}
                type="checkbox"
                className="checkbox"
                onChange={onChange}
                defaultChecked
            />
            <label htmlFor={id} className="ml-1 text-gray-900">
                {agencyLongName}
            </label>
        </div>
    );
}

export interface HeaderProps {
    children?: ReactNode
}
export function Header({ children }: HeaderProps) {
    return <h2 className="font-medium text-md border-b mt-2 md:mt-3">{children}</h2>;
}

export function AgencyForm({ agencies, header, updateValues }) {
    const values = useRef(Object.fromEntries(agencies.map((ag) => [ag.agencyCode, true])));

    useEffect(() => {
        updateValues(values.current);
    }, []);
    const setSelectValue = (value, status) => {
        values.current[value] = status;
        updateValues(values.current);
    };
    const agencyList = agencies
        .filter((ag) => ag.shouldShow || ag.shouldShow === undefined)
        .map((ag) => <AgencyEntry {...ag} setSelectValue={setSelectValue} key={ag.agencyCode} />);

    return (
        <div>
            <Header>{header}</Header>

            <form id="agency-form" className="mt-2 max-h-56 overflow-y-scroll">
                {agencyList}
            </form>
        </div>
    );
}

export interface SidebarProps {
    positioning?: string
    children?: any[]
    zi?: number
    style?: Record<string, any>
}

export function Sidebar({ children, zi, positioning, style }: SidebarProps) {
    let classes = `opacity-90 absolute m-4 w-3/12 md:max-w-sm p-5 pt-4 ${BG_WHITE_COLOR} border border-slate-400 rounded-lg drop-shadow-2xl shadow-inner `;
    classes += positioning ?? "";

    return (
        <div className={classes} style={{ zIndex: zi ?? 0, ...style }}>
            {children}
        </div>
    );
}

function getID(): string {
    // Check in localstorage for randomly generated ID string
    let id = localStorage.getItem("time2reach-random-id");
    if (!id) {
        // Generate a 64 bit random id
        id = (Math.random() * Math.pow(10, 18)).toString(10);
        localStorage.setItem("time2reach-random-id", id);
    }

    return id;
}
async function fetchAgencies(): Promise<Agency[]> {
    const result = await fetch(`${baseUrl}/agencies?id=${getID()}`);
    const json = await result.json();
    return json.map((agency) => {
        return {
            agencyCode: agency.short_code,
            agencyLongName: agency.public_name,
            city: agency.city,
        };
    });
}

function useAgencies() {
    return useQuery("agencies", fetchAgencies);
}

const MODES = [
    {
        agencyCode: "bus",
        agencyLongName: "Bus",
    },
    {
        agencyCode: "subway",
        agencyLongName: "Subway",
    },
    {
        agencyCode: "tram",
        agencyLongName: "Tram",
    },
    {
        agencyCode: "rail",
        agencyLongName: "Train",
    },
    {
        agencyCode: "ferry",
        agencyLongName: "Ferry",
    },
];

export function ControlSidebar({ defaultStartLoc, currentCity }) {
    const { isLoading, data } = useAgencies();

    const filtered = data
        ? data.map((ag) => {
              return {
                  shouldShow: (ag.city === currentCity) || (ag.city === "Toronto" && currentCity === "Kitchener-Waterloo"),
                  ...ag,
              };
          })
        : null;

    const agencies = useRef<object>({});

    const [duration, setDuration] = useState(2700);
    const [startTime, setStartTime] = useState(17 * 3600 + 40 * 60);
    const [minDuration, setMinDuration] = useState(0);

    const [currentOptions, setOptions] = useState<any>({ startTime, minDuration, duration });
    const [currentStartingLoc, setCurrentStartingLoc] = useState(defaultStartLoc);
    const [lastWorkingLocation, setLastWorkingLocation] = useState(defaultStartLoc);
    const [spinner, setSpinner] = useState(true);

    const [paintProperty, setPaintProperty] = useState<any>(null);
    const [timeData, setTimeData] = useState<any>(null);

    const [transferPenalty, setTransferPenalty] = useState(0);

    const cityLocation = CITY_LOCATION[currentCity];
    console.log("Current city mapbox", currentCity, cityLocation)

    useEffect(() => {
        setLastWorkingLocation(defaultStartLoc);
        setCurrentStartingLoc(defaultStartLoc);

        if (GIF_RENDER) {
            // setStartTime(4 * 3600)
            setStartTime(GIF_RENDER_START_TIME)
            // setStartTime(60240)
        }
    }, [defaultStartLoc]);

    useEffect(() => {
        if (!isLoading && currentOptions.agencies && currentOptions.modes && currentStartingLoc) {
            const joinedOptions = {
                ...currentOptions,
                duration,
                minDuration,
                startTime,
                transferPenalty
            };
            setSpinner(true);
            console.log("Start time is", formatTime(startTime))
            console.log("Location is", currentStartingLoc)
            setAndColorNewOriginLocation(currentStartingLoc, joinedOptions)
                .then((data) => {
                    setPaintProperty(data.m);
                    setTimeData(data);
                    setLastWorkingLocation(currentStartingLoc);
                })
                .catch((err) => {
                    setCurrentStartingLoc(lastWorkingLocation);
                    console.error("Got error in setAndColorNewOriginLocation", err);
                });
        }
    }, [currentOptions, currentStartingLoc, isLoading, duration, minDuration, startTime, transferPenalty]);

    // Activates only when GIF_RENDER = true
    useGifRenderNewAnimationFrame(spinner, startTime, setStartTime);

    const onAgencyChange = (agencies1: object) => {
        track("agency-change", agencies1);
        console.log("agency change!");
        agencies.current = agencies1;
        setOptions((options: any) => {
            options = options || {};
            // setOptions({
            //     duration,
            //     startTime,
            //     agencies: agencies.current,
            //     modes: modes.current,
            //     minDuration,
            // });
            return {
                ...options,
                agencies: agencies.current,
            };
        });
    };

    const onModeChange = (modes1: object) => {
        track("mode-change", modes1);
        console.log("mode change!");
        setOptions((options: any) => {
            options = options || {};
            // setOptions({
            //     duration,
            //     startTime,
            //     agencies: agencies.current,
            //     modes: modes.current,
            //     minDuration,
            // });
            return {
                ...options,
                modes: modes1,
            };
        });
    };

    return (
        <>
            <LoadingSpinner display={spinner} />
            <Sidebar
                zi={10}
                positioning="sm:top-0 sm:right-0 sm:block sm:hover:opacity-90 sm:opacity-30 transition-opacity sm:max-h-screen overflow-y-scroll
               top40"
            >
                <p className="text-gray-700">
                    <ul>
                        <li>Double click anywhere to set starting location.</li>
                        <li>Hover over a point to see the fastest path to get there.</li>
                    </ul>
                </p>
                {filtered ? (
                    <AgencyForm
                        agencies={filtered}
                        header="Agencies"
                        updateValues={onAgencyChange}
                    />
                ) : null}

                <AgencyForm agencies={MODES} header="Modes" updateValues={onModeChange} />

                <TimeSlider
                    duration={duration}
                    setDuration={(e) => {
                        setDuration(e);
                    }}
                    minDuration={minDuration}
                    setMinDuration={setMinDuration}
                    startTime={startTime}
                    setStartTime={setStartTime}
                    transferPenalty={transferPenalty}
                    setTransferPenalty={setTransferPenalty}
                />
                <p className="text-xs border-t mt-6 pt-3">
                    Find this project on{" "}
                    <a
                        href="https://github.com/econaxis/time2reach"
                        target="_blank"
                        rel="me noreferrer"
                        className="underline"
                    >
                        Github!
                    </a>
                </p>

                <p className="text-xs pt-3">
                    See my other projects{" "}
                    <a
                        href="https://henryn.xyz"
                        target="_blank"
                        className="underline"
                        rel="noreferrer"
                    >
                        here!
                    </a>
                </p>

            </Sidebar>
            <MapboxMap
                timeData={timeData}
                paintProperty={paintProperty}
                setLatLng={setCurrentStartingLoc}
                setSpinnerLoading={setSpinner}
                currentPos={cityLocation}
            />
        </>
    );
}
