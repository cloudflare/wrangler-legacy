"use strict";
const hasURL = require("hasurl");

const {URL, URLSearchParams} = require( hasURL() ? "url" : "whatwg-url" );

const shim = () =>
{
	global.URL = URL;
	global.URLSearchParams = URLSearchParams;
};



module.exports = { shim, URL, URLSearchParams };
