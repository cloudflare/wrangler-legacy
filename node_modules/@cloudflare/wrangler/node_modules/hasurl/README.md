# hasurl [![NPM Version][npm-image]][npm-url] [![Build Status][travis-image]][travis-url]

> Determine whether Node.js' native [WHATWG `URL`](https://nodejs.org/api/url.html#url_the_whatwg_url_api) implementation is available.


## Installation

[Node.js](http://nodejs.org/) `>= 4` is required. To install, type this at the command line:
```shell
npm install hasurl
```


## Usage

```js
const hasURL = require('hasurl');

if (hasURL()) {
	// supported
} else {
	// fallback
}
```


[npm-image]: https://img.shields.io/npm/v/hasurl.svg
[npm-url]: https://npmjs.org/package/hasurl
[travis-image]: https://img.shields.io/travis/stevenvachon/hasurl.svg
[travis-url]: https://travis-ci.org/stevenvachon/hasurl
