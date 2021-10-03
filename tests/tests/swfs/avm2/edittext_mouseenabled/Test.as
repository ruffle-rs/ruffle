package {
	public class Test {
		
	}
}

import flash.text.TextField;

trace("///var text = new TextField();");
var text = new TextField();

trace("///(Initial state of event enabled flags...)");

trace("///text.mouseEnabled");
trace(text.mouseEnabled);

trace("///text.doubleClickEnabled");
trace(text.doubleClickEnabled);

trace("///text.doubleClickEnabled = true");
text.doubleClickEnabled = true;

trace("///text.mouseEnabled");
trace(text.mouseEnabled);

trace("///text.doubleClickEnabled");
trace(text.doubleClickEnabled);

trace("///text.mouseEnabled = false");
text.mouseEnabled = false;

trace("///text.mouseEnabled");
trace(text.mouseEnabled);

trace("///text.doubleClickEnabled");
trace(text.doubleClickEnabled);

trace("///text.doubleClickEnabled = false");
text.doubleClickEnabled = false;

trace("///text.mouseEnabled");
trace(text.mouseEnabled);

trace("///text.doubleClickEnabled");
trace(text.doubleClickEnabled);

trace("///text.mouseEnabled = true");
text.mouseEnabled = true;

trace("///text.mouseEnabled");
trace(text.mouseEnabled);

trace("///text.doubleClickEnabled");
trace(text.doubleClickEnabled);