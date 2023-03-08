package {

	public class Test {
	}
}

trace("Valid sequences");
trace("unescape(\"%32%33\")");
trace(unescape("%32%33"));

trace("unescape(\"aa %32%33\")");
trace(unescape("aa %32%33"));

trace("unescape(\"%32 aa %33\")");
trace(unescape("%32 aa %33"));

trace("unescape(\"%32%33 aa\")");
trace(unescape("%32%33 aa"));

trace("unescape(escape(\"😊\"))");
trace(unescape(escape("😊")));

trace("unescape(escape(\"&& 😊 😊 😊 😊 😊bb\"))");
trace(unescape(escape("&& 😊 😊 😊 😊 😊bb")));

trace("Invalid sequences");
trace("unescape(\"%32%3\")");
trace(unescape("%32%3"));

trace("unescape(\"%%3\")");
trace(unescape("%%3"));

trace("unescape(\"%G3\")");
trace(unescape("%G3 %25"));

trace("unescape(\"%u\")");
trace(unescape("%u"));

trace("unescape(\"%u33\")");
trace(unescape("%u33"));

trace("unescape(\"%U3333\")");
trace(unescape("%U3333"));

trace("unescape(\"%u333G\")");
trace(unescape("%u333G"));