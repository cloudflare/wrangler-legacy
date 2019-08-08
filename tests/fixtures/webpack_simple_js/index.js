addEventListener('fetch', event => {
    event.respondWith(handleRequest(event.request))
  })
  
  /**
   * Fetch and log a request
   * @param {Request} request
   */
  async function handleRequest(request) {
    return new Response('Hello worker!', { status: 200 })
  }