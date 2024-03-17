import { useContext, useState } from "react";
import { formatDuration, formatTime } from "./format-details";
import { Header } from "./control-sidebar";
import track from "./analytics";
import { BrightnessContext } from "./app";

export function TimeSliderInner({ duration, setDuration, text, min, max, formatFunc, title }) {
    const [iduration, setIduration] = useState(duration);

    const onChange = (element) => {
        const dur = parseInt(element.target.value);
        setIduration(dur);
    };

    const onMouseUp = (element) => {
        track("range-change", { text });
        const dur = parseInt(element.target.value);
        setDuration(dur);
    };

    return (
        <div className="mt-2" title={title}>
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
                onChange={onChange}
                onMouseUp={onMouseUp}
                onTouchEnd={onMouseUp}
            />
        </div>
    );
}

export interface BrightnessContextInt {
    brightness: number
    setBrightness: (value: number) => void
}

export function TimeSlider({
    duration,
    setDuration,
    minDuration,
    setMinDuration,
    startTime,
    setStartTime,
    transferPenalty,
    setTransferPenalty
}) {
    const { brightness, setBrightness } = useContext<BrightnessContextInt>(BrightnessContext);
    return (
        <div className="mt-2">
            <Header>Time Settings</Header>

            <TimeSliderInner
                duration={startTime}
                setDuration={setStartTime}
                formatFunc={formatTime}
                min="10800"
                max="104400"
                text="Starting time"
            />
            <TimeSliderInner
                duration={duration}
                setDuration={setDuration}
                formatFunc={(duration) => {
                    return formatDuration(duration).substring(0, 5);
                }}
                min="1800"
                max="12240"
                text="Maximum trip duration"
            />

            <TimeSliderInner
                duration={transferPenalty}
                setDuration={setTransferPenalty}
                formatFunc={(duration) => {
                    return duration;
                }}
                min="0"
                max={"1200"}
                text={"Transfer Penalty"}
                title={"Penalty (seconds) to add to total trip time for each transfer"}
            />
            {false && (
                <TimeSliderInner
                    duration={minDuration}
                    setDuration={setMinDuration}
                    formatFunc={(duration) => {
                        // 00:44:26
                        return formatDuration(duration).substring(0, 5);
                    }}
                    min="0"
                    max={duration.toString()}
                    text="Minimum trip duration"
                />
            )}

            <TimeSliderInner
                duration={brightness}
                setDuration={setBrightness}
                formatFunc={(brightness) => {
                    return brightness.toString();
                }}
                min="100"
                max="300"
                text="Brightness"
            />
        </div>
    );
}
