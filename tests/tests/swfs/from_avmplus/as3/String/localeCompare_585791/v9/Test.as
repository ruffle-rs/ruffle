/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "String";
//     var VERSION = "as3";
//     var TITLE   = "bug 585791";


    function sign(n:int):int
    {
        if (n < 0) return -1;
        else if (n > 0) return 1;
        else return 0;
    }
    
    var r;

    r = sign("m".localeCompare(null));
    Assert.expectEq('Test "m".localeCompare(null) with SWF9 behavior',
      0,
      r);

    r = sign("null".localeCompare(null));
    Assert.expectEq('Test "null".localeCompare(null) with SWF9 behavior',
      0,
      r);

    r = sign("o".localeCompare(null));
    Assert.expectEq('Test "o".localeCompare(null) with SWF9 behavior',
      0,
      r);

    r = sign("".localeCompare(null));
    Assert.expectEq('Test "".localeCompare(null) with SWF9 behavior',
      1,
      r);

    r = sign("t".localeCompare(undefined));
    Assert.expectEq('Test "t".localeCompare(undefined) with SWF9 behavior',
      0,
      r);

    r = sign("undefined".localeCompare(undefined));
    Assert.expectEq('Test "undefined".localeCompare(undefined) with SWF9 behavior',
      0,
      r);

    r = sign("v".localeCompare(undefined));
    Assert.expectEq('Test "v".localeCompare(undefined) with SWF9 behavior',
      0,
      r);

    r = sign("".localeCompare(undefined));
    Assert.expectEq('Test "".localeCompare(undefined) with SWF9 behavior',
      1,
      r);
