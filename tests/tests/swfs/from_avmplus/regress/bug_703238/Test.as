/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

// var SECTION = "regress_703238";
// var VERSION = "AS3";
// var TITLE   = "overflow the 31-bit signed integer range";
// var bug = "703238";


function f()
{
  var t = 0;
  for ( var d = 0; d < 777600000; d += 86400000 ) {
    t += d;
    print(t);
    print(t/86400000);
    Assert.expectEq("Results should always stay positive", true,   (t/86400000)>-1);
  }
}
f();

