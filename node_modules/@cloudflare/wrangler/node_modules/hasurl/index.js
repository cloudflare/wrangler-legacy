"use strict";
let hasNative;



const hasURL = () =>
{
	if (hasNative === undefined)
	{
		hasNative = "URL" in require("url");
	}

	return hasNative;
};



module.exports = hasURL;
