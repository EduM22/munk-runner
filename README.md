# munk
Munk - js run service 

## Setup
  * `env:MUNK_DB_PATH` -> Set custom save path for db | Default = `/var/lib/munk/`
  * `env:MUNK_ENCRYPTION_KEY` -> Set encryption key for db
  * `env:MUNK_AUTH_HEADER_VALUE` -> Set the admin auth header value

## Info
The runtime is trying to conform to the [wintertc](https://min-common-api.proposal.wintertc.org/) spec.   
The navigator.userAgent value should be = `Munk`

### Limits
The code execution has a timeout of 15s and cpu limit of 50ms

### Custom API

```
Munk.env.get(key: string) -> string | undefined | null
Munk.env.toObject() -> { key: value } | undefined | null

Munk.serve((req: Request) => Response)
```

### Add new Function

POST - `/api/function`  

Headers - [  
  `munk-function-id`: `main`,  
  `munk-auth`: `${VALUE_SET_IN_SETUP}`  
]  

Body
```
{
    "code": "Munk.serve(async (req) => new Response(`Hello from munk ${Munk.env.get('test')}`))",
    "envs": [
        { "test": "this works, soo cool" }
    ]
}
```

#### Returns

header `munk-function-id` with the id of the created function.

### Call function

Add header `munk-function-id` with the id of the function, then you will be redirected to the function.

ex: `'munk-function-id': '604qi60u0h0v'`

## Setup Prod

You should probably use a reverse proxy like `nginx` or `caddy` and setup something like this:  
`https://{functionId}.domain.run`  
->  
`http://localhost:3000`  
  * with header `'munk-function-id': '{functionId}'`

## Data from other providers:

* IP address data is powered by [IPinfo](https://ipinfo.io/lite)
* JS runtime from [deno_core](https://github.com/denoland/deno_core)