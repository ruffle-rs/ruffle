/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "8.6.1 Constructor Methods";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "SuperExpression on Object";       // Provide ECMA section title or a description
var BUGNUMBER = "";



///////////////////////////////////////////////////////////////
// add your tests here

import SuperObjectCallPkg.*

import com.adobe.test.Assert;
import com.adobe.test.Utils;
var soc = new SuperObjectCall();
var thisException = "no exception thrown";
try {
    var s = soc.whatIsIt();
} catch (e) {
    thisException = e.toString();
} finally {
    Assert.expectEq( "super call to Object toString() should fail", Utils.REFERENCEERROR+1070, Utils.referenceError( thisException ) );
}

//
////////////////////////////////////////////////////////////////

              // displays results.
