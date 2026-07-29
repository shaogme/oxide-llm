"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const server_1 = require("./server");
const PORT = Number(process.env.PORT) || 3000;
const server = (0, server_1.createMockServer)(PORT);
async function main() {
    const url = await server.start();
    console.log(`Mock AI Server running at ${url}`);
}
main().catch((err) => {
    console.error(err);
    process.exit(1);
});
