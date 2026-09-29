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
  * `env:MUNK_IPINFO_PATH` -> Set custom path for GeoIP database (`ipinfo_lite.mmdb`) | Defaults = `$MUNK_DB_PATH/ipinfo_lite.mmdb`
  * `env:MUNK_ENCRYPTION_KEY` -> Set encryption key for db
  * `env:MUNK_AUTH_HEADER_VALUE` -> Set the admin auth header value
  * `env:MUNK_LOG_TOKEN` -> Set the auth token for viewing logs | Defaults = `env:MUNK_AUTH_HEADER_VALUE`
  * `env:MUNK_USE_SESSION` -> Set to `true` to use `MunkSession` for persistent function execution | Defaults = `false`
  * `env:MUNK_SESSION_WORKERS` -> Set the number of worker threads per `MunkSession` | Defaults = `4`
  * `env:MUNK_MAX_BODY_SIZE` -> Set maximum request body size (e.g. `10MB`, `2048`, `500KB`) | Defaults = `10MB`
  * `env:MUNK_TELEMETRY` -> Set it to `false` to not send telemetry | Defaults = `true`

## Info
The runtime is trying to conform to the [wintertc](https://min-common-api.proposal.wintertc.org/) spec.   
The navigator.userAgent value should be = `Munk`

The use of `env:MUNK_USE_SESSION` CAN increase the throughput of the function BUT it will cache the v8/deno_core runtime per function, So if you need every request to use a clean deno_core isolate DO NOT use `env:MUNK_USE_SESSION`

### Limits
The code execution has default timeout of 15s and cpu limit of 50ms, this can be changed when creating the function.

> [!NOTE]  
> Users without a license are limited to 5 functions. Free-tier functions also have maximum limits of 60s walltime and 30s cputime.

### Permissions

Function permissions restrict what system capabilities (network access, environment variables) the function's JavaScript isolate can perform.

```json
"permissions": {
    "allow_net": ["api.example.com"],
    "allow_env": ["API_KEY"],
    "allow_private_net": false
}
```

* `allow_net`: Controls network outbound connections (e.g. `fetch()`).
* `allow_env`: Controls access to environment variables.
* `allow_private_net`: Controls whether outbound connections are permitted to private / local network IP addresses (optional boolean, default `false`).

#### Permission Levels:

* **Omitted** *(Default)*:
  * **Network (`allow_net`)**: **No network access** by default. Outbound requests (e.g., `fetch()`) require explicit permission via `["*"]` or an allowlist.
  * **Environment (`allow_env`)**: **Unrestricted access** if the `permissions` block is omitted entirely; **No access** if `permissions` is provided but `allow_env` is omitted.
* **Empty Array (`[]`)**: **No access granted**. Outbound network calls or environment variable reads throw a permission error.
* **Wildcard (`["*"]`)**: **Unrestricted access**. Allows connecting to any public host/domain (set `allow_private_net: true` for private IPs) or reading any environment variable.
* **Specific Values (Allowlist)**: **Restricted access**. Allows connecting or reading **only** the specified domains, IP addresses, or environment variable names (e.g., `["api.github.com", "example.com"]`).

### Custom API

```javascript
Munk.env.get(key: string) -> string | undefined | null
Munk.env.toObject() -> { key: value } | undefined | null
Munk.waitUntil(promise: Promise<any>)

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

### Health Check

GET - `/health`

#### Returns

```json
{
    "status": "ok",
    "version": "0.10.0",
    "uptime_secs": 120
}
```

### Response Headers
All responses returned by Munk include the headers:
```http
Server: Munk
x-munk-request-id: {request_id}
```

### List functions

GET - `/api/functions` or `/api/function`  

Headers - [  
  `munk-function-id`: `main`,  
  `munk-auth`: `${VALUE_SET_IN_SETUP}`  
] 

> [!NOTE]  
> API management endpoints are rate-limited to 30 requests per minute per IP (returns `429 Too Many Requests` when exceeded).

#### Returns

```json
{
    "functions": [
        {
            "id": "{functionId}",
            "name": "my-func",
            "created_at": "{function_created_at}",
            "limits": {
                "walltime": "15s",
                "cputime": "50ms"
            },
            "permissions": {
                "allow_net": ["api.example.com"],
                "allow_env": ["DATABASE_URL"],
                "allow_private_net": false
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
    "name": "my-func", // (optional)
    "code": "Munk.serve(async (req) => new Response(`Hello from munk ${Munk.env.get('test')}`))",
    "envs": [
        { "test": "this works, soo cool" }
    ],
    "limits": { // (optional)
        "walltime": "10s", // Default: 15s (Free tier max: 60s)
        "cputime": "75ms" // Default: 50ms (Free tier max: 30s)
    },
    "permissions": { // (optional)
        "allow_net": ["api.example.com"],
        "allow_env": ["API_KEY"],
        "allow_private_net": false // (optional, default: false)
    }
}
```

#### Returns

header `munk-function-id` with the id of the created function.

### Call function

Add header `munk-function-id` with the ID of the function, then you will be routed to the function.

ex: `'munk-function-id': '604qi60u0h0v'`

### Delete function

DELETE - `/api/functions?id={munk-function-id}` or `/api/function?id={munk-function-id}`  

Headers - [  
  `munk-function-id`: `main`,  
  `munk-auth`: `${VALUE_SET_IN_SETUP}`  
]  

Deleting a function also deletes all associated logs.

#### Returns

status code `204`

### Stream Logs (SSE)

GET - `/api/logs/stream`  

Headers - [  
  `munk-function-id`: `main`,  
  `Authorization`: `Bearer ${MUNK_LOG_TOKEN}` (or `Bearer ${MUNK_AUTH_HEADER_VALUE}`) or `munk-auth`: `${MUNK_LOG_TOKEN}` (or `${MUNK_AUTH_HEADER_VALUE}`)  
]  

Query Parameters (enables direct browser `EventSource` connections without custom headers):
* `token`: `${MUNK_LOG_TOKEN}` (or `${MUNK_AUTH_HEADER_VALUE}`)

Example URL for browser `EventSource`:
```javascript
const eventSource = new EventSource('/api/logs/stream?token=YOUR_LOG_TOKEN');
```

> [!NOTE]  
> Log endpoints are rate-limited to 30 requests per minute per IP (returns `429 Too Many Requests` when exceeded).

#### Returns

A real-time Server-Sent Events (SSE) stream of type `text/event-stream`. Each event data is a JSON string representing a function execution log:

```json
{
    "id": "{munk-function-id}",
    "is_error": false,
    "message": "log message from code execution"
}
```

### Get Logs (History)

GET - `/api/logs?id={function_id}&limit={limit}`  

Headers - [  
  `munk-function-id`: `main`,  
  `munk-auth`: `${VALUE_SET_IN_SETUP}`  
]  

Query parameters:
* `id` (required): function ID
* `limit` (optional): max number of logs to fetch (default: 100, max: 1000)

> [!NOTE]  
> Log endpoints are rate-limited to 30 requests per minute per IP (returns `429 Too Many Requests` when exceeded).

#### Returns

```json
{
    "logs": [
        {
            "id": "{log_id}",
            "function_id": "{function_id}",
            "is_error": false,
            "message": "log message from code execution",
            "created_at": "2026-07-19 14:00:00"
        }
    ]
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


