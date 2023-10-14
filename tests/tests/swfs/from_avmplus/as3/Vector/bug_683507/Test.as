/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Vector";
// var VERSION = "AS3";

var expected = "TypeError: Error #1128";
var err = "exception not thrown";
try {
    var vError:Vector.<Number, int> = new Vector.<Number, int>;
}
catch (e:Error){
    err = e.toString();
}
Assert.expectEq("TypeError: Error #1128: Incorrect number of type parameters",
  expected,
  Utils.parseError(err, expected.length));
