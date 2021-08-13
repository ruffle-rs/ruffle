package {
	public class Test {
	}
}

trace("///var silence = new Silence();");
var silence = new Silence();

trace("///silence.bytesLoaded;");
trace(silence.bytesLoaded);

trace("///silence.bytesTotal;");
trace(silence.bytesTotal);

trace("///silence.isBuffering;");
trace(silence.isBuffering);

trace("///silence.isURLInaccessible;");
trace(silence.isURLInaccessible);

trace("///silence.length;");
trace(silence.length);

trace("///silence.url;");
trace(silence.url);

trace("///var noise = new Noise();");
var noise = new Noise();

trace("///noise.bytesLoaded;");
trace(noise.bytesLoaded);

trace("///noise.bytesTotal;");
trace(noise.bytesTotal);

trace("///noise.isBuffering;");
trace(noise.isBuffering);

trace("///noise.isURLInaccessible;");
trace(noise.isURLInaccessible);

trace("///noise.length;");
trace(noise.length);

trace("///noise.url;");
trace(noise.url);