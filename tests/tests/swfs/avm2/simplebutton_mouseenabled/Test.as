package {
	public class Test {
		
	}
}

import flash.display.SimpleButton;

trace("///var btn = new SimpleButton();");
var btn = new SimpleButton();

trace("///(Initial state of event enabled flags...)");

trace("///btn.mouseEnabled");
trace(btn.mouseEnabled);

trace("///btn.doubleClickEnabled");
trace(btn.doubleClickEnabled);

trace("///btn.doubleClickEnabled = true");
btn.doubleClickEnabled = true;

trace("///btn.mouseEnabled");
trace(btn.mouseEnabled);

trace("///btn.doubleClickEnabled");
trace(btn.doubleClickEnabled);

trace("///btn.mouseEnabled = false");
btn.mouseEnabled = false;

trace("///btn.mouseEnabled");
trace(btn.mouseEnabled);

trace("///btn.doubleClickEnabled");
trace(btn.doubleClickEnabled);

trace("///btn.doubleClickEnabled = false");
btn.doubleClickEnabled = false;

trace("///btn.mouseEnabled");
trace(btn.mouseEnabled);

trace("///btn.doubleClickEnabled");
trace(btn.doubleClickEnabled);

trace("///btn.mouseEnabled = true");
btn.mouseEnabled = true;

trace("///btn.mouseEnabled");
trace(btn.mouseEnabled);

trace("///btn.doubleClickEnabled");
trace(btn.doubleClickEnabled);