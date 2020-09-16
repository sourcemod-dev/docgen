addEventListener('fetch', event => {
    event.respondWith(handleRequest(event.request))
});

class TitleHandler {
    text(text) {
        text = 'new text';
    }
}

class DescriptionHandler {
    element(element) {
        console.log(element);
    }
}

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest(request) {
    const res = await fetch(request);

    return new HTMLRewriter()
        .on('title', new TitleHandler())
        .on('meta', new DescriptionHandler())
        .transform(res)

    // const { greet } = wasm_bindgen;
    // await wasm_bindgen(wasm)
    // const greeting = greet()
    // return new Response(greeting, {status: 200})
}
