package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.events.NetFilterEvent;
import flash.utils.ByteArray;

var event = new NetFilterEvent("netfilter", false, false, null, new ByteArray());
trace(event.toString());
trace(event.clone().toString());

event = new NetFilterEvent("netfilter", false, true, new ByteArray(), null);
trace(event.toString());
trace(event.clone().toString());

var a = new ByteArray();
var b = new ByteArray();
event = new NetFilterEvent("netfilter", true, false, a, b);
trace(event.toString());
trace("event.header === a: " + (event.header === a));
trace("event.data === b: " + (event.data === b));
var clone = event.clone();
trace(clone.toString());
trace("clone.header === a: " + (clone.header === a));
trace("clone.data === b: " + (clone.data === b));