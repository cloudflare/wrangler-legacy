"use strict";
const output = {};
let g, hasNative;



if (typeof window !== "undefined")
{
	g = window;
}
else if (typeof global !== "undefined")
{
	g = global;
}
else if (typeof self !== "undefined")
{
	g = self;
}
else
{
	g = this;
}



try
{
	const url = new g.URL("http://domain.com");
	const params = new g.URLSearchParams("?param=value")

	hasNative = "searchParams" in url && params.get("param") === "value";
}
catch (error)
{
	hasNative = false;
}



if (hasNative)
{
	output.URL = g.URL;
	output.URLSearchParams = g.URLSearchParams;
}
else
{
	const lib = require("whatwg-url");

	output.URL = lib.URL;
	output.URLSearchParams = lib.URLSearchParams;
}



output.shim = () =>
{
	g.URL = output.URL;
	g.URLSearchParams = output.URLSearchParams;
};



module.exports = output;
