// import { format_time } from "./format-details";
// import { map, refetch_data } from "./ol";
// import { getData } from "./data-promise";
//
// function setup_agency_form() {
//     document.getElementById("agency-form").onchange = async () => {
//         await refetch_data(undefined);
//     };
// }
//
// export const duration_range = <HTMLInputElement>(
//     document.getElementById("duration-range")
// );
// const duration_label = document.getElementById("duration-label");
// function setup_duration_range() {
//     const value = parseInt(duration_range.value);
//     duration_label.innerText = format_time(value).substring(0, 5);
//
//     duration_range.addEventListener("change", () => {
//         const value = parseInt(duration_range.value);
//         duration_label.innerText = format_time(value).substring(0, 5);
//
//         if (getData()) {
//             getData().max = getData().min + value;
//             getData().calculate_colors();
//             map.setPaintProperty("transit-layer", "line-color", [
//                 "get",
//                 ["to-string", ["id"]],
//                 ["literal", getData().m],
//             ]);
//         }
//     });
// }
//
// export const modes_form = document.getElementById("modes-form");
// function setup_modes() {
//     modes_form.addEventListener("change", async () => {
//         await refetch_data(undefined);
//     });
// }
//
// export default function setup() {
//     setup_agency_form();
//     setup_duration_range();
//     setup_modes();
// }
