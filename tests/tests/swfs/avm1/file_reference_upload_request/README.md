# Running the test

To verify the actual requests sent over the network, run `server.py` from this directory. It answers like the test runner's fake responses do and saves every request it receives to `requests/`. Then, run `http://localhost:8000/test.swf` in either Flash Player or the Ruffle Desktop player. Loading the SWF from the server lets it send requests there without a trust file.

Flash Player only allows browsing for a file in response to a user action, so click the stage to start each of the five uploads there, and select a text file containing `Hello, World!`.

The saved requests from both players should match, except for the multipart boundary string, which Flash Player generates randomly.
