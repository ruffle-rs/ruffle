# NetConnection StrictArray undefined deserialization test

The `localhost/test1` file is a raw AMF0 packet fixture used automatically by
Ruffle's SWF test navigator. The ActionScript connects to
`http://localhost:8000/test1`, and the test harness resolves that request to
this fixture file.

For a manual Flash Player run, start `server.py` in this directory before
opening the compiled `test.swf`. The server returns the same checked-in AMF0
packet. Flash Player may also require a Flash Player Trust entry allowing the
SWF to connect to localhost.

The packet's `/1/onResult` value is a StrictArray of length two containing
`undefined` at index 0 and the string `keep` at index 1.
