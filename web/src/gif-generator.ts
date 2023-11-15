import { saveAs } from "file-saver";
import { useEffect } from "react";
import { IS_LOCAL } from "./dev-api";

export function saveImage(name: string) {
    Object.defineProperty(window, "devicePixelRatio", {
        get: function() {
            return 350 / 96;
        },
    });

    const CANVAS_CLASS = ".mapboxgl-canvas";
    const canvas = document.querySelector(CANVAS_CLASS) as HTMLCanvasElement;

    // @ts-expect-error
    canvas.toBlob((blob) => {
        saveAs(blob, "test/" + name + ".png");
    });
}

export function useGifRenderNewAnimationFrame(
    spinner: boolean,
    startTime: number,
    setStartTime: (value: ((prevState: number) => number) | number) => void
) {
    useEffect(() => {
        if (!spinner && GIF_RENDER) {
            setTimeout(() => {
                saveImage(startTime.toString());
                setStartTime((prev) => prev + 60);
            }, 1500);
        }
    }, [spinner])
}

export let GIF_RENDER_START_TIME = 5 * 3600;
GIF_RENDER_START_TIME = 43500;
export const GIF_RENDER = IS_LOCAL && true;
