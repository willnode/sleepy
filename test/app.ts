import { Readable } from "stream";

setTimeout(() => {
    const server = Bun.serve({
        port: parseInt(process.env.PORT || '4000'),
        fetch(request) {
            const path = (new URL(request.url)).pathname;
            if (path.startsWith("/ws")) {
                console.log('SERVER incoming request');
                if (this.upgrade(request)) {
                    return;
                }
            }
            if (path.startsWith("/streaming")) {
                const stream = new ReadableStream({
                    // @ts-ignore
                    type: "direct",
                    async pull(controller) {
                        for (let i = 1; i <= 10; i++) {
                            // @ts-ignore
                            controller.write(`Number: ${i}\n`);
                            await new Promise((resolve) => setTimeout(resolve, 1000)); // 1 second delay
                        }
                        // @ts-ignore
                        controller.write("Welcome to Bun!\n");
                        controller.close();
                    },
                });

                return new Response(stream, {
                    headers: {
                        "Content-Type": "text/plain",
                        "Transfer-Encoding": "chunked", // Optional, helps some clients
                    },
                });
            }

            console.log(`SERVER responded from: ${request.url}`);
            return new Response("Welcome to Bun!\n");
        },
        websocket: {
            open(ws) {
                console.log(`SERVER open`);
                ws.ping();
            },
            close(ws, code, reason) {
                console.log(`SERVER close: code=${code} reason=${reason}`);
            },
            message(ws, message) {
                console.log(`SERVER message: ${Bun.inspect(message)}`);
                ws.send(`You said: ${message}`);
            }
        },
    });
    console.log(`SERVER started: ${server.port}`);
}, 3000) // This timeout is intended to simulate slow startup within proxy
