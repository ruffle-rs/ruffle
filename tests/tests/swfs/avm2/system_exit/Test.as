package {
  import flash.display.Sprite;
  import flash.system.System;

  public class Test extends Sprite {
    public function Test() {
    	try {
    		System.exit(0);
    	} catch (e:*) {
    		trace(e.getStackTrace())
    	}
    }
  }
}
