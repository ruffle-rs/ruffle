package {

import flash.display.Sprite;
import flash.trace.Trace;
import flash.text.TextField;

public class Test extends Sprite {
	var txt:TextField = new TextField();

	public function Test() {
		txt.width = 1000;
		txt.height = 1000;
		addChild(txt);
		
		trace_("OFF: " + Trace.OFF);
		trace_("METHODS: " + Trace.METHODS);
		trace_("METHODS_WITH_ARGS: " + Trace.METHODS_WITH_ARGS);
		trace_("METHODS_AND_LINES: " + Trace.METHODS_AND_LINES);
		trace_("METHODS_AND_LINES_WITH_ARGS: " + Trace.METHODS_AND_LINES_WITH_ARGS);
		trace_("FILE: " + Trace.FILE);
		trace_("Listener: " + Trace.LISTENER);
		
		trace_("getLevel.length: " + Trace.getLevel.length);
		trace_("getListener.length: " + Trace.getListener.length);
		trace_("setLevel.length: " + Trace.setLevel.length);
		trace_("setListener.length: " + Trace.setListener.length);

		trace_("getLevel(): " + Trace.getLevel());
		trace_("getListener(): " + Trace.getListener());
		Trace.setListener(trace_);
		Trace.setLevel(4);
		Trace.setLevel(4, 1);
		Trace.setLevel(4, 2);
		trace_("getLevel(): " + Trace.getLevel());
		trace_("getLevel(1): " + Trace.getLevel(1));
		trace_("getLevel(2): " + Trace.getLevel(2));
		trace_("getListener(): " + Trace.getListener());
	}

	private function trace_(arg:*) {
		txt.text += arg + "\n";
		trace(arg);
	}
}

}
