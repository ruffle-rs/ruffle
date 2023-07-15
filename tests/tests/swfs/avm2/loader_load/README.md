# Running the test

To verify the actual headers sent over the network, run 'server.py'
from this directory. Then, run 'test.swf' in either Flash Player
or the Ruffle Desktop player.

When running under flash player, you'll need to allow the SWF to
make network connections. On Linux, this can be done by creating
the file `/etc/adobe/FlashPlayerTrust/test.cfg` with the following contents:

```
/ancestor/of/swf/path
```

where `ancestor/of/swf/path` is any path that's an ancestor of the path of `test.swf`
(e.g. `/home/username/`)
