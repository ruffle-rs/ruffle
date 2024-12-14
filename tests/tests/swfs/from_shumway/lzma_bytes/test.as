/**
 * Compiled with:
 * node ./utils/compileabc.js --swf LzmaBytesTest,600,600,60 -p test/swfs/lzma_bytes.as
 */

package {

import flash.display.Sprite;
import flash.utils.ByteArray;
import flash.system.fscommand;

public class LzmaBytesTest extends Sprite {

  public function LzmaBytesTest() {

    var bytes = [93, 0, 0, 16, 0, 19, 0, 0, 0, 0, 0, 0, 0,
      0, 34, 25, 73, -89, 30, -111, 20, -25, 96,
      -50, -33, -3, -28, -126, 36, 55, -94, 119, 0];

    var ba:ByteArray = new ByteArray();
    var i:int;
    for (i = 0; i < bytes.length; i++) {
      ba.writeByte(bytes[i]);
    }
    ba.position = 0;

    ba.uncompress('lzma');

    var s:String = '';
    ba.position = 0;
    for (i = 0; i < ba.length; i++) {
      s += String.fromCharCode(ba.readByte());
    }

    trace(ba.length);
    trace(s);

    fscommand("quit");
  }
}
}
