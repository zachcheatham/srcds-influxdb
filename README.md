# SRCDS Query to InfluxDB v2

A simple docker image for recording srcds statistics to InfluxDB!

## Getting Started

To build and run this repo, perform the following steps

```
$ git clone https://github.com/zachcheatham/srcds-influxdb.git srcds-influxdb
$ docker build -t srcds-influxdb:latest -f Dockerfile srcds-influxdb
$ docker run -v ./config.yaml:/config.yaml srcds-influxdb:latest
```

Be sure to create a `config.yaml` before running. See below for an example file.

## Example `config.yaml`

```
influxdb:
  host: http://ip.or.host:8086
  organization: MyOrg
  bucket: my_bucket_name
  token: token
servers:
  - host: 127.0.0.1
    port: 27015
  - host: 127.0.0.1
    port: 27016
frequency_secs: 30
```
