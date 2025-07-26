// -*- mode: js; indent-tabs-mode: nil -*-
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

/*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=559565
*
*/
//-----------------------------------------------------------------------------

import com.adobe.test.Assert;
// var SECTION = "559565";
// var VERSION = "";
// var TITLE   = "delete arr[1]; set arr[0] should not assert";
// var bug = "557933";


var arr = []

arr[1] = 1
delete arr[1]

arr[0] = 1

Assert.expectEq("This test asserts at arr[0] assignment above if bug is present",
            true, true);

