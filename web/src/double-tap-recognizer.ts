import type mapboxgl from "mapbox-gl";

export function installDoubleTap(
    map: mapboxgl.Map,
    handler: (evt: mapboxgl.MapTouchEvent) => void
) {
    // @ts-expect-error unused
    const _unused = new DoubleTapRecognizer(map, handler);
}

class DoubleTapRecognizer {
    handler: (evt: mapboxgl.MapTouchEvent) => void;
    lastTime: number = 0;
    lastLocation?: mapboxgl.Point;

    constructor(map: mapboxgl.Map, handler: (evt: mapboxgl.MapTouchEvent) => void) {
        this.handler = handler;
        map.on("touchend", this.ontouchend.bind(this));
    }

    ontouchend(evt: mapboxgl.MapTouchEvent) {
        if (this.lastTime && this.lastLocation) {
            // Allow buffer time for double tap
            if (Date.now() - this.lastTime < 400) {
                if (this.lastLocation.dist(evt.point) < 18) {
                    // Is double touch. Call handler
                    this.handler(evt);
                }
            }
        }

        this.lastLocation = evt.point;
        this.lastTime = Date.now();
    }
}
