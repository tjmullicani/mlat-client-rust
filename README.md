# mlat-client

WORK IN PROGRESS PORT OF MLAT-CLIENT
IN PROGRESS:
  * BEAST MESSAGE DECODING
  * TCP PORT STREAMING
  * UDP PORT STREAMING
TODO:
  * Multilateration math

This is a client that selectively forwards Mode S messages to a
server that resolves the transmitter position by multilateration of the same
message received by multiple clients.

The corresponding server code is available at
https://github.com/mutability/mlat-server.

## Building

TODO

## Running

If you are connecting to a third party multilateration server, contact the
server's administrator for configuration instructions.

```console
$ cat beast-test-capture.bin | nc -l 127.0.0.1 3000
$ socat -u UDP4-RECVFROM:30004,reuseaddr,fork exec:"xxd -"
$ cargo run -- --lat 1 --lon 1 --alt 1 --user 1 --server 127.0.0.1:30004
```

## Supported receivers

TODO

## Unsupported receivers

* The FlightRadar24 radarcape-based receiver. This produces a deliberately
crippled timestamp in its output, making it useless for multilateration.
If you have one of these, you should ask FR24 to fix this.

## License

Copyright 2023, [Timothy Mullican](mailto:timothy.j.mullican@gmail.com).

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received [a copy of the GNU General Public License](LICENSE)
along with this program.  If not, see <http://www.gnu.org/licenses/>.
