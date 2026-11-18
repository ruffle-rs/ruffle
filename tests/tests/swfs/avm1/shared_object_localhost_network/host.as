var so = SharedObject.getLocal("so", "/");
trace("host: " + so.data.writer);
so.data.writer = "host";
trace("host: " + so.flush());
