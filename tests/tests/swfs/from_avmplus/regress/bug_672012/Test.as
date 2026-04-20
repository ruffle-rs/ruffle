/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

// var SECTION = "Regression Tests";
// var VERSION = "AS3";
// var TITLE   = "Bug 672012";
// var bug = "672012";


var subject:String = "AAA";
subject.match(/(((((((((((((((((((a*)(abc|b))))))))))))))))))*.)*(...)*/g);
subject.match(/((((((((((((((((((d|.*)))))))))))))))))*.)*(...)*/g);
subject.match(/((((((((((((((((((a+)*))))))))))))))))*.)*(...)*/g);

Assert.expectEq("Completed", true, true);


