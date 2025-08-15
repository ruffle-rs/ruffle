/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "8.6.1 Constructor Methods";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "SuperStatement for Object";       // Provide ECMA section title or a description
var BUGNUMBER = "";



///////////////////////////////////////////////////////////////
// add your tests here

import SuperObjectPkg.*

import com.adobe.test.Assert;
var so = new SuperObject()
Assert.expectEq( "super statement initializes Object", 1, 1 );

//
////////////////////////////////////////////////////////////////

              // displays results.
