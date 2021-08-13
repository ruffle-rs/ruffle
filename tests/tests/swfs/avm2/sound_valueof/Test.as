package {
	public class Test {
	}
}

import flash.media.Sound;

trace("///var empty = new Sound();");
var empty = new Sound();

trace("empty");
trace(empty);

trace("empty.toString();");
trace(empty.toString());

trace("empty.valueOf();");
trace(empty.valueOf());

trace("Object.prototype.toString.apply(empty);");
trace(Object.prototype.toString.apply(empty));

trace("Object.prototype.valueOf.apply(empty);");
trace(Object.prototype.valueOf.apply(empty));

trace("///var silence = new Silence();");
var silence = new Silence();

trace("silence");
trace(silence);

trace("silence.toString();");
trace(silence.toString());

trace("silence.valueOf();");
trace(silence.valueOf());

trace("Object.prototype.toString.apply(silence);");
trace(Object.prototype.toString.apply(silence));

trace("Object.prototype.valueOf.apply(silence);");
trace(Object.prototype.valueOf.apply(silence));

trace("///var noise = new Noise();");
var noise = new Noise();

trace("noise");
trace(noise);

trace("noise.toString();");
trace(noise.toString());

trace("noise.valueOf();");
trace(noise.valueOf());

trace("Object.prototype.toString.apply(noise);");
trace(Object.prototype.toString.apply(noise));

trace("Object.prototype.valueOf.apply(noise);");
trace(Object.prototype.valueOf.apply(noise));