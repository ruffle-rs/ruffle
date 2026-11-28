var so = SharedObject.getLocal("avm1");
so.data.count = (so.data.count == undefined ? 0 : so.data.count) + 1;
trace("avm1 child: " + so.data.count + ", " + so.flush());
