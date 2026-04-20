/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "8.6.1 Constructor Methods";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "Implicit SuperStatement";       // Provide ECMA section title or a description
var BUGNUMBER = "";



///////////////////////////////////////////////////////////////
// add your tests here

import SuperRuntimeErrorPkg.*

import com.adobe.test.Assert;
import com.adobe.test.Utils;
var srte = new SuperRuntimeError();
var result = "";
var thisError = "";

thisError = "no exception thrown";
try {
    result = srte.missingSuperMethod();
} catch (e) {
    thisError = e.toString();
} finally {
    Assert.expectEq( "call missing base method via super", Utils.REFERENCEERROR+1070, Utils.referenceError( thisError ) );
}

thisError = "no exception thrown";
try {
    result = srte.callSuperPrivate();
} catch (e) {
    thisError = e.toString();
} finally {
    Assert.expectEq( "call private base method via super", Utils.REFERENCEERROR+1070, Utils.referenceError( thisError ) );
}

thisError = "no exception thrown";
try {
    result = srte.callSuperOtherPackage();
} catch (e) {
    thisError = e.toString();
} finally {
    Assert.expectEq( "call internal base method from different package via super", Utils.REFERENCEERROR+1070, Utils.referenceError( thisError ) );
}

//
////////////////////////////////////////////////////////////////

              // displays results.
