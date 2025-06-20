setTimeout(() => {
    const server = Bun.serve({
        port: parseInt(process.env.PORT || '4000'),
        idleTimeout: 100,
        routes: {
            '/': req => {
                console.log('SERVER incoming /');
                return new Response("Welcome to Bun!\n");
            },
            '/ws': req => {
                console.log('SERVER incoming /ws');
                if (this.upgrade(req)) {
                    return;
                }
            },
            '/sleep': async req => {
                console.log('SERVER incoming /sleep');
                await new Promise((resolve) => setTimeout(resolve, 5000)); // 5 second delay
                return new Response("Welcome to Bun!\n");
            },
            '/streaming': req => {
                console.log('SERVER incoming /streaming');
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
            },
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
