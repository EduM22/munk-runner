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

> [!NOTE]  
> Users without a license is limited to 5 functions.

### Custom API

```
Munk.env.get(key: string) -> string | undefined | null
Munk.env.toObject() -> { key: value } | undefined | null

Munk.serve((req: Request) => Response)
```

### Custom request headers
These are inserted into the request to the function
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

GET - `/api/function`  

Headers - [  
  `munk-function-id`: `main`,  
  `munk-auth`: `${VALUE_SET_IN_SETUP}`  
] 

#### Returns

```
{
  functions: [
    {
      id: '{functionId}',
      created_at: '{function_created_at}'
    }
  ]
}
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

### Delete function

DELETE - `/api/function?id={munk-function-id}`  

Headers - [  
  `munk-function-id`: `main`,  
  `munk-auth`: `${VALUE_SET_IN_SETUP}`  
]  

#### Returns

status code `201`

## Setup Prod

You should probably use a reverse proxy like `nginx` or `caddy` and setup something like this:  
`https://{functionId}.domain.run`  
->  
`http://localhost:3000`  
  * with header `'munk-function-id': '{functionId}'`

## Data from other providers:

* IP address data is powered by [IPinfo](https://ipinfo.io/lite)
* JS runtime from [deno_core](https://github.com/denoland/deno_core)