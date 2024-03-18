import { useContext, useState } from "react";
import { formatDuration, formatTime } from "./format-details";
import { Header } from "./control-sidebar";
import track from "./analytics";
import { BrightnessContext } from "./app";

interface TimeSliderInnerProps {
    duration: number
    setDuration: (duration: number) => void
    text: string
    min: string
    max: string
    formatFunc: (duration: number) => string
    title?: string
}

export function TimeSliderInner({ duration, setDuration, text, min, max, formatFunc, title }: TimeSliderInnerProps) {
    const [iduration, setIduration] = useState(duration);

    const onChange = (element: React.ChangeEvent<HTMLInputElement>) => {
        const dur = parseInt(element.target.value);
        setIduration(dur);
    };

    const onMouseUp = (element: React.MouseEvent<HTMLInputElement> | React.TouchEvent<HTMLInputElement>) => {
        track("range-change", { text });
        track("range-change", { text });
        const touchTarget = element.target as HTMLInputElement;
        const dur = parseInt(touchTarget.value);
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

interface TimeSliderProps {
    duration: number
    setDuration: (duration: number) => void
    startTime: number
    setStartTime: (time: number) => void
    transferPenalty: number
    setTransferPenalty: (penalty: number) => void
}

export function TimeSlider({
                               duration,
                               setDuration,
                               startTime,
                               setStartTime,
                               transferPenalty,
                               setTransferPenalty,
                           }: TimeSliderProps) {
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
                    return duration.toString();
                }}
                min="0"
                max="1800"
                text="Transfer Penalty (seconds)"
                title="Penalty to add to total trip time for each transfer"
            />
            {/* {false && ( */}
            {/*    <TimeSliderInner */}
            {/*        duration={minDuration} */}
            {/*        setDuration={setMinDuration} */}
            {/*        formatFunc={(duration) => { */}
            {/*            // 00:44:26 */}
            {/*            return formatDuration(duration).substring(0, 5); */}
            {/*        }} */}
            {/*        min="0" */}
            {/*        max={duration.toString()} */}
            {/*        text="Minimum trip duration" */}
            {/*    /> */}
            {/* )} */}

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
