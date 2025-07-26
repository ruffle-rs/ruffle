/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

/*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=538107
*
*/
//-----------------------------------------------------------------------------



import com.adobe.test.Assert;
// ED B0 80 = UTF-8 for 0xDC00
// ED A0 80 = UTF-8 for 0xD800
// 0xDC00 0xD800 is an invalid UTF-16 sequence
var s = "%ED%B0%80%ED%A0%80";
decodeURIComponent(s);
// test here is actually to make sure that this will run, previously
// the shell would have already thrown a runtime error:
//  URIError: Error #1052: Invalid URI passed to decodeURIComponent function.
Assert.expectEq("decodeURIComponent", 2, decodeURIComponent(s).length);


decodeURI(s);
// test here is actually to make sure that this will run, previously
// the shell would have already thrown a runtime error:
//  URIError: Error #1052: Invalid URI passed to decodeURI function.
Assert.expectEq("decodeURI", 2, decodeURI(s).length);


