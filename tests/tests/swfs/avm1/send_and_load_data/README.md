# Running the test

To verify the actual requests sent over the network, run `server.py` from this directory. It answers like the test runner's fake responses do and saves every request it receives to `requests/`. Then, run `http://localhost:8000/test.swf` in either Flash Player or the Ruffle Desktop player. Loading the SWF from the server lets it send requests there without a trust file.

The saved requests from both players should match.
