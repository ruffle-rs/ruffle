/* Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/ */

/*
 Compiled with:
 node utils/compileabc.js --swf Preloader,100,100,60 -p test/swfs/as3-loader/loaderinfo/Preloader.as
 */

package {

import flash.display.MovieClip;

public class Preloader extends MovieClip {
  public function Preloader() {
    var loaderInfo = root.loaderInfo;
    var url = loaderInfo.loaderURL;
    trace(url.indexOf("test.swf") >= 0);
  }
}
}
