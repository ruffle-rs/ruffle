/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

/**
 *  This template is a modified version of the test case
 *  templates taken from mozilla.org.  This template or
 *  any test cases written from it are not for external
 *  use or distribution.
 *
 *  Author: Brent Baker <brbaker@adobe.com>
 *  Date: 10/02/2009
 *
 *  Modifications: (Name   :Date)

 */

// var SECTION = "";       // provide a document reference (ie, Actionscript section)
// var VERSION = "";        // Version of ECMAScript or ActionScript
// var TITLE   = "E4X truncate String when it found the '\0'";       // Provide ECMA section title or a description
var BUGNUMBER = "478501";


// add your tests here

var s = "a" + String.fromCharCode(0) + "a";
Assert.expectEq( "<e a={s}/>.toXMLString()", '<e a="a&#x0;a"/>', <e a={s}/>.toXMLString() );


              // displays results.
