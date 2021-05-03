package {
	public class Test {
	}
}

import flash.display.Scene;
import flash.display.FrameLabel;

trace("//var label1 = new FrameLabel(\"test\", 123);");
var label1 = new FrameLabel("test", 123);

trace("//var val = new Scene(\"test\", [label1], 456);");
var val = new Scene("test", [label1], 456);

trace("//val.labels");
trace(val.labels);

trace("//val.name");
trace(val.name);

trace("//val.numFrames");
trace(val.numFrames);