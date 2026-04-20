/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

/*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=549389
*
*/
//-----------------------------------------------------------------------------


import com.adobe.test.Assert;

var s:String = "abc";
var badChar:String = String.fromCharCode(0xD801);
s += badChar;
s.replace(/\uD801/, "\\uD801");
// test here is actually to make sure that this will run, previously
// the shell would have already crashed
Assert.expectEq("test", 4, s.length);


