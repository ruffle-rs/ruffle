/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
// var SECTION="";
// var VERSION = "ECMA_1";


var v1=new Vector.<String>();
Assert.expectEq(
  "slice no args on empty vector",
  "",
  v1.slice().toString());
