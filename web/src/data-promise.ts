import { TimeColorMapper } from "./colors";
let data_promise: TimeColorMapper = null;

export function getData(): TimeColorMapper {
    return data_promise;
}

export function setData(p) {
    data_promise = p;
}

