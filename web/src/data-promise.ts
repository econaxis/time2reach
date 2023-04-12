import { TimeColorMapper } from "./colors";
let data_promise: TimeColorMapper | null = null;

export function getData(): TimeColorMapper {
    if(!data_promise) throw Error("Data promise not defined yet")
    return data_promise;
}

export function setData(p) {
    data_promise = p;
}
