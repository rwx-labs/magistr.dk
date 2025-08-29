# magistr.dk

This is the website for magistr.dk.

## Environment variables

This is a list of environment variables that the application supports.


| Name | Type | Default | Description |
|------|------|---------|-------------|
| `MAGISTR_TRACING_ENABLED` | Boolean | `true` | Enable tracing exporter |
| `MAGISTR_DATABASE_URL` | String | `postgresql://magistr:password@localhost/magistr_development` | Connection URL to the database |
| `MAGISTR_DATABASE_MAX_CONNECTIONS` | Number | 5 | Maximum number of connections the cool maintains at once |
| `MAGISTR_DATABASE_IDLE_TIMEOUT` | Number | 30 | How long a connection in the database connection pool can stay idle until closed |
| `MAGISTR_HTTP_ADDRESS` | String | `[::]:3000` | Address to start listening for HTTP requests on |
| `MAGISTR_HTTP_COMPRESSION` | Boolean | `true` | Support compressing responses if the client supports it |

