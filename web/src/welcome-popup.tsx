import { BG_WHITE_COLOR } from "./app"
import { useEffect } from "preact/hooks"

export function WelcomePopup ({ acceptedPopupCallback }) {
    useEffect(() => {
        const storedData = localStorage.getItem('popup-accepted')
        if (storedData) {
            acceptedPopupCallback(true)
        }
    }, [])

    const acceptedPopup = () => {
        localStorage.setItem('popup-accepted', 'true')
        acceptedPopupCallback(true)
    }

    return <div className = "h-screen w-screen bg-transparent absolute" onClick={acceptedPopup}>
        <div className="z-50 relative top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-full max-w-xl" onClick={(evt) => {
            evt.stopPropagation()
        }}>
            {/* Modal content */}
            <div className={`relative ${BG_WHITE_COLOR} rounded-lg shadow`}>
                {/* Modal header */}
                <div className="flex items-start justify-between p-3.5 border-b rounded-t border-slate-300">
                    <h3 className="text-xl font-semibold text-gray-900 ">
                        Welcome!
                    </h3>
                    <button
                        type="button"
                        className="text-gray-400 bg-transparent hover:bg-gray-200 hover:text-gray-900 rounded-lg text-sm p-1.5 ml-auto inline-flex items-center"
                        data-modal-hide="defaultModal"
                        onClick={acceptedPopup}
                    >
                        <svg
                            aria-hidden="true"
                            className="w-5 h-5"
                            fill="currentColor"
                            viewBox="0 0 20 20"
                            xmlns="http://www.w3.org/2000/svg"
                        >
                            <path
                                fillRule="evenodd"
                                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                                clipRule="evenodd"
                            ></path>
                        </svg>
                        <span className="sr-only">Close modal</span>
                    </button>
                </div>
                {/* Modal body */}
                <div className="p-6 space-y-6 text-gray-500">
                    <p>
                        <b>time2reach</b> is an interactive map that lets you see how far you can go just using public transit.
                    </p>
                    <p>
                        <b>Double-click on the map to set your starting origin.</b> The map is colour-coded by how far and how quickly
                        you can travel from that point. Explore around to see how transit-accessible different parts of your city are!
                    </p>
                    <p>
                        Hover/click over any other point to view the route you must take to arrive their from your previously selected origin.
                    </p>
                </div>
                {/* Modal footer */}
                <div className="flex items-center justify-end p-6 space-x-2 border-t border-slate-300 rounded-b">
                    <button
                        data-modal-hide="defaultModal"
                        type="button"
                        className="text-gray-500 bg-white hover:bg-gray-100 focus:ring-4 focus:outline-none focus:ring-blue-300 rounded-lg border border-gray-200 text-sm font-medium px-5 py-2.5 hover:text-gray-900 focus:z-10 dark:bg-gray-700 dark:text-gray-300 dark:border-gray-500 dark:hover:text-white dark:hover:bg-gray-600 dark:focus:ring-gray-600"
                    >
                        Learn More
                    </button>
                    <button
                        data-modal-hide="defaultModal"
                        type="button"
                        className="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800"
                        onClick={acceptedPopup}
                    >
                        Go to map
                    </button>
                </div>
            </div>
        </div>
    </div>
}

export function BlurBackground ({ enabled, children }) {
    let classes: string
    if (enabled) {
        classes = "blur-sm pointer-events-none"
    } else {
        classes = ""
    }

    return <div className={classes}>
        {children}
    </div>
};
