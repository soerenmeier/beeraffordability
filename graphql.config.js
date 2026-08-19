const { readFileSync } = require("fs");

// read ../.env file and get the endpoint url
// export it as schema
const ctn = readFileSync("./craft/.env", "utf8");
const ENDPOINT_REGEX = /^\s*ENDPOINT_URL=\s*"?(.*?)"?\s*$/m;
const endpointUrl = ctn.match(ENDPOINT_REGEX)[1];
const TOKEN_REGEX = /^\s*ENDPOINT_TOKEN=\s*"?(.*?)"?\s*$/m;
const endpointToken = ctn.match(TOKEN_REGEX)?.[1];

const schema = { [endpointUrl]: {} };

if (endpointToken)
	schema[endpointUrl].headers = { Authorization: `Bearer ${endpointToken}` };

module.exports = { schema };
