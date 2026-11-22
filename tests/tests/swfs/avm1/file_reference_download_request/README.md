# Running the test

To verify the actual requests sent over the network, run `server.py` from this directory. It serves the files in `localhost` in chunks, and without a `Content-Length` where the test asks for it, like the test runner does, and saves every request it receives to `requests/`. Then, run `http://localhost:8000/test.swf` in either Flash Player or the Ruffle Desktop player. Loading the SWF from the server lets it send requests there without a trust file.

Flash Player only allows downloading in response to a user action, so click the stage to start each of the four downloads there, and save the file anywhere.

The saved requests from both players should match, and both should report the progress for every chunk.
