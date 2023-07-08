import { useState } from "react";
import { formatDuration, formatTime } from "./format-details";
import { Header } from "./control-sidebar";
import track from "./analytics";

export function TimeSliderInner({ duration, setDuration, text, min, max, formatFunc }) {
    const [iduration, setIduration] = useState(duration);

    const onChange = (element) => {
        const dur = parseInt(element.target.value);
        console.log("CHANGED1!!", setDuration)
        setIduration(dur);
    };

    const onMouseUp = (element) => {
        track("range-change", { text });
        console.log("CHANGED2!!", setDuration)
        const dur = parseInt(element.target.value);
        setDuration(dur);
    };

    return (
        <div className="mt-2">
            <div>
                <label
                    htmlFor="duration-range"
                    className="float-left mb-1 text-sm font-medium text-gray-900"
                >
                    {text}
                </label>
                <span
                    id="duration-label"
                    className="float-right inline-block mb-1 text-sm font-light text-gray-700"
                >
                    {formatFunc(iduration)}
                </span>
            </div>
            <input
                id="duration-range"
                type="range"
                min={min}
                max={max}
                value={iduration}
                className="w-full h-1 bg-slate-300 rounded-lg appearance-none cursor-pointer"
                onChange={() => {
                    console.log("hello1")
                }}
                onMouseUp={() => {
                    console.log("hello2")
                }}
                onMouseOut={() => {
                    console.log("hello3")
                }}
                onInput={() => {
                    console.log("hello4")
                }}
            />
        </div>
    );
}

export function TimeSlider({ duration, setDuration, minDuration, setMinDuration, startTime, setStartTime }) {
    return (
        <div className="mt-2">
            <Header>Time Settings</Header>

            <TimeSliderInner
                duration={startTime}
                setDuration={setStartTime}
                formatFunc={formatTime}
                min="18000"
                max="104400"
                text="Starting time"
            />
            <TimeSliderInner
                duration={duration}
                setDuration={setDuration}
                formatFunc={(duration) => {
                    return formatDuration(duration).substring(0, 5)
                }}
                min="1800"
                max="8100"
                text="Maximum trip duration"
            />

            <TimeSliderInner
                duration={minDuration}
                setDuration={setMinDuration}
                formatFunc={(duration) => {
                    // 00:44:26
                    return formatDuration(duration).substring(0, 5)
                }}
                min="0"
                max={duration.toString()}
                text="Minimum trip duration"
            />
        </div>
    );
}
