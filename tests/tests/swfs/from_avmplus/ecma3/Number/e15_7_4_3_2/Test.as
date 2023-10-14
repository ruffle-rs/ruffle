/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.7.4.3-2";
//     var VERSION = "ECMA_4";
    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    v = Number.prototype.valueOf;
    Number.prototype.valueOf=v;
    num = 3;
    array[item++] = Assert.expectEq(
            "v = Number.prototype.valueOf; Number.prototype.valueof=v;num = 3;num.valueOf()",
            3,
            num.valueOf() );

    num = new Number(3);
    array[item++] = Assert.expectEq(
            "v = Number.prototype.valueOf; Number.prototype.valueof=v;num = 3;num.valueOf()",
            3,
            num.valueOf() );

    num = new Number();
    array[item++] = Assert.expectEq(
            "v = Number.prototype.valueOf; Number.prototype.valueof=v;num = 3;num.valueOf()",
            0,
            num.valueOf() );

    return ( array );
}
