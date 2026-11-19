# AMF0 Date deserialization edge cases

`localhost/test2` is a raw AMF0 remoting response containing a native Date. Each Date-binding scenario uses a fresh `NetConnection`, so each request uses the first response URI (`/1`); this keeps the request log focused on Date deserialization rather than response-ID sequencing across separate HTTP requests.
The ActionScript first receives it with the built-in `Date` global, then
replaces `_global.Date` with a script-defined constructor and receives the
same response again. The resulting object uses the replacement constructor's
stored prototype without invoking the script constructor or creating native
Date state. It then replaces `Date` with a non-callable object and with the
native `Number` constructor, checking the undefined fallback and ensuring
the AMF0 timestamp is not written into a non-Date native object.

The separate `localconnection_deserialize_date_invalid_binding` test covers
missing and non-callable Date bindings during LocalConnection deserialization.

For manual Flash Player testing, rebuild `test.swf` from `test.as` with the configured Rascal compiler, then run `python3 server.py` from this directory before opening the SWF.
