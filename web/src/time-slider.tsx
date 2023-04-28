import { useEffect, useState } from 'preact/hooks'
import { Header } from './app'
import { formatTime } from './format-details'

export function TimeSlider ({ setDuration }) {
    const [IDuration, setIDuration] = useState(3600)

    const onChange = (element) => {
        const dur = parseInt(element.target.value)
        setIDuration(dur)
        setDuration(dur)
    }
    useEffect(() => {
        setDuration(IDuration)
    }, [])
    return (
        <div className="mt-2">
            <Header>Time Settings</Header>

            <div className="mt-2">
                <div>
                    <label
                        htmlFor="duration-range"
                        className="float-left mb-1 text-sm font-medium text-gray-900"
                    >
                        Maximum duration of trip
                    </label>
                    <span
                        id="duration-label"
                        className="float-right inline-block mb-1 text-sm font-light text-gray-700"
                    >{formatTime(IDuration)}</span>
                </div>
                <input
                    id="duration-range"
                    type="range"
                    min="1800"
                    max="5400"
                    value={IDuration}
                    className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
                    onMouseUp={onChange}
                />
            </div>
        </div>
    )
}
