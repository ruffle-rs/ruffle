/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.3.3.1-1";
//     var VERSION = "ECMA_2";
//     var TITLE   = "Function.prototype";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // Result is false. See http://flashqa.macromedia.com/bugapp/detail.asp?ID=122988
    array[item++] = Assert.expectEq(  "Function.constructor.prototype == Function.prototype", false, Function.constructor.prototype == Function.prototype );
    
    return array;
}
