import { useQuery } from "react-query";
import { useEffect, useRef, useState } from "preact/hooks";
import { TimeSlider } from "./time-slider";
import { baseUrl } from "./dev-api";
import { BG_WHITE_COLOR } from "./app";
import track from "./analytics";

interface Agency {
    agencyCode: string;
    agencyLongName: string;
    city: string;
}

export interface AgencyEntryProps {
    setSelectValue: (value: string, status: string) => void;
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

export function Header({ children }) {
    return <h2 className="font-medium text-lg font-bold border-b mt-3">{children}</h2>;
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

            <form id="agency-form" className="mt-2">
                {agencyList}
            </form>
        </div>
    );
}

export interface SidebarProps {
    positioning?: string;
    children?: any[];
    zi?: number;
}
export function Sidebar({ children, zi, positioning }: SidebarProps) {
    let classes = `absolute m-5 w-3/12 p-5 ${BG_WHITE_COLOR} border border-slate-400 rounded-lg drop-shadow-2xl shadow-inner `;
    classes += positioning ?? "";

    return (
        <div className={classes} style={{ zIndex: zi ?? 0 }}>
            {children}
        </div>
    );
}

async function fetchAgencies(): Promise<Agency[]> {
    const result = await fetch(`${baseUrl}/agencies`);
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
];

export function ControlSidebar({ setOptions, currentCity }) {
    const { isLoading, data } = useAgencies();

    if (isLoading) return null;
    if (data == null) throw new Error("data is null");

    const filtered = data.map((ag) => {
        return {
            shouldShow: ag.city === currentCity,
            ...ag,
        };
    });
    const agencies = useRef<object>(filtered);
    const modes = useRef<object>(MODES);

    const [duration, setDuration] = useState(2700);
    const [startTime, setStartTime] = useState(17 * 3600 + 40 * 60);
    const [minDuration, setMinDuration] = useState(0);

    const triggerRefetch = () => {
        console.log("Setting options", agencies.current, modes.current);
        setOptions({
            duration,
            startTime,
            agencies: agencies.current,
            modes: modes.current,
            minDuration,
        });
    };

    useEffect(triggerRefetch, []);
    useEffect(() => {
        triggerRefetch();
    }, [duration, startTime, minDuration]);
    const onAgencyChange = (agencies1: object) => {
        console.log("onAgencyChange");
        track("agency-change", agencies1);
        agencies.current = agencies1;
        triggerRefetch();
    };

    const onModeChange = (modes1: object) => {
        console.log("onModeChange");
        track("mode-change", modes1);
        modes.current = modes1;
        triggerRefetch();
    };

    return (
        <Sidebar zi={10} positioning="top-0 right-0">
            <p className="text-gray-700">
                <ul>
                    <li>Double click anywhere to set starting location.</li>
                    <li>Hover over a point to see the fastest path to get there.</li>
                </ul>
            </p>
            <AgencyForm agencies={filtered} header="Agencies" updateValues={onAgencyChange} />

            <AgencyForm agencies={MODES} header="Modes" updateValues={onModeChange} />

            <TimeSlider
                duration={duration}
                setDuration={setDuration}
                minDuration={minDuration}
                setMinDuration={setMinDuration}
                startTime={startTime}
                setStartTime={setStartTime}
            />
        </Sidebar>
    );
}
