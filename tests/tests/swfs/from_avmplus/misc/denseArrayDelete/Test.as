/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

// var SECTION = "Array";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "test";       // Provide ECMA section title or a description
var BUGNUMBER = "";


// add your tests here
function arrayCrasher():Boolean
{
    var a:Array = new Array();
    a[0] = 1; // so that the array "hasDense"
    a[0x10000000] = 3;
    delete a[0x10000000]; // stack overflow
    return true;
}

// Lack of a crash means the bug is fixed
Assert.expectEq( "var helloWorld = 'Hello World'", true, arrayCrasher() );


              // displays results.
