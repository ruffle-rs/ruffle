/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
// var SECTION = "Bug 703238";
// var VERSION = "AS3";
// var TITLE   = "64bit jit mishandling untyped variable";


function f()
{
    var t = 0;
    for ( var d = 0; d < 777600000; d += 86400000 ) {
        t += d;
        /*
        We go wrong once we overflow the 31-bit signed integer range.
        Test to make sure that once we pass the 31-bit signed integer
        range that we start working in the Number range, test this
        by marking sure that the results here stay positive.
        */
        Assert.expectEq("Result should always stay positive", true, t/86400000 >= 0);
    }
}
f();

