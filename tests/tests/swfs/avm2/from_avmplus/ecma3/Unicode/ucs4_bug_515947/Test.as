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

// var SECTION = "Unicode";       // provide a document reference (ie, Actionscript section)
// var VERSION = "ECMAScript";        // Version of ECMAScript or ActionScript
// var TITLE   = "UTF16 surrogate pairs not being translated correctly from UTF8";
var BUGNUMBER = "515947";


// add your tests here

var s:String = "𠂊";
Assert.expectEq( "s.length", 2, s.length );
Assert.expectEq( "0xd840", "d840", s.charCodeAt(0).toString(16) );
Assert.expectEq( "0xdc8a", "dc8a", s.charCodeAt(1).toString(16) );


              // displays results.
              



