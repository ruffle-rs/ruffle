/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import voidEvaluation.*
import com.adobe.test.Assert;


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Function Return Type";       // Provide ECMA section title or a description
var BUGNUMBER = "108487";


var TESTOBJ = new TestObj();

var result = "exception";
try {
    TESTOBJ.g();
    result = "no exception";
}  catch (e1) {
    result = e1.toString();
}

Assert.expectEq("Assign function that returns void", "no exception",  result);

Assert.expectEq("Test for g being called", "hello from g", TESTOBJ.varG);

Assert.expectEq("Test for f being called", "hello from f", TESTOBJ.varF);

              // displays results.
