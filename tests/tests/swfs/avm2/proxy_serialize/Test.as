package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.net.ObjectEncoding;
import flash.net.registerClassAlias;
import flash.utils.ByteArray;
import flash.utils.flash_proxy;
import flash.utils.Proxy;

dynamic class MyProxy extends Proxy {
  flash_proxy override function nextNameIndex(index: int): int {
    trace("nextNameIndex: " + index);

    if (index < 2) {
      return index + 1;
    } else {
      return 0;
    }
  }

  flash_proxy override  function nextName(index:int):String {
    trace("nextName: " + index);
    return "name" + index;
  }

  flash_proxy override  function nextValue(index:int):* {
    trace("nextValue: " + index);
    return "value" + index;
  }
}

var ba = new ByteArray();
ba.objectEncoding = ObjectEncoding.AMF3;
ba.writeObject(new MyProxy());

trace("");

var dump = "";
for (var i = 0; i < ba.length; i++) {
  var hex = ba[i].toString(16)
  dump += hex.length < 2 ? "0" + hex : hex;
}
trace("bytes: " + dump);
