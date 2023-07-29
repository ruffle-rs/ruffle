/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "15.4.3.1-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array.prototype";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var ARRAY_PROTO = Array.prototype;

    var props = '';
    for ( var p in Array  ) {
        props += p
    }
    array[item++] = Assert.expectEq(   "var props = '', for ( p in Array  ) { props += p } props", "", props );

    // avmplus will throw a ReferenceError here. Becaue we are using const instead of enum, and this
    // is expected(?) behavior when using const
    var thisError:String = "no error";


    try{
        Array.prototype = null;
        result = "no exception thrown"

    } catch (e:ReferenceError) {
        thisError = e.toString();



    } finally {
        array[item++] = Assert.expectEq(   "Array.prototype = null; Array.prototype ",
"ReferenceError: Error #1074", Utils.referenceError(thisError) );

    }
    array[item++] = Assert.expectEq(   "delete Array.prototype",                   false,       delete Array.prototype );
    array[item++] = Assert.expectEq(   "delete Array.prototype; Array.prototype",  ARRAY_PROTO, (delete Array.prototype, Array.prototype) );

    return ( array );
}
