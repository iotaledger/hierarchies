import url from "@iota/hierarchies/web/hierarchies_wasm_bg.wasm?url";

import { init } from "@iota/hierarchies/web";
import { main } from "../../../examples/dist/web/web-main";

export const runTest = async (example: string) => {
    try {
        await main(example);
        console.log("success");
    } catch (error) {
        throw (error);
    }
};

init(url)
    .then(() => {
        console.log("init");
    });