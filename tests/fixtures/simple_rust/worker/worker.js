addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest(request) {
    const { greet } = wasm_bindgen;
    await wasm_bindgen(wasm)
    const greeting = greet()
    return new Response(greeting, {status: 200})
}
