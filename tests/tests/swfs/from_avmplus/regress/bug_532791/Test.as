/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

// Regression testcase for https://bugzilla.mozilla.org/show_bug.cgi?id=532791


var r:RegExp = /(^#\{)|(\}$)/;
Assert.expectEq("var r:RegExp = /(^#\{)|(\}$)/; r.exec('# {blah blah}')", "},,}", r.exec('# {blah blah}').toString());

