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
 *  Date: 10/02/2009
 *
 *  Modifications: (Name   :Date)

 */

// var SECTION = "15.5.4.8";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "lastindexof(null)";       // Provide ECMA section title or a description
var BUGNUMBER = "477132";


// add your tests here

Assert.expectEq( "''.lastIndexOf(null)", -1, "".lastIndexOf(null) );


              // displays results.
