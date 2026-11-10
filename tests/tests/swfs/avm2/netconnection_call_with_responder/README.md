# Running the test

Start the local HTTP server from this directory:

```sh
python3 server.py
```

Then run `test.swf` in Flash Player Debugger. The server accepts the
AMF remoting POST and returns a minimal AMF0 `/1/onResult` response so
the `Responder` callback completes without producing extra output.

When running under Flash Player, allow the SWF to make network connections.
On Linux, create `/etc/adobe/FlashPlayerTrust/test.cfg` with an ancestor
directory of the SWF path, for example:

```
/home/username/
```
