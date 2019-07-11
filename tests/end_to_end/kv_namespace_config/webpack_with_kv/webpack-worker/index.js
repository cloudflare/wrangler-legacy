addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

/**
 * read value "bar" from key "foo" in KV namespace "TEST_KV_INTEGRATION"
 * @param {Request} request
 */
async function handleRequest(request) {
	try {
		let value = await TEST_KV_INTEGRATION.get('foo')

	    return new Response(`The value at key foo is ${value}`, { status: 200 })
	} catch (e) {
		return new Response(`internal error: ${e.message}`, { status: 500 })
	}
}
