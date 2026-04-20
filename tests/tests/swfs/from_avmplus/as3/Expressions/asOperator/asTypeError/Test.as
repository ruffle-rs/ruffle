/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "as operator";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "test";       // Provide ECMA section title or a description
var BUGNUMBER = "";


///////////////////////////////////////////////////////////////
// add your tests here

try {
    var x = 13;
    var y:String = "";
    var z = (x as notAValidType);

} catch(e) {
    y = e;
} finally {
    Assert.expectEq( "as Type Error: (x as notAValidType)", "ReferenceError: Error #1065", Utils.referenceError(y) );
}

try {
    x = 13;
    var temp = "hello";
    y = "";
    z = (x as temp);

} catch(e:TypeError) {
    y = e;
} finally {
    Assert.expectEq( "as Type Error: temp='hello'; (x as temp)", "TypeError: Error #1009", Utils.typeError(y) );
}

/* errors are now compile-time in falcon
try {
    x = 13;
    y = "";
    z = (x as undefined);

} catch(e:TypeError) {
    y = e;
} finally {
    Assert.expectEq( "as Type Error: (x as undefined)", "TypeError: Error #1010", Utils.typeError(y) );
}

try {
    x = 13;
    y = "";
    z = (x as 3333);

} catch(e:TypeError) {
    y = e;
} finally {
    Assert.expectEq( "as Type Error: (x as 3333)", "TypeError: Error #1009", Utils.typeError(y) );
}

try {
    x = 13;
    y = "";
    z = (x as "string");

} catch(e:TypeError) {
    y = e;
} finally {
    Assert.expectEq( "as Type Error: (x as 'string')", "TypeError: Error #1009", Utils.typeError(y) );
}

*/

//
////////////////////////////////////////////////////////////////

              // displays results.

