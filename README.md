# munk
Munk - JS/TS & WASM run service  
Use it for edge functions, A/B testing, security features or anything else you can create

### Status
This project is in: Beta

### Upload cli
use [hilly](https://github.com/EduM22/hilly) to upload code to munk easier

## Example Function

```javascript
Munk.serve(async (req) => {
    const res = await fetch('https://jsonplaceholder.typicode.com/todos/1');
    const data = await res.json();

    return new Response(JSON.stringify({ data }))
})
```

## Setup
Get image from `docker pull ghcr.io/edum22/munk-runner:<VERSION>`

  * `env:MUNK_DB_PATH` -> Set custom save path for db | Default = `/var/lib/munk/`
  * `env:MUNK_ENCRYPTION_KEY` -> Set encryption key for db
  * `env:MUNK_AUTH_HEADER_VALUE` -> Set the admin auth header value
  * `env:MUNK_LOG_TOKEN` -> Set the auth token for viewing logs | Defaults = `env:MUNK_AUTH_HEADER_VALUE`
  * `env:MUNK_USE_SESSION` -> Set to `true` to use `MunkSession` for persistent function execution | Defaults = `false`
  * `env:MUNK_TELEMETRY` -> Set it to `false` to not send telemetry | Defaults = `true`

## Info
The runtime is trying to conform to the [wintertc](https://min-common-api.proposal.wintertc.org/) spec.   
The navigator.userAgent value should be = `Munk`

The use of `env:MUNK_USE_SESSION` CAN increase the throughput of the function BUT it will cache the v8/deno_core runtime per function, So if you need every request to use a clean deno_core isolate DO NOT use `env:MUNK_USE_SESSION`

### Limits
The code execution has default timeout of 15s and cpu limit of 50ms, this can be changed when creating the function

> [!NOTE]  
> Users without a license is limited to 5 functions.

### Custom API

```javascript
Munk.env.get(key: string) -> string | undefined | null
Munk.env.toObject() -> { key: value } | undefined | null

Munk.serve((req: Request) => Response | Promise<Response>)
```

### Custom request headers
These are inserted into the request to the function.   
The Ip is taken from header: X-Forwarded-For (If you are not using a proxy like `caddy` this header can be spoofed by client)
```
x-munk-geo-as-domain: 'google.com'
x-munk-geo-as-name: 'Google LLC'
x-munk-geo-asn: 'AS15169'
x-munk-geo-continent: 'North America'
x-munk-geo-continent-code: 'NA'
x-munk-geo-country: 'United States'
x-munk-geo-country-code: 'US'
x-munk-geo-ip: '8.8.8.8'
```

### List functions

GET - `/api/functions` or `/api/function`  

Headers - [  
  `munk-function-id`: `main`,  
  `munk-auth`: `${VALUE_SET_IN_SETUP}`  
] 

#### Returns

```json
{
    "functions": [
        {
            "id": "{functionId}",
            "created_at": "{function_created_at}",
            "limits": {
                "walltime": "15s",
                "cputime": "50ms"
            }
        }
    ]
}
```

### Add new Function

POST - `/api/functions` or `/api/function`  

Headers - [  
  `munk-function-id`: `main`,  
  `munk-auth`: `${VALUE_SET_IN_SETUP}`  
]  

Body
```json
{
    "code": "Munk.serve(async (req) => new Response(`Hello from munk ${Munk.env.get('test')}`))",
    "envs": [
        { "test": "this works, soo cool" }
    ],
    "limits": { // (optional)
        "walltime": "10s", // Default: 15s
        "cputime": "75ms" // Default: 50ms
    }
}
```

#### Returns

header `munk-function-id` with the id of the created function.

### Call function

Add header `munk-function-id` with the id of the function, then you will be redirected to the function.

ex: `'munk-function-id': '604qi60u0h0v'`

### Delete function

DELETE - `/api/functions?id={munk-function-id}` or `/api/function?id={munk-function-id}`  

Headers - [  
  `munk-function-id`: `main`,  
  `munk-auth`: `${VALUE_SET_IN_SETUP}`  
]  

#### Returns

status code `204`

### Stream Logs (SSE)

GET - `/api/logs/stream`  

Headers - [  
  `Authorization`: `Bearer ${MUNK_LOG_TOKEN}` (or `Bearer ${MUNK_AUTH_HEADER_VALUE}` if no log token is set)  
]  

#### Returns

A real-time Server-Sent Events (SSE) stream of type `text/event-stream`. Each event data is a JSON string representing a function execution log:

```json
{
    "id": "{munk-function-id}",
    "is_error": false,
    "message": "log message from code execution"
}
```

## Setup Prod

You should probably use a reverse proxy like `nginx` or `caddy` and setup something like this:  
`https://{functionId}.domain.run`  
->  
`http://localhost:3000`  
  * with header `'munk-function-id': '{functionId}'`

## Data from other providers:

* IP address data is powered by [IPinfo](https://ipinfo.io/lite)
* JS runtime from [deno_core](https://github.com/denoland/deno_core)
