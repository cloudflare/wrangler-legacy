# universal-url [![NPM Version][npm-image]][npm-url] [![Build Status][travis-image]][travis-url] [![Dependency Monitor][greenkeeper-image]][greenkeeper-url]

> WHATWG [`URL`](https://developer.mozilla.org/en/docs/Web/API/URL) for Node & Browser.


* For Node.js versions `>= 8`, the native implementation will be used.
* For Node.js versions `< 8`, a [shim](https://npmjs.com/whatwg-url) will be used.
* For web browsers without a native implementation, the same shim will be used.


## Installation

[Node.js](http://nodejs.org/) `>= 6` is required. To install, type this at the command line:
```shell
npm install universal-url
```


## Usage

```js
const {URL, URLSearchParams} = require('universal-url');

const url = new URL('http://domain/');
const params = new URLSearchParams('?param=value');
```

Global shim:
```js
require('universal-url').shim();

const url = new URL('http://domain/');
const params = new URLSearchParams('?param=value');
```


## Browserify/etc

The bundled file size of this library can be large for a web browser. If this is a problem, try using [universal-url-lite](https://npmjs.com/universal-url-lite) in your build as an alias for this module.


[npm-image]: https://img.shields.io/npm/v/universal-url.svg
[npm-url]: https://npmjs.org/package/universal-url
[travis-image]: https://img.shields.io/travis/stevenvachon/universal-url.svg
[travis-url]: https://travis-ci.org/stevenvachon/universal-url
[greenkeeper-image]: https://badges.greenkeeper.io/stevenvachon/universal-url.svg
[greenkeeper-url]: https://greenkeeper.io/
