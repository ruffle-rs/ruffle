/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

/**
 *  This template is a modified version of the test case
 *  templates taken from mozilla.org.  This template or
 *  any test cases written from it are not for external
 *  use or distribution.
 *
 *  Author: Brent Baker <brbaker@adobe.com>
 *  Date: 10/06/2009
 *
 *  Modifications: (Name   :Date)

 */

// var SECTION = "";       // provide a document reference (ie, Actionscript section)
// var VERSION = "";        // Version of ECMAScript or ActionScript
// var TITLE   = "Surrogate pairs are mangled by String.Replace";       // Provide ECMA section title or a description
var BUGNUMBER = "439458";


// add your tests here

var text:String = String.fromCharCode(0xD840, 0xDC0B);
var dblSpacePattern:RegExp = /[\s]{2,}/g;
var strippedText:String = text.replace(dblSpacePattern, " ");
Assert.expectEq( "strippedText.charCodeAt(0).toString(16)", "d840", strippedText.charCodeAt(0).toString(16) );
Assert.expectEq( "strippedText.charCodeAt(1).toString(16)", "dc0b", strippedText.charCodeAt(1).toString(16) );
Assert.expectEq( "strippedText.charCodeAt(2).toString(16)", "NaN", strippedText.charCodeAt(2).toString(16) );
Assert.expectEq( "strippedText.charCodeAt(3).toString(16)", "NaN", strippedText.charCodeAt(3).toString(16) );
Assert.expectEq( "strippedText==text", true, strippedText==text );


              // displays results.

